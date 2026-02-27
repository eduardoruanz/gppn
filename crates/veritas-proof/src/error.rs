/// Zero-knowledge proof errors.
#[derive(Debug, thiserror::Error)]
pub enum ProofError {
    #[error("proof generation failed: {0}")]
    GenerationFailed(String),

    #[error("proof verification failed: {0}")]
    VerificationFailed(String),

    #[error("invalid proof data: {0}")]
    InvalidProofData(String),

    #[error("value out of range: {0}")]
    OutOfRange(String),

    #[error("value not in set")]
    NotInSet,

    #[error("crypto error: {0}")]
    Crypto(#[from] veritas_crypto::CryptoError),

    #[error("serialization error: {0}")]
    Serialization(String),
}
