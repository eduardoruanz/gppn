use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

use crate::error::CryptoError;
use crate::keys::KeyPair;

/// Encrypted payload containing ciphertext, nonce, and ephemeral public key.
#[derive(Debug, Clone)]
pub struct EncryptedPayload {
    /// Ephemeral X25519 public key used for key exchange.
    pub ephemeral_pubkey: [u8; 32],
    /// 12-byte nonce for ChaCha20-Poly1305.
    pub nonce: [u8; 12],
    /// Encrypted data (ciphertext + 16-byte Poly1305 tag).
    pub ciphertext: Vec<u8>,
}

impl EncryptedPayload {
    /// Serialize to bytes: ephemeral_pubkey (32) + nonce (12) + ciphertext (variable).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(32 + 12 + self.ciphertext.len());
        out.extend_from_slice(&self.ephemeral_pubkey);
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&self.ciphertext);
        out
    }

    /// Deserialize from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() < 44 {
            return Err(CryptoError::DecryptionError(
                "payload too short".into(),
            ));
        }
        let mut ephemeral_pubkey = [0u8; 32];
        ephemeral_pubkey.copy_from_slice(&bytes[..32]);
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&bytes[32..44]);
        let ciphertext = bytes[44..].to_vec();
        Ok(Self {
            ephemeral_pubkey,
            nonce,
            ciphertext,
        })
    }
}

/// Derive an X25519 static secret from an Ed25519 key pair's seed.
/// Uses BLAKE3 key derivation for domain separation.
fn derive_x25519_secret(keypair: &KeyPair) -> StaticSecret {
    let seed = keypair.secret_bytes();
    let derived = blake3::derive_key("GPPN-x25519-key-derivation-v1", &seed);
    StaticSecret::from(derived)
}

/// Derive the X25519 public key corresponding to a KeyPair.
/// This is the public key that should be used for encryption.
pub fn x25519_public_key(keypair: &KeyPair) -> [u8; 32] {
    let secret = derive_x25519_secret(keypair);
    let pubkey = X25519PublicKey::from(&secret);
    pubkey.to_bytes()
}

/// Encrypt plaintext for a recipient using X25519 key exchange + ChaCha20-Poly1305.
///
/// The sender creates an ephemeral X25519 key pair, performs Diffie-Hellman
/// with the recipient's X25519 public key (derived from their Ed25519 key),
/// and uses the shared secret as the ChaCha20-Poly1305 key.
///
/// `recipient_x25519_pubkey` should be obtained via `x25519_public_key()`.
pub fn encrypt(
    plaintext: &[u8],
    recipient_x25519_pubkey: &[u8; 32],
) -> Result<EncryptedPayload, CryptoError> {
    // Generate ephemeral X25519 key pair
    let mut ephemeral_secret_bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut ephemeral_secret_bytes);
    let ephemeral_secret = StaticSecret::from(ephemeral_secret_bytes);
    let ephemeral_pubkey = X25519PublicKey::from(&ephemeral_secret);

    // Perform Diffie-Hellman key exchange
    let recipient_pubkey = X25519PublicKey::from(*recipient_x25519_pubkey);
    let shared_secret = ephemeral_secret.diffie_hellman(&recipient_pubkey);

    // Derive symmetric key from shared secret using BLAKE3
    let symmetric_key = blake3::derive_key("GPPN-encryption-v1", shared_secret.as_bytes());

    // Generate random nonce
    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt with ChaCha20-Poly1305
    let cipher = ChaCha20Poly1305::new_from_slice(&symmetric_key)
        .map_err(|e| CryptoError::EncryptionError(format!("cipher init failed: {}", e)))?;
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionError(format!("encryption failed: {}", e)))?;

    Ok(EncryptedPayload {
        ephemeral_pubkey: ephemeral_pubkey.to_bytes(),
        nonce: nonce_bytes,
        ciphertext,
    })
}

