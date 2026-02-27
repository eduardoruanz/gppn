use ed25519_dalek::Signer;
use ed25519_dalek::Verifier;

use crate::error::CryptoError;
use crate::keys::{KeyPair, PublicKey};

/// Ed25519 signature (64 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    inner: ed25519_dalek::Signature,
}

impl Signature {
    /// Get the raw bytes (64 bytes).
    pub fn to_bytes(&self) -> [u8; 64] {
        self.inner.to_bytes()
    }

    /// Create from raw bytes (64 bytes).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != 64 {
            return Err(CryptoError::InvalidInput(format!(
                "signature must be 64 bytes, got {}",
                bytes.len()
            )));
        }
        let bytes_arr: [u8; 64] = bytes
            .try_into()
            .map_err(|_| CryptoError::InvalidInput("invalid signature length".into()))?;
        let inner = ed25519_dalek::Signature::from_bytes(&bytes_arr);
        Ok(Self { inner })
    }

    /// Encode as hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }
}

/// Sign a message using Ed25519.
pub fn sign(message: &[u8], keypair: &KeyPair) -> Signature {
    let sig = keypair.signing_key().sign(message);
    Signature { inner: sig }
}

/// Verify an Ed25519 signature.
pub fn verify(
    message: &[u8],
    signature: &Signature,
    pubkey: &PublicKey,
) -> Result<(), CryptoError> {
    pubkey
        .verifying_key()
        .verify(message, &signature.inner)
        .map_err(|_| CryptoError::SignatureVerificationFailed)
}

/// Sign a credential payload (serialized bytes) using Ed25519.
pub fn sign_credential(payload: &[u8], keypair: &KeyPair) -> Signature {
    sign(payload, keypair)
}

/// Verify a credential payload's Ed25519 signature.
pub fn verify_credential(
    payload: &[u8],
    signature: &Signature,
    pubkey: &PublicKey,
) -> Result<(), CryptoError> {
    verify(payload, signature, pubkey)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::KeyPair;

    #[test]
    fn test_sign_verify_roundtrip() {
        let kp = KeyPair::generate();
        let message = b"hello Veritas protocol";
        let sig = sign(message, &kp);
        assert!(verify(message, &sig, &kp.public_key()).is_ok());
    }

    #[test]
    fn test_verify_wrong_message_fails() {
        let kp = KeyPair::generate();
        let sig = sign(b"correct message", &kp);
        let result = verify(b"wrong message", &sig, &kp.public_key());
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_wrong_key_fails() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();
        let sig = sign(b"test message", &kp1);
        let result = verify(b"test message", &sig, &kp2.public_key());
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_empty_message() {
        let kp = KeyPair::generate();
        let sig = sign(b"", &kp);
        assert!(verify(b"", &sig, &kp.public_key()).is_ok());
    }

    #[test]
    fn test_sign_large_message() {
        let kp = KeyPair::generate();
        let message = vec![0xABu8; 10_000];
        let sig = sign(&message, &kp);
        assert!(verify(&message, &sig, &kp.public_key()).is_ok());
    }

    #[test]
    fn test_signature_bytes_roundtrip() {
        let kp = KeyPair::generate();
        let sig = sign(b"test", &kp);
        let bytes = sig.to_bytes();
        assert_eq!(bytes.len(), 64);
        let sig2 = Signature::from_bytes(&bytes).unwrap();
        assert_eq!(sig, sig2);
    }

    #[test]
    fn test_signature_hex() {
        let kp = KeyPair::generate();
        let sig = sign(b"test", &kp);
        let hex_str = sig.to_hex();
        assert_eq!(hex_str.len(), 128); // 64 bytes = 128 hex chars
    }

    #[test]
    fn test_signature_from_invalid_bytes() {
        let result = Signature::from_bytes(&[0u8; 32]);
        assert!(result.is_err());
    }

    #[test]
    fn test_deterministic_signatures() {
        let kp = KeyPair::from_seed(&[99u8; 32]);
        let sig1 = sign(b"deterministic test", &kp);
        let sig2 = sign(b"deterministic test", &kp);
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_sign_verify_credential() {
        let kp = KeyPair::generate();
        let payload = b"credential-payload-bytes";
        let sig = sign_credential(payload, &kp);
        assert!(verify_credential(payload, &sig, &kp.public_key()).is_ok());
    }

    #[test]
    fn test_verify_credential_wrong_key() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();
        let payload = b"credential-payload-bytes";
        let sig = sign_credential(payload, &kp1);
        assert!(verify_credential(payload, &sig, &kp2.public_key()).is_err());
    }

    #[test]
    fn test_verify_credential_tampered_payload() {
        let kp = KeyPair::generate();
        let payload = b"original-payload";
        let sig = sign_credential(payload, &kp);
        assert!(verify_credential(b"tampered-payload", &sig, &kp.public_key()).is_err());
    }
}
