//! Veritas P2P Networking Crate
//!
//! This crate provides the peer-to-peer networking layer for the
//! Veritas decentralized identity protocol. Built on top of libp2p,
//! it implements:
//!
//! - **GossipSub** for broadcasting credentials and proof requests across the network
//! - **Kademlia** for distributed peer and DID discovery (DHT)
//! - **mDNS** for local network peer discovery
//! - **Identify** for exchanging peer identity information
//! - **Request-Response** for direct peer-to-peer messaging
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use veritas_network::{VeritasNode, NodeConfig};
//! use libp2p::identity::Keypair;
//!
//! #[tokio::main]
//! async fn main() {
//!     let keypair = Keypair::generate_ed25519();
//!     let config = NodeConfig::default();
//!     let mut node = VeritasNode::new(keypair, config).unwrap();
//!     node.start().await.unwrap();
//! }
//! ```

pub mod behaviour;
pub mod discovery;
pub mod error;
pub mod events;
pub mod gossip;
pub mod node;
pub mod protocol;
pub mod transport;

// Re-exports for convenience.
pub use behaviour::{VeritasBehaviour, VeritasBehaviourEvent};
pub use discovery::PeerDiscovery;
pub use error::NetworkError;
pub use events::{
    DirectMessage, IncomingCredential, NetworkEvent, PeerConnected, PeerDisconnected,
    ProofRequestEvent, ProofResponseEvent,
};
pub use gossip::TopicManager;
pub use node::{NetworkCommand, NodeConfig, VeritasNode};
pub use protocol::{VeritasCodec, VeritasRequest, VeritasResponse, VERITAS_PROTOCOL};

// Re-export commonly used libp2p types for downstream convenience.
pub use libp2p::{identity::Keypair, Multiaddr, PeerId};
