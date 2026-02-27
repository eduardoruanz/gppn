use uuid::Uuid;

/// Settlement-layer errors.
#[derive(Debug, thiserror::Error)]
pub enum SettlementError {
    #[error("settlement not found: {0}")]
    NotFound(Uuid),

    #[error("adapter not registered: {0}")]
    AdapterNotFound(String),

    #[error("invalid settlement state transition: {0}")]
    InvalidStateTransition(String),

    #[error("settlement already exists: {0}")]
    AlreadyExists(Uuid),

    #[error("settlement expired: {0}")]
    Expired(Uuid),

    #[error("HTLC error: {0}")]
    HtlcError(String),

    #[error("preimage mismatch for HTLC {0}")]
    PreimageMismatch(Uuid),

    #[error("HTLC not expired yet: {0}")]
    HtlcNotExpired(Uuid),

    #[error("insufficient balance: available {available}, required {required}")]
    InsufficientBalance { available: u128, required: u128 },

    #[error("unsupported currency: {0}")]
    UnsupportedCurrency(String),

    #[error("internal error: {0}")]
    Internal(String),
}
