use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use gppn_core::types::{Amount, Currency, CryptoCurrency, Did, FiatCurrency};
use std::time::Duration;
use uuid::Uuid;

use crate::error::SettlementError;
use crate::traits::ISettlement;
use crate::types::{SettlementId, SettlementReceipt, SettlementStatus};

/// An individual ledger entry in the double-entry bookkeeping system.
#[derive(Debug, Clone)]
struct LedgerEntry {
    /// Unique ID for this entry.
    _id: Uuid,
    /// The DID whose account is affected.
    did: Did,
    /// Positive = credit, negative = debit (stored as signed i128).
    delta: i128,
    /// Associated settlement.
    settlement_id: SettlementId,
    /// Currency of the entry.
    currency: Currency,
}

/// Internal record for a settlement.
#[derive(Debug, Clone)]
struct InternalSettlement {
    id: SettlementId,
    pm_id: Uuid,
    amount: Amount,
    sender_did: Did,
    receiver_did: Did,
    status: SettlementStatus,
}

/// Off-chain, zero-cost settlement adapter.
///
/// Implements an in-memory double-entry ledger for instant settlement
/// within the same GPPN node.  Useful for testing and for local
/// inter-account transfers that do not require an external rail.
pub struct InternalAdapter {
    /// Settlement records keyed by SettlementId.
    settlements: DashMap<Uuid, InternalSettlement>,
    /// Double-entry ledger.
    ledger: DashMap<Uuid, LedgerEntry>,
    /// Balance tracker: (DID string, Currency display) -> signed balance.
    balances: DashMap<String, i128>,
}

impl InternalAdapter {
    /// Create a new internal adapter with empty ledger.
    pub fn new() -> Self {
        Self {
            settlements: DashMap::new(),
            ledger: DashMap::new(),
            balances: DashMap::new(),
        }
    }

    /// Build a composite balance key from a DID and currency.
    fn balance_key(did: &Did, currency: &Currency) -> String {
        format!("{}:{}", did.uri(), currency)
    }

    /// Get the current balance for a DID + currency pair.
    pub fn get_balance(&self, did: &Did, currency: &Currency) -> i128 {
        let key = Self::balance_key(did, currency);
        self.balances.get(&key).map(|v| *v).unwrap_or(0)
    }

    /// Record a double-entry pair: debit sender, credit receiver.
    fn record_entries(
        &self,
        settlement_id: SettlementId,
        sender: &Did,
        receiver: &Did,
        amount: &Amount,
    ) {
        let value = amount.value as i128;

        // Debit sender
        let debit_id = Uuid::now_v7();
        self.ledger.insert(
            debit_id,
            LedgerEntry {
                _id: debit_id,
                did: sender.clone(),
                delta: -value,
                settlement_id,
                currency: amount.currency.clone(),
            },
        );

        // Credit receiver
        let credit_id = Uuid::now_v7();
        self.ledger.insert(
            credit_id,
            LedgerEntry {
                _id: credit_id,
                did: receiver.clone(),
                delta: value,
                settlement_id,
                currency: amount.currency.clone(),
            },
        );

        // Update balances
        let sender_key = Self::balance_key(sender, &amount.currency);
        let receiver_key = Self::balance_key(receiver, &amount.currency);

        self.balances
            .entry(sender_key)
            .and_modify(|b| *b -= value)
            .or_insert(-value);
        self.balances
            .entry(receiver_key)
            .and_modify(|b| *b += value)
            .or_insert(value);
    }

    /// Reverse a double-entry pair (for rollback).
    fn reverse_entries(
        &self,
        sender: &Did,
        receiver: &Did,
        amount: &Amount,
    ) {
        let value = amount.value as i128;

        let sender_key = Self::balance_key(sender, &amount.currency);
        let receiver_key = Self::balance_key(receiver, &amount.currency);

        self.balances
            .entry(sender_key)
            .and_modify(|b| *b += value)
            .or_insert(value);
        self.balances
            .entry(receiver_key)
            .and_modify(|b| *b -= value)
            .or_insert(-value);
    }
}

impl Default for InternalAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ISettlement for InternalAdapter {
    async fn initiate(
        &self,
        pm_id: Uuid,
        amount: Amount,
        sender_did: Did,
        receiver_did: Did,
    ) -> Result<SettlementId, SettlementError> {
        let settlement_id = SettlementId::new();
        let record = InternalSettlement {
            id: settlement_id,
            pm_id,
            amount,
            sender_did,
            receiver_did,
            status: SettlementStatus::Initiated,
        };
        self.settlements.insert(settlement_id.0, record);
        tracing::info!(settlement_id = %settlement_id, "Internal settlement initiated");
        Ok(settlement_id)
    }

