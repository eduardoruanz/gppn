//! GPPN P2P Networking Crate
//!
//! This crate provides the peer-to-peer networking layer for the
//! Global Payment Protocol Network (GPPN). Built on top of libp2p,
//! it implements:
//!
//! - **GossipSub** for broadcasting payment messages across the network
//! - **Kademlia** for distributed peer and route discovery (DHT)
//! - **mDNS** for local network peer discovery
//! - **Identify** for exchanging peer identity information
//! - **Request-Response** for direct peer-to-peer messaging
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use gppn_network::{GppnNode, NodeConfig};
//! use libp2p::identity::Keypair;
//!
//! #[tokio::main]
//! async fn main() {
//!     let keypair = Keypair::generate_ed25519();
//!     let config = NodeConfig::default();
//!     let mut node = GppnNode::new(keypair, config).unwrap();
//!     node.start().await.unwrap();
//! }
//! ```

pub mod error;
pub mod events;
pub mod protocol;
pub mod transport;
pub mod behaviour;
pub mod gossip;
pub mod discovery;
pub mod node;

// Re-exports for convenience.
pub use error::NetworkError;
pub use events::{
    DirectMessage, IncomingPaymentMessage, NetworkEvent, PeerConnected, PeerDisconnected,
    RouteRequest, RouteResponse,
};
pub use protocol::{GppnCodec, GppnRequest, GppnResponse, GPPN_PROTOCOL};
pub use behaviour::{GppnBehaviour, GppnBehaviourEvent};
pub use gossip::TopicManager;
pub use discovery::PeerDiscovery;
pub use node::{GppnNode, NetworkCommand, NodeConfig};

// Re-export commonly used libp2p types for downstream convenience.
pub use libp2p::{identity::Keypair, Multiaddr, PeerId};
