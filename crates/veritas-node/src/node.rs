//! The full Veritas node orchestrator.
//!
//! Ties together all layers: network, credentials, proofs, identity.
//! Spawns the P2P network in a background task and runs an HTTP API server.

use anyhow::Result;
use libp2p::identity::Keypair;
use libp2p::PeerId;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

use veritas_credentials::{CredentialIssuer, CredentialVerifier, CredentialWallet, SchemaRegistry};
use veritas_crypto::KeyPair;
use veritas_identity::{DidManager, TrustGraph};
use veritas_network::{NetworkCommand, NetworkEvent, VeritasNode};

use crate::commands::{
    CredentialResponse, DidResponse, NodeCommand, ProofResponse, TrustResponse, VerifyCheck,
    VerifyResponse,
};
use crate::config::VeritasConfig;
use crate::state::NodeState;
use crate::storage::Storage;

/// The full Veritas node, orchestrating all protocol layers.
pub struct VeritasFullNode {
    /// Node configuration.
    config: VeritasConfig,
    /// The ed25519 keypair for P2P identity.
    keypair: Keypair,
    /// The node's DID.
    did: String,
    /// The P2P network layer (None after start moves it into a background task).
    _network: Option<VeritasNode>,
    /// Credential issuer.
    credential_issuer: Arc<CredentialIssuer>,
    /// Credential verifier.
    credential_verifier: Arc<CredentialVerifier>,
    /// Credential wallet (holder).
    _credential_wallet: Arc<CredentialWallet>,
    /// Schema registry.
    schema_registry: Arc<SchemaRegistry>,
    /// DID manager.
    did_manager: Arc<DidManager>,
    /// Trust graph.
    trust_graph: Arc<TrustGraph>,
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

impl VeritasFullNode {
    /// Create a new full node with the given config.
    pub fn new(config: VeritasConfig) -> Result<Self> {
        let keypair = if let Some(ref path) = config.identity.keypair_path {
            Self::load_or_generate_keypair(path)?
        } else {
            tracing::info!("generating ephemeral keypair");
            Keypair::generate_ed25519()
        };

        let signing_keypair = KeyPair::generate();
        let peer_id = PeerId::from(keypair.public());
        let did = format!("did:veritas:key:{}", peer_id);

        let schema_registry = Arc::new(SchemaRegistry::new());
        let credential_issuer = Arc::new(CredentialIssuer::new(
            did.clone(),
            signing_keypair,
            schema_registry.clone(),
        ));
        let credential_verifier = Arc::new(CredentialVerifier::new(schema_registry.clone()));
        let credential_wallet = Arc::new(CredentialWallet::new(did.clone()));
        let did_manager = Arc::new(DidManager::new());
        let trust_graph = Arc::new(TrustGraph::new());

        tracing::info!(%peer_id, %did, "Veritas full node created");

        Ok(Self {
            config,
            keypair,
            did,
            _network: None,
            credential_issuer,
            credential_verifier,
            _credential_wallet: credential_wallet,
            schema_registry,
            did_manager,
            trust_graph,
            storage: None,
            node_state: None,
            command_rx: None,
            event_rx: None,
            net_command_tx: None,
        })
    }

    /// Initialize and start the node: storage, P2P network, HTTP API.
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("starting Veritas full node");

        // Initialize storage
        let storage = Storage::open(&self.config.storage.data_dir)?;
        self.storage = Some(storage);
        tracing::info!(path = %self.config.storage.data_dir.display(), "storage initialized");

        // Create and start P2P network
        let net_config = veritas_network::NodeConfig {
            listen_addr: self.config.p2p_multiaddr(),
            bootstrap_peers: self.config.network.bootstrap_peers.clone(),
            event_channel_capacity: 256,
        };

        let mut network = VeritasNode::new(self.keypair.clone(), net_config)?;
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
        let node_state = Arc::new(NodeState::new(peer_id, self.did.clone(), command_tx));

        // Spawn the HTTP API server
        let api_addr: SocketAddr =
            format!("{}:{}", self.config.api.listen_addr, self.config.api.port).parse()?;

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
        let node_state = self
            .node_state
            .clone()
            .ok_or_else(|| anyhow::anyhow!("node not started"))?;

