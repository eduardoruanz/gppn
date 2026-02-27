use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;

use veritas_crypto::KeyPair;
use veritas_identity::VerifiableCredential;

use crate::error::CredentialError;
use crate::schema::SchemaRegistry;

/// Issues verifiable credentials signed by the issuer's keypair.
pub struct CredentialIssuer {
    /// DID of the issuer.
    did: String,
    /// Issuer's signing keypair.
    keypair: KeyPair,
    /// Schema registry for validation.
    schema_registry: Arc<SchemaRegistry>,
}

impl CredentialIssuer {
    /// Create a new credential issuer.
    pub fn new(did: String, keypair: KeyPair, schema_registry: Arc<SchemaRegistry>) -> Self {
        Self {
            did,
            keypair,
            schema_registry,
        }
    }

    /// Get the issuer's DID.
    pub fn did(&self) -> &str {
        &self.did
    }

    /// Issue a credential with no expiration.
    pub fn issue(
        &self,
        subject_did: &str,
        credential_type: Vec<String>,
        claims: serde_json::Value,
    ) -> Result<VerifiableCredential, CredentialError> {
        let vc = VerifiableCredential::new(
            self.did.clone(),
            subject_did.to_string(),
            credential_type,
            claims,
        );

        let signed = vc
            .issue(&self.keypair)
            .map_err(|e| CredentialError::IssuanceFailed(e.to_string()))?;

        tracing::info!(
            issuer = %self.did,
            subject = subject_did,
            credential_id = %signed.id,
            "credential issued"
        );

        Ok(signed)
    }

    /// Issue a credential with an expiration duration.
    pub fn issue_with_expiry(
        &self,
        subject_did: &str,
        credential_type: Vec<String>,
        claims: serde_json::Value,
        duration: Duration,
    ) -> Result<VerifiableCredential, CredentialError> {
        let expiration = Utc::now() + duration;
        self.issue_with_expiration(subject_did, credential_type, claims, expiration)
    }

    /// Issue a credential with a specific expiration date.
    pub fn issue_with_expiration(
        &self,
        subject_did: &str,
        credential_type: Vec<String>,
        claims: serde_json::Value,
        expiration: DateTime<Utc>,
    ) -> Result<VerifiableCredential, CredentialError> {
        let vc = VerifiableCredential::new(
            self.did.clone(),
            subject_did.to_string(),
            credential_type,
            claims,
        )
        .with_expiration(expiration);

        let signed = vc
            .issue(&self.keypair)
            .map_err(|e| CredentialError::IssuanceFailed(e.to_string()))?;

        tracing::info!(
            issuer = %self.did,
            subject = subject_did,
            credential_id = %signed.id,
            expires = %expiration,
            "credential issued with expiration"
        );

        Ok(signed)
    }

    /// Issue a credential validated against a schema.
    pub fn issue_with_schema(
        &self,
        subject_did: &str,
        schema_id: &str,
        credential_type: Vec<String>,
        claims: serde_json::Value,
    ) -> Result<VerifiableCredential, CredentialError> {
        self.schema_registry.validate_claims(schema_id, &claims)?;
        self.issue(subject_did, credential_type, claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_issuer() -> CredentialIssuer {
        let kp = KeyPair::generate();
        let registry = Arc::new(SchemaRegistry::new());
        CredentialIssuer::new("did:veritas:key:issuer".into(), kp, registry)
    }

    #[test]
    fn test_issue_credential() {
        let issuer = test_issuer();
        let claims = serde_json::json!({"name": "Alice"});
        let vc = issuer
            .issue(
                "did:veritas:key:subject",
                vec!["TestCredential".into()],
                claims,
            )
            .unwrap();
        assert!(vc.is_signed());
        assert_eq!(vc.issuer, "did:veritas:key:issuer");
        assert_eq!(vc.subject, "did:veritas:key:subject");
    }

    #[test]
    fn test_issue_with_expiry() {
        let issuer = test_issuer();
        let claims = serde_json::json!({"name": "Bob"});
        let vc = issuer
            .issue_with_expiry(
                "did:veritas:key:bob",
                vec!["TestCredential".into()],
                claims,
                Duration::hours(24),
            )
            .unwrap();
        assert!(vc.is_signed());
        assert!(vc.expiration_date.is_some());
        assert!(!vc.is_expired());
    }

    #[test]
    fn test_issue_already_expired() {
        let issuer = test_issuer();
        let claims = serde_json::json!({"name": "Charlie"});
        let vc = issuer
            .issue_with_expiration(
                "did:veritas:key:charlie",
                vec!["TestCredential".into()],
                claims,
                Utc::now() - Duration::hours(1),
            )
            .unwrap();
        assert!(vc.is_expired());
    }

    #[test]
    fn test_issue_with_schema_valid() {
        let issuer = test_issuer();
        let claims = serde_json::json!({
            "full_name": "Alice",
            "date_of_birth": "1990-01-15",
            "country": "BR"
        });
        let vc = issuer
            .issue_with_schema(
                "did:veritas:key:alice",
                "kyc-basic-v1",
                vec!["KycBasic".into()],
                claims,
            )
            .unwrap();
        assert!(vc.is_signed());
    }

    #[test]
    fn test_issue_with_schema_missing_field() {
        let issuer = test_issuer();
        let claims = serde_json::json!({"full_name": "Alice"});
        let result = issuer.issue_with_schema(
            "did:veritas:key:alice",
            "kyc-basic-v1",
            vec!["KycBasic".into()],
            claims,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_issue_with_unknown_schema() {
        let issuer = test_issuer();
        let claims = serde_json::json!({});
        let result = issuer.issue_with_schema(
            "did:veritas:key:alice",
            "nonexistent",
            vec!["Test".into()],
            claims,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_issuer_did() {
        let issuer = test_issuer();
        assert_eq!(issuer.did(), "did:veritas:key:issuer");
    }

    #[test]
    fn test_verify_issued_credential() {
        let kp = KeyPair::generate();
        let pubkey = kp.public_key();
        let registry = Arc::new(SchemaRegistry::new());
        let issuer = CredentialIssuer::new("did:veritas:key:issuer".into(), kp, registry);
        let vc = issuer
            .issue(
                "did:veritas:key:bob",
                vec!["Test".into()],
                serde_json::json!({}),
            )
            .unwrap();
        assert!(vc.verify_proof(&pubkey).is_ok());
    }
}
