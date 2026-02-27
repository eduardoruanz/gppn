/// Credential system errors.
#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error("schema not found: {0}")]
    SchemaNotFound(String),

    #[error("invalid schema: {0}")]
    InvalidSchema(String),

    #[error("credential not found: {0}")]
    CredentialNotFound(String),

    #[error("issuance failed: {0}")]
    IssuanceFailed(String),

    #[error("verification failed: {0}")]
    VerificationFailed(String),

    #[error("untrusted issuer: {0}")]
    UntrustedIssuer(String),

    #[error("expired credential")]
    Expired,

    #[error("presentation error: {0}")]
    PresentationError(String),

    #[error("crypto error: {0}")]
    Crypto(#[from] veritas_crypto::CryptoError),

    #[error("identity error: {0}")]
    Identity(#[from] veritas_identity::IdentityError),

    #[error("serialization error: {0}")]
    Serialization(String),
}
