use chrono::{DateTime, Utc};
use dashmap::DashMap;
use gppn_core::types::{Amount, Did};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::SettlementError;

/// Status of a Hash Time-Locked Contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HtlcStatus {
    /// HTLC is active and awaiting claim or expiry.
    Active,
    /// The receiver has claimed the HTLC by revealing the preimage.
    Claimed,
    /// The HTLC has expired and funds were refunded to the sender.
    Refunded,
    /// The HTLC has expired but has not yet been refunded.
    Expired,
}

impl std::fmt::Display for HtlcStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Claimed => write!(f, "Claimed"),
            Self::Refunded => write!(f, "Refunded"),
            Self::Expired => write!(f, "Expired"),
        }
    }
}

/// A Hash Time-Locked Contract (HTLC).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Htlc {
    /// Unique identifier for this HTLC.
    pub id: Uuid,
    /// BLAKE3 hash of the preimage; the receiver must reveal the preimage to claim.
    pub hash_lock: [u8; 32],
    /// Absolute expiry time in milliseconds since UNIX epoch.
    pub time_lock: u64,
    /// Locked amount.
    pub amount: Amount,
    /// Sender DID (funds originator).
    pub sender_did: Did,
    /// Receiver DID (funds destination).
    pub receiver_did: Did,
    /// Current status.
    pub status: HtlcStatus,
    /// When the HTLC was created.
    pub created_at: DateTime<Utc>,
}

/// Manager for HTLC lifecycle operations.
///
/// Thread-safe: uses `DashMap` for concurrent access.
pub struct HtlcManager {
    htlcs: DashMap<Uuid, Htlc>,
}

impl HtlcManager {
    /// Create a new, empty HTLC manager.
    pub fn new() -> Self {
        Self {
            htlcs: DashMap::new(),
        }
    }

    /// Create a new HTLC from a preimage.
    ///
    /// The `hash_lock` is computed as `blake3(preimage)`.
    /// The `time_lock` is the absolute expiry in milliseconds since epoch.
    pub fn create_htlc(
        &self,
        preimage: &[u8],
        time_lock: u64,
        amount: Amount,
        sender_did: Did,
        receiver_did: Did,
    ) -> Htlc {
        let hash_lock = *blake3::hash(preimage).as_bytes();
        let htlc = Htlc {
            id: Uuid::now_v7(),
            hash_lock,
            time_lock,
            amount,
            sender_did,
            receiver_did,
            status: HtlcStatus::Active,
            created_at: Utc::now(),
        };
        self.htlcs.insert(htlc.id, htlc.clone());
        tracing::info!(htlc_id = %htlc.id, "HTLC created");
        htlc
    }

    /// Claim an HTLC by revealing the preimage.
    ///
    /// The preimage is hashed with BLAKE3 and compared against the stored `hash_lock`.
    pub fn claim(&self, htlc_id: Uuid, preimage: &[u8]) -> Result<Htlc, SettlementError> {
        let mut entry = self
            .htlcs
            .get_mut(&htlc_id)
            .ok_or(SettlementError::NotFound(htlc_id))?;

        let htlc = entry.value_mut();

        if htlc.status != HtlcStatus::Active {
            return Err(SettlementError::InvalidStateTransition(format!(
                "cannot claim HTLC in status {}",
                htlc.status
            )));
        }

        // Check expiry
        let now_ms = Utc::now().timestamp_millis() as u64;
        if now_ms >= htlc.time_lock {
            htlc.status = HtlcStatus::Expired;
            return Err(SettlementError::Expired(htlc_id));
        }

        // Verify preimage
        let hash = *blake3::hash(preimage).as_bytes();
        if hash != htlc.hash_lock {
            return Err(SettlementError::PreimageMismatch(htlc_id));
        }

        htlc.status = HtlcStatus::Claimed;
        tracing::info!(htlc_id = %htlc_id, "HTLC claimed");
        Ok(htlc.clone())
    }