/// Decrypt an encrypted payload using the recipient's key pair.
pub fn decrypt(
    payload: &EncryptedPayload,
    recipient_keypair: &KeyPair,
) -> Result<Vec<u8>, CryptoError> {
    let ephemeral_pubkey = X25519PublicKey::from(payload.ephemeral_pubkey);
    let recipient_x25519 = derive_x25519_secret(recipient_keypair);
    let shared_secret = recipient_x25519.diffie_hellman(&ephemeral_pubkey);
    let symmetric_key = blake3::derive_key("GPPN-encryption-v1", shared_secret.as_bytes());

    let nonce = Nonce::from_slice(&payload.nonce);
    let cipher = ChaCha20Poly1305::new_from_slice(&symmetric_key)
        .map_err(|e| CryptoError::DecryptionError(format!("cipher init failed: {}", e)))?;
    let plaintext = cipher
        .decrypt(nonce, payload.ciphertext.as_slice())
        .map_err(|e| CryptoError::DecryptionError(format!("decryption failed: {}", e)))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::KeyPair;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let recipient = KeyPair::generate();
        let recipient_pubkey = x25519_public_key(&recipient);
        let plaintext = b"Hello, GPPN payment metadata!";

        let encrypted = encrypt(plaintext, &recipient_pubkey).unwrap();
        let decrypted = decrypt(&encrypted, &recipient).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_empty_message() {
        let recipient = KeyPair::generate();
        let recipient_pubkey = x25519_public_key(&recipient);

        let encrypted = encrypt(b"", &recipient_pubkey).unwrap();
        let decrypted = decrypt(&encrypted, &recipient).unwrap();
        assert_eq!(decrypted, b"");
    }

    #[test]
    fn test_encrypt_large_message() {
        let recipient = KeyPair::generate();
        let recipient_pubkey = x25519_public_key(&recipient);
        let plaintext = vec![0xFFu8; 100_000];

        let encrypted = encrypt(&plaintext, &recipient_pubkey).unwrap();
        let decrypted = decrypt(&encrypted, &recipient).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let recipient = KeyPair::generate();
        let wrong_recipient = KeyPair::generate();
        let recipient_pubkey = x25519_public_key(&recipient);

        let encrypted = encrypt(b"secret data", &recipient_pubkey).unwrap();
        let result = decrypt(&encrypted, &wrong_recipient);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let recipient = KeyPair::generate();
        let recipient_pubkey = x25519_public_key(&recipient);

        let mut encrypted = encrypt(b"secret data", &recipient_pubkey).unwrap();
        if let Some(byte) = encrypted.ciphertext.first_mut() {
            *byte ^= 0xFF;
        }
        let result = decrypt(&encrypted, &recipient);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypted_payload_serialization() {
        let recipient = KeyPair::generate();
        let recipient_pubkey = x25519_public_key(&recipient);
        let plaintext = b"test payload serialization";

        let encrypted = encrypt(plaintext, &recipient_pubkey).unwrap();
        let bytes = encrypted.to_bytes();
        let deserialized = EncryptedPayload::from_bytes(&bytes).unwrap();

        assert_eq!(encrypted.ephemeral_pubkey, deserialized.ephemeral_pubkey);
        assert_eq!(encrypted.nonce, deserialized.nonce);
        assert_eq!(encrypted.ciphertext, deserialized.ciphertext);

        let decrypted = decrypt(&deserialized, &recipient).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_payload_from_bytes_too_short() {
        let result = EncryptedPayload::from_bytes(&[0u8; 10]);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_encryptions_produce_different_ciphertexts() {
        let recipient = KeyPair::generate();
        let recipient_pubkey = x25519_public_key(&recipient);
        let plaintext = b"same message";

        let enc1 = encrypt(plaintext, &recipient_pubkey).unwrap();
        let enc2 = encrypt(plaintext, &recipient_pubkey).unwrap();
        assert_ne!(enc1.ciphertext, enc2.ciphertext);
    }

    #[test]
    fn test_x25519_public_key_deterministic() {
        let kp = KeyPair::from_seed(&[42u8; 32]);
        let pk1 = x25519_public_key(&kp);
        let pk2 = x25519_public_key(&kp);
        assert_eq!(pk1, pk2);
    }
}
