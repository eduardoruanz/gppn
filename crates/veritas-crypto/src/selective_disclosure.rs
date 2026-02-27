use std::collections::HashMap;

use crate::error::CryptoError;
use crate::hashing::{self, Hash};
use crate::zkp::Commitment;

/// Selective disclosure of claims using BLAKE3 commitments.
///
/// Each claim is committed individually, allowing the holder to reveal
/// only chosen claims while keeping others hidden behind commitments.
#[derive(Debug, Clone)]
pub struct SelectiveDisclosure {
    /// Map of claim_name → (commitment, nonce).
    commitments: HashMap<String, (Commitment, [u8; 32])>,
}

impl SelectiveDisclosure {
    /// Create a new empty selective disclosure set.
    pub fn new() -> Self {
        Self {
            commitments: HashMap::new(),
        }
    }

    /// Add a claim and commit to its value.
    /// Returns the commitment for this claim.
    pub fn add_claim(&mut self, name: impl Into<String>, value: &[u8]) -> Commitment {
        let (commitment, nonce) = Commitment::commit(value);
        let c = commitment.clone();
        self.commitments.insert(name.into(), (commitment, nonce));
        c
    }

    /// Get the commitment for a claim.
    pub fn commitment_for(&self, name: &str) -> Option<&Commitment> {
        self.commitments.get(name).map(|(c, _)| c)
    }

    /// Reveal a claim: returns the nonce needed for verification.
    /// The verifier can then check: H(value || nonce) == commitment.
    pub fn reveal_claim(&self, name: &str) -> Option<(Commitment, [u8; 32])> {
        self.commitments.get(name).cloned()
    }

    /// Get all claim names.
    pub fn claim_names(&self) -> Vec<&str> {
        self.commitments.keys().map(|s| s.as_str()).collect()
    }

    /// Number of committed claims.
    pub fn len(&self) -> usize {
        self.commitments.len()
    }

    /// Whether there are no committed claims.
    pub fn is_empty(&self) -> bool {
        self.commitments.is_empty()
    }

    /// Compute a Merkle root over all commitment hashes (sorted by claim name).
    /// This provides a single hash representing all committed claims.
    pub fn commitment_root(&self) -> Hash {
        let mut sorted_names: Vec<&String> = self.commitments.keys().collect();
        sorted_names.sort();

        let hashes: Vec<Hash> = sorted_names
            .iter()
            .map(|name| self.commitments[*name].0.hash)
            .collect();

        hashing::merkle_root(&hashes)
    }

    /// Verify a revealed claim value against its commitment.
    pub fn verify_revealed(
        commitment: &Commitment,
        value: &[u8],
        nonce: &[u8; 32],
    ) -> Result<bool, CryptoError> {
        Ok(commitment.verify(value, nonce))
    }
}

impl Default for SelectiveDisclosure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_reveal_claim() {
        let mut sd = SelectiveDisclosure::new();
        sd.add_claim("name", b"Alice");
        sd.add_claim("country", b"BR");
        sd.add_claim("dob", b"1990-01-15");

        assert_eq!(sd.len(), 3);
        assert!(!sd.is_empty());

        // Reveal only country
        let (commitment, nonce) = sd.reveal_claim("country").unwrap();
        assert!(SelectiveDisclosure::verify_revealed(&commitment, b"BR", &nonce).unwrap());
    }

    #[test]
    fn test_verify_wrong_value() {
        let mut sd = SelectiveDisclosure::new();
        sd.add_claim("age", b"25");

        let (commitment, nonce) = sd.reveal_claim("age").unwrap();
        assert!(!SelectiveDisclosure::verify_revealed(&commitment, b"30", &nonce).unwrap());
    }

    #[test]
    fn test_commitment_for() {
        let mut sd = SelectiveDisclosure::new();
        sd.add_claim("email", b"alice@example.com");

        assert!(sd.commitment_for("email").is_some());
        assert!(sd.commitment_for("phone").is_none());
    }

    #[test]
    fn test_claim_names() {
        let mut sd = SelectiveDisclosure::new();
        sd.add_claim("a", b"1");
        sd.add_claim("b", b"2");

        let names = sd.claim_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
    }

    #[test]
    fn test_commitment_root_deterministic() {
        let mut sd1 = SelectiveDisclosure::new();
        sd1.add_claim("x", b"1");
        sd1.add_claim("y", b"2");

        // Different insertion order but same data won't match because nonces are random,
        // but the root should be deterministic for a given SelectiveDisclosure instance
        let root1 = sd1.commitment_root();
        let root2 = sd1.commitment_root();
        assert_eq!(root1, root2);
    }

    #[test]
    fn test_empty_selective_disclosure() {
        let sd = SelectiveDisclosure::new();
        assert!(sd.is_empty());
        assert_eq!(sd.len(), 0);
        assert!(sd.reveal_claim("nothing").is_none());
    }

    #[test]
    fn test_selective_reveal_subset() {
        let mut sd = SelectiveDisclosure::new();
        sd.add_claim("name", b"Alice");
        sd.add_claim("dob", b"1990-01-15");
        sd.add_claim("country", b"BR");
        sd.add_claim("kyc_level", b"3");

        // Verifier asks for country and kyc_level only
        let (country_c, country_n) = sd.reveal_claim("country").unwrap();
        let (kyc_c, kyc_n) = sd.reveal_claim("kyc_level").unwrap();

        // Verify revealed claims
        assert!(SelectiveDisclosure::verify_revealed(&country_c, b"BR", &country_n).unwrap());
        assert!(SelectiveDisclosure::verify_revealed(&kyc_c, b"3", &kyc_n).unwrap());

        // Name and DOB remain hidden (only commitment visible)
        assert!(sd.commitment_for("name").is_some());
        assert!(sd.commitment_for("dob").is_some());
    }
}
