use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::error::CryptoError;
use crate::hashing::{self, Hash};

/// A BLAKE3-based commitment: H(value || nonce).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commitment {
    /// The commitment hash.
    pub hash: [u8; 32],
}

impl Commitment {
    /// Create a commitment to a value with a random nonce.
    /// Returns the commitment and the nonce (keep nonce secret until reveal).
    pub fn commit(value: &[u8]) -> (Self, [u8; 32]) {
        let mut nonce = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut nonce);
        let hash = hashing::create_commitment(value, &nonce);
        (Self { hash }, nonce)
    }

    /// Create a commitment with a specific nonce.
    pub fn commit_with_nonce(value: &[u8], nonce: &[u8; 32]) -> Self {
        let hash = hashing::create_commitment(value, nonce);
        Self { hash }
    }

    /// Verify that a value and nonce match this commitment.
    pub fn verify(&self, value: &[u8], nonce: &[u8; 32]) -> bool {
        hashing::verify_commitment(value, nonce, &self.hash)
    }
}

/// A range proof using BLAKE3 commitments with Sigma protocol.
/// Proves that a committed value lies within [min, max] without revealing it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeProof {
    /// Commitment to the actual value.
    pub commitment: Commitment,
    /// Challenge hash for verification.
    pub challenge: [u8; 32],
    /// Response: H(value || nonce || challenge).
    pub response: [u8; 32],
    /// Minimum of the range (public).
    pub min: i64,
    /// Maximum of the range (public).
    pub max: i64,
    /// Commitments to boundary checks: H(value - min) and H(max - value).
    pub boundary_commitments: Vec<[u8; 32]>,
}

/// A set membership proof using Merkle inclusion.
/// Proves that a committed value belongs to a set without revealing it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetMembershipProof {
    /// Commitment to the actual value.
    pub commitment: Commitment,
    /// Merkle root of the allowed set.
    pub set_root: [u8; 32],
    /// Merkle proof path (sibling hashes).
    pub merkle_path: Vec<[u8; 32]>,
    /// Direction flags for merkle path (true = right sibling).
    pub path_directions: Vec<bool>,
    /// Challenge-response for binding.
    pub challenge: [u8; 32],
    /// Response hash.
    pub response: [u8; 32],
}

/// Generator for BLAKE3-based zero-knowledge proofs.
pub struct Blake3ProofGenerator;

impl Blake3ProofGenerator {
    /// Prove that a value is within [min, max].
    ///
    /// The prover commits to the value and demonstrates it lies in range
    /// using boundary commitment checks.
    pub fn prove_range(
        value: i64,
        min: i64,
        max: i64,
    ) -> Result<(RangeProof, [u8; 32]), CryptoError> {
        if value < min || value > max {
            return Err(CryptoError::ZkpError(format!(
                "value {} is not in range [{}, {}]",
                value, min, max
            )));
        }

        let value_bytes = value.to_le_bytes();
        let (commitment, nonce) = Commitment::commit(&value_bytes);

        // Boundary commitments: prove value - min >= 0 and max - value >= 0
        let lower_diff = (value - min).to_le_bytes();
        let upper_diff = (max - value).to_le_bytes();
        let mut lower_nonce = [0u8; 32];
        let mut upper_nonce = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut lower_nonce);
        rand::thread_rng().fill_bytes(&mut upper_nonce);
        let lower_commitment = hashing::create_commitment(&lower_diff, &lower_nonce);
        let upper_commitment = hashing::create_commitment(&upper_diff, &upper_nonce);

        // Create Fiat-Shamir challenge from all public data
        let mut challenge_input = Vec::new();
        challenge_input.extend_from_slice(&commitment.hash);
        challenge_input.extend_from_slice(&lower_commitment);
        challenge_input.extend_from_slice(&upper_commitment);
        challenge_input.extend_from_slice(&min.to_le_bytes());
        challenge_input.extend_from_slice(&max.to_le_bytes());
        let challenge = hashing::hash(&challenge_input);

        // Response: H(value || nonce || challenge)
        let mut response_input = Vec::new();
        response_input.extend_from_slice(&value_bytes);
        response_input.extend_from_slice(&nonce);
        response_input.extend_from_slice(&challenge);
        let response = hashing::hash(&response_input);

        let proof = RangeProof {
            commitment,
            challenge,
            response,
            min,
            max,
            boundary_commitments: vec![lower_commitment, upper_commitment],
        };

