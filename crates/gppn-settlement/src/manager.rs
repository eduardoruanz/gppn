use std::collections::HashMap;

use gppn_core::types::{Amount, Currency, Did};
use std::time::Duration;

use crate::error::SettlementError;
use crate::traits::ISettlement;
use crate::types::{SettlementId, SettlementReceipt, SettlementStatus};

/// Central settlement manager that dispatches operations to registered adapters.
pub struct SettlementManager {
    adapters: HashMap<String, Box<dyn ISettlement>>,
}

impl SettlementManager {
    /// Create a new settlement manager with no adapters registered.
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Register a settlement adapter.
    ///
    /// The adapter is keyed by its `adapter_id()`.
    pub fn register_adapter(&mut self, adapter: Box<dyn ISettlement>) {
        let id = adapter.adapter_id().to_string();
        tracing::info!(adapter_id = %id, "Registering settlement adapter");
        self.adapters.insert(id, adapter);
    }

    /// Unregister an adapter by its ID.
    pub fn unregister_adapter(&mut self, adapter_id: &str) -> Option<Box<dyn ISettlement>> {
        self.adapters.remove(adapter_id)
    }

    /// Get a reference to an adapter by its ID.
    pub fn get_adapter(&self, adapter_id: &str) -> Result<&dyn ISettlement, SettlementError> {
        self.adapters
            .get(adapter_id)
            .map(|a| a.as_ref())
            .ok_or_else(|| SettlementError::AdapterNotFound(adapter_id.to_string()))
    }

    /// List all registered adapter IDs.
    pub fn adapter_ids(&self) -> Vec<String> {
        self.adapters.keys().cloned().collect()
    }

    /// Initiate a settlement using the specified adapter.
    pub async fn initiate(
        &self,
        adapter_id: &str,
        pm_id: uuid::Uuid,
        amount: Amount,
        sender_did: Did,
        receiver_did: Did,
    ) -> Result<SettlementId, SettlementError> {
        let adapter = self.get_adapter(adapter_id)?;
        adapter.initiate(pm_id, amount, sender_did, receiver_did).await
    }

    /// Confirm a settlement using the specified adapter.
    pub async fn confirm(
        &self,
        adapter_id: &str,
        settlement_id: SettlementId,
    ) -> Result<SettlementReceipt, SettlementError> {
        let adapter = self.get_adapter(adapter_id)?;
        adapter.confirm(settlement_id).await
    }

    /// Rollback a settlement using the specified adapter.
    pub async fn rollback(
        &self,
        adapter_id: &str,
        settlement_id: SettlementId,
    ) -> Result<(), SettlementError> {
        let adapter = self.get_adapter(adapter_id)?;
        adapter.rollback(settlement_id).await
    }

    /// Get the status of a settlement using the specified adapter.
    pub async fn get_status(
        &self,
        adapter_id: &str,
        settlement_id: SettlementId,
    ) -> Result<SettlementStatus, SettlementError> {
        let adapter = self.get_adapter(adapter_id)?;
        adapter.get_status(settlement_id).await
    }

    /// Estimate cost for a settlement through the specified adapter.
    pub async fn estimate_cost(
        &self,
        adapter_id: &str,
        amount: &Amount,
    ) -> Result<Amount, SettlementError> {
        let adapter = self.get_adapter(adapter_id)?;
        adapter.estimate_cost(amount).await
    }

    /// Estimate latency for a settlement through the specified adapter.
    pub async fn estimate_latency(
        &self,
        adapter_id: &str,
        amount: &Amount,
    ) -> Result<Duration, SettlementError> {
        let adapter = self.get_adapter(adapter_id)?;
        adapter.estimate_latency(amount).await
    }

    /// List currencies supported by the specified adapter.
    pub fn supported_currencies(
        &self,
        adapter_id: &str,
    ) -> Result<Vec<Currency>, SettlementError> {
        let adapter = self.get_adapter(adapter_id)?;
        Ok(adapter.supported_currencies())
    }

