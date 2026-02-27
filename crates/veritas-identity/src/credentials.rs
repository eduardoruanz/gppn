use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use veritas_crypto::{sign, verify, KeyPair, PublicKey, Signature};

use crate::error::IdentityError;

/// A W3C-inspired Verifiable Credential for the Veritas protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiableCredential {
    /// Unique credential identifier.
    pub id: String,
    /// Type(s) of the credential (e.g., ["VerifiableCredential", "VeritasNodeAttestation"]).
    pub credential_type: Vec<String>,
    /// DID of the issuer.
    pub issuer: String,
    /// DID of the subject.
    pub subject: String,
    /// When the credential was issued.
    pub issuance_date: DateTime<Utc>,
    /// Optional expiration date.
    pub expiration_date: Option<DateTime<Utc>>,
    /// Credential claims as arbitrary JSON.
    pub claims: serde_json::Value,
    /// Ed25519 signature over the canonical credential payload (hex-encoded).
    pub proof: Option<CredentialProof>,
}

/// Proof attached to a verifiable credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialProof {
    /// Proof type.
    pub proof_type: String,
    /// When the proof was created.
    pub created: DateTime<Utc>,
    /// Verification method ID (e.g., "did:veritas:key:abc#keys-1").
    pub verification_method: String,
    /// The signature value (hex-encoded).
    pub signature_hex: String,
}

impl VerifiableCredential {
    /// Create a new unsigned credential.
    pub fn new(
        issuer: String,
        subject: String,
        credential_type: Vec<String>,
        claims: serde_json::Value,
    ) -> Self {
        let mut types = vec!["VerifiableCredential".to_string()];
        for t in credential_type {
            if t != "VerifiableCredential" {
                types.push(t);
            }
        }

        Self {
            id: format!("urn:uuid:{}", Uuid::now_v7()),
            credential_type: types,
            issuer,
            subject,
            issuance_date: Utc::now(),
            expiration_date: None,
            claims,
            proof: None,
        }
    }

    /// Set the expiration date.
    pub fn with_expiration(mut self, expiration: DateTime<Utc>) -> Self {
        self.expiration_date = Some(expiration);
        self
    }

    /// Compute the canonical signing payload for this credential.
    ///
    /// This is a deterministic JSON representation of the credential
    /// without the proof field.
    pub fn signing_payload(&self) -> Vec<u8> {
        // Create a canonical representation without the proof
        let canonical = serde_json::json!({
            "id": self.id,
            "type": self.credential_type,
            "issuer": self.issuer,
            "subject": self.subject,
            "issuanceDate": self.issuance_date.to_rfc3339(),
            "expirationDate": self.expiration_date.map(|d| d.to_rfc3339()),
            "claims": self.claims,
        });
        serde_json::to_vec(&canonical).unwrap_or_default()
    }

    /// Issue (sign) this credential with the given keypair.
    ///
    /// The issuer DID must match the keypair.
    pub fn issue(mut self, keypair: &KeyPair) -> Result<Self, IdentityError> {
        let payload = self.signing_payload();
        let sig = sign(&payload, keypair);

        self.proof = Some(CredentialProof {
            proof_type: "Ed25519Signature2020".to_string(),
            created: Utc::now(),
            verification_method: format!("{}#keys-1", self.issuer),
            signature_hex: sig.to_hex(),
        });

        Ok(self)
    }

    /// Verify the credential's proof against the given public key.
    pub fn verify_proof(&self, public_key: &PublicKey) -> Result<(), IdentityError> {
        let proof = self.proof.as_ref().ok_or_else(|| {
            IdentityError::CredentialVerification("no proof attached".to_string())
        })?;

        // Check expiration
        if let Some(exp) = self.expiration_date {
            if Utc::now() > exp {
                return Err(IdentityError::CredentialVerification(
                    "credential has expired".to_string(),
                ));
            }
        }

        let payload = self.signing_payload();

        let sig_bytes = hex::decode(&proof.signature_hex).map_err(|e| {
            IdentityError::CredentialVerification(format!("invalid signature hex: {}", e))
        })?;
        let signature = Signature::from_bytes(&sig_bytes).map_err(|e| {
            IdentityError::CredentialVerification(format!("invalid signature: {}", e))
        })?;

        verify(&payload, &signature, public_key).map_err(|_| {
            IdentityError::CredentialVerification("signature verification failed".to_string())
        })
    }

