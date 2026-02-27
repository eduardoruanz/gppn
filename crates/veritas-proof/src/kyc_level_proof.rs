use serde::{Deserialize, Serialize};

use veritas_crypto::Blake3ProofGenerator;

use crate::error::ProofError;

/// Proves that a KYC level is >= a required level without revealing the exact level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycLevelProof {
    /// The minimum required level that was proven.
    pub min_level: i64,
    /// Maximum possible KYC level.
    pub max_level: i64,
    /// The range proof.
    pub range_proof: veritas_crypto::RangeProof,
    /// When the proof was generated.
    pub generated_at: String,
}

impl KycLevelProof {
    /// Create a KYC level proof showing level >= min_level.
    ///
    /// KYC levels: 0 = none, 1 = basic, 2 = enhanced, 3 = full.
    pub fn create(actual_level: i64, min_level: i64) -> Result<Self, ProofError> {
        let max_level = 3; // Maximum KYC level

        let (range_proof, _nonce) =
            Blake3ProofGenerator::prove_range(actual_level, min_level, max_level).map_err(|e| {
                ProofError::GenerationFailed(format!("KYC level proof failed: {}", e))
            })?;

        Ok(Self {
            min_level,
            max_level,
            range_proof,
            generated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Verify the KYC level proof.
    pub fn verify(&self) -> Result<bool, ProofError> {
        Blake3ProofGenerator::verify_range(&self.range_proof)
            .map_err(|e| ProofError::VerificationFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kyc_level_proof_valid() {
        let proof = KycLevelProof::create(3, 2).unwrap();
        assert!(proof.verify().unwrap());
        assert_eq!(proof.min_level, 2);
    }

    #[test]
    fn test_kyc_level_proof_exact_match() {
        let proof = KycLevelProof::create(2, 2).unwrap();
        assert!(proof.verify().unwrap());
    }

    #[test]
    fn test_kyc_level_proof_max() {
        let proof = KycLevelProof::create(3, 1).unwrap();
        assert!(proof.verify().unwrap());
    }

    #[test]
    fn test_kyc_level_proof_insufficient() {
        let result = KycLevelProof::create(1, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_kyc_level_proof_zero() {
        let result = KycLevelProof::create(0, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_kyc_level_proof_serialization() {
        let proof = KycLevelProof::create(3, 1).unwrap();
        let json = serde_json::to_string(&proof).unwrap();
        let back: KycLevelProof = serde_json::from_str(&json).unwrap();
        assert_eq!(back.min_level, 1);
    }
}
