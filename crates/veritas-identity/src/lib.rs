//! Veritas Identity Layer
//!
//! Provides decentralised identity primitives for the Veritas protocol:
//! - DID creation and resolution
//! - DID Documents (W3C-compatible)
//! - Verifiable Credentials with Ed25519 proofs
//! - TrustGraph with EigenTrust-like score computation
//! - Composite TrustScore for network participants
//! - Humanity verification (AI-resistant proof of humanity)
//! - DID resolution (local, network, composite)

pub mod credentials;
pub mod did;
pub mod did_resolver;
pub mod document;
pub mod error;
pub mod humanity;
pub mod trust_graph;
pub mod trust_score;

pub use credentials::VerifiableCredential;
pub use did::DidManager;
pub use did_resolver::{CompositeDidResolver, DidResolver, LocalDidResolver};
pub use document::DidDocument;
pub use error::IdentityError;
pub use humanity::{HumanityStatus, HumanityVerificationMethod, HumanityVerifier};
pub use trust_graph::{TrustEdge, TrustGraph};
pub use trust_score::TrustScore;
