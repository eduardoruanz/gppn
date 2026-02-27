//! Veritas Core — Fundamental types, errors, and constants for the
//! Veritas decentralized identity protocol.

pub mod config;
pub mod credential_state;
pub mod error;
pub mod types;

/// Generated protobuf types from proto/veritas/v1/*.proto.
pub mod proto {
    pub mod veritas {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/veritas.v1.rs"));
        }
    }
}

pub use config::NodeConfig;
pub use credential_state::{CredentialEvent, CredentialState, CredentialStateMachine};
pub use error::CoreError;
pub use types::{Claim, ClaimValue, CredentialType, Did, ProofType, SchemaId};