    /// Check if the credential has been signed.
    pub fn is_signed(&self) -> bool {
        self.proof.is_some()
    }

    /// Check if the credential has expired.
    pub fn is_expired(&self) -> bool {
        self.expiration_date
            .map(|exp| Utc::now() > exp)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn test_keypair() -> KeyPair {
        KeyPair::generate()
    }

    fn test_credential(issuer: &str, subject: &str) -> VerifiableCredential {
        VerifiableCredential::new(
            issuer.to_string(),
            subject.to_string(),
            vec!["VeritasNodeAttestation".to_string()],
            serde_json::json!({
                "nodeVersion": "0.1.0",
                "uptime": 0.99,
            }),
        )
    }

    #[test]
    fn test_create_credential() {
        let vc = test_credential("did:veritas:key:issuer", "did:veritas:key:subject");
        assert!(vc.id.starts_with("urn:uuid:"));
        assert!(vc
            .credential_type
            .contains(&"VerifiableCredential".to_string()));
        assert!(vc
            .credential_type
            .contains(&"VeritasNodeAttestation".to_string()));
        assert!(!vc.is_signed());
    }

    #[test]
    fn test_issue_and_verify() {
        let kp = test_keypair();
        let vc = test_credential("did:veritas:key:issuer", "did:veritas:key:subject");
        let signed = vc.issue(&kp).unwrap();

        assert!(signed.is_signed());
        assert!(signed.verify_proof(&kp.public_key()).is_ok());
    }

    #[test]
    fn test_verify_wrong_key() {
        let kp1 = test_keypair();
        let kp2 = test_keypair();
        let vc = test_credential("did:veritas:key:issuer", "did:veritas:key:subject");
        let signed = vc.issue(&kp1).unwrap();

        let result = signed.verify_proof(&kp2.public_key());
        assert!(matches!(
            result,
            Err(IdentityError::CredentialVerification(_))
        ));
    }

    #[test]
    fn test_verify_unsigned() {
        let kp = test_keypair();
        let vc = test_credential("did:veritas:key:issuer", "did:veritas:key:subject");
        let result = vc.verify_proof(&kp.public_key());
        assert!(matches!(
            result,
            Err(IdentityError::CredentialVerification(_))
        ));
    }

    #[test]
    fn test_credential_with_expiration() {
        let kp = test_keypair();
        let future = Utc::now() + Duration::hours(24);
        let vc = test_credential("did:veritas:key:issuer", "did:veritas:key:subject")
            .with_expiration(future);
        let signed = vc.issue(&kp).unwrap();

        assert!(!signed.is_expired());
        assert!(signed.verify_proof(&kp.public_key()).is_ok());
    }

    #[test]
    fn test_expired_credential() {
        let kp = test_keypair();
        let past = Utc::now() - Duration::hours(1);
        let vc = test_credential("did:veritas:key:issuer", "did:veritas:key:subject")
            .with_expiration(past);
        let signed = vc.issue(&kp).unwrap();

        assert!(signed.is_expired());
        let result = signed.verify_proof(&kp.public_key());
        assert!(matches!(
            result,
            Err(IdentityError::CredentialVerification(_))
        ));
    }

    #[test]
    fn test_signing_payload_deterministic() {
        let vc = test_credential("did:veritas:key:issuer", "did:veritas:key:subject");
        let p1 = vc.signing_payload();
        let p2 = vc.signing_payload();
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let kp = test_keypair();
        let vc = test_credential("did:veritas:key:issuer", "did:veritas:key:subject");
        let signed = vc.issue(&kp).unwrap();

        let json = serde_json::to_string(&signed).unwrap();
        let deserialized: VerifiableCredential = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, signed.id);
        assert!(deserialized.proof.is_some());
    }

    #[test]
    fn test_no_duplicate_vc_type() {
        let vc = VerifiableCredential::new(
            "did:veritas:key:issuer".to_string(),
            "did:veritas:key:subject".to_string(),
            vec!["VerifiableCredential".to_string(), "Custom".to_string()],
            serde_json::json!({}),
        );
        // Should not duplicate "VerifiableCredential"
        let vc_count = vc
            .credential_type
            .iter()
            .filter(|t| *t == "VerifiableCredential")
            .count();
        assert_eq!(vc_count, 1);
    }
}
