//! GossipSub topic management for the Veritas identity network.
//!
//! Defines the standard topics used by Veritas nodes for broadcasting
//! credentials, DID announcements, proof requests, and peer announcements.

use libp2p::gossipsub;

/// Veritas GossipSub topic names.
pub mod topics {
    /// Topic for broadcasting verifiable credentials across the network.
    pub const CREDENTIALS: &str = "veritas/credentials/v1";

    /// Topic for DID document announcements and updates.
    pub const DID_ANNOUNCE: &str = "veritas/did-announce/v1";

    /// Topic for proof requests and responses.
    pub const PROOF_REQUESTS: &str = "veritas/proof-requests/v1";

    /// Topic for peer announcements (capabilities, supported credential types).
    pub const PEER_ANNOUNCE: &str = "veritas/peer-announce/v1";
}

/// Manager for Veritas GossipSub topics.
///
/// Provides convenient access to pre-built topic and hash objects
/// for the standard Veritas protocol topics.
#[derive(Debug, Clone)]
pub struct TopicManager {
    /// The credentials topic.
    pub credentials: gossipsub::IdentTopic,
    /// The DID announce topic.
    pub did_announce: gossipsub::IdentTopic,
    /// The proof requests topic.
    pub proof_requests: gossipsub::IdentTopic,
    /// The peer announcements topic.
    pub peer_announce: gossipsub::IdentTopic,
}

impl TopicManager {
    /// Create a new TopicManager with all standard Veritas topics.
    pub fn new() -> Self {
        Self {
            credentials: gossipsub::IdentTopic::new(topics::CREDENTIALS),
            did_announce: gossipsub::IdentTopic::new(topics::DID_ANNOUNCE),
            proof_requests: gossipsub::IdentTopic::new(topics::PROOF_REQUESTS),
            peer_announce: gossipsub::IdentTopic::new(topics::PEER_ANNOUNCE),
        }
    }

    /// Get the hash for the credentials topic.
    pub fn credentials_hash(&self) -> gossipsub::TopicHash {
        self.credentials.hash()
    }

    /// Get the hash for the DID announce topic.
    pub fn did_announce_hash(&self) -> gossipsub::TopicHash {
        self.did_announce.hash()
    }

    /// Get the hash for the proof requests topic.
    pub fn proof_requests_hash(&self) -> gossipsub::TopicHash {
        self.proof_requests.hash()
    }

    /// Get the hash for the peer announce topic.
    pub fn peer_announce_hash(&self) -> gossipsub::TopicHash {
        self.peer_announce.hash()
    }

    /// Return all standard topic references for subscription.
    pub fn all_topics(&self) -> Vec<&gossipsub::IdentTopic> {
        vec![
            &self.credentials,
            &self.did_announce,
            &self.proof_requests,
            &self.peer_announce,
        ]
    }

    /// Subscribe to all standard topics on the given gossipsub behaviour.
    pub fn subscribe_all(
        &self,
        gossipsub: &mut gossipsub::Behaviour,
    ) -> Result<(), crate::error::NetworkError> {
        for topic in self.all_topics() {
            gossipsub.subscribe(topic).map_err(|e| {
                crate::error::NetworkError::Gossipsub(format!(
                    "failed to subscribe to {}: {}",
                    topic.hash(),
                    e
                ))
            })?;
        }
        Ok(())
    }

    /// Determine the topic name from a TopicHash.
    pub fn topic_name_from_hash(&self, hash: &gossipsub::TopicHash) -> Option<&'static str> {
        if *hash == self.credentials_hash() {
            Some(topics::CREDENTIALS)
        } else if *hash == self.did_announce_hash() {
            Some(topics::DID_ANNOUNCE)
        } else if *hash == self.proof_requests_hash() {
            Some(topics::PROOF_REQUESTS)
        } else if *hash == self.peer_announce_hash() {
            Some(topics::PEER_ANNOUNCE)
        } else {
            None
        }
    }
}

impl Default for TopicManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_constants() {
        assert_eq!(topics::CREDENTIALS, "veritas/credentials/v1");
        assert_eq!(topics::DID_ANNOUNCE, "veritas/did-announce/v1");
        assert_eq!(topics::PROOF_REQUESTS, "veritas/proof-requests/v1");
        assert_eq!(topics::PEER_ANNOUNCE, "veritas/peer-announce/v1");
    }

    #[test]
    fn test_topic_manager_creation() {
        let tm = TopicManager::new();
        assert_eq!(
            tm.credentials.hash(),
            gossipsub::IdentTopic::new(topics::CREDENTIALS).hash()
        );
        assert_eq!(
            tm.did_announce.hash(),
            gossipsub::IdentTopic::new(topics::DID_ANNOUNCE).hash()
        );
    }

    #[test]
    fn test_topic_manager_default() {
        let tm = TopicManager::default();
        assert_eq!(
            tm.credentials.hash(),
            gossipsub::IdentTopic::new(topics::CREDENTIALS).hash()
        );
    }

    #[test]
    fn test_all_topics_returns_four() {
        let tm = TopicManager::new();
        let all = tm.all_topics();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_topic_hashes_are_distinct() {
        let tm = TopicManager::new();
        let hashes = vec![
            tm.credentials_hash(),
            tm.did_announce_hash(),
            tm.proof_requests_hash(),
            tm.peer_announce_hash(),
        ];

        for i in 0..hashes.len() {
            for j in (i + 1)..hashes.len() {
                assert_ne!(hashes[i], hashes[j], "topic hashes should be distinct");
            }
        }
    }

    #[test]
    fn test_topic_name_from_hash() {
        let tm = TopicManager::new();

        assert_eq!(
            tm.topic_name_from_hash(&tm.credentials_hash()),
            Some(topics::CREDENTIALS)
        );
        assert_eq!(
            tm.topic_name_from_hash(&tm.did_announce_hash()),
            Some(topics::DID_ANNOUNCE)
        );
        assert_eq!(
            tm.topic_name_from_hash(&tm.proof_requests_hash()),
            Some(topics::PROOF_REQUESTS)
        );
        assert_eq!(
            tm.topic_name_from_hash(&tm.peer_announce_hash()),
            Some(topics::PEER_ANNOUNCE)
        );
    }

    #[test]
    fn test_topic_name_from_unknown_hash() {
        let tm = TopicManager::new();
        let unknown = gossipsub::IdentTopic::new("unknown/topic").hash();
        assert_eq!(tm.topic_name_from_hash(&unknown), None);
    }

    #[test]
    fn test_subscribe_all() {
        use libp2p::identity::Keypair;

        let keypair = Keypair::generate_ed25519();
        let mut behaviour =
            crate::behaviour::VeritasBehaviour::new(&keypair).expect("behaviour creation failed");
        let tm = TopicManager::new();

        let result = tm.subscribe_all(&mut behaviour.gossipsub);
        assert!(result.is_ok());
    }
}
