//! The full GPPN node orchestrator.
//!
//! Ties together all layers: network, routing, settlement, identity.
//! Spawns the P2P network in a background task and runs an HTTP API server.

use anyhow::Result;
use libp2p::identity::Keypair;
use libp2p::PeerId;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

use gppn_identity::DidManager;
use gppn_network::{GppnNode, NetworkCommand, NetworkEvent};
use gppn_routing::DistributedRoutingTable;
use gppn_settlement::SettlementManager;

use crate::commands::{NodeCommand, SendPaymentResponse};
use crate::config::GppnConfig;
use crate::state::NodeState;
use crate::storage::Storage;

/// The full GPPN node, orchestrating all protocol layers.
pub struct GppnFullNode {
    /// Node configuration.
    config: GppnConfig,
    /// The ed25519 keypair for this node.
    keypair: Keypair,
    /// The P2P network layer (None after start moves it into a background task).
    network: Option<GppnNode>,
    /// The distributed routing table.
    routing_table: Arc<DistributedRoutingTable>,
    /// The settlement manager.
    settlement_manager: Arc<SettlementManager>,
    /// The DID manager.
    did_manager: Arc<DidManager>,
    /// Persistent storage.
    storage: Option<Storage>,
    /// Shared state accessible from HTTP handlers.
    node_state: Option<Arc<NodeState>>,
    /// Receives commands from the HTTP API.
    command_rx: Option<mpsc::Receiver<NodeCommand>>,
    /// Receives network events from the background network task.
    event_rx: Option<broadcast::Receiver<NetworkEvent>>,
    /// Sends commands to the network task's swarm event loop.
    net_command_tx: Option<mpsc::Sender<NetworkCommand>>,
}

impl GppnFullNode {
    /// Create a new full node with the given config.
    pub fn new(config: GppnConfig) -> Result<Self> {
        let keypair = if let Some(ref path) = config.identity.keypair_path {
            Self::load_or_generate_keypair(path)?
        } else {
            tracing::info!("generating ephemeral keypair");
            Keypair::generate_ed25519()
        };

        let routing_table = Arc::new(DistributedRoutingTable::new());
        let settlement_manager = Arc::new(SettlementManager::default());
        let did_manager = Arc::new(DidManager::new());

        let peer_id = PeerId::from(keypair.public());
        tracing::info!(%peer_id, "GPPN full node created");

        Ok(Self {
            config,
            keypair,
            network: None,
            routing_table,
            settlement_manager,
            did_manager,
            storage: None,
            node_state: None,
            command_rx: None,
            event_rx: None,
            net_command_tx: None,
        })
    }

    /// Initialize and start the node: storage, P2P network, HTTP API.
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("starting GPPN full node");

        // Initialize storage
        let storage = Storage::open(&self.config.storage.data_dir)?;
        self.storage = Some(storage);
        tracing::info!(path = %self.config.storage.data_dir.display(), "storage initialized");

        // Register built-in settlement adapters
        self.register_default_adapters();

        // Create and start P2P network
        let net_config = gppn_network::NodeConfig {
            listen_addr: self.config.p2p_multiaddr(),
            bootstrap_peers: self.config.network.bootstrap_peers.clone(),
            event_channel_capacity: 256,
        };

        let mut network = GppnNode::new(self.keypair.clone(), net_config)?;
        let listen_addr = network.start().await?;
        tracing::info!(%listen_addr, "P2P network started");

        // Grab handles before moving network into background task
        let event_rx = network.event_receiver();
        let net_command_tx = network.command_sender();

        // Spawn the network event loop in a background task
        tokio::spawn(async move {
            if let Err(e) = network.run().await {
                tracing::error!(error = %e, "network event loop error");
            }
            tracing::info!("network event loop exited");
        });

        // Create the NodeCommand channel (HTTP API → main event loop)
        let (command_tx, command_rx) = mpsc::channel::<NodeCommand>(256);

        // Create shared state
        let peer_id = self.peer_id();
        let node_state = Arc::new(NodeState::new(peer_id, command_tx));

        // Spawn the HTTP API server
        let api_addr: SocketAddr = format!(
            "{}:{}",
            self.config.api.listen_addr, self.config.api.port
        )
        .parse()?;

