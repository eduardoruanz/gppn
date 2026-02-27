//! GossipSub topic management for the GPPN network.
//!
//! Defines the standard topics used by GPPN nodes for broadcasting
//! payment messages, routing announcements, and network telemetry.

use libp2p::gossipsub;

/// GPPN GossipSub topic names.
pub mod topics {
    /// Topic for broadcasting payment messages across the network.
    pub const PAYMENTS: &str = "gppn/payments/v1";

    /// Topic for routing table announcements and updates.
    pub const ROUTING: &str = "gppn/routing/v1";

    /// Topic for settlement confirmations.
    pub const SETTLEMENTS: &str = "gppn/settlements/v1";

    /// Topic for peer announcements (capabilities, supported currencies).
    pub const PEER_ANNOUNCE: &str = "gppn/peer-announce/v1";
}

/// Manager for GPPN GossipSub topics.
///
/// Provides convenient access to pre-built topic and hash objects
/// for the standard GPPN protocol topics.
#[derive(Debug, Clone)]
pub struct TopicManager {
    /// The payments topic.
    pub payments: gossipsub::IdentTopic,
    /// The routing topic.
    pub routing: gossipsub::IdentTopic,
    /// The settlements topic.
    pub settlements: gossipsub::IdentTopic,
    /// The peer announcements topic.
    pub peer_announce: gossipsub::IdentTopic,
}

impl TopicManager {
    /// Create a new TopicManager with all standard GPPN topics.
    pub fn new() -> Self {
        Self {
            payments: gossipsub::IdentTopic::new(topics::PAYMENTS),
            routing: gossipsub::IdentTopic::new(topics::ROUTING),
            settlements: gossipsub::IdentTopic::new(topics::SETTLEMENTS),
            peer_announce: gossipsub::IdentTopic::new(topics::PEER_ANNOUNCE),
        }
    }

    /// Get the hash for the payments topic.
    pub fn payments_hash(&self) -> gossipsub::TopicHash {
        self.payments.hash()
    }

    /// Get the hash for the routing topic.
    pub fn routing_hash(&self) -> gossipsub::TopicHash {
        self.routing.hash()
    }

    /// Get the hash for the settlements topic.
    pub fn settlements_hash(&self) -> gossipsub::TopicHash {
        self.settlements.hash()
    }

    /// Get the hash for the peer announce topic.
    pub fn peer_announce_hash(&self) -> gossipsub::TopicHash {
        self.peer_announce.hash()
    }

    /// Return all standard topic references for subscription.
    pub fn all_topics(&self) -> Vec<&gossipsub::IdentTopic> {
        vec![
            &self.payments,
            &self.routing,
            &self.settlements,
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
        if *hash == self.payments_hash() {
            Some(topics::PAYMENTS)
        } else if *hash == self.routing_hash() {
            Some(topics::ROUTING)
        } else if *hash == self.settlements_hash() {
            Some(topics::SETTLEMENTS)
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
        assert_eq!(topics::PAYMENTS, "gppn/payments/v1");
        assert_eq!(topics::ROUTING, "gppn/routing/v1");
        assert_eq!(topics::SETTLEMENTS, "gppn/settlements/v1");
        assert_eq!(topics::PEER_ANNOUNCE, "gppn/peer-announce/v1");
    }

    #[test]
    fn test_topic_manager_creation() {
        let tm = TopicManager::new();
        assert_eq!(tm.payments.hash(), gossipsub::IdentTopic::new(topics::PAYMENTS).hash());
        assert_eq!(tm.routing.hash(), gossipsub::IdentTopic::new(topics::ROUTING).hash());
    }

    #[test]
    fn test_topic_manager_default() {
        let tm = TopicManager::default();
        assert_eq!(tm.payments.hash(), gossipsub::IdentTopic::new(topics::PAYMENTS).hash());
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
            tm.payments_hash(),
            tm.routing_hash(),
            tm.settlements_hash(),
            tm.peer_announce_hash(),
        ];

        // Verify all hashes are unique
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
            tm.topic_name_from_hash(&tm.payments_hash()),
            Some(topics::PAYMENTS)
        );
        assert_eq!(
            tm.topic_name_from_hash(&tm.routing_hash()),
            Some(topics::ROUTING)
        );
        assert_eq!(
            tm.topic_name_from_hash(&tm.settlements_hash()),
            Some(topics::SETTLEMENTS)
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
        let mut behaviour = crate::behaviour::GppnBehaviour::new(&keypair)
            .expect("behaviour creation failed");
        let tm = TopicManager::new();

        let result = tm.subscribe_all(&mut behaviour.gossipsub);
        assert!(result.is_ok());
    }
}
