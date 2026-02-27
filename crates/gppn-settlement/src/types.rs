use chrono::{DateTime, Utc};
use gppn_core::types::{Amount, Did};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SettlementId(pub Uuid);

impl SettlementId {
    /// Create a new random settlement ID (UUID v7 — time-ordered).
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Create from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SettlementId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SettlementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The lifecycle status of a settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementStatus {
    /// Settlement has been created but not yet submitted.
    Initiated,
    /// Settlement has been submitted and is awaiting confirmation.
    Pending,
    /// Settlement has been confirmed on the underlying rail.
    Confirmed,
    /// Settlement has failed (non-recoverable).
    Failed,
    /// Settlement has been rolled back.
    RolledBack,
}

impl std::fmt::Display for SettlementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initiated => write!(f, "Initiated"),
            Self::Pending => write!(f, "Pending"),
            Self::Confirmed => write!(f, "Confirmed"),
            Self::Failed => write!(f, "Failed"),
            Self::RolledBack => write!(f, "RolledBack"),
        }
    }
}

/// Proof that a settlement completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementReceipt {
    /// Settlement identifier.
    pub settlement_id: SettlementId,
    /// Adapter that processed the settlement.
    pub adapter_id: String,
    /// Final status (should be Confirmed).
    pub status: SettlementStatus,
    /// Amount settled.
    pub amount: Amount,
    /// Sender DID.
    pub sender_did: Did,
    /// Receiver DID.
    pub receiver_did: Did,
    /// Timestamp when the settlement was confirmed.
    pub confirmed_at: DateTime<Utc>,
    /// Optional transaction reference on the underlying rail.
    pub tx_ref: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use gppn_core::types::{Currency, FiatCurrency};

    #[test]
    fn test_settlement_id_creation() {
        let id1 = SettlementId::new();
        let id2 = SettlementId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_settlement_id_display() {
        let id = SettlementId::new();
        let s = format!("{}", id);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_settlement_status_display() {
        assert_eq!(format!("{}", SettlementStatus::Initiated), "Initiated");
        assert_eq!(format!("{}", SettlementStatus::Pending), "Pending");
        assert_eq!(format!("{}", SettlementStatus::Confirmed), "Confirmed");
        assert_eq!(format!("{}", SettlementStatus::Failed), "Failed");
        assert_eq!(format!("{}", SettlementStatus::RolledBack), "RolledBack");
    }

    #[test]
    fn test_settlement_receipt_creation() {
        let receipt = SettlementReceipt {
            settlement_id: SettlementId::new(),
            adapter_id: "sa-internal".to_string(),
            status: SettlementStatus::Confirmed,
            amount: Amount::new(1000, Currency::Fiat(FiatCurrency::USD)),
            sender_did: Did::from_parts("key", "alice"),
            receiver_did: Did::from_parts("key", "bob"),
            confirmed_at: Utc::now(),
            tx_ref: Some("tx-001".to_string()),
        };
        assert_eq!(receipt.status, SettlementStatus::Confirmed);
        assert_eq!(receipt.adapter_id, "sa-internal");
    }
}
