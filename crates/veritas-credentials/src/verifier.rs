use dashmap::DashMap;
use std::sync::Arc;

use veritas_crypto::PublicKey;
use veritas_identity::VerifiableCredential;

use crate::error::CredentialError;
use crate::schema::SchemaRegistry;

/// Result of credential verification.
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the credential is valid.
    pub valid: bool,
    /// Individual check results.
    pub checks: Vec<VerificationCheck>,
}

/// An individual verification check.
#[derive(Debug, Clone)]
pub struct VerificationCheck {
    /// Name of the check.
    pub name: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Optional detail message.
    pub detail: Option<String>,
}

/// Verifies verifiable credentials and presentations.
pub struct CredentialVerifier {
    /// Trusted issuer DIDs → public keys.
    trusted_issuers: DashMap<String, PublicKey>,
    /// Schema registry for validation.
    _schema_registry: Arc<SchemaRegistry>,
}

impl CredentialVerifier {
    /// Create a new credential verifier.
    pub fn new(schema_registry: Arc<SchemaRegistry>) -> Self {
        Self {
            trusted_issuers: DashMap::new(),
            _schema_registry: schema_registry,
        }
    }

    /// Add a trusted issuer.
    pub fn add_trusted_issuer(&self, did: String, public_key: PublicKey) {
        self.trusted_issuers.insert(did, public_key);
    }

    /// Remove a trusted issuer.
    pub fn remove_trusted_issuer(&self, did: &str) -> bool {
        self.trusted_issuers.remove(did).is_some()
    }

    /// Check if an issuer is trusted.
    pub fn is_trusted_issuer(&self, did: &str) -> bool {
        self.trusted_issuers.contains_key(did)
    }

    /// Number of trusted issuers.
    pub fn trusted_issuer_count(&self) -> usize {
        self.trusted_issuers.len()
    }

    /// Verify a credential.
    pub fn verify_credential(
        &self,
        credential: &VerifiableCredential,
    ) -> Result<VerificationResult, CredentialError> {
        let mut checks = Vec::new();

        // Check 1: Is the credential signed?
        let signed = credential.is_signed();
        checks.push(VerificationCheck {
            name: "signature_present".into(),
            passed: signed,
            detail: if signed {
                None
            } else {
                Some("credential is not signed".into())
            },
        });

        if !signed {
            return Ok(VerificationResult {
                valid: false,
                checks,
            });
        }

        // Check 2: Is the issuer trusted?
        let issuer_trusted = self.is_trusted_issuer(&credential.issuer);
        checks.push(VerificationCheck {
            name: "issuer_trusted".into(),
            passed: issuer_trusted,
            detail: if issuer_trusted {
                None
            } else {
                Some(format!("issuer {} is not trusted", credential.issuer))
            },
        });

        // Check 3: Verify signature (if issuer is trusted)
        let sig_valid = if issuer_trusted {
            let pubkey = self.trusted_issuers.get(&credential.issuer).unwrap();
            credential.verify_proof(&pubkey).is_ok()
        } else {
            false
        };
        checks.push(VerificationCheck {
            name: "signature_valid".into(),
            passed: sig_valid,
            detail: if sig_valid {
                None
            } else {
                Some("signature verification failed or issuer unknown".into())
            },
        });

        // Check 4: Is the credential expired?
        let not_expired = !credential.is_expired();
        checks.push(VerificationCheck {
            name: "not_expired".into(),
            passed: not_expired,
            detail: if not_expired {
                None
            } else {
                Some("credential has expired".into())
            },
        });

        let valid = signed && issuer_trusted && sig_valid && not_expired;

        Ok(VerificationResult { valid, checks })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veritas_crypto::KeyPair;

    fn setup() -> (CredentialVerifier, KeyPair) {
        let kp = KeyPair::generate();
        let registry = Arc::new(SchemaRegistry::new());
        let verifier = CredentialVerifier::new(registry);
        verifier.add_trusted_issuer("did:veritas:key:issuer".into(), kp.public_key());
        (verifier, kp)
    }

    fn make_credential(kp: &KeyPair) -> VerifiableCredential {
        VerifiableCredential::new(
            "did:veritas:key:issuer".into(),
            "did:veritas:key:subject".into(),
            vec!["TestCredential".into()],
            serde_json::json!({"test": true}),
        )
        .issue(kp)
        .unwrap()
    }

    #[test]
    fn test_verify_valid_credential() {
        let (verifier, kp) = setup();
        let vc = make_credential(&kp);
        let result = verifier.verify_credential(&vc).unwrap();
        assert!(result.valid);
        assert!(result.checks.iter().all(|c| c.passed));
    }

    #[test]
    fn test_verify_unsigned_credential() {
        let (verifier, _kp) = setup();
        let vc = VerifiableCredential::new(
            "did:veritas:key:issuer".into(),
            "did:veritas:key:subject".into(),
            vec!["Test".into()],
            serde_json::json!({}),
        );
        let result = verifier.verify_credential(&vc).unwrap();
        assert!(!result.valid);
    }

    #[test]
    fn test_verify_untrusted_issuer() {
        let registry = Arc::new(SchemaRegistry::new());
        let verifier = CredentialVerifier::new(registry);
        // Don't add any trusted issuers
        let kp = KeyPair::generate();
        let vc = make_credential(&kp);
        let result = verifier.verify_credential(&vc).unwrap();
        assert!(!result.valid);
        assert!(result
            .checks
            .iter()
            .any(|c| c.name == "issuer_trusted" && !c.passed));
    }

    #[test]
    fn test_verify_wrong_key() {
        let registry = Arc::new(SchemaRegistry::new());
        let verifier = CredentialVerifier::new(registry);
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();
        // Trust the wrong key
        verifier.add_trusted_issuer("did:veritas:key:issuer".into(), kp2.public_key());
        let vc = make_credential(&kp1);
        let result = verifier.verify_credential(&vc).unwrap();
        assert!(!result.valid);
        assert!(result
            .checks
            .iter()
            .any(|c| c.name == "signature_valid" && !c.passed));
    }

    #[test]
    fn test_verify_expired_credential() {
        let (verifier, kp) = setup();
        let vc = VerifiableCredential::new(
            "did:veritas:key:issuer".into(),
            "did:veritas:key:subject".into(),
            vec!["Test".into()],
            serde_json::json!({}),
        )
        .with_expiration(chrono::Utc::now() - chrono::Duration::hours(1))
        .issue(&kp)
        .unwrap();
        let result = verifier.verify_credential(&vc).unwrap();
        assert!(!result.valid);
        assert!(result
            .checks
            .iter()
            .any(|c| c.name == "not_expired" && !c.passed));
    }

    #[test]
    fn test_add_remove_trusted_issuer() {
        let registry = Arc::new(SchemaRegistry::new());
        let verifier = CredentialVerifier::new(registry);
        let kp = KeyPair::generate();
        verifier.add_trusted_issuer("did:veritas:key:x".into(), kp.public_key());
        assert!(verifier.is_trusted_issuer("did:veritas:key:x"));
        assert_eq!(verifier.trusted_issuer_count(), 1);
        assert!(verifier.remove_trusted_issuer("did:veritas:key:x"));
        assert!(!verifier.is_trusted_issuer("did:veritas:key:x"));
    }

    #[test]
    fn test_verification_result_checks() {
        let (verifier, kp) = setup();
        let vc = make_credential(&kp);
        let result = verifier.verify_credential(&vc).unwrap();
        assert_eq!(result.checks.len(), 4);
    }
}
