use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};

use crate::error::CryptoError;

/// Derive a 32-byte key from a password using Argon2id.
/// Returns the derived key bytes and the salt used.
pub fn derive_key(password: &[u8]) -> Result<(Vec<u8>, String), CryptoError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password, &salt)
        .map_err(|e| CryptoError::KeyDerivationError(format!("argon2 hash failed: {}", e)))?;

    let _hash_str = hash.to_string();
    // Extract the raw hash output
    let hash_output = hash
        .hash
        .ok_or_else(|| CryptoError::KeyDerivationError("no hash output".into()))?;

    Ok((hash_output.as_bytes().to_vec(), salt.to_string()))
}

/// Verify a password against a stored Argon2id hash.
pub fn verify_password(password: &[u8], hash_str: &str) -> Result<bool, CryptoError> {
    let argon2 = Argon2::default();
    let parsed_hash = argon2::PasswordHash::new(hash_str)
        .map_err(|e| CryptoError::KeyDerivationError(format!("invalid hash format: {}", e)))?;

    match argon2.verify_password(password, &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(CryptoError::KeyDerivationError(format!(
            "verification error: {}",
            e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key() {
        let (key, salt) = derive_key(b"test-password-123").unwrap();
        assert!(!key.is_empty());
        assert!(!salt.is_empty());
    }

    #[test]
    fn test_different_passwords_different_keys() {
        let (key1, _) = derive_key(b"password1").unwrap();
        let (key2, _) = derive_key(b"password2").unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_derive_key_empty_password() {
        let result = derive_key(b"");
        assert!(result.is_ok());
    }
}
