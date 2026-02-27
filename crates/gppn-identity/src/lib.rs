//! GPPN Identity Layer
//!
//! Provides decentralised identity primitives for the GPPN protocol:
//! - DID creation and resolution
//! - DID Documents (W3C-compatible)
//! - Verifiable Credentials with Ed25519 proofs
//! - TrustGraph with EigenTrust-like score computation
//! - Composite TrustScore for network participants

pub mod error;
pub mod document;
pub mod did;
pub mod credentials;
pub mod trust_graph;
pub mod trust_score;

pub use error::IdentityError;
pub use document::DidDocument;
pub use did::DidManager;
pub use credentials::VerifiableCredential;
pub use trust_graph::{TrustGraph, TrustEdge};
pub use trust_score::TrustScore;
