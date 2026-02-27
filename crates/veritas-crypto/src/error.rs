/// Cryptographic operation errors.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("key generation failed: {0}")]
    KeyGenerationError(String),

    #[error("invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("signature verification failed")]
    SignatureVerificationFailed,

    #[error("signing failed: {0}")]
    SigningError(String),

    #[error("encryption failed: {0}")]
    EncryptionError(String),

    #[error("decryption failed: {0}")]
    DecryptionError(String),

    #[error("key derivation failed: {0}")]
    KeyDerivationError(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("zero-knowledge proof error: {0}")]
    ZkpError(String),
}
