use crate::state_machine::PaymentState;

/// Core protocol errors.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("invalid state transition from {from} to {to}")]
    InvalidStateTransition {
        from: PaymentState,
        to: PaymentState,
    },

    #[error("payment message validation failed: {0}")]
    ValidationError(String),

    #[error("serialization error: {0}")]
    SerializationError(#[from] prost::EncodeError),

    #[error("deserialization error: {0}")]
    DeserializationError(#[from] prost::DecodeError),

    #[error("payment expired: TTL exceeded")]
    PaymentExpired,

    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("invalid amount: {0}")]
    InvalidAmount(String),

    #[error("invalid DID format: {0}")]
    InvalidDid(String),

    #[error("signature error: {0}")]
    SignatureError(String),
}