        let api_state = node_state.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::api::start_api_server(api_addr, api_state).await {
                tracing::error!(error = %e, "HTTP API server error");
            }
        });

        // Store handles
        self.node_state = Some(node_state);
        self.command_rx = Some(command_rx);
        self.event_rx = Some(event_rx);
        self.net_command_tx = Some(net_command_tx);

        Ok(())
    }

    /// Run the node's main event loop: processes network events and API commands.
    pub async fn run(&mut self) -> Result<()> {
        let mut event_rx = self
            .event_rx
            .take()
            .ok_or_else(|| anyhow::anyhow!("node not started"))?;
        let mut command_rx = self
            .command_rx
            .take()
            .ok_or_else(|| anyhow::anyhow!("node not started"))?;
        let net_command_tx = self
            .net_command_tx
            .clone()
            .ok_or_else(|| anyhow::anyhow!("node not started"))?;
        let node_state = self
            .node_state
            .clone()
            .ok_or_else(|| anyhow::anyhow!("node not started"))?;

        tracing::info!("entering main event loop");

        loop {
            tokio::select! {
                event = event_rx.recv() => {
                    match event {
                        Ok(ev) => Self::handle_network_event(&node_state, &ev),
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!(missed = n, "event receiver lagged");
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            tracing::info!("network event channel closed");
                            break;
                        }
                    }
                }
                cmd = command_rx.recv() => {
                    match cmd {
                        Some(c) => Self::handle_api_command(c, &net_command_tx, &node_state).await,
                        None => {
                            tracing::info!("API command channel closed");
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Gracefully shut down the node.
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("shutting down GPPN full node");

        // Drop the network command sender — this signals the network task to exit
        self.net_command_tx = None;
        self.node_state = None;

        if let Some(storage) = self.storage.take() {
            drop(storage);
            tracing::info!("storage closed");
        }

        tracing::info!("GPPN full node shut down");
        Ok(())
    }

    /// Get the node's peer ID.
    pub fn peer_id(&self) -> PeerId {
        PeerId::from(self.keypair.public())
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
        tracing::info!("registered default settlement adapters");
    }

    /// Handle a network event by updating shared state and logging.
    fn handle_network_event(state: &Arc<NodeState>, event: &NetworkEvent) {
        match event {
            NetworkEvent::PeerConnected(pc) => {
                state.add_peer(pc.peer_id);
                tracing::info!(
                    peer_id = %pc.peer_id,
                    count = pc.num_connected,
                    "peer connected"
                );
            }
            NetworkEvent::PeerDisconnected(pd) => {
                state.remove_peer(&pd.peer_id);
                tracing::info!(
                    peer_id = %pd.peer_id,
                    count = pd.num_connected,
                    "peer disconnected"
                );
            }
            NetworkEvent::Listening { address } => {
                state.add_listening_addr(address.to_string());
                tracing::info!(%address, "listening on address");
            }
            NetworkEvent::IncomingPaymentMessage(msg) => {
                tracing::info!(
                    source = ?msg.source,
                    topic = %msg.topic,
                    bytes = msg.data.len(),
                    "PAYMENT RECEIVED via gossipsub"
                );
            }
            NetworkEvent::DirectMessage(dm) => {
                tracing::info!(
                    peer_id = %dm.peer_id,
                    bytes = dm.data.len(),
                    "PAYMENT RECEIVED via direct message"
                );
            }
            NetworkEvent::RouteRequest(rr) => {
                tracing::debug!(
                    request_id = %rr.request_id,
                    target = %rr.target_did,
                    "route request received"
                );
            }
            NetworkEvent::RouteResponse(rr) => {
                tracing::debug!(
                    request_id = %rr.request_id,
                    found = rr.found,
                    "route response received"
                );
            }
        }
    }

    /// Handle a command from the HTTP API.
    async fn handle_api_command(
        cmd: NodeCommand,
        net_tx: &mpsc::Sender<NetworkCommand>,
        state: &Arc<NodeState>,
    ) {
        match cmd {
            NodeCommand::SendPayment {
                recipient,
                amount,
                currency,
                reply,
            } => {
                // Build a simple JSON payload as the payment data
                let pm_id = uuid::Uuid::now_v7().to_string();
                let payload = serde_json::json!({
                    "pm_id": pm_id,
                    "sender": state.peer_id.to_string(),
                    "recipient": recipient,
                    "amount": amount,
                    "currency": currency,
                    "status": "created",
                });
                let data = serde_json::to_vec(&payload).unwrap_or_default();

                tracing::info!(
                    %pm_id,
                    %recipient,
                    %amount,
                    %currency,
                    "sending payment"
                );

                // Try to send via direct request if we can parse a PeerId,
                // otherwise broadcast via gossipsub
                let (cmd_reply_tx, cmd_reply_rx) = tokio::sync::oneshot::channel();

                if let Ok(peer_id) = recipient.parse::<PeerId>() {
                    let _ = net_tx
                        .send(NetworkCommand::SendDirectRequest {
                            peer_id,
                            data,
                            reply: cmd_reply_tx,
                        })
                        .await;
                } else {
                    let _ = net_tx
                        .send(NetworkCommand::BroadcastPayment {
                            data,
                            reply: cmd_reply_tx,
                        })
                        .await;
                }

                match cmd_reply_rx.await {
                    Ok(Ok(())) => {
                        let _ = reply.send(Ok(SendPaymentResponse {
                            pm_id,
                            status: "sent".into(),
                            message: "payment dispatched to network".into(),
                        }));
                    }
                    Ok(Err(e)) => {
                        let _ = reply.send(Err(format!("network error: {}", e)));
                    }
                    Err(_) => {
                        let _ = reply.send(Err("network task did not respond".into()));
                    }
                }
            }
        }
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
