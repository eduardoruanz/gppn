use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::CoreError;

/// Decentralized Identifier (DID) in the Veritas protocol.
/// Format: `did:veritas:<method>:<identifier>`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Did(pub String);

impl Did {
    /// Create a new DID from a full URI string.
    pub fn new(uri: String) -> Result<Self, CoreError> {
        if !uri.starts_with("did:veritas:") {
            return Err(CoreError::InvalidDid(format!(
                "DID must start with 'did:veritas:', got: {}",
                uri
            )));
        }
        let parts: Vec<&str> = uri.split(':').collect();
        if parts.len() < 4 {
            return Err(CoreError::InvalidDid(format!(
                "DID must have format 'did:veritas:<method>:<identifier>', got: {}",
                uri
            )));
        }
        Ok(Self(uri))
    }

    /// Create a DID from method and identifier components.
    pub fn from_parts(method: &str, identifier: &str) -> Self {
        Self(format!("did:veritas:{}:{}", method, identifier))
    }

    /// Get the full DID URI.
    pub fn uri(&self) -> &str {
        &self.0
    }

    /// Extract the method (key, web, peer).
    pub fn method(&self) -> Option<&str> {
        self.0.split(':').nth(2)
    }

    /// Extract the identifier.
    pub fn identifier(&self) -> Option<&str> {
        let parts: Vec<&str> = self.0.splitn(4, ':').collect();
        parts.get(3).copied()
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Types of verifiable credentials supported by Veritas.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CredentialType {
    /// Basic KYC verification (name, DOB, country).
    KycBasic,
    /// Enhanced KYC with address and document verification.
    KycEnhanced,
    /// Age verification (proves age >= threshold).
    AgeVerification,
    /// Residency proof (proves country/region of residence).
    Residency,
    /// AI-resistant proof of humanity.
    HumanityProof,
    /// Custom credential type.
    Custom(String),
}

impl fmt::Display for CredentialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KycBasic => write!(f, "KycBasic"),
            Self::KycEnhanced => write!(f, "KycEnhanced"),
            Self::AgeVerification => write!(f, "AgeVerification"),
            Self::Residency => write!(f, "Residency"),
            Self::HumanityProof => write!(f, "HumanityProof"),
            Self::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Value of a claim within a verifiable credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimValue {
    /// UTF-8 string value.
    String(String),
    /// Integer value (signed 64-bit).
    Integer(i64),
    /// Boolean value.
    Boolean(bool),
    /// Date in ISO 8601 format (YYYY-MM-DD).
    Date(String),
    /// Raw bytes.
    Bytes(Vec<u8>),
}

impl fmt::Display for ClaimValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", s),
            Self::Integer(i) => write!(f, "{}", i),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Date(d) => write!(f, "{}", d),
            Self::Bytes(b) => write!(f, "<{} bytes>", b.len()),
        }
    }
}

/// A single claim (name-value pair) in a credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    /// Claim name (e.g., "date_of_birth", "country", "kyc_level").
    pub name: String,
    /// Claim value.
    pub value: ClaimValue,
}

impl Claim {
    /// Create a string claim.
    pub fn string(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: ClaimValue::String(value.into()),
        }
    }

    /// Create an integer claim.
    pub fn integer(name: impl Into<String>, value: i64) -> Self {
        Self {
            name: name.into(),
            value: ClaimValue::Integer(value),
        }
    }

    /// Create a boolean claim.
    pub fn boolean(name: impl Into<String>, value: bool) -> Self {
        Self {
            name: name.into(),
            value: ClaimValue::Boolean(value),
        }
    }

    /// Create a date claim (ISO 8601).
    pub fn date(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: ClaimValue::Date(value.into()),
        }
    }

    /// Create a bytes claim.
    pub fn bytes(name: impl Into<String>, value: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            value: ClaimValue::Bytes(value),
        }
    }
}

impl fmt::Display for Claim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.name, self.value)
    }
}

/// Cryptographic proof types used in Veritas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProofType {
    /// Ed25519 digital signature (2020 suite).
    Ed25519Signature2020,
    /// BLAKE3 hash commitment for ZKP.
    Blake3Commitment,
    /// Sigma protocol interactive proof.
    SigmaProtocol,
}

impl fmt::Display for ProofType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ed25519Signature2020 => write!(f, "Ed25519Signature2020"),
            Self::Blake3Commitment => write!(f, "Blake3Commitment"),
            Self::SigmaProtocol => write!(f, "SigmaProtocol"),
        }
    }
}

/// Identifier for a credential schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SchemaId(pub String);

impl SchemaId {
    /// Create a new schema identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the schema ID string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SchemaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_did_new_valid() {
        let did = Did::new("did:veritas:key:abc123".into()).unwrap();
        assert_eq!(did.uri(), "did:veritas:key:abc123");
        assert_eq!(did.method(), Some("key"));
        assert_eq!(did.identifier(), Some("abc123"));
    }

    #[test]
    fn test_did_new_invalid_prefix() {
        let result = Did::new("did:other:key:abc123".into());
        assert!(result.is_err());
    }

    #[test]
    fn test_did_new_too_few_parts() {
        let result = Did::new("did:veritas:".into());
        assert!(result.is_err());
    }

    #[test]
    fn test_did_from_parts() {
        let did = Did::from_parts("web", "example.com");
        assert_eq!(did.uri(), "did:veritas:web:example.com");
        assert_eq!(did.method(), Some("web"));
    }

    #[test]
    fn test_did_display() {
        let did = Did::from_parts("key", "z6Mk...");
        assert_eq!(format!("{}", did), "did:veritas:key:z6Mk...");
    }

    #[test]
    fn test_credential_type_display() {
        assert_eq!(format!("{}", CredentialType::KycBasic), "KycBasic");
        assert_eq!(
            format!("{}", CredentialType::Custom("DriverLicense".into())),
            "Custom(DriverLicense)"
        );
    }

    #[test]
    fn test_claim_constructors() {
        let c = Claim::string("name", "Alice");
        assert_eq!(c.name, "name");
        assert_eq!(c.value, ClaimValue::String("Alice".into()));

        let c = Claim::integer("kyc_level", 3);
        assert_eq!(c.value, ClaimValue::Integer(3));

        let c = Claim::boolean("verified", true);
        assert_eq!(c.value, ClaimValue::Boolean(true));

        let c = Claim::date("dob", "1990-01-15");
        assert_eq!(c.value, ClaimValue::Date("1990-01-15".into()));

        let c = Claim::bytes("photo_hash", vec![1, 2, 3]);
        assert_eq!(c.value, ClaimValue::Bytes(vec![1, 2, 3]));
    }

    #[test]
    fn test_claim_display() {
        let c = Claim::string("country", "BR");
        assert_eq!(format!("{}", c), "country=BR");

        let c = Claim::integer("age", 25);
        assert_eq!(format!("{}", c), "age=25");
    }

    #[test]
    fn test_proof_type_display() {
        assert_eq!(
            format!("{}", ProofType::Ed25519Signature2020),
            "Ed25519Signature2020"
        );
        assert_eq!(
            format!("{}", ProofType::Blake3Commitment),
            "Blake3Commitment"
        );
    }

    #[test]
    fn test_schema_id() {
        let id = SchemaId::new("kyc-basic-v1");
        assert_eq!(id.as_str(), "kyc-basic-v1");
        assert_eq!(format!("{}", id), "kyc-basic-v1");
    }
}
