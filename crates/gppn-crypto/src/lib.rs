pub mod error;
pub mod keys;
pub mod signing;
pub mod encryption;
pub mod hashing;
pub mod kdf;
pub mod zkp;

pub use error::CryptoError;
pub use keys::{KeyPair, PublicKey};
pub use signing::{sign, verify, Signature};
pub use encryption::{encrypt, decrypt, x25519_public_key, EncryptedPayload};
pub use hashing::{hash, hash_payment_message, merkle_root};
