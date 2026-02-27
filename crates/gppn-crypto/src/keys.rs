use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use zeroize::Zeroize;

use crate::error::CryptoError;

/// Ed25519 key pair for signing operations.
/// Private key material is zeroized on drop.
pub struct KeyPair {
    signing_key: SigningKey,
}

impl KeyPair {
    /// Generate a new random key pair using OS-provided entropy.
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    /// Create a key pair from a 32-byte seed.
    /// The seed is used directly as the Ed25519 private key.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        Self { signing_key }
    }

    /// Create a key pair from raw bytes (32 bytes).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 32,
                actual: bytes.len(),
            });
        }
        let mut seed = [0u8; 32];
        seed.copy_from_slice(bytes);
        let kp = Self::from_seed(&seed);
        seed.zeroize();
        Ok(kp)
    }

    /// Get the public key.
    pub fn public_key(&self) -> PublicKey {
        PublicKey {
            verifying_key: self.signing_key.verifying_key(),
        }
    }

    /// Get the raw private key bytes (32 bytes).
    /// Use with caution — prefer using sign() methods instead.
    pub fn secret_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Access the underlying ed25519-dalek SigningKey for signing operations.
    pub(crate) fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }
}

impl Drop for KeyPair {
    fn drop(&mut self) {
        // SigningKey internally zeroizes on drop via ed25519-dalek
        // This is a safety net
        let _ = &self.signing_key;
    }
}

/// Ed25519 public key for verification operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey {
    verifying_key: VerifyingKey,
}

impl PublicKey {
    /// Create from raw bytes (32 bytes).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 32,
                actual: bytes.len(),
            });
        }
        let bytes_arr: [u8; 32] = bytes.try_into().map_err(|_| CryptoError::InvalidKeyLength {
            expected: 32,
            actual: bytes.len(),
        })?;
        let verifying_key = VerifyingKey::from_bytes(&bytes_arr)
            .map_err(|e| CryptoError::InvalidInput(format!("invalid public key: {}", e)))?;
        Ok(Self { verifying_key })
    }

    /// Get the raw bytes (32 bytes).
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.verifying_key.as_bytes()
    }

    /// Encode as hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.as_bytes())
    }

    /// Decode from hex string.
    pub fn from_hex(hex_str: &str) -> Result<Self, CryptoError> {
        let bytes = hex::decode(hex_str)
            .map_err(|e| CryptoError::InvalidInput(format!("invalid hex: {}", e)))?;
        Self::from_bytes(&bytes)
    }

    /// Encode as base58.
    pub fn to_bs58(&self) -> String {
        bs58::encode(self.as_bytes()).into_string()
    }

    /// Decode from base58.
    pub fn from_bs58(bs58_str: &str) -> Result<Self, CryptoError> {
        let bytes = bs58::decode(bs58_str)
            .into_vec()
            .map_err(|e| CryptoError::InvalidInput(format!("invalid base58: {}", e)))?;
        Self::from_bytes(&bytes)
    }

    /// Access the underlying verifying key.
    pub(crate) fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let kp = KeyPair::generate();
        let pk = kp.public_key();
        assert_eq!(pk.as_bytes().len(), 32);
    }

    #[test]
    fn test_from_seed_deterministic() {
        let seed = [42u8; 32];
        let kp1 = KeyPair::from_seed(&seed);
        let kp2 = KeyPair::from_seed(&seed);
        assert_eq!(kp1.public_key(), kp2.public_key());
    }

    #[test]
    fn test_from_bytes() {
        let kp = KeyPair::generate();
        let bytes = kp.secret_bytes();
        let kp2 = KeyPair::from_bytes(&bytes).unwrap();
        assert_eq!(kp.public_key(), kp2.public_key());
    }

    #[test]
    fn test_from_bytes_invalid_length() {
        let result = KeyPair::from_bytes(&[0u8; 16]);
        assert!(result.is_err());
    }

    #[test]
    fn test_public_key_hex_roundtrip() {
        let kp = KeyPair::generate();
        let pk = kp.public_key();
        let hex_str = pk.to_hex();
        let pk2 = PublicKey::from_hex(&hex_str).unwrap();
        assert_eq!(pk, pk2);
    }

    #[test]
    fn test_public_key_bs58_roundtrip() {
        let kp = KeyPair::generate();
        let pk = kp.public_key();
        let bs58_str = pk.to_bs58();
        let pk2 = PublicKey::from_bs58(&bs58_str).unwrap();
        assert_eq!(pk, pk2);
    }

    #[test]
    fn test_public_key_from_bytes_invalid() {
        let result = PublicKey::from_bytes(&[0u8; 31]);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_seeds_different_keys() {
        let kp1 = KeyPair::from_seed(&[1u8; 32]);
        let kp2 = KeyPair::from_seed(&[2u8; 32]);
        assert_ne!(kp1.public_key(), kp2.public_key());
    }
}
