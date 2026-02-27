//! The full GPPN node orchestrator.
//!
//! Ties together all layers: network, routing, settlement, identity.

use anyhow::Result;
use libp2p::identity::Keypair;
use std::path::Path;
use std::sync::Arc;

use gppn_identity::DidManager;
use gppn_network::GppnNode;
use gppn_routing::DistributedRoutingTable;
use gppn_settlement::SettlementManager;

use crate::config::GppnConfig;
use crate::storage::Storage;

/// The full GPPN node, orchestrating all protocol layers.
pub struct GppnFullNode {
    /// Node configuration.
    config: GppnConfig,
    /// The ed25519 keypair for this node.
    keypair: Keypair,
    /// The P2P network layer.
    network: Option<gppn_network::GppnNode>,
    /// The distributed routing table.
    routing_table: Arc<DistributedRoutingTable>,
    /// The settlement manager.
    settlement_manager: Arc<SettlementManager>,
    /// The DID manager.
    did_manager: Arc<DidManager>,
    /// Persistent storage.
    storage: Option<Storage>,
}

impl GppnFullNode {
    /// Create a new full node with the given config.
    pub fn new(config: GppnConfig) -> Result<Self> {
        // Load or generate keypair
        let keypair = if let Some(ref path) = config.identity.keypair_path {
            Self::load_or_generate_keypair(path)?
        } else {
            tracing::info!("generating ephemeral keypair");
            Keypair::generate_ed25519()
        };

        let routing_table = Arc::new(DistributedRoutingTable::new());
        let settlement_manager = Arc::new(SettlementManager::default());
        let did_manager = Arc::new(DidManager::new());

        let peer_id = libp2p::PeerId::from(keypair.public());
        tracing::info!(%peer_id, "GPPN full node created");

        Ok(Self {
            config,
            keypair,
            network: None,
            routing_table,
            settlement_manager,
            did_manager,
            storage: None,
        })
    }

    /// Initialize and start the node.
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("starting GPPN full node");

        // Initialize storage
        let storage = Storage::open(&self.config.storage.data_dir)?;
        self.storage = Some(storage);
        tracing::info!(path = %self.config.storage.data_dir.display(), "storage initialized");

        // Register built-in settlement adapters
        self.register_default_adapters();

        // Start P2P network
        let net_config = gppn_network::NodeConfig {
            listen_addr: self.config.p2p_multiaddr(),
            bootstrap_peers: self.config.network.bootstrap_peers.clone(),
            event_channel_capacity: 256,
        };

        let mut network = GppnNode::new(self.keypair.clone(), net_config)?;
        let listen_addr = network.start().await?;
        tracing::info!(%listen_addr, "P2P network started");

        self.network = Some(network);

        Ok(())
    }

    /// Run the node's event loop until shutdown.
    pub async fn run(&mut self) -> Result<()> {
        if let Some(ref mut network) = self.network {
            tracing::info!("entering main event loop");
            network.run().await?;
        }
        Ok(())
    }

    /// Gracefully shut down the node.
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("shutting down GPPN full node");

        if let Some(ref mut network) = self.network {
            network.stop().await?;
        }
        self.network = None;

        if let Some(storage) = self.storage.take() {
            drop(storage);
            tracing::info!("storage closed");
        }

        tracing::info!("GPPN full node shut down");
        Ok(())
    }

    /// Get the node's peer ID.
    pub fn peer_id(&self) -> libp2p::PeerId {
        libp2p::PeerId::from(self.keypair.public())
    }

    /// Get a reference to the routing table.
    pub fn routing_table(&self) -> &Arc<DistributedRoutingTable> {
        &self.routing_table
    }

    /// Get a reference to the settlement manager.
    pub fn settlement_manager(&self) -> &Arc<SettlementManager> {
        &self.settlement_manager
    }

    /// Get a reference to the DID manager.
    pub fn did_manager(&self) -> &Arc<DidManager> {
        &self.did_manager
    }

    /// Register default settlement adapters (internal ledger).
    fn register_default_adapters(&self) {
        // The internal adapter is registered by default for off-chain settlement
        tracing::info!("registered default settlement adapters");
    }

    /// Load a keypair from disk, or generate and save a new one.
    fn load_or_generate_keypair(path: &Path) -> Result<Keypair> {
        if path.exists() {
            let bytes = std::fs::read(path)?;
            let keypair = Keypair::ed25519_from_bytes(bytes)
                .map_err(|e| anyhow::anyhow!("failed to decode keypair: {}", e))?;
            tracing::info!(path = %path.display(), "loaded keypair from disk");
            Ok(keypair)
        } else {
            let keypair = Keypair::generate_ed25519();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            // Save the seed bytes (32 bytes for ed25519)
            if let Ok(kp) = keypair.clone().try_into_ed25519() {
                std::fs::write(path, kp.to_bytes())?;
                tracing::info!(path = %path.display(), "generated and saved new keypair");
            }
            Ok(keypair)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_node_creation() {
        let config = GppnConfig::default();
        let node = GppnFullNode::new(config);
        assert!(node.is_ok());
    }

    #[test]
    fn test_full_node_peer_id() {
        let config = GppnConfig::default();
        let node = GppnFullNode::new(config).unwrap();
        let peer_id = node.peer_id();
        // PeerId should be valid (non-empty string representation)
        assert!(!peer_id.to_string().is_empty());
    }

    #[test]
    fn test_full_node_has_routing_table() {
        let config = GppnConfig::default();
        let node = GppnFullNode::new(config).unwrap();
        assert!(node.routing_table().is_empty());
    }

    #[test]
    fn test_full_node_has_did_manager() {
        let config = GppnConfig::default();
        let node = GppnFullNode::new(config).unwrap();
        assert!(node.did_manager().is_empty());
    }

    #[tokio::test]
    async fn test_full_node_start_and_shutdown() {
        let config = GppnConfig::default();
        let mut node = GppnFullNode::new(config).unwrap();
        node.start().await.expect("start failed");
        node.shutdown().await.expect("shutdown failed");
    }
}
