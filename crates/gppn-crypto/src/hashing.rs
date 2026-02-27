/// BLAKE3 hash (32 bytes).
pub type Hash = [u8; 32];

/// Hash arbitrary data using BLAKE3.
pub fn hash(data: &[u8]) -> Hash {
    *blake3::hash(data).as_bytes()
}

/// Hash a payment message's signing payload using BLAKE3.
pub fn hash_payment_message(pm: &gppn_core::PaymentMessage) -> Hash {
    let payload = pm.signing_payload();
    hash(&payload)
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
        let mut next_level = Vec::with_capacity((current_level.len() + 1) / 2);
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
    use gppn_core::types::{Amount, Currency, FiatCurrency, Did};

    #[test]
    fn test_hash_deterministic() {
        let data = b"GPPN protocol test data";
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
        // BLAKE3 hash of empty string is well-known
        assert_ne!(h, [0u8; 32]);
    }

    #[test]
    fn test_hash_length() {
        let h = hash(b"test");
        assert_eq!(h.len(), 32);
    }

    #[test]
    fn test_hash_payment_message() {
        let pm = gppn_core::PaymentMessage::builder()
            .sender(Did::from_parts("key", "alice"))
            .receiver(Did::from_parts("key", "bob"))
            .amount(Amount::new(100, Currency::Fiat(FiatCurrency::USD)))
            .ttl(60)
            .build()
            .unwrap();

        let h1 = hash_payment_message(&pm);
        let h2 = hash_payment_message(&pm);
        assert_eq!(h1, h2);
        assert_ne!(h1, [0u8; 32]);
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
        // Root should be hash(h1 || h2)
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