        let issuer = self.credential_issuer.clone();
        let verifier = self.credential_verifier.clone();
        let did_manager = self.did_manager.clone();
        let trust_graph = self.trust_graph.clone();
        let node_did = self.did.clone();

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
                        Some(c) => Self::handle_api_command(
                            c, &issuer, &verifier,
                            &did_manager, &trust_graph, &node_did,
                        ),
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
        tracing::info!("shutting down Veritas full node");

        self.net_command_tx = None;
        self.node_state = None;

        if let Some(storage) = self.storage.take() {
            drop(storage);
            tracing::info!("storage closed");
        }

        tracing::info!("Veritas full node shut down");
        Ok(())
    }

    /// Get the node's peer ID.
    pub fn peer_id(&self) -> PeerId {
        PeerId::from(self.keypair.public())
    }

    /// Get the node's DID.
    pub fn did(&self) -> &str {
        &self.did
    }

    /// Get a reference to the DID manager.
    pub fn did_manager(&self) -> &Arc<DidManager> {
        &self.did_manager
    }

    /// Get a reference to the schema registry.
    pub fn schema_registry(&self) -> &Arc<SchemaRegistry> {
        &self.schema_registry
    }

    /// Get a reference to the trust graph.
    pub fn trust_graph(&self) -> &Arc<TrustGraph> {
        &self.trust_graph
    }

