//! The GPPN network node.
//!
//! `GppnNode` is the main entry point for the P2P networking layer.
//! It owns the libp2p `Swarm`, manages subscriptions, handles incoming events,
//! and exposes a high-level API for the application layer.

use futures::StreamExt;
use libp2p::identity::Keypair;
use libp2p::swarm::SwarmEvent;
use libp2p::{gossipsub, identify, kad, mdns, Multiaddr, PeerId, Swarm};
use std::collections::HashSet;
use std::str::FromStr;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::behaviour::{GppnBehaviour, GppnBehaviourEvent};
use crate::discovery::PeerDiscovery;
use crate::error::NetworkError;
use crate::events::{
    DirectMessage, IncomingPaymentMessage, NetworkEvent, PeerConnected, PeerDisconnected,
    RouteRequest, RouteResponse,
};
use crate::gossip::TopicManager;
use crate::transport;

/// Configuration for the GppnNode.
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// The listen address, e.g. "/ip4/0.0.0.0/tcp/9000".
    pub listen_addr: String,
    /// Bootstrap peer multiaddresses.
    pub bootstrap_peers: Vec<String>,
    /// Broadcast channel capacity for network events.
    pub event_channel_capacity: usize,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/0".into(),
            bootstrap_peers: Vec::new(),
            event_channel_capacity: 256,
        }
    }
}

/// Commands that can be sent to the network event loop from external tasks.
pub enum NetworkCommand {
    /// Broadcast a payment message via gossipsub.
    BroadcastPayment {
        data: Vec<u8>,
        reply: oneshot::Sender<Result<(), NetworkError>>,
    },
    /// Send a direct request to a specific peer.
    SendDirectRequest {
        peer_id: PeerId,
        data: Vec<u8>,
        reply: oneshot::Sender<Result<(), NetworkError>>,
    },
}

/// The GPPN P2P network node.
///
/// Manages the libp2p swarm, topic subscriptions, peer discovery,
/// and event propagation to the application layer.
pub struct GppnNode {
    /// The libp2p keypair for this node.
    keypair: Keypair,
    /// Our local PeerId.
    local_peer_id: PeerId,
    /// Node configuration.
    config: NodeConfig,
    /// The topic manager for GossipSub.
    topic_manager: TopicManager,
    /// Peer discovery state.
    discovery: PeerDiscovery,
    /// The libp2p swarm (set after start).
    swarm: Option<Swarm<GppnBehaviour>>,
    /// Broadcast sender for network events.
    event_tx: broadcast::Sender<NetworkEvent>,
    /// Shutdown signal sender.
    shutdown_tx: Option<mpsc::Sender<()>>,
    /// Connected peers tracking.
    connected_peers: HashSet<PeerId>,
    /// Command sender (cloneable, handed out via `command_sender()`).
    command_tx: mpsc::Sender<NetworkCommand>,
    /// Command receiver for external tasks to request actions on the swarm.
    command_rx: Option<mpsc::Receiver<NetworkCommand>>,
}

impl GppnNode {
    /// Create a new GppnNode with the given keypair and config.
    pub fn new(keypair: Keypair, config: NodeConfig) -> Result<Self, NetworkError> {
        let local_peer_id = PeerId::from(keypair.public());
        let topic_manager = TopicManager::new();
        let discovery = PeerDiscovery::new(config.bootstrap_peers.clone())?;
        let (event_tx, _) = broadcast::channel(config.event_channel_capacity);
        let (command_tx, command_rx) = mpsc::channel(256);

        tracing::info!(%local_peer_id, "creating GPPN node");

        Ok(Self {
            keypair,
            local_peer_id,
            config,
            topic_manager,
            discovery,
            swarm: None,
            event_tx,
            shutdown_tx: None,
            connected_peers: HashSet::new(),
            command_tx,
            command_rx: Some(command_rx),
        })
    }

