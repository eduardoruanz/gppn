use dashmap::DashMap;
use gppn_core::types::Did;
use gppn_crypto::KeyPair;

use crate::document::DidDocument;
use crate::error::IdentityError;

/// Manages DID creation, storage, and resolution.
///
/// Uses an in-memory `DashMap` as the local DID document store.
pub struct DidManager {
    /// DID URI -> DidDocument
    store: DashMap<String, DidDocument>,
}

impl DidManager {
    /// Create a new, empty DID manager.
    pub fn new() -> Self {
        Self {
            store: DashMap::new(),
        }
    }

    /// Create a new DID from a method name and a keypair.
    ///
    /// The DID format is: `did:gppn:<method>:<bs58_pubkey>`
    ///
    /// A DID Document is automatically created and stored.
    pub fn create_did(&self, method: &str, keypair: &KeyPair) -> Result<Did, IdentityError> {
        let pubkey_bs58 = bs58::encode(keypair.public_key().as_bytes()).into_string();
        let did_uri = format!("did:gppn:{}:{}", method, pubkey_bs58);

        // Check for duplicate
        if self.store.contains_key(&did_uri) {
            return Err(IdentityError::DuplicateDid(did_uri));
        }

        let doc = DidDocument::new(did_uri.clone(), pubkey_bs58);
        self.store.insert(did_uri.clone(), doc);

        tracing::info!(did = %did_uri, "DID created");

        Did::new(did_uri).map_err(|e| IdentityError::InvalidDid(e.to_string()))
    }

    /// Resolve a DID to its document.
    ///
    /// Returns `None` if the DID is not in the local store.
    pub fn resolve_did(&self, did: &str) -> Option<DidDocument> {
        self.store.get(did).map(|entry| entry.clone())
    }

    /// Update a DID document in the store.
    pub fn update_document(&self, doc: DidDocument) -> Result<(), IdentityError> {
        if !self.store.contains_key(&doc.id) {
            return Err(IdentityError::DidNotFound(doc.id.clone()));
        }
        self.store.insert(doc.id.clone(), doc);
        Ok(())
    }

    /// Remove a DID from the store.
    pub fn remove_did(&self, did: &str) -> Option<DidDocument> {
        self.store.remove(did).map(|(_, doc)| doc)
    }

    /// Get the number of DIDs in the store.
    pub fn count(&self) -> usize {
        self.store.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// List all stored DID URIs.
    pub fn list_dids(&self) -> Vec<String> {
        self.store.iter().map(|entry| entry.key().clone()).collect()
    }
}

impl Default for DidManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gppn_crypto::KeyPair;

    #[test]
    fn test_create_did() {
        let mgr = DidManager::new();
        let kp = KeyPair::generate();
        let did = mgr.create_did("key", &kp).unwrap();

        assert!(did.uri().starts_with("did:gppn:key:"));
        assert_eq!(did.method(), Some("key"));
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn test_create_did_format() {
        let mgr = DidManager::new();
        let kp = KeyPair::generate();
        let did = mgr.create_did("key", &kp).unwrap();

        let expected_bs58 = bs58::encode(kp.public_key().as_bytes()).into_string();
        let expected_uri = format!("did:gppn:key:{}", expected_bs58);
        assert_eq!(did.uri(), expected_uri);
    }

    #[test]
    fn test_resolve_did() {
        let mgr = DidManager::new();
        let kp = KeyPair::generate();
        let did = mgr.create_did("key", &kp).unwrap();

        let doc = mgr.resolve_did(did.uri()).unwrap();
        assert_eq!(doc.id, did.uri());
        assert_eq!(doc.verification_methods.len(), 1);
        assert_eq!(
            doc.verification_methods[0].method_type,
            "Ed25519VerificationKey2020"
        );
    }

    #[test]
    fn test_resolve_nonexistent_did() {
        let mgr = DidManager::new();
        assert!(mgr.resolve_did("did:gppn:key:nonexistent").is_none());
    }

    #[test]
    fn test_duplicate_did() {
        let mgr = DidManager::new();
        let kp = KeyPair::generate();
        mgr.create_did("key", &kp).unwrap();

        let result = mgr.create_did("key", &kp);
        assert!(matches!(result, Err(IdentityError::DuplicateDid(_))));
    }

    #[test]
    fn test_update_document() {
        let mgr = DidManager::new();
        let kp = KeyPair::generate();
        let did = mgr.create_did("key", &kp).unwrap();

        let mut doc = mgr.resolve_did(did.uri()).unwrap();
        doc.add_service("GppnNode", "https://node.example.com");
        mgr.update_document(doc).unwrap();

        let updated = mgr.resolve_did(did.uri()).unwrap();
        assert_eq!(updated.services.len(), 1);
    }

    #[test]
    fn test_update_nonexistent_document() {
        let mgr = DidManager::new();
        let doc = DidDocument::new("did:gppn:key:fake".to_string(), "fake".to_string());
        let result = mgr.update_document(doc);
        assert!(matches!(result, Err(IdentityError::DidNotFound(_))));
    }

    #[test]
    fn test_remove_did() {
        let mgr = DidManager::new();
        let kp = KeyPair::generate();
        let did = mgr.create_did("key", &kp).unwrap();

        let removed = mgr.remove_did(did.uri());
        assert!(removed.is_some());
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_list_dids() {
        let mgr = DidManager::new();
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();
        let did1 = mgr.create_did("key", &kp1).unwrap();
        let did2 = mgr.create_did("key", &kp2).unwrap();

        let dids = mgr.list_dids();
        assert_eq!(dids.len(), 2);
        assert!(dids.contains(&did1.uri().to_string()));
        assert!(dids.contains(&did2.uri().to_string()));
    }

    #[test]
    fn test_manager_default() {
        let mgr = DidManager::default();
        assert!(mgr.is_empty());
    }

    #[test]
    fn test_different_methods() {
        let mgr = DidManager::new();
        let kp = KeyPair::generate();
        let did = mgr.create_did("web", &kp).unwrap();
        assert!(did.uri().starts_with("did:gppn:web:"));
        assert_eq!(did.method(), Some("web"));
    }
}
