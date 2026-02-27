/// Identity-layer errors.
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("DID not found: {0}")]
    DidNotFound(String),

    #[error("invalid DID format: {0}")]
    InvalidDid(String),

    #[error("duplicate DID: {0}")]
    DuplicateDid(String),

    #[error("credential issuance failed: {0}")]
    CredentialIssuance(String),

    #[error("credential verification failed: {0}")]
    CredentialVerification(String),

    #[error("trust graph error: {0}")]
    TrustGraph(String),

    #[error("invalid trust weight: {0} (must be between -1.0 and 1.0)")]
    InvalidTrustWeight(f64),

    #[error("crypto error: {0}")]
    Crypto(#[from] gppn_crypto::CryptoError),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("internal error: {0}")]
    Internal(String),
}
