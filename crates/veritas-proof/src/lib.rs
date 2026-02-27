//! Veritas Proof — Zero-knowledge proof system for decentralized identity.
//!
//! Provides proof generation and verification for:
//! - Age proofs (prove age >= threshold without revealing DOB)
//! - Residency proofs (prove country membership without revealing address)
//! - KYC level proofs (prove KYC level >= required)
//! - Humanity proof bundles (composite proofs combining multiple signals)

pub mod age_proof;
pub mod error;
pub mod humanity_proof;
pub mod kyc_level_proof;
pub mod residency_proof;

pub use age_proof::AgeProof;
pub use error::ProofError;
pub use humanity_proof::HumanityProofBundle;
pub use kyc_level_proof::KycLevelProof;
pub use residency_proof::ResidencyProof;