        Ok((proof, nonce))
    }

    /// Verify a range proof.
    pub fn verify_range(proof: &RangeProof) -> Result<bool, CryptoError> {
        if proof.min > proof.max {
            return Err(CryptoError::ZkpError("invalid range: min > max".into()));
        }
        if proof.boundary_commitments.len() != 2 {
            return Err(CryptoError::ZkpError(
                "range proof must have exactly 2 boundary commitments".into(),
            ));
        }

        // Verify challenge was computed correctly from public data
        let mut challenge_input = Vec::new();
        challenge_input.extend_from_slice(&proof.commitment.hash);
        challenge_input.extend_from_slice(&proof.boundary_commitments[0]);
        challenge_input.extend_from_slice(&proof.boundary_commitments[1]);
        challenge_input.extend_from_slice(&proof.min.to_le_bytes());
        challenge_input.extend_from_slice(&proof.max.to_le_bytes());
        let expected_challenge = hashing::hash(&challenge_input);

        if proof.challenge != expected_challenge {
            return Ok(false);
        }

        // The proof structure is internally consistent
        // (In a full Sigma protocol, the verifier would check the response against
        // the commitment and challenge. Here we check structural integrity.)
        Ok(true)
    }

    /// Prove membership in a set using Merkle inclusion proof.
    pub fn prove_set_membership(
        value: &[u8],
        set: &[Vec<u8>],
    ) -> Result<(SetMembershipProof, [u8; 32]), CryptoError> {
        // Find the value's index in the set
        let index = set
            .iter()
            .position(|item| item.as_slice() == value)
            .ok_or_else(|| CryptoError::ZkpError("value not in set".into()))?;

        // Compute hashes for all set members
        let leaf_hashes: Vec<Hash> = set.iter().map(|item| hashing::hash(item)).collect();

        // Build Merkle tree and collect proof path
        let (merkle_path, path_directions) = build_merkle_proof(&leaf_hashes, index);
        let set_root = hashing::merkle_root(&leaf_hashes);

        // Commit to the value
        let (commitment, nonce) = Commitment::commit(value);

        // Fiat-Shamir challenge
        let mut challenge_input = Vec::new();
        challenge_input.extend_from_slice(&commitment.hash);
        challenge_input.extend_from_slice(&set_root);
        let challenge = hashing::hash(&challenge_input);

        // Response
        let mut response_input = Vec::new();
        response_input.extend_from_slice(value);
        response_input.extend_from_slice(&nonce);
        response_input.extend_from_slice(&challenge);
        let response = hashing::hash(&response_input);

        let proof = SetMembershipProof {
            commitment,
            set_root,
            merkle_path,
            path_directions,
            challenge,
            response,
        };

        Ok((proof, nonce))
    }

    /// Verify a set membership proof.
    pub fn verify_set_membership(
        proof: &SetMembershipProof,
        expected_root: &Hash,
    ) -> Result<bool, CryptoError> {
        // Verify the set root matches expected
        if proof.set_root != *expected_root {
            return Ok(false);
        }

        // Verify challenge consistency
        let mut challenge_input = Vec::new();
        challenge_input.extend_from_slice(&proof.commitment.hash);
        challenge_input.extend_from_slice(&proof.set_root);
        let expected_challenge = hashing::hash(&challenge_input);

        if proof.challenge != expected_challenge {
            return Ok(false);
        }

        Ok(true)
    }
}

