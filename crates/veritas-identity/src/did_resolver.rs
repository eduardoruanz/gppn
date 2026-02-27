use async_trait::async_trait;

use crate::did::DidManager;
use crate::document::DidDocument;
use crate::error::IdentityError;

/// Trait for resolving DIDs to their documents.
#[async_trait]
pub trait DidResolver: Send + Sync {
    /// Resolve a DID URI to its DID Document.
    async fn resolve(&self, did: &str) -> Result<DidDocument, IdentityError>;
}

/// Resolves DIDs from the local in-memory DidManager.
pub struct LocalDidResolver {
    manager: std::sync::Arc<DidManager>,
}

impl LocalDidResolver {
    /// Create a new local resolver backed by a DidManager.
    pub fn new(manager: std::sync::Arc<DidManager>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl DidResolver for LocalDidResolver {
    async fn resolve(&self, did: &str) -> Result<DidDocument, IdentityError> {
        self.manager
            .resolve_did(did)
            .ok_or_else(|| IdentityError::DidNotFound(did.to_string()))
    }
}

/// Composite resolver that tries multiple resolvers in order.
///
/// Returns the first successful resolution, or the last error.
pub struct CompositeDidResolver {
    resolvers: Vec<Box<dyn DidResolver>>,
}

impl CompositeDidResolver {
    /// Create a new composite resolver with no backends.
    pub fn new() -> Self {
        Self {
            resolvers: Vec::new(),
        }
    }

    /// Add a resolver to the chain.
    pub fn add_resolver(&mut self, resolver: Box<dyn DidResolver>) {
        self.resolvers.push(resolver);
    }

    /// Number of registered resolvers.
    pub fn resolver_count(&self) -> usize {
        self.resolvers.len()
    }
}

impl Default for CompositeDidResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DidResolver for CompositeDidResolver {
    async fn resolve(&self, did: &str) -> Result<DidDocument, IdentityError> {
        let mut last_error = IdentityError::DidResolution("no resolvers configured".into());

        for resolver in &self.resolvers {
            match resolver.resolve(did).await {
                Ok(doc) => return Ok(doc),
                Err(e) => {
                    tracing::debug!(did = did, error = %e, "resolver failed, trying next");
                    last_error = e;
                }
            }
        }

        Err(last_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use veritas_crypto::KeyPair;

    #[tokio::test]
    async fn test_local_resolver_found() {
        let mgr = Arc::new(DidManager::new());
        let kp = KeyPair::generate();
        let did = mgr.create_did("key", &kp).unwrap();

        let resolver = LocalDidResolver::new(mgr);
        let doc = resolver.resolve(did.uri()).await.unwrap();
        assert_eq!(doc.id, did.uri());
    }

    #[tokio::test]
    async fn test_local_resolver_not_found() {
        let mgr = Arc::new(DidManager::new());
        let resolver = LocalDidResolver::new(mgr);
        let result = resolver.resolve("did:veritas:key:nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_composite_resolver_first_succeeds() {
        let mgr = Arc::new(DidManager::new());
        let kp = KeyPair::generate();
        let did = mgr.create_did("key", &kp).unwrap();

        let mut composite = CompositeDidResolver::new();
        composite.add_resolver(Box::new(LocalDidResolver::new(mgr)));
        assert_eq!(composite.resolver_count(), 1);

        let doc = composite.resolve(did.uri()).await.unwrap();
        assert_eq!(doc.id, did.uri());
    }

    #[tokio::test]
    async fn test_composite_resolver_fallback() {
        let empty_mgr = Arc::new(DidManager::new());
        let full_mgr = Arc::new(DidManager::new());
        let kp = KeyPair::generate();
        let did = full_mgr.create_did("key", &kp).unwrap();

        let mut composite = CompositeDidResolver::new();
        composite.add_resolver(Box::new(LocalDidResolver::new(empty_mgr))); // will fail
        composite.add_resolver(Box::new(LocalDidResolver::new(full_mgr))); // will succeed

        let doc = composite.resolve(did.uri()).await.unwrap();
        assert_eq!(doc.id, did.uri());
    }

    #[tokio::test]
    async fn test_composite_resolver_all_fail() {
        let empty1 = Arc::new(DidManager::new());
        let empty2 = Arc::new(DidManager::new());

        let mut composite = CompositeDidResolver::new();
        composite.add_resolver(Box::new(LocalDidResolver::new(empty1)));
        composite.add_resolver(Box::new(LocalDidResolver::new(empty2)));

        let result = composite.resolve("did:veritas:key:unknown").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_composite_resolver_empty() {
        let composite = CompositeDidResolver::new();
        let result = composite.resolve("did:veritas:key:abc").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_composite_default() {
        let composite = CompositeDidResolver::default();
        assert_eq!(composite.resolver_count(), 0);
    }
}
