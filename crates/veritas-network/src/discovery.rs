//! Peer discovery logic for the Veritas network.
//!
//! Handles multiple discovery mechanisms:
//! - **mDNS**: Discovers peers on the local network automatically.
//! - **Kademlia bootstrap**: Connects to known bootstrap nodes to discover the wider network.
//! - **Manual**: Explicitly dial peers by multiaddr.

use libp2p::{Multiaddr, PeerId};
use std::collections::HashSet;
use std::str::FromStr;

use crate::error::NetworkError;

/// Manages peer discovery state for a Veritas node.
#[derive(Debug)]
pub struct PeerDiscovery {
    /// Bootstrap peer addresses to connect to on startup.
    bootstrap_addrs: Vec<Multiaddr>,
    /// Set of peers discovered via mDNS.
    mdns_peers: HashSet<PeerId>,
    /// Set of peers discovered via Kademlia.
    kad_peers: HashSet<PeerId>,
    /// Set of manually added peers.
    manual_peers: HashSet<PeerId>,
}

impl PeerDiscovery {
    /// Create a new PeerDiscovery with the given bootstrap addresses.
    pub fn new(bootstrap_addrs: Vec<String>) -> Result<Self, NetworkError> {
        let mut parsed_addrs = Vec::new();
        for addr_str in &bootstrap_addrs {
            let addr = Multiaddr::from_str(addr_str).map_err(|e| {
                NetworkError::Transport(format!("invalid bootstrap addr '{}': {}", addr_str, e))
            })?;
            parsed_addrs.push(addr);
        }

        Ok(Self {
            bootstrap_addrs: parsed_addrs,
            mdns_peers: HashSet::new(),
            kad_peers: HashSet::new(),
            manual_peers: HashSet::new(),
        })
    }

    /// Get the parsed bootstrap multiaddresses.
    pub fn bootstrap_addrs(&self) -> &[Multiaddr] {
        &self.bootstrap_addrs
    }

    /// Record a peer discovered via mDNS.
    pub fn add_mdns_peer(&mut self, peer_id: PeerId) {
        self.mdns_peers.insert(peer_id);
        tracing::debug!(%peer_id, "mDNS peer discovered");
    }

    /// Remove a peer that expired from mDNS.
    pub fn remove_mdns_peer(&mut self, peer_id: &PeerId) {
        self.mdns_peers.remove(peer_id);
        tracing::debug!(%peer_id, "mDNS peer expired");
    }

    /// Record a peer discovered via Kademlia.
    pub fn add_kad_peer(&mut self, peer_id: PeerId) {
        self.kad_peers.insert(peer_id);
        tracing::debug!(%peer_id, "Kademlia peer discovered");
    }

    /// Manually add a peer.
    pub fn add_manual_peer(&mut self, peer_id: PeerId) {
        self.manual_peers.insert(peer_id);
        tracing::debug!(%peer_id, "manual peer added");
    }

    /// Get all known peers from all discovery sources (deduplicated).
    pub fn all_known_peers(&self) -> HashSet<PeerId> {
        let mut all = self.mdns_peers.clone();
        all.extend(&self.kad_peers);
        all.extend(&self.manual_peers);
        all
    }

    /// Get the number of uniquely known peers.
    pub fn known_peer_count(&self) -> usize {
        self.all_known_peers().len()
    }

    /// Check if a peer is known through any discovery mechanism.
    pub fn is_known(&self, peer_id: &PeerId) -> bool {
        self.mdns_peers.contains(peer_id)
            || self.kad_peers.contains(peer_id)
            || self.manual_peers.contains(peer_id)
    }

    /// Get peers discovered via mDNS.
    pub fn mdns_peers(&self) -> &HashSet<PeerId> {
        &self.mdns_peers
    }

    /// Get peers discovered via Kademlia.
    pub fn kad_peers(&self) -> &HashSet<PeerId> {
        &self.kad_peers
    }

    /// Get manually added peers.
    pub fn manual_peers(&self) -> &HashSet<PeerId> {
        &self.manual_peers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty_bootstrap() {
        let discovery = PeerDiscovery::new(vec![]).expect("should succeed with empty bootstrap");
        assert_eq!(discovery.bootstrap_addrs().len(), 0);
        assert_eq!(discovery.known_peer_count(), 0);
    }

    #[test]
    fn test_new_with_valid_bootstrap() {
        let addrs = vec![
            "/ip4/127.0.0.1/tcp/9000".to_string(),
            "/ip4/192.168.1.1/tcp/9001".to_string(),
        ];
        let discovery = PeerDiscovery::new(addrs).expect("should parse valid addrs");
        assert_eq!(discovery.bootstrap_addrs().len(), 2);
    }

    #[test]
    fn test_new_with_invalid_bootstrap() {
        let addrs = vec!["not-a-valid-multiaddr".to_string()];
        let result = PeerDiscovery::new(addrs);
        assert!(result.is_err());
    }

    #[test]
    fn test_mdns_peer_lifecycle() {
        let mut discovery = PeerDiscovery::new(vec![]).expect("creation");
        let peer = PeerId::random();

        assert!(!discovery.is_known(&peer));
        discovery.add_mdns_peer(peer);
        assert!(discovery.is_known(&peer));
        assert!(discovery.mdns_peers().contains(&peer));
        assert_eq!(discovery.known_peer_count(), 1);

        discovery.remove_mdns_peer(&peer);
        assert!(!discovery.is_known(&peer));
        assert_eq!(discovery.known_peer_count(), 0);
    }

    #[test]
    fn test_kad_peer() {
        let mut discovery = PeerDiscovery::new(vec![]).expect("creation");
        let peer = PeerId::random();

        discovery.add_kad_peer(peer);
        assert!(discovery.is_known(&peer));
        assert!(discovery.kad_peers().contains(&peer));
        assert_eq!(discovery.known_peer_count(), 1);
    }

    #[test]
    fn test_manual_peer() {
        let mut discovery = PeerDiscovery::new(vec![]).expect("creation");
        let peer = PeerId::random();

        discovery.add_manual_peer(peer);
        assert!(discovery.is_known(&peer));
        assert!(discovery.manual_peers().contains(&peer));
        assert_eq!(discovery.known_peer_count(), 1);
    }

    #[test]
    fn test_all_known_peers_deduplication() {
        let mut discovery = PeerDiscovery::new(vec![]).expect("creation");
        let peer = PeerId::random();

        // Add the same peer via all discovery mechanisms
        discovery.add_mdns_peer(peer);
        discovery.add_kad_peer(peer);
        discovery.add_manual_peer(peer);

        // Should still be counted only once
        assert_eq!(discovery.known_peer_count(), 1);
        let all = discovery.all_known_peers();
        assert_eq!(all.len(), 1);
        assert!(all.contains(&peer));
    }

    #[test]
    fn test_multiple_distinct_peers() {
        let mut discovery = PeerDiscovery::new(vec![]).expect("creation");
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();

        discovery.add_mdns_peer(peer1);
        discovery.add_kad_peer(peer2);
        discovery.add_manual_peer(peer3);

        assert_eq!(discovery.known_peer_count(), 3);
        assert!(discovery.is_known(&peer1));
        assert!(discovery.is_known(&peer2));
        assert!(discovery.is_known(&peer3));
    }

    #[test]
    fn test_unknown_peer() {
        let discovery = PeerDiscovery::new(vec![]).expect("creation");
        let unknown_peer = PeerId::random();
        assert!(!discovery.is_known(&unknown_peer));
    }
}