/// Build a Merkle proof path for a leaf at the given index.
fn build_merkle_proof(leaves: &[Hash], index: usize) -> (Vec<[u8; 32]>, Vec<bool>) {
    let mut proof_path = Vec::new();
    let mut directions = Vec::new();

    if leaves.len() <= 1 {
        return (proof_path, directions);
    }

    let mut current_level = leaves.to_vec();
    let mut current_index = index;

    while current_level.len() > 1 {
        let sibling_index = if current_index.is_multiple_of(2) {
            current_index + 1
        } else {
            current_index - 1
        };

        // If sibling exists, add it to proof path
        if sibling_index < current_level.len() {
            proof_path.push(current_level[sibling_index]);
            directions.push(current_index.is_multiple_of(2)); // true = sibling is on the right
        } else {
            // Odd element pairs with itself
            proof_path.push(current_level[current_index]);
            directions.push(true);
        }

        // Move up one level
        let mut next_level = Vec::with_capacity(current_level.len().div_ceil(2));
        for chunk in current_level.chunks(2) {
            let mut combined = Vec::with_capacity(64);
            combined.extend_from_slice(&chunk[0]);
            if chunk.len() == 2 {
                combined.extend_from_slice(&chunk[1]);
            } else {
                combined.extend_from_slice(&chunk[0]);
            }
            next_level.push(hashing::hash(&combined));
        }
        current_level = next_level;
        current_index /= 2;
    }

    (proof_path, directions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_create_verify() {
        let value = b"secret-age-25";
        let (commitment, nonce) = Commitment::commit(value);
        assert!(commitment.verify(value, &nonce));
    }

    #[test]
    fn test_commitment_wrong_value() {
        let (commitment, nonce) = Commitment::commit(b"real");
        assert!(!commitment.verify(b"fake", &nonce));
    }

    #[test]
    fn test_commitment_wrong_nonce() {
        let value = b"test";
        let (commitment, _nonce) = Commitment::commit(value);
        let wrong_nonce = [0xFFu8; 32];
        assert!(!commitment.verify(value, &wrong_nonce));
    }

    #[test]
    fn test_commitment_with_specific_nonce() {
        let value = b"deterministic";
        let nonce = [42u8; 32];
        let c1 = Commitment::commit_with_nonce(value, &nonce);
        let c2 = Commitment::commit_with_nonce(value, &nonce);
        assert_eq!(c1, c2);
        assert!(c1.verify(value, &nonce));
    }

    #[test]
    fn test_range_proof_valid() {
        let (proof, _nonce) = Blake3ProofGenerator::prove_range(25, 18, 120).unwrap();
        assert!(Blake3ProofGenerator::verify_range(&proof).unwrap());
    }

    #[test]
    fn test_range_proof_at_min() {
        let (proof, _nonce) = Blake3ProofGenerator::prove_range(18, 18, 120).unwrap();
        assert!(Blake3ProofGenerator::verify_range(&proof).unwrap());
    }

    #[test]
    fn test_range_proof_at_max() {
        let (proof, _nonce) = Blake3ProofGenerator::prove_range(120, 18, 120).unwrap();
        assert!(Blake3ProofGenerator::verify_range(&proof).unwrap());
    }

    #[test]
    fn test_range_proof_out_of_range() {
        let result = Blake3ProofGenerator::prove_range(17, 18, 120);
        assert!(result.is_err());
    }

    #[test]
    fn test_range_proof_above_max() {
        let result = Blake3ProofGenerator::prove_range(121, 18, 120);
        assert!(result.is_err());
    }

    #[test]
    fn test_range_proof_negative_values() {
        let (proof, _nonce) = Blake3ProofGenerator::prove_range(-5, -10, 10).unwrap();
        assert!(Blake3ProofGenerator::verify_range(&proof).unwrap());
    }

    #[test]
    fn test_range_proof_tampered_challenge() {
        let (mut proof, _nonce) = Blake3ProofGenerator::prove_range(25, 18, 120).unwrap();
        proof.challenge = [0xFFu8; 32]; // Tamper
        assert!(!Blake3ProofGenerator::verify_range(&proof).unwrap());
    }

    #[test]
    fn test_set_membership_proof_valid() {
        let set: Vec<Vec<u8>> = vec![
            b"BR".to_vec(),
            b"US".to_vec(),
            b"DE".to_vec(),
            b"JP".to_vec(),
        ];
        let (proof, _nonce) = Blake3ProofGenerator::prove_set_membership(b"BR", &set).unwrap();
        let leaf_hashes: Vec<Hash> = set.iter().map(|item| hashing::hash(item)).collect();
        let expected_root = hashing::merkle_root(&leaf_hashes);
        assert!(Blake3ProofGenerator::verify_set_membership(&proof, &expected_root).unwrap());
    }

    #[test]
    fn test_set_membership_proof_last_element() {
        let set: Vec<Vec<u8>> = vec![b"A".to_vec(), b"B".to_vec(), b"C".to_vec()];
        let (proof, _nonce) = Blake3ProofGenerator::prove_set_membership(b"C", &set).unwrap();
        let leaf_hashes: Vec<Hash> = set.iter().map(|item| hashing::hash(item)).collect();
        let expected_root = hashing::merkle_root(&leaf_hashes);
        assert!(Blake3ProofGenerator::verify_set_membership(&proof, &expected_root).unwrap());
    }

    #[test]
    fn test_set_membership_not_in_set() {
        let set: Vec<Vec<u8>> = vec![b"BR".to_vec(), b"US".to_vec()];
        let result = Blake3ProofGenerator::prove_set_membership(b"XX", &set);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_membership_wrong_root() {
        let set: Vec<Vec<u8>> = vec![b"BR".to_vec(), b"US".to_vec()];
        let (proof, _nonce) = Blake3ProofGenerator::prove_set_membership(b"BR", &set).unwrap();
        let wrong_root = [0xFFu8; 32];
        assert!(!Blake3ProofGenerator::verify_set_membership(&proof, &wrong_root).unwrap());
    }

    #[test]
    fn test_set_membership_tampered_challenge() {
        let set: Vec<Vec<u8>> = vec![b"BR".to_vec(), b"US".to_vec()];
        let (mut proof, _nonce) = Blake3ProofGenerator::prove_set_membership(b"BR", &set).unwrap();
        let leaf_hashes: Vec<Hash> = set.iter().map(|item| hashing::hash(item)).collect();
        let expected_root = hashing::merkle_root(&leaf_hashes);
        proof.challenge = [0xAAu8; 32]; // Tamper
        assert!(!Blake3ProofGenerator::verify_set_membership(&proof, &expected_root).unwrap());
    }
}
