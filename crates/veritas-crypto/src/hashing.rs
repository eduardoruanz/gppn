/// BLAKE3 hash (32 bytes).
pub type Hash = [u8; 32];

/// Hash arbitrary data using BLAKE3.
pub fn hash(data: &[u8]) -> Hash {
    *blake3::hash(data).as_bytes()
}

/// Create a BLAKE3 commitment: H(value || nonce).
/// Used for zero-knowledge proof commitments.
pub fn create_commitment(value: &[u8], nonce: &[u8; 32]) -> Hash {
    let mut input = Vec::with_capacity(value.len() + 32);
    input.extend_from_slice(value);
    input.extend_from_slice(nonce);
    hash(&input)
}

/// Verify a BLAKE3 commitment by recomputing H(value || nonce).
pub fn verify_commitment(value: &[u8], nonce: &[u8; 32], commitment: &Hash) -> bool {
    let computed = create_commitment(value, nonce);
    computed == *commitment
}

/// Compute the Merkle root of a list of hashes.
/// Returns the single root hash. If the input is empty, returns a zero hash.
/// If the input has one element, returns that element.
pub fn merkle_root(hashes: &[Hash]) -> Hash {
    if hashes.is_empty() {
        return [0u8; 32];
    }
    if hashes.len() == 1 {
        return hashes[0];
    }

    let mut current_level: Vec<Hash> = hashes.to_vec();

    while current_level.len() > 1 {
        let mut next_level = Vec::with_capacity(current_level.len().div_ceil(2));
        for chunk in current_level.chunks(2) {
            if chunk.len() == 2 {
                let mut combined = Vec::with_capacity(64);
                combined.extend_from_slice(&chunk[0]);
                combined.extend_from_slice(&chunk[1]);
                next_level.push(hash(&combined));
            } else {
                // Odd element: hash it with itself
                let mut combined = Vec::with_capacity(64);
                combined.extend_from_slice(&chunk[0]);
                combined.extend_from_slice(&chunk[0]);
                next_level.push(hash(&combined));
            }
        }
        current_level = next_level;
    }

    current_level[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_deterministic() {
        let data = b"Veritas protocol test data";
        let h1 = hash(data);
        let h2 = hash(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_different_inputs() {
        let h1 = hash(b"data A");
        let h2 = hash(b"data B");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_empty() {
        let h = hash(b"");
        assert_eq!(h.len(), 32);
        assert_ne!(h, [0u8; 32]);
    }

    #[test]
    fn test_hash_length() {
        let h = hash(b"test");
        assert_eq!(h.len(), 32);
    }

    #[test]
    fn test_commitment_roundtrip() {
        let value = b"secret-value-42";
        let nonce = [0xABu8; 32];
        let commitment = create_commitment(value, &nonce);
        assert!(verify_commitment(value, &nonce, &commitment));
    }

    #[test]
    fn test_commitment_wrong_value() {
        let nonce = [0xCDu8; 32];
        let commitment = create_commitment(b"real-value", &nonce);
        assert!(!verify_commitment(b"fake-value", &nonce, &commitment));
    }

    #[test]
    fn test_commitment_wrong_nonce() {
        let value = b"test-value";
        let nonce1 = [0x01u8; 32];
        let nonce2 = [0x02u8; 32];
        let commitment = create_commitment(value, &nonce1);
        assert!(!verify_commitment(value, &nonce2, &commitment));
    }

    #[test]
    fn test_commitment_deterministic() {
        let value = b"deterministic-value";
        let nonce = [0x99u8; 32];
        let c1 = create_commitment(value, &nonce);
        let c2 = create_commitment(value, &nonce);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_commitment_different_nonces_differ() {
        let value = b"same-value";
        let nonce1 = [0x01u8; 32];
        let nonce2 = [0x02u8; 32];
        let c1 = create_commitment(value, &nonce1);
        let c2 = create_commitment(value, &nonce2);
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_merkle_root_empty() {
        let root = merkle_root(&[]);
        assert_eq!(root, [0u8; 32]);
    }

    #[test]
    fn test_merkle_root_single() {
        let h = hash(b"single leaf");
        let root = merkle_root(&[h]);
        assert_eq!(root, h);
    }

    #[test]
    fn test_merkle_root_two() {
        let h1 = hash(b"leaf 1");
        let h2 = hash(b"leaf 2");
        let root = merkle_root(&[h1, h2]);
        let mut combined = Vec::new();
        combined.extend_from_slice(&h1);
        combined.extend_from_slice(&h2);
        assert_eq!(root, hash(&combined));
    }

    #[test]
    fn test_merkle_root_three() {
        let h1 = hash(b"leaf 1");
        let h2 = hash(b"leaf 2");
        let h3 = hash(b"leaf 3");
        let root = merkle_root(&[h1, h2, h3]);
        assert_ne!(root, [0u8; 32]);
        assert_eq!(root.len(), 32);
    }

    #[test]
    fn test_merkle_root_deterministic() {
        let leaves: Vec<Hash> = (0..8).map(|i| hash(&[i])).collect();
        let root1 = merkle_root(&leaves);
        let root2 = merkle_root(&leaves);
        assert_eq!(root1, root2);
    }

    #[test]
    fn test_merkle_root_order_matters() {
        let h1 = hash(b"A");
        let h2 = hash(b"B");
        let root1 = merkle_root(&[h1, h2]);
        let root2 = merkle_root(&[h2, h1]);
        assert_ne!(root1, root2);
    }
}
