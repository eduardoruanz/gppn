use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A verification method within a DID Document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    /// Verification method identifier (e.g., "did:gppn:key:abc#keys-1").
    pub id: String,
    /// Type of the verification method (e.g., "Ed25519VerificationKey2020").
    pub method_type: String,
    /// The DID that controls this verification method.
    pub controller: String,
    /// Base58-encoded public key material.
    pub public_key_bs58: String,
}

/// A service endpoint in a DID Document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    /// Service identifier (e.g., "did:gppn:key:abc#gppn-endpoint").
    pub id: String,
    /// Service type (e.g., "GppnNode", "GppnRelay").
    pub service_type: String,
    /// Service endpoint URL.
    pub service_endpoint: String,
}

/// W3C-compatible DID Document for the GPPN protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDocument {
    /// The DID subject (e.g., "did:gppn:key:<bs58_pubkey>").
    pub id: String,
    /// Verification methods (public keys) associated with this DID.
    pub verification_methods: Vec<VerificationMethod>,
    /// Service endpoints.
    pub services: Vec<Service>,
    /// When the document was created.
    pub created: DateTime<Utc>,
    /// When the document was last updated.
    pub updated: DateTime<Utc>,
}

impl DidDocument {
    /// Create a new DID Document with a single verification method.
    pub fn new(id: String, public_key_bs58: String) -> Self {
        let now = Utc::now();
        let vm = VerificationMethod {
            id: format!("{}#keys-1", id),
            method_type: "Ed25519VerificationKey2020".to_string(),
            controller: id.clone(),
            public_key_bs58,
        };
        Self {
            id,
            verification_methods: vec![vm],
            services: Vec::new(),
            created: now,
            updated: now,
        }
    }

    /// Add a service endpoint.
    pub fn add_service(&mut self, service_type: &str, endpoint: &str) {
        let idx = self.services.len() + 1;
        let service = Service {
            id: format!("{}#service-{}", self.id, idx),
            service_type: service_type.to_string(),
            service_endpoint: endpoint.to_string(),
        };
        self.services.push(service);
        self.updated = Utc::now();
    }

    /// Add a verification method.
    pub fn add_verification_method(
        &mut self,
        method_type: &str,
        public_key_bs58: &str,
    ) {
        let idx = self.verification_methods.len() + 1;
        let vm = VerificationMethod {
            id: format!("{}#keys-{}", self.id, idx),
            method_type: method_type.to_string(),
            controller: self.id.clone(),
            public_key_bs58: public_key_bs58.to_string(),
        };
        self.verification_methods.push(vm);
        self.updated = Utc::now();
    }

    /// Get the primary public key (first verification method) in base58.
    pub fn primary_public_key_bs58(&self) -> Option<&str> {
        self.verification_methods
            .first()
            .map(|vm| vm.public_key_bs58.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_did_document() {
        let doc = DidDocument::new(
            "did:gppn:key:abc123".to_string(),
            "abc123".to_string(),
        );
        assert_eq!(doc.id, "did:gppn:key:abc123");
        assert_eq!(doc.verification_methods.len(), 1);
        assert_eq!(
            doc.verification_methods[0].method_type,
            "Ed25519VerificationKey2020"
        );
        assert_eq!(doc.verification_methods[0].public_key_bs58, "abc123");
        assert!(doc.services.is_empty());
    }

    #[test]
    fn test_add_service() {
        let mut doc = DidDocument::new(
            "did:gppn:key:abc".to_string(),
            "abc".to_string(),
        );
        doc.add_service("GppnNode", "https://node.example.com");
        assert_eq!(doc.services.len(), 1);
        assert_eq!(doc.services[0].service_type, "GppnNode");
        assert_eq!(
            doc.services[0].service_endpoint,
            "https://node.example.com"
        );
    }

    #[test]
    fn test_add_verification_method() {
        let mut doc = DidDocument::new(
            "did:gppn:key:abc".to_string(),
            "abc".to_string(),
        );
        doc.add_verification_method("X25519KeyAgreementKey2020", "def456");
        assert_eq!(doc.verification_methods.len(), 2);
        assert_eq!(
            doc.verification_methods[1].method_type,
            "X25519KeyAgreementKey2020"
        );
    }

    #[test]
    fn test_primary_public_key() {
        let doc = DidDocument::new(
            "did:gppn:key:xyz".to_string(),
            "xyz".to_string(),
        );
        assert_eq!(doc.primary_public_key_bs58(), Some("xyz"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut doc = DidDocument::new(
            "did:gppn:key:test".to_string(),
            "test".to_string(),
        );
        doc.add_service("GppnNode", "https://node.example.com");

        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: DidDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, doc.id);
        assert_eq!(
            deserialized.verification_methods.len(),
            doc.verification_methods.len()
        );
        assert_eq!(deserialized.services.len(), doc.services.len());
    }
}
