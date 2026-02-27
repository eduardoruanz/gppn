use crate::credential_state::CredentialState;

/// Core protocol errors.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("invalid state transition from {from} to {to}")]
    InvalidStateTransition {
        from: CredentialState,
        to: CredentialState,
    },

    #[error("validation error: {0}")]
    ValidationError(String),

    #[error("serialization error: {0}")]
    SerializationError(#[from] prost::EncodeError),

    #[error("deserialization error: {0}")]
    DeserializationError(#[from] prost::DecodeError),

    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("invalid DID format: {0}")]
    InvalidDid(String),

    #[error("signature error: {0}")]
    SignatureError(String),

    #[error("credential error: {0}")]
    CredentialError(String),

    #[error("proof error: {0}")]
    ProofError(String),

    #[error("schema error: {0}")]
    SchemaError(String),

    #[error("expired: {0}")]
    Expired(String),
}