    /// Refund an HTLC that has expired.
    ///
    /// Fails if the HTLC has not yet expired or is not in an active/expired state.
    pub fn refund(&self, htlc_id: Uuid) -> Result<Htlc, SettlementError> {
        let mut entry = self
            .htlcs
            .get_mut(&htlc_id)
            .ok_or(SettlementError::NotFound(htlc_id))?;

        let htlc = entry.value_mut();

        match htlc.status {
            HtlcStatus::Active | HtlcStatus::Expired => {}
            _ => {
                return Err(SettlementError::InvalidStateTransition(format!(
                    "cannot refund HTLC in status {}",
                    htlc.status
                )));
            }
        }

        let now_ms = Utc::now().timestamp_millis() as u64;
        if now_ms < htlc.time_lock {
            return Err(SettlementError::HtlcNotExpired(htlc_id));
        }

        htlc.status = HtlcStatus::Refunded;
        tracing::info!(htlc_id = %htlc_id, "HTLC refunded");
        Ok(htlc.clone())
    }

    /// Check all active HTLCs and mark expired ones.
    ///
    /// Returns the number of HTLCs that were marked as expired.
    pub fn check_expiry(&self) -> usize {
        let now_ms = Utc::now().timestamp_millis() as u64;
        let mut expired_count = 0;

        for mut entry in self.htlcs.iter_mut() {
            let htlc = entry.value_mut();
            if htlc.status == HtlcStatus::Active && now_ms >= htlc.time_lock {
                htlc.status = HtlcStatus::Expired;
                expired_count += 1;
                tracing::debug!(htlc_id = %htlc.id, "HTLC expired");
            }
        }

        expired_count
    }

    /// Get an HTLC by its ID.
    pub fn get(&self, htlc_id: &Uuid) -> Option<Htlc> {
        self.htlcs.get(htlc_id).map(|entry| entry.clone())
    }

    /// Get the number of tracked HTLCs.
    pub fn len(&self) -> usize {
        self.htlcs.len()
    }

    /// Check if the manager has no HTLCs.
    pub fn is_empty(&self) -> bool {
        self.htlcs.is_empty()
    }
}

