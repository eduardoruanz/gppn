//! Veritas Credentials — Issuer, holder wallet, verifier, and schema registry.

pub mod error;
pub mod holder;
pub mod issuer;
pub mod schema;
pub mod verifier;

pub use error::CredentialError;
pub use holder::CredentialWallet;
pub use issuer::CredentialIssuer;
pub use schema::{ClaimDefinition, CredentialSchema, SchemaRegistry};
pub use verifier::{CredentialVerifier, VerificationCheck, VerificationResult};