    async fn confirm(
        &self,
        settlement_id: SettlementId,
    ) -> Result<SettlementReceipt, SettlementError> {
        let mut entry = self
            .settlements
            .get_mut(&settlement_id.0)
            .ok_or(SettlementError::NotFound(settlement_id.0))?;

        let record = entry.value_mut();

        match record.status {
            SettlementStatus::Initiated | SettlementStatus::Pending => {}
            _ => {
                return Err(SettlementError::InvalidStateTransition(format!(
                    "cannot confirm settlement in status {}",
                    record.status
                )));
            }
        }

        // Record the double-entry
        self.record_entries(
            settlement_id,
            &record.sender_did,
            &record.receiver_did,
            &record.amount,
        );

        record.status = SettlementStatus::Confirmed;

        let receipt = SettlementReceipt {
            settlement_id,
            adapter_id: self.adapter_id().to_string(),
            status: SettlementStatus::Confirmed,
            amount: record.amount.clone(),
            sender_did: record.sender_did.clone(),
            receiver_did: record.receiver_did.clone(),
            confirmed_at: Utc::now(),
            tx_ref: Some(format!("internal-{}", record.pm_id)),
        };

        tracing::info!(settlement_id = %settlement_id, "Internal settlement confirmed");
        Ok(receipt)
    }

    async fn rollback(&self, settlement_id: SettlementId) -> Result<(), SettlementError> {
        let mut entry = self
            .settlements
            .get_mut(&settlement_id.0)
            .ok_or(SettlementError::NotFound(settlement_id.0))?;

        let record = entry.value_mut();

        match record.status {
            SettlementStatus::Initiated | SettlementStatus::Pending => {
                // No entries were recorded, just change status.
            }
            SettlementStatus::Confirmed => {
                // Reverse the double-entry.
                self.reverse_entries(
                    &record.sender_did,
                    &record.receiver_did,
                    &record.amount,
                );
            }
            _ => {
                return Err(SettlementError::InvalidStateTransition(format!(
                    "cannot rollback settlement in status {}",
                    record.status
                )));
            }
        }

        record.status = SettlementStatus::RolledBack;
        tracing::info!(settlement_id = %settlement_id, "Internal settlement rolled back");
        Ok(())
    }

    async fn get_status(
        &self,
        settlement_id: SettlementId,
    ) -> Result<SettlementStatus, SettlementError> {
        let entry = self
            .settlements
            .get(&settlement_id.0)
            .ok_or(SettlementError::NotFound(settlement_id.0))?;
        Ok(entry.status)
    }

    async fn estimate_cost(&self, amount: &Amount) -> Result<Amount, SettlementError> {
        // Internal settlement is zero-cost.
        Ok(Amount::new(0, amount.currency.clone()))
    }

    async fn estimate_latency(&self, _amount: &Amount) -> Result<Duration, SettlementError> {
        // Instant — in-memory.
        Ok(Duration::from_millis(0))
    }

    fn supported_currencies(&self) -> Vec<Currency> {
        vec![
            Currency::Fiat(FiatCurrency::USD),
            Currency::Fiat(FiatCurrency::EUR),
            Currency::Fiat(FiatCurrency::BRL),
            Currency::Fiat(FiatCurrency::GBP),
            Currency::Fiat(FiatCurrency::JPY),
            Currency::Crypto(CryptoCurrency::BTC),
            Currency::Crypto(CryptoCurrency::ETH),
            Currency::Crypto(CryptoCurrency::USDC),
            Currency::Crypto(CryptoCurrency::USDT),
        ]
    }