impl Default for HtlcManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gppn_core::types::{Currency, FiatCurrency};

    fn test_amount() -> Amount {
        Amount::new(1_000_000, Currency::Fiat(FiatCurrency::USD))
    }

    fn sender_did() -> Did {
        Did::from_parts("key", "alice")
    }

    fn receiver_did() -> Did {
        Did::from_parts("key", "bob")
    }

    fn future_time_lock() -> u64 {
        // 1 hour from now
        (Utc::now().timestamp_millis() as u64) + 3_600_000
    }

    fn past_time_lock() -> u64 {
        // 1 hour ago
        (Utc::now().timestamp_millis() as u64).saturating_sub(3_600_000)
    }

    #[test]
    fn test_create_htlc() {
        let mgr = HtlcManager::new();
        let preimage = b"secret-preimage";
        let htlc = mgr.create_htlc(
            preimage,
            future_time_lock(),
            test_amount(),
            sender_did(),
            receiver_did(),
        );

        assert_eq!(htlc.status, HtlcStatus::Active);
        assert_eq!(htlc.hash_lock, *blake3::hash(preimage).as_bytes());
        assert_eq!(mgr.len(), 1);
    }

    #[test]
    fn test_claim_htlc_success() {
        let mgr = HtlcManager::new();
        let preimage = b"secret-preimage";
        let htlc = mgr.create_htlc(
            preimage,
            future_time_lock(),
            test_amount(),
            sender_did(),
            receiver_did(),
        );

        let claimed = mgr.claim(htlc.id, preimage).unwrap();
        assert_eq!(claimed.status, HtlcStatus::Claimed);
    }

    #[test]
    fn test_claim_htlc_wrong_preimage() {
        let mgr = HtlcManager::new();
        let htlc = mgr.create_htlc(
            b"correct-preimage",
            future_time_lock(),
            test_amount(),
            sender_did(),
            receiver_did(),
        );

        let result = mgr.claim(htlc.id, b"wrong-preimage");
        assert!(matches!(result, Err(SettlementError::PreimageMismatch(_))));
    }

    #[test]
    fn test_claim_expired_htlc() {
        let mgr = HtlcManager::new();
        let preimage = b"secret";
        let htlc = mgr.create_htlc(
            preimage,
            past_time_lock(),
            test_amount(),
            sender_did(),
            receiver_did(),
        );

        let result = mgr.claim(htlc.id, preimage);
        assert!(matches!(result, Err(SettlementError::Expired(_))));
    }

    #[test]
    fn test_refund_expired_htlc() {
        let mgr = HtlcManager::new();
        let htlc = mgr.create_htlc(
            b"preimage",
            past_time_lock(),
            test_amount(),
            sender_did(),
            receiver_did(),
        );

        let refunded = mgr.refund(htlc.id).unwrap();
        assert_eq!(refunded.status, HtlcStatus::Refunded);
    }

    #[test]
    fn test_refund_not_expired_htlc() {
        let mgr = HtlcManager::new();
        let htlc = mgr.create_htlc(
            b"preimage",
            future_time_lock(),
            test_amount(),
            sender_did(),
            receiver_did(),
        );

        let result = mgr.refund(htlc.id);
        assert!(matches!(result, Err(SettlementError::HtlcNotExpired(_))));
    }

    #[test]
    fn test_refund_already_claimed() {
        let mgr = HtlcManager::new();
        let preimage = b"preimage";
        let htlc = mgr.create_htlc(
            preimage,
            future_time_lock(),
            test_amount(),
            sender_did(),
            receiver_did(),
        );

        mgr.claim(htlc.id, preimage).unwrap();
        let result = mgr.refund(htlc.id);
        assert!(matches!(
            result,
            Err(SettlementError::InvalidStateTransition(_))
        ));
    }

    #[test]
    fn test_check_expiry() {
        let mgr = HtlcManager::new();
        // Create some active HTLCs with past time lock
        mgr.create_htlc(
            b"a",
            past_time_lock(),
            test_amount(),
            sender_did(),
            receiver_did(),
        );
        mgr.create_htlc(
            b"b",
            past_time_lock(),
            test_amount(),
            sender_did(),
            receiver_did(),
        );
        // One still active
        mgr.create_htlc(
            b"c",
            future_time_lock(),
            test_amount(),
            sender_did(),
            receiver_did(),
        );

        let expired = mgr.check_expiry();
        assert_eq!(expired, 2);
    }

    #[test]
    fn test_get_htlc() {
        let mgr = HtlcManager::new();
        let htlc = mgr.create_htlc(
            b"p",
            future_time_lock(),
            test_amount(),
            sender_did(),
            receiver_did(),
        );

        let found = mgr.get(&htlc.id).unwrap();
        assert_eq!(found.id, htlc.id);
        assert_eq!(found.hash_lock, htlc.hash_lock);
    }

    #[test]
    fn test_get_nonexistent_htlc() {
        let mgr = HtlcManager::new();
        assert!(mgr.get(&Uuid::now_v7()).is_none());
    }

    #[test]
    fn test_htlc_manager_default() {
        let mgr = HtlcManager::default();
        assert!(mgr.is_empty());
    }

    #[test]
    fn test_double_claim_fails() {
        let mgr = HtlcManager::new();
        let preimage = b"preimage";
        let htlc = mgr.create_htlc(
            preimage,
            future_time_lock(),
            test_amount(),
            sender_did(),
            receiver_did(),
        );

        mgr.claim(htlc.id, preimage).unwrap();
        let result = mgr.claim(htlc.id, preimage);
        assert!(matches!(
            result,
            Err(SettlementError::InvalidStateTransition(_))
        ));
    }

    #[test]
    fn test_claim_nonexistent_htlc() {
        let mgr = HtlcManager::new();
        let result = mgr.claim(Uuid::now_v7(), b"preimage");
        assert!(matches!(result, Err(SettlementError::NotFound(_))));
    }

    #[test]
    fn test_htlc_status_display() {
        assert_eq!(format!("{}", HtlcStatus::Active), "Active");
        assert_eq!(format!("{}", HtlcStatus::Claimed), "Claimed");
        assert_eq!(format!("{}", HtlcStatus::Refunded), "Refunded");
        assert_eq!(format!("{}", HtlcStatus::Expired), "Expired");
    }
}
