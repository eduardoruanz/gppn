use dashmap::DashMap;

use veritas_identity::VerifiableCredential;

use crate::error::CredentialError;

/// Credential wallet for a holder — stores and manages verifiable credentials.
pub struct CredentialWallet {
    /// DID of the wallet owner.
    owner_did: String,
    /// Credential ID → VerifiableCredential.
    credentials: DashMap<String, VerifiableCredential>,
}

impl CredentialWallet {
    /// Create a new credential wallet.
    pub fn new(owner_did: String) -> Self {
        Self {
            owner_did,
            credentials: DashMap::new(),
        }
    }

    /// Get the wallet owner's DID.
    pub fn owner_did(&self) -> &str {
        &self.owner_did
    }

    /// Store a credential in the wallet.
    pub fn store(&self, credential: VerifiableCredential) -> Result<(), CredentialError> {
        if credential.subject != self.owner_did {
            return Err(CredentialError::VerificationFailed(format!(
                "credential subject {} does not match wallet owner {}",
                credential.subject, self.owner_did
            )));
        }
        let id = credential.id.clone();
        self.credentials.insert(id.clone(), credential);
        tracing::debug!(credential_id = %id, "credential stored in wallet");
        Ok(())
    }

    /// Get a credential by ID.
    pub fn get(&self, id: &str) -> Option<VerifiableCredential> {
        self.credentials.get(id).map(|e| e.clone())
    }

    /// List all credential IDs.
    pub fn list(&self) -> Vec<String> {
        self.credentials.iter().map(|e| e.key().clone()).collect()
    }

    /// List credentials by type.
    pub fn list_by_type(&self, credential_type: &str) -> Vec<VerifiableCredential> {
        self.credentials
            .iter()
            .filter(|e| e.credential_type.iter().any(|t| t == credential_type))
            .map(|e| e.value().clone())
            .collect()
    }

    /// Number of credentials in the wallet.
    pub fn count(&self) -> usize {
        self.credentials.len()
    }

    /// Check if the wallet is empty.
    pub fn is_empty(&self) -> bool {
        self.credentials.is_empty()
    }

    /// Remove a credential from the wallet.
    pub fn remove(&self, id: &str) -> Option<VerifiableCredential> {
        self.credentials.remove(id).map(|(_, vc)| vc)
    }

    /// Get all non-expired credentials.
    pub fn active_credentials(&self) -> Vec<VerifiableCredential> {
        self.credentials
            .iter()
            .filter(|e| !e.is_expired())
            .map(|e| e.value().clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veritas_crypto::KeyPair;

    fn make_credential(subject: &str, cred_type: &str) -> VerifiableCredential {
        let kp = KeyPair::generate();
        VerifiableCredential::new(
            "did:veritas:key:issuer".into(),
            subject.into(),
            vec![cred_type.into()],
            serde_json::json!({"test": true}),
        )
        .issue(&kp)
        .unwrap()
    }

    #[test]
    fn test_store_and_get() {
        let wallet = CredentialWallet::new("did:veritas:key:alice".into());
        let vc = make_credential("did:veritas:key:alice", "KycBasic");
        let id = vc.id.clone();
        wallet.store(vc).unwrap();
        assert_eq!(wallet.count(), 1);
        assert!(wallet.get(&id).is_some());
    }

    #[test]
    fn test_store_wrong_subject() {
        let wallet = CredentialWallet::new("did:veritas:key:alice".into());
        let vc = make_credential("did:veritas:key:bob", "KycBasic");
        let result = wallet.store(vc);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_credentials() {
        let wallet = CredentialWallet::new("did:veritas:key:alice".into());
        let vc1 = make_credential("did:veritas:key:alice", "KycBasic");
        let vc2 = make_credential("did:veritas:key:alice", "AgeVerification");
        wallet.store(vc1).unwrap();
        wallet.store(vc2).unwrap();
        assert_eq!(wallet.list().len(), 2);
    }

    #[test]
    fn test_list_by_type() {
        let wallet = CredentialWallet::new("did:veritas:key:alice".into());
        let vc1 = make_credential("did:veritas:key:alice", "KycBasic");
        let vc2 = make_credential("did:veritas:key:alice", "AgeVerification");
        let vc3 = make_credential("did:veritas:key:alice", "KycBasic");
        wallet.store(vc1).unwrap();
        wallet.store(vc2).unwrap();
        wallet.store(vc3).unwrap();
        let kyc = wallet.list_by_type("KycBasic");
        assert_eq!(kyc.len(), 2);
    }

    #[test]
    fn test_remove_credential() {
        let wallet = CredentialWallet::new("did:veritas:key:alice".into());
        let vc = make_credential("did:veritas:key:alice", "KycBasic");
        let id = vc.id.clone();
        wallet.store(vc).unwrap();
        assert_eq!(wallet.count(), 1);
        let removed = wallet.remove(&id);
        assert!(removed.is_some());
        assert_eq!(wallet.count(), 0);
    }

    #[test]
    fn test_empty_wallet() {
        let wallet = CredentialWallet::new("did:veritas:key:alice".into());
        assert!(wallet.is_empty());
        assert_eq!(wallet.count(), 0);
        assert!(wallet.list().is_empty());
    }

    #[test]
    fn test_owner_did() {
        let wallet = CredentialWallet::new("did:veritas:key:alice".into());
        assert_eq!(wallet.owner_did(), "did:veritas:key:alice");
    }
}