    /// Get the number of registered adapters.
    pub fn adapter_count(&self) -> usize {
        self.adapters.len()
    }
}

impl Default for SettlementManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::internal::InternalAdapter;
    use gppn_core::types::{Currency, FiatCurrency};

    #[tokio::test]
    async fn test_register_and_list_adapters() {
        let mut mgr = SettlementManager::new();
        mgr.register_adapter(Box::new(InternalAdapter::new()));

        let ids = mgr.adapter_ids();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&"sa-internal".to_string()));
    }

    #[tokio::test]
    async fn test_unregister_adapter() {
        let mut mgr = SettlementManager::new();
        mgr.register_adapter(Box::new(InternalAdapter::new()));
        assert_eq!(mgr.adapter_count(), 1);

        mgr.unregister_adapter("sa-internal");
        assert_eq!(mgr.adapter_count(), 0);
    }

    #[tokio::test]
    async fn test_get_adapter_not_found() {
        let mgr = SettlementManager::new();
        let result = mgr.get_adapter("nonexistent");
        assert!(matches!(result, Err(SettlementError::AdapterNotFound(_))));
    }

    #[tokio::test]
    async fn test_initiate_via_manager() {
        let mut mgr = SettlementManager::new();
        mgr.register_adapter(Box::new(InternalAdapter::new()));

        let amount = Amount::new(1000, Currency::Fiat(FiatCurrency::USD));
        let sender = Did::from_parts("key", "alice");
        let receiver = Did::from_parts("key", "bob");

        let settlement_id = mgr
            .initiate("sa-internal", uuid::Uuid::now_v7(), amount, sender, receiver)
            .await
            .unwrap();

        let status = mgr
            .get_status("sa-internal", settlement_id)
            .await
            .unwrap();
        assert_eq!(status, SettlementStatus::Initiated);
    }

    #[tokio::test]
    async fn test_full_settlement_lifecycle_via_manager() {
        let mut mgr = SettlementManager::new();
        mgr.register_adapter(Box::new(InternalAdapter::new()));

        let amount = Amount::new(5000, Currency::Fiat(FiatCurrency::BRL));
        let sender = Did::from_parts("key", "alice");
        let receiver = Did::from_parts("key", "bob");

        let sid = mgr
            .initiate("sa-internal", uuid::Uuid::now_v7(), amount, sender, receiver)
            .await
            .unwrap();

        let receipt = mgr.confirm("sa-internal", sid).await.unwrap();
        assert_eq!(receipt.status, SettlementStatus::Confirmed);
    }

    #[tokio::test]
    async fn test_estimate_cost_via_manager() {
        let mut mgr = SettlementManager::new();
        mgr.register_adapter(Box::new(InternalAdapter::new()));

        let amount = Amount::new(10_000, Currency::Fiat(FiatCurrency::USD));
        let cost = mgr.estimate_cost("sa-internal", &amount).await.unwrap();
        assert_eq!(cost.value, 0); // internal adapter is zero-cost
    }

    #[tokio::test]
    async fn test_supported_currencies_via_manager() {
        let mut mgr = SettlementManager::new();
        mgr.register_adapter(Box::new(InternalAdapter::new()));

        let currencies = mgr.supported_currencies("sa-internal").unwrap();
        assert!(!currencies.is_empty());
    }

    #[tokio::test]
    async fn test_initiate_nonexistent_adapter() {
        let mgr = SettlementManager::new();
        let amount = Amount::new(100, Currency::Fiat(FiatCurrency::USD));
        let result = mgr
            .initiate(
                "sa-nonexistent",
                uuid::Uuid::now_v7(),
                amount,
                Did::from_parts("key", "a"),
                Did::from_parts("key", "b"),
            )
            .await;
        assert!(matches!(result, Err(SettlementError::AdapterNotFound(_))));
    }

    #[test]
    fn test_manager_default() {
        let mgr = SettlementManager::default();
        assert_eq!(mgr.adapter_count(), 0);
    }
}