    /// Handle a network event by updating shared state.
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
            NetworkEvent::IncomingCredential(msg) => {
                tracing::info!(
                    source = ?msg.source,
                    topic = %msg.topic,
                    bytes = msg.data.len(),
                    "credential received via gossipsub"
                );
            }
            NetworkEvent::DirectMessage(dm) => {
                tracing::info!(
                    peer_id = %dm.peer_id,
                    bytes = dm.data.len(),
                    "direct message received"
                );
            }
            NetworkEvent::ProofRequestEvent(rr) => {
                tracing::debug!(
                    request_id = %rr.request_id,
                    verifier = %rr.verifier_did,
                    proof_type = %rr.proof_type,
                    "proof request received"
                );
            }
            NetworkEvent::ProofResponseEvent(rr) => {
                tracing::debug!(
                    request_id = %rr.request_id,
                    valid = rr.valid,
                    "proof response received"
                );
            }
        }
    }

    /// Handle a command from the HTTP API.
    fn handle_api_command(
        cmd: NodeCommand,
        issuer: &Arc<CredentialIssuer>,
        verifier: &Arc<CredentialVerifier>,
        did_manager: &Arc<DidManager>,
        trust_graph: &Arc<TrustGraph>,
        node_did: &str,
    ) {
        match cmd {
            NodeCommand::IssueCredential {
                subject_did,
                credential_type,
                claims,
                reply,
            } => {
                let result = issuer.issue(&subject_did, credential_type.clone(), claims);
                match result {
                    Ok(vc) => {
                        let _ = reply.send(Ok(CredentialResponse {
                            credential_id: vc.id.clone(),
                            issuer: vc.issuer.clone(),
                            subject: vc.subject.clone(),
                            status: "issued".into(),
                        }));
                    }
                    Err(e) => {
                        let _ = reply.send(Err(format!("issuance failed: {}", e)));
                    }
                }
            }
            NodeCommand::VerifyCredential {
                credential_json,
                reply,
            } => {
                match serde_json::from_str::<veritas_identity::VerifiableCredential>(
                    &credential_json,
                ) {
                    Ok(vc) => {
                        let result = verifier.verify_credential(&vc);
                        match result {
                            Ok(vr) => {
                                let checks = vr
                                    .checks
                                    .iter()
                                    .map(|c| VerifyCheck {
                                        name: c.name.clone(),
                                        passed: c.passed,
                                        detail: c.detail.clone(),
                                    })
                                    .collect();
                                let _ = reply.send(Ok(VerifyResponse {
                                    valid: vr.valid,
                                    checks,
                                }));
                            }
                            Err(e) => {
                                let _ = reply.send(Err(format!("verification failed: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        let _ = reply.send(Err(format!("invalid credential JSON: {}", e)));
                    }
                }
            }
            NodeCommand::GenerateProof {
                proof_type,
                params,
                reply,
            } => {
                let result = match proof_type.as_str() {
                    "age" => {
                        let dob = params
                            .get("date_of_birth")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let min_age = params.get("min_age").and_then(|v| v.as_i64()).unwrap_or(18);
                        match chrono::NaiveDate::parse_from_str(dob, "%Y-%m-%d") {
                            Ok(dob_date) => {
                                match veritas_proof::AgeProof::create(dob_date, min_age) {
                                    Ok(proof) => {
                                        Ok(serde_json::to_string(&proof).unwrap_or_default())
                                    }
                                    Err(e) => Err(format!("age proof failed: {}", e)),
                                }
                            }
                            Err(e) => Err(format!("invalid date_of_birth: {}", e)),
                        }
                    }
                    "residency" => {
                        let country = params.get("country").and_then(|v| v.as_str()).unwrap_or("");
                        let allowed: Vec<String> = params
                            .get("allowed_countries")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let allowed_refs: Vec<&str> = allowed.iter().map(|s| s.as_str()).collect();
                        match veritas_proof::ResidencyProof::create(country, &allowed_refs) {
                            Ok(proof) => Ok(serde_json::to_string(&proof).unwrap_or_default()),
                            Err(e) => Err(format!("residency proof failed: {}", e)),
                        }
                    }
                    "kyc_level" => {
                        let level = params
                            .get("actual_level")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        let min = params
                            .get("min_level")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(1);
                        match veritas_proof::KycLevelProof::create(level, min) {
                            Ok(proof) => Ok(serde_json::to_string(&proof).unwrap_or_default()),
                            Err(e) => Err(format!("kyc level proof failed: {}", e)),
                        }
                    }
                    other => Err(format!("unknown proof type: {}", other)),
                };

                match result {
                    Ok(proof_json) => {
                        let _ = reply.send(Ok(ProofResponse {
                            proof_type,
                            proof_json,
                            status: "generated".into(),
                        }));
                    }
                    Err(e) => {
                        let _ = reply.send(Err(e));
                    }
                }
            }
            NodeCommand::ResolveDid { did, reply } => match did_manager.resolve_did(&did) {
                Some(doc) => {
                    let doc_json = serde_json::to_value(&doc).unwrap_or_default();
                    let _ = reply.send(Ok(DidResponse {
                        did,
                        document: doc_json,
                    }));
                }
                None => {
                    let _ = reply.send(Err(format!("DID not found: {}", did)));
                }
            },
            NodeCommand::AttestTrust {
                subject_did,
                score,
                category: _,
                reply,
            } => {
                let _ = trust_graph.add_edge(node_did, &subject_did, score);
                let _ = reply.send(Ok(TrustResponse {
                    subject_did,
                    score,
                    status: "attested".into(),
                }));
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
        let config = VeritasConfig::default();
        let node = VeritasFullNode::new(config);
        assert!(node.is_ok());
    }

    #[test]
    fn test_full_node_peer_id() {
        let config = VeritasConfig::default();
        let node = VeritasFullNode::new(config).unwrap();
        let peer_id = node.peer_id();
        assert!(!peer_id.to_string().is_empty());
    }

    #[test]
    fn test_full_node_did() {
        let config = VeritasConfig::default();
        let node = VeritasFullNode::new(config).unwrap();
        assert!(node.did().starts_with("did:veritas:key:"));
    }

    #[test]
    fn test_full_node_has_did_manager() {
        let config = VeritasConfig::default();
        let node = VeritasFullNode::new(config).unwrap();
        assert!(node.did_manager().is_empty());
    }

    #[test]
    fn test_full_node_has_schema_registry() {
        let config = VeritasConfig::default();
        let node = VeritasFullNode::new(config).unwrap();
        // Schema registry has built-in schemas
        assert!(!node.schema_registry().list().is_empty());
    }

    #[tokio::test]
    async fn test_full_node_start_and_shutdown() {
        let dir = std::env::temp_dir().join(format!("veritas-node-test-{}", rand::random::<u64>()));
        let mut config = VeritasConfig::default();
        config.storage.data_dir = dir.clone();
        let mut node = VeritasFullNode::new(config).unwrap();
        node.start().await.expect("start failed");
        node.shutdown().await.expect("shutdown failed");
        std::fs::remove_dir_all(&dir).ok();
    }
}