    fn adapter_id(&self) -> &str {
        "sa-internal"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gppn_core::types::FiatCurrency;

    fn usd(value: u128) -> Amount {
        Amount::new(value, Currency::Fiat(FiatCurrency::USD))
    }

    fn alice() -> Did {
        Did::from_parts("key", "alice")
    }

    fn bob() -> Did {
        Did::from_parts("key", "bob")
    }

    #[tokio::test]
    async fn test_initiate_settlement() {
        let adapter = InternalAdapter::new();
        let sid = adapter
            .initiate(Uuid::now_v7(), usd(1000), alice(), bob())
            .await
            .unwrap();

        let status = adapter.get_status(sid).await.unwrap();
        assert_eq!(status, SettlementStatus::Initiated);
    }

    #[tokio::test]
    async fn test_confirm_settlement() {
        let adapter = InternalAdapter::new();
        let sid = adapter
            .initiate(Uuid::now_v7(), usd(5000), alice(), bob())
            .await
            .unwrap();

        let receipt = adapter.confirm(sid).await.unwrap();
        assert_eq!(receipt.status, SettlementStatus::Confirmed);
        assert_eq!(receipt.adapter_id, "sa-internal");
        assert_eq!(receipt.amount.value, 5000);
    }

    #[tokio::test]
    async fn test_double_entry_balances() {
        let adapter = InternalAdapter::new();
        let sid = adapter
            .initiate(Uuid::now_v7(), usd(10_000), alice(), bob())
            .await
            .unwrap();
        adapter.confirm(sid).await.unwrap();

        let alice_balance = adapter.get_balance(&alice(), &Currency::Fiat(FiatCurrency::USD));
        let bob_balance = adapter.get_balance(&bob(), &Currency::Fiat(FiatCurrency::USD));

        assert_eq!(alice_balance, -10_000);
        assert_eq!(bob_balance, 10_000);
    }

    #[tokio::test]
    async fn test_rollback_initiated() {
        let adapter = InternalAdapter::new();
        let sid = adapter
            .initiate(Uuid::now_v7(), usd(1000), alice(), bob())
            .await
            .unwrap();

        adapter.rollback(sid).await.unwrap();
        let status = adapter.get_status(sid).await.unwrap();
        assert_eq!(status, SettlementStatus::RolledBack);
    }

    #[tokio::test]
    async fn test_rollback_confirmed_reverses_balances() {
        let adapter = InternalAdapter::new();
        let sid = adapter
            .initiate(Uuid::now_v7(), usd(2000), alice(), bob())
            .await
            .unwrap();
        adapter.confirm(sid).await.unwrap();

        // Verify balances before rollback.
        assert_eq!(
            adapter.get_balance(&alice(), &Currency::Fiat(FiatCurrency::USD)),
            -2000
        );
        assert_eq!(
            adapter.get_balance(&bob(), &Currency::Fiat(FiatCurrency::USD)),
            2000
        );

        adapter.rollback(sid).await.unwrap();

        // Balances should be zeroed out.
        assert_eq!(
            adapter.get_balance(&alice(), &Currency::Fiat(FiatCurrency::USD)),
            0
        );
        assert_eq!(
            adapter.get_balance(&bob(), &Currency::Fiat(FiatCurrency::USD)),
            0
        );

        let status = adapter.get_status(sid).await.unwrap();
        assert_eq!(status, SettlementStatus::RolledBack);
    }

    #[tokio::test]
    async fn test_double_confirm_fails() {
        let adapter = InternalAdapter::new();
        let sid = adapter
            .initiate(Uuid::now_v7(), usd(100), alice(), bob())
            .await
            .unwrap();
        adapter.confirm(sid).await.unwrap();

        let result = adapter.confirm(sid).await;
        assert!(matches!(
            result,
            Err(SettlementError::InvalidStateTransition(_))
        ));
    }

    #[tokio::test]
    async fn test_double_rollback_fails() {
        let adapter = InternalAdapter::new();
        let sid = adapter
            .initiate(Uuid::now_v7(), usd(100), alice(), bob())
            .await
            .unwrap();
        adapter.rollback(sid).await.unwrap();

        let result = adapter.rollback(sid).await;
        assert!(matches!(
            result,
            Err(SettlementError::InvalidStateTransition(_))
        ));
    }

    #[tokio::test]
    async fn test_estimate_cost_is_zero() {
        let adapter = InternalAdapter::new();
        let cost = adapter.estimate_cost(&usd(1_000_000)).await.unwrap();
        assert_eq!(cost.value, 0);
    }

    #[tokio::test]
    async fn test_estimate_latency_is_zero() {
        let adapter = InternalAdapter::new();
        let latency = adapter.estimate_latency(&usd(1_000_000)).await.unwrap();
        assert_eq!(latency, Duration::from_millis(0));
    }

    #[tokio::test]
    async fn test_supported_currencies() {
        let adapter = InternalAdapter::new();
        let currencies = adapter.supported_currencies();
        assert!(currencies.contains(&Currency::Fiat(FiatCurrency::USD)));
        assert!(currencies.contains(&Currency::Fiat(FiatCurrency::BRL)));
        assert!(currencies.contains(&Currency::Crypto(CryptoCurrency::BTC)));
    }

    #[tokio::test]
    async fn test_adapter_id() {
        let adapter = InternalAdapter::new();
        assert_eq!(adapter.adapter_id(), "sa-internal");
    }

    #[tokio::test]
    async fn test_get_status_not_found() {
        let adapter = InternalAdapter::new();
        let fake_id = SettlementId::new();
        let result = adapter.get_status(fake_id).await;
        assert!(matches!(result, Err(SettlementError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_multiple_settlements() {
        let adapter = InternalAdapter::new();

        let s1 = adapter
            .initiate(Uuid::now_v7(), usd(1000), alice(), bob())
            .await
            .unwrap();
        let s2 = adapter
            .initiate(Uuid::now_v7(), usd(2000), alice(), bob())
            .await
            .unwrap();

        adapter.confirm(s1).await.unwrap();
        adapter.confirm(s2).await.unwrap();

        let alice_balance = adapter.get_balance(&alice(), &Currency::Fiat(FiatCurrency::USD));
        let bob_balance = adapter.get_balance(&bob(), &Currency::Fiat(FiatCurrency::USD));

        assert_eq!(alice_balance, -3000);
        assert_eq!(bob_balance, 3000);
    }
}
