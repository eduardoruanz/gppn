use serde::{Deserialize, Serialize};

use veritas_crypto::{hashing, Blake3ProofGenerator};

use crate::error::ProofError;

/// Proves that a person's country is in an allowed set without revealing it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidencyProof {
    /// The set membership proof.
    pub membership_proof: veritas_crypto::SetMembershipProof,
    /// Merkle root of the allowed country set (public).
    pub allowed_set_root: [u8; 32],
    /// Number of countries in the allowed set.
    pub set_size: usize,
    /// When the proof was generated.
    pub generated_at: String,
}

impl ResidencyProof {
    /// Create a residency proof showing that country is in the allowed set.
    pub fn create(country: &str, allowed_countries: &[&str]) -> Result<Self, ProofError> {
        let set: Vec<Vec<u8>> = allowed_countries
            .iter()
            .map(|c| c.as_bytes().to_vec())
            .collect();

        let leaf_hashes: Vec<[u8; 32]> = set.iter().map(|item| hashing::hash(item)).collect();
        let set_root = hashing::merkle_root(&leaf_hashes);

        let (membership_proof, _nonce) =
            Blake3ProofGenerator::prove_set_membership(country.as_bytes(), &set).map_err(|e| {
                ProofError::GenerationFailed(format!("residency proof failed: {}", e))
            })?;

        Ok(Self {
            membership_proof,
            allowed_set_root: set_root,
            set_size: allowed_countries.len(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Verify the residency proof.
    pub fn verify(&self) -> Result<bool, ProofError> {
        Blake3ProofGenerator::verify_set_membership(&self.membership_proof, &self.allowed_set_root)
            .map_err(|e| ProofError::VerificationFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EU_COUNTRIES: &[&str] = &[
        "AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "FR", "DE", "GR", "HU", "IE", "IT",
        "LV", "LT", "LU", "MT", "NL", "PL", "PT", "RO", "SK", "SI", "ES", "SE",
    ];

    const MERCOSUL: &[&str] = &["AR", "BR", "PY", "UY"];

    #[test]
    fn test_residency_proof_in_set() {
        let proof = ResidencyProof::create("BR", MERCOSUL).unwrap();
        assert!(proof.verify().unwrap());
        assert_eq!(proof.set_size, 4);
    }

    #[test]
    fn test_residency_proof_not_in_set() {
        let result = ResidencyProof::create("US", MERCOSUL);
        assert!(result.is_err());
    }

    #[test]
    fn test_residency_proof_eu() {
        let proof = ResidencyProof::create("DE", EU_COUNTRIES).unwrap();
        assert!(proof.verify().unwrap());
    }

    #[test]
    fn test_residency_proof_last_element() {
        let proof = ResidencyProof::create("UY", MERCOSUL).unwrap();
        assert!(proof.verify().unwrap());
    }

    #[test]
    fn test_residency_proof_first_element() {
        let proof = ResidencyProof::create("AR", MERCOSUL).unwrap();
        assert!(proof.verify().unwrap());
    }

    #[test]
    fn test_residency_proof_serialization() {
        let proof = ResidencyProof::create("BR", MERCOSUL).unwrap();
        let json = serde_json::to_string(&proof).unwrap();
        let back: ResidencyProof = serde_json::from_str(&json).unwrap();
        assert_eq!(back.set_size, 4);
    }
}
