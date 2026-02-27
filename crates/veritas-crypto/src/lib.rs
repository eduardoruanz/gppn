pub mod encryption;
pub mod error;
pub mod hashing;
pub mod kdf;
pub mod keys;
pub mod selective_disclosure;
pub mod signing;
pub mod zkp;

pub use encryption::{decrypt, encrypt, x25519_public_key, EncryptedPayload};
pub use error::CryptoError;
pub use hashing::{create_commitment, hash, merkle_root, verify_commitment};
pub use keys::{KeyPair, PublicKey};
pub use selective_disclosure::SelectiveDisclosure;
pub use signing::{sign, sign_credential, verify, verify_credential, Signature};
pub use zkp::{Blake3ProofGenerator, Commitment, RangeProof, SetMembershipProof};
