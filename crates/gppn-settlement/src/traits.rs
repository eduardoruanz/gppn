use async_trait::async_trait;
use gppn_core::types::{Amount, Currency, Did};
use std::time::Duration;

use crate::error::SettlementError;
use crate::types::{SettlementId, SettlementReceipt, SettlementStatus};

/// Settlement adapter interface.
///
/// Each implementation bridges the GPPN protocol to a concrete settlement
/// rail (blockchain, PIX, SWIFT, internal ledger, etc.).
#[async_trait]
pub trait ISettlement: Send + Sync {
    /// Initiate a new settlement between two parties.
    async fn initiate(
        &self,
        pm_id: uuid::Uuid,
        amount: Amount,
        sender_did: Did,
        receiver_did: Did,
    ) -> Result<SettlementId, SettlementError>;

    /// Confirm a previously initiated settlement.
    async fn confirm(
        &self,
        settlement_id: SettlementId,
    ) -> Result<SettlementReceipt, SettlementError>;

    /// Rollback / cancel a settlement that has not yet been confirmed.
    async fn rollback(&self, settlement_id: SettlementId) -> Result<(), SettlementError>;

    /// Query the current status of a settlement.
    async fn get_status(
        &self,
        settlement_id: SettlementId,
    ) -> Result<SettlementStatus, SettlementError>;

    /// Estimate the fee / cost for settling the given amount.
    async fn estimate_cost(&self, amount: &Amount) -> Result<Amount, SettlementError>;

    /// Estimate the latency for settling the given amount.
    async fn estimate_latency(&self, amount: &Amount) -> Result<Duration, SettlementError>;

    /// List the currencies supported by this adapter.
    fn supported_currencies(&self) -> Vec<Currency>;

    /// Return the unique identifier of this adapter (e.g. "sa-internal").
    fn adapter_id(&self) -> &str;
}