    /// Get the local PeerId.
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }

    /// Get a receiver for network events.
    pub fn event_receiver(&self) -> broadcast::Receiver<NetworkEvent> {
        self.event_tx.subscribe()
    }

    /// Get the topic manager.
    pub fn topic_manager(&self) -> &TopicManager {
        &self.topic_manager
    }

    /// Get the peer discovery state.
    pub fn discovery(&self) -> &PeerDiscovery {
        &self.discovery
    }

    /// Get the set of currently connected peers.
    pub fn connected_peers(&self) -> &HashSet<PeerId> {
        &self.connected_peers
    }

    /// Get the number of currently connected peers.
    pub fn connected_peer_count(&self) -> usize {
        self.connected_peers.len()
    }

    /// Check if the node's swarm has been started.
    pub fn is_running(&self) -> bool {
        self.swarm.is_some()
    }

    /// Get a command sender that can be used from other tasks to
    /// request actions on the swarm (e.g., broadcasting a payment).
    pub fn command_sender(&self) -> mpsc::Sender<NetworkCommand> {
        self.command_tx.clone()
    }

    /// Start the node: build the swarm, listen on the configured address,
    /// subscribe to topics, and begin the event loop.
    ///
    /// Returns the actual listen address (useful when port is 0).
    pub async fn start(&mut self) -> Result<Multiaddr, NetworkError> {
        if self.swarm.is_some() {
            return Err(NetworkError::AlreadyRunning);
        }

        tracing::info!(
            listen_addr = %self.config.listen_addr,
            peer_id = %self.local_peer_id,
            "starting GPPN node"
        );

        // Build the swarm
        let mut swarm = transport::build_swarm(self.keypair.clone(), |key| {
            GppnBehaviour::new(key)
        })?;

        // Subscribe to all GPPN topics
        self.topic_manager
            .subscribe_all(&mut swarm.behaviour_mut().gossipsub)?;

        // Start listening
        let listen_addr = Multiaddr::from_str(&self.config.listen_addr)
            .map_err(|e| NetworkError::Listen(format!("invalid listen address: {}", e)))?;

        swarm
            .listen_on(listen_addr)
            .map_err(|e| NetworkError::Listen(e.to_string()))?;

        // Dial bootstrap peers
        for addr in self.discovery.bootstrap_addrs() {
            tracing::info!(addr = %addr, "dialing bootstrap peer");
            if let Err(e) = swarm.dial(addr.clone()) {
                tracing::warn!(addr = %addr, error = %e, "failed to dial bootstrap peer");
            }
        }

        // Bootstrap Kademlia if we have bootstrap peers
        if !self.discovery.bootstrap_addrs().is_empty() {
            if let Err(e) = swarm.behaviour_mut().kademlia.bootstrap() {
                tracing::warn!(error = %e, "kademlia bootstrap failed");
            }
        }

        self.swarm = Some(swarm);

        // Determine actual listen address (port may have been assigned)
        let actual_addr = Multiaddr::from_str(&self.config.listen_addr)
            .unwrap_or_else(|_| Multiaddr::empty());

        Ok(actual_addr)
    }

    /// Run the event loop. This should be called in a tokio::spawn after start().
    ///
    /// Processes incoming swarm events, external commands, and emits high-level NetworkEvents.
    /// Returns when the shutdown signal is received or the command channel closes.
    pub async fn run(&mut self) -> Result<(), NetworkError> {
        if self.swarm.is_none() {
            return Err(NetworkError::NotStarted);
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let mut command_rx = self.command_rx.take()
            .ok_or(NetworkError::NotStarted)?;

        tracing::info!(peer_id = %self.local_peer_id, "GPPN node event loop started");

        enum Action {
            SwarmEvent(SwarmEvent<GppnBehaviourEvent>),
            Command(NetworkCommand),
            Shutdown,
            CommandChannelClosed,
        }

        loop {
            let action = {
                let swarm = match self.swarm.as_mut() {
                    Some(s) => s,
                    None => break,
                };
                tokio::select! {
                    event = swarm.select_next_some() => Action::SwarmEvent(event),
                    cmd = command_rx.recv() => match cmd {
                        Some(c) => Action::Command(c),
                        None => Action::CommandChannelClosed,
                    },
                    _ = shutdown_rx.recv() => Action::Shutdown,
                }
            };

            match action {
                Action::SwarmEvent(event) => self.handle_swarm_event(event),
                Action::Command(cmd) => self.handle_command(cmd),
                Action::Shutdown => {
                    tracing::info!("GPPN node shutting down (signal)");
                    break;
                }
                Action::CommandChannelClosed => {
                    tracing::info!("GPPN node shutting down (command channel closed)");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Stop the node gracefully.
    pub async fn stop(&mut self) -> Result<(), NetworkError> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        self.swarm = None;
        self.connected_peers.clear();
        tracing::info!(peer_id = %self.local_peer_id, "GPPN node stopped");
        Ok(())
    }

    /// Broadcast a serialized payment message over GossipSub.
    pub fn broadcast_pm(&mut self, data: Vec<u8>) -> Result<(), NetworkError> {
        let swarm = self.swarm.as_mut().ok_or(NetworkError::NotStarted)?;
        let topic = &self.topic_manager.payments;

        swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic.clone(), data)
            .map_err(|e| NetworkError::Gossipsub(format!("publish failed: {}", e)))?;

        tracing::debug!("broadcast payment message to gossipsub");
        Ok(())
    }

    /// Publish a message to a specific gossipsub topic.
    pub fn publish(
        &mut self,
        topic: &gossipsub::IdentTopic,
        data: Vec<u8>,
    ) -> Result<(), NetworkError> {
        let swarm = self.swarm.as_mut().ok_or(NetworkError::NotStarted)?;

        swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic.clone(), data)
            .map_err(|e| NetworkError::Gossipsub(format!("publish failed: {}", e)))?;

        Ok(())
    }

    /// Send a direct request to a peer.
    pub fn send_request(
        &mut self,
        peer_id: &PeerId,
        request: crate::protocol::GppnRequest,
    ) -> Result<libp2p::request_response::OutboundRequestId, NetworkError> {
        let swarm = self.swarm.as_mut().ok_or(NetworkError::NotStarted)?;

        let request_id = swarm
            .behaviour_mut()
            .request_response
            .send_request(peer_id, request);

        Ok(request_id)
    }

    /// Handle a command from an external task (e.g., HTTP API).
    fn handle_command(&mut self, cmd: NetworkCommand) {
        match cmd {
            NetworkCommand::BroadcastPayment { data, reply } => {
                let result = self.broadcast_pm(data);
                let _ = reply.send(result);
            }
            NetworkCommand::SendDirectRequest { peer_id, data, reply } => {
                let request = crate::protocol::GppnRequest::PaymentMessage { data };
                let result = self.send_request(&peer_id, request).map(|_| ());
                let _ = reply.send(result);
            }
        }
    }

    /// Handle a swarm event dispatched from the event loop.
    fn handle_swarm_event(&mut self, event: SwarmEvent<GppnBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(behaviour_event) => {
                self.handle_behaviour_event(behaviour_event);
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                num_established,
                ..
            } => {
                self.connected_peers.insert(peer_id);
                tracing::info!(
                    %peer_id,
                    num_established,
                    total_connected = self.connected_peers.len(),
                    "connection established"
                );
                let _ = self.event_tx.send(NetworkEvent::PeerConnected(PeerConnected {
                    peer_id,
                    num_connected: self.connected_peers.len(),
                }));
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                num_established,
                ..
            } => {
                if num_established == 0 {
                    self.connected_peers.remove(&peer_id);
                }
                tracing::info!(
                    %peer_id,
                    num_established,
                    total_connected = self.connected_peers.len(),
                    "connection closed"
                );
                let _ = self.event_tx.send(NetworkEvent::PeerDisconnected(
                    PeerDisconnected {
                        peer_id,
                        num_connected: self.connected_peers.len(),
                    },
                ));
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                tracing::info!(address = %address, "listening on new address");
                let _ = self
                    .event_tx
                    .send(NetworkEvent::Listening { address });
            }
            SwarmEvent::IncomingConnection { .. } => {
                tracing::debug!("incoming connection");
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                tracing::warn!(
                    ?peer_id,
                    error = %error,
                    "outgoing connection error"
                );
            }
            SwarmEvent::IncomingConnectionError { error, .. } => {
                tracing::warn!(error = %error, "incoming connection error");
            }
            SwarmEvent::ListenerClosed { .. } => {
                tracing::info!("listener closed");
            }
            SwarmEvent::ListenerError { error, .. } => {
                tracing::error!(error = %error, "listener error");
            }
            _ => {}
        }
    }

    /// Handle a behaviour-level event from one of the sub-behaviours.
    fn handle_behaviour_event(&mut self, event: GppnBehaviourEvent) {
        match event {
            GppnBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source,
                message,
                ..
            }) => {
                let topic_str = self
                    .topic_manager
                    .topic_name_from_hash(&message.topic)
                    .unwrap_or("unknown")
                    .to_string();

                tracing::debug!(
                    source = %propagation_source,
                    topic = %topic_str,
                    bytes = message.data.len(),
                    "gossipsub message received"
                );

                if message.topic == self.topic_manager.payments_hash() {
                    let _ = self.event_tx.send(NetworkEvent::IncomingPaymentMessage(
                        IncomingPaymentMessage {
                            source: message.source,
                            data: message.data,
                            topic: topic_str,
                        },
                    ));
                }
            }
            GppnBehaviourEvent::Gossipsub(gossipsub::Event::Subscribed { peer_id, topic }) => {
                tracing::debug!(%peer_id, %topic, "peer subscribed to topic");
            }
            GppnBehaviourEvent::Gossipsub(gossipsub::Event::Unsubscribed { peer_id, topic }) => {
                tracing::debug!(%peer_id, %topic, "peer unsubscribed from topic");
            }
            GppnBehaviourEvent::Gossipsub(_) => {}

            GppnBehaviourEvent::Kademlia(kad::Event::RoutingUpdated {
                peer, is_new_peer, ..
            }) => {
                tracing::debug!(%peer, is_new_peer, "kademlia routing updated");
                self.discovery.add_kad_peer(peer);
            }
            GppnBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {
                result, ..
            }) => {
                tracing::debug!(?result, "kademlia query progressed");
            }
            GppnBehaviourEvent::Kademlia(_) => {}

            GppnBehaviourEvent::Mdns(mdns::Event::Discovered(peers)) => {
                let mut swarm = self.swarm.as_mut();
                for (peer_id, addr) in peers {
                    tracing::debug!(%peer_id, %addr, "mDNS peer discovered");
                    self.discovery.add_mdns_peer(peer_id);
                    if let Some(ref mut s) = swarm {
                        s.behaviour_mut()
                            .gossipsub
                            .add_explicit_peer(&peer_id);
                        s.behaviour_mut()
                            .kademlia
                            .add_address(&peer_id, addr);
                    }
                }
            }
            GppnBehaviourEvent::Mdns(mdns::Event::Expired(peers)) => {
                for (peer_id, _addr) in peers {
                    tracing::debug!(%peer_id, "mDNS peer expired");
                    self.discovery.remove_mdns_peer(&peer_id);
                }
            }

            GppnBehaviourEvent::Identify(identify::Event::Received {
                peer_id, info, ..
            }) => {
                tracing::debug!(
                    %peer_id,
                    protocol_version = %info.protocol_version,
                    agent_version = %info.agent_version,
                    "identify: received peer info"
                );
                // Add identified peer addresses to Kademlia
                if let Some(ref mut swarm) = self.swarm {
                    for addr in &info.listen_addrs {
                        swarm
                            .behaviour_mut()
                            .kademlia
                            .add_address(&peer_id, addr.clone());
                    }
                }
            }
            GppnBehaviourEvent::Identify(_) => {}

            GppnBehaviourEvent::RequestResponse(
                libp2p::request_response::Event::Message { peer, message },
            ) => {
                match message {
                    libp2p::request_response::Message::Request {
                        request, channel, ..
                    } => {
                        self.handle_incoming_request(peer, request, channel);
                    }
                    libp2p::request_response::Message::Response {
                        response, ..
                    } => {
                        self.handle_incoming_response(peer, response);
                    }
                }
            }
            GppnBehaviourEvent::RequestResponse(
                libp2p::request_response::Event::OutboundFailure {
                    peer, error, ..
                },
            ) => {
                tracing::warn!(%peer, error = %error, "outbound request failed");
            }
            GppnBehaviourEvent::RequestResponse(
                libp2p::request_response::Event::InboundFailure {
                    peer, error, ..
                },
            ) => {
                tracing::warn!(%peer, error = %error, "inbound request failed");
            }
            GppnBehaviourEvent::RequestResponse(_) => {}
        }
    }

    /// Handle an incoming request from the request-response protocol.
    fn handle_incoming_request(
        &mut self,
        peer: PeerId,
        request: crate::protocol::GppnRequest,
        channel: libp2p::request_response::ResponseChannel<crate::protocol::GppnResponse>,
    ) {
        tracing::debug!(%peer, "incoming request");

        match request {
            crate::protocol::GppnRequest::RouteRequest {
                request_id,
                target_did,
                source_currency,
                destination_currency,
                amount,
                max_hops,
            } => {
                let _ = self.event_tx.send(NetworkEvent::RouteRequest(RouteRequest {
                    request_id: request_id.clone(),
                    target_did,
                    source_currency,
                    destination_currency,
                    amount,
                    max_hops,
                }));

                // Send a default "not found" response; the application layer
                // should handle the RouteRequest event and send a proper response.
                let response = crate::protocol::GppnResponse::RouteResponse {
                    request_id,
                    found: false,
                    path: vec![],
                    estimated_fee: 0,
                    hop_count: 0,
                };
                if let Some(ref mut swarm) = self.swarm {
                    let _ = swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, response);
                }
            }
            crate::protocol::GppnRequest::PaymentMessage { data } => {
                let _ = self.event_tx.send(NetworkEvent::DirectMessage(DirectMessage {
                    peer_id: peer,
                    data,
                }));

                let response = crate::protocol::GppnResponse::PaymentAck {
                    accepted: true,
                    reason: None,
                };
                if let Some(ref mut swarm) = self.swarm {
                    let _ = swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, response);
                }
            }
            crate::protocol::GppnRequest::Ping => {
                if let Some(ref mut swarm) = self.swarm {
                    let _ = swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, crate::protocol::GppnResponse::Pong);
                }
            }
        }
    }

    /// Handle an incoming response from the request-response protocol.
    fn handle_incoming_response(
        &self,
        peer: PeerId,
        response: crate::protocol::GppnResponse,
    ) {
        tracing::debug!(%peer, "incoming response");

        match response {
            crate::protocol::GppnResponse::RouteResponse {
                request_id,
                found,
                path,
                estimated_fee,
                hop_count,
            } => {
                let _ = self.event_tx.send(NetworkEvent::RouteResponse(RouteResponse {
                    request_id,
                    found,
                    path,
                    estimated_fee,
                    hop_count,
                }));
            }
            crate::protocol::GppnResponse::PaymentAck { accepted, reason } => {
                tracing::debug!(
                    %peer,
                    accepted,
                    ?reason,
                    "payment ack received"
                );
            }
            crate::protocol::GppnResponse::Pong => {
                tracing::debug!(%peer, "pong received");
            }
            crate::protocol::GppnResponse::Error { message } => {
                tracing::warn!(%peer, error = %message, "error response received");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_keypair() -> Keypair {
        Keypair::generate_ed25519()
    }

    #[test]
    fn test_node_creation() {
        let keypair = make_keypair();
        let config = NodeConfig::default();
        let node = GppnNode::new(keypair, config);
        assert!(node.is_ok());
    }

    #[test]
    fn test_node_local_peer_id() {
        let keypair = make_keypair();
        let expected_peer_id = PeerId::from(keypair.public());
        let config = NodeConfig::default();
        let node = GppnNode::new(keypair, config).expect("node creation");
        assert_eq!(*node.local_peer_id(), expected_peer_id);
    }

    #[test]
    fn test_node_not_running_initially() {
        let keypair = make_keypair();
        let config = NodeConfig::default();
        let node = GppnNode::new(keypair, config).expect("node creation");
        assert!(!node.is_running());
    }

    #[test]
    fn test_node_connected_peers_empty() {
        let keypair = make_keypair();
        let config = NodeConfig::default();
        let node = GppnNode::new(keypair, config).expect("node creation");
        assert!(node.connected_peers().is_empty());
        assert_eq!(node.connected_peer_count(), 0);
    }

    #[test]
    fn test_node_topic_manager() {
        let keypair = make_keypair();
        let config = NodeConfig::default();
        let node = GppnNode::new(keypair, config).expect("node creation");
        let tm = node.topic_manager();
        assert_eq!(tm.all_topics().len(), 4);
    }

    #[test]
    fn test_node_event_receiver() {
        let keypair = make_keypair();
        let config = NodeConfig::default();
        let node = GppnNode::new(keypair, config).expect("node creation");
        let _rx = node.event_receiver();
        // Just verify we can obtain a receiver without panic
    }

    #[test]
    fn test_broadcast_pm_before_start() {
        let keypair = make_keypair();
        let config = NodeConfig::default();
        let mut node = GppnNode::new(keypair, config).expect("node creation");
        let result = node.broadcast_pm(vec![1, 2, 3]);
        assert!(matches!(result, Err(NetworkError::NotStarted)));
    }

    #[test]
    fn test_send_request_before_start() {
        let keypair = make_keypair();
        let config = NodeConfig::default();
        let mut node = GppnNode::new(keypair, config).expect("node creation");
        let peer = PeerId::random();
        let result = node.send_request(&peer, crate::protocol::GppnRequest::Ping);
        assert!(matches!(result, Err(NetworkError::NotStarted)));
    }

    #[test]
    fn test_node_config_default() {
        let config = NodeConfig::default();
        assert_eq!(config.listen_addr, "/ip4/0.0.0.0/tcp/0");
        assert!(config.bootstrap_peers.is_empty());
        assert_eq!(config.event_channel_capacity, 256);
    }

    #[test]
    fn test_node_config_custom() {
        let config = NodeConfig {
            listen_addr: "/ip4/127.0.0.1/tcp/9000".into(),
            bootstrap_peers: vec!["/ip4/1.2.3.4/tcp/9000".into()],
            event_channel_capacity: 512,
        };
        assert_eq!(config.listen_addr, "/ip4/127.0.0.1/tcp/9000");
        assert_eq!(config.bootstrap_peers.len(), 1);
        assert_eq!(config.event_channel_capacity, 512);
    }

    #[tokio::test]
    async fn test_node_start_and_stop() {
        let keypair = make_keypair();
        let config = NodeConfig::default();
        let mut node = GppnNode::new(keypair, config).expect("node creation");

        let result = node.start().await;
        assert!(result.is_ok(), "start failed: {:?}", result.err());
        assert!(node.is_running());

        // Starting again should fail
        let result2 = node.start().await;
        assert!(matches!(result2, Err(NetworkError::AlreadyRunning)));

        let stop_result = node.stop().await;
        assert!(stop_result.is_ok());
        assert!(!node.is_running());
    }

    #[tokio::test]
    async fn test_node_broadcast_after_start() {
        let keypair = make_keypair();
        let config = NodeConfig::default();
        let mut node = GppnNode::new(keypair, config).expect("node creation");

        node.start().await.expect("start failed");

        // Broadcasting with no peers subscribed will fail with InsufficientPeers,
        // but it should not return NotStarted.
        let result = node.broadcast_pm(vec![1, 2, 3]);
        assert!(!matches!(result, Err(NetworkError::NotStarted)));

        node.stop().await.expect("stop failed");
    }

    #[tokio::test]
    async fn test_node_stop_when_not_started() {
        let keypair = make_keypair();
        let config = NodeConfig::default();
        let mut node = GppnNode::new(keypair, config).expect("node creation");

        // Stopping when not started should not error
        let result = node.stop().await;
        assert!(result.is_ok());
    }
}
