//! Shared node state for cross-task communication.

use libp2p::PeerId;
use std::collections::HashSet;
use std::sync::RwLock;
use std::time::Instant;
use tokio::sync::mpsc;

use crate::commands::NodeCommand;

/// Shared state for the running node, accessible from HTTP handlers.
pub struct NodeState {
    /// The local peer ID of this node.
    pub peer_id: PeerId,
    /// When the node started.
    pub start_time: Instant,
    /// Current connected peers (updated by the event loop).
    peers: RwLock<HashSet<PeerId>>,
    /// Listening addresses (updated when swarm reports them).
    addrs: RwLock<Vec<String>>,
    /// Channel to send commands to the event loop.
    pub command_tx: mpsc::Sender<NodeCommand>,
}

impl NodeState {
    pub fn new(peer_id: PeerId, command_tx: mpsc::Sender<NodeCommand>) -> Self {
        Self {
            peer_id,
            start_time: Instant::now(),
            peers: RwLock::new(HashSet::new()),
            addrs: RwLock::new(Vec::new()),
            command_tx,
        }
    }

    pub fn add_peer(&self, peer_id: PeerId) {
        self.peers.write().unwrap().insert(peer_id);
    }

    pub fn remove_peer(&self, peer_id: &PeerId) {
        self.peers.write().unwrap().remove(peer_id);
    }

    pub fn peer_count(&self) -> usize {
        self.peers.read().unwrap().len()
    }

    pub fn connected_peers(&self) -> HashSet<PeerId> {
        self.peers.read().unwrap().clone()
    }

    pub fn add_listening_addr(&self, addr: String) {
        self.addrs.write().unwrap().push(addr);
    }

    pub fn listening_addrs(&self) -> Vec<String> {
        self.addrs.read().unwrap().clone()
    }
}
