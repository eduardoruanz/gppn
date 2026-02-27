//! Network event types for the GPPN P2P layer.
//!
//! These events are emitted by the network layer to the application layer
//! so that higher-level components can react to network activity.

use libp2p::PeerId;
use serde::{Deserialize, Serialize};

/// High-level events emitted by the GPPN network layer.
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// A new peer connected to this node.
    PeerConnected(PeerConnected),

    /// A peer disconnected from this node.
    PeerDisconnected(PeerDisconnected),

    /// An incoming payment message was received via GossipSub.
    IncomingPaymentMessage(IncomingPaymentMessage),

    /// A route request was received from a peer.
    RouteRequest(RouteRequest),

    /// A route response was received from a peer.
    RouteResponse(RouteResponse),

    /// A direct message was received from a peer.
    DirectMessage(DirectMessage),

    /// This node started listening on an address.
    Listening {
        /// The address we are now listening on.
        address: libp2p::Multiaddr,
    },
}

/// Emitted when a new peer connects.
#[derive(Debug, Clone)]
pub struct PeerConnected {
    /// The PeerId of the connected peer.
    pub peer_id: PeerId,
    /// Number of currently connected peers (including this one).
    pub num_connected: usize,
}

/// Emitted when a peer disconnects.
#[derive(Debug, Clone)]
pub struct PeerDisconnected {
    /// The PeerId of the disconnected peer.
    pub peer_id: PeerId,
    /// Number of remaining connected peers.
    pub num_connected: usize,
}

/// A payment message received over the gossipsub network.
#[derive(Debug, Clone)]
pub struct IncomingPaymentMessage {
    /// The PeerId of the gossipsub source (propagating peer), if available.
    pub source: Option<PeerId>,
    /// Raw serialized PaymentMessage bytes (protobuf).
    pub data: Vec<u8>,
    /// The gossipsub topic the message arrived on.
    pub topic: String,
}

/// A route request received from a peer via direct messaging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRequest {
    /// Unique request identifier.
    pub request_id: String,
    /// The DID of the target receiver.
    pub target_did: String,
    /// Source currency code.
    pub source_currency: String,
    /// Destination currency code.
    pub destination_currency: String,
    /// Amount in atomic units.
    pub amount: u128,
    /// Maximum allowed hops.
    pub max_hops: u32,
}

/// A route response sent back to the requester.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResponse {
    /// The request identifier this is responding to.
    pub request_id: String,
    /// Whether a route was found.
    pub found: bool,
    /// The route as a list of peer DIDs (empty if not found).
    pub path: Vec<String>,
    /// Estimated total fee in atomic units (0 if no route).
    pub estimated_fee: u128,
    /// Number of hops in the route.
    pub hop_count: u32,
}

/// A direct message from a specific peer (request-response protocol).
#[derive(Debug, Clone)]
pub struct DirectMessage {
    /// The peer who sent the message.
    pub peer_id: PeerId,
    /// Raw message bytes.
    pub data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_connected_event() {
        let peer_id = PeerId::random();
        let event = PeerConnected {
            peer_id,
            num_connected: 5,
        };
        assert_eq!(event.peer_id, peer_id);
        assert_eq!(event.num_connected, 5);
    }

    #[test]
    fn test_peer_disconnected_event() {
        let peer_id = PeerId::random();
        let event = PeerDisconnected {
            peer_id,
            num_connected: 3,
        };
        assert_eq!(event.peer_id, peer_id);
        assert_eq!(event.num_connected, 3);
    }

    #[test]
    fn test_incoming_payment_message() {
        let source = PeerId::random();
        let msg = IncomingPaymentMessage {
            source: Some(source),
            data: vec![1, 2, 3, 4],
            topic: "gppn/payments/v1".into(),
        };
        assert_eq!(msg.source, Some(source));
        assert_eq!(msg.data.len(), 4);
        assert_eq!(msg.topic, "gppn/payments/v1");
    }

    #[test]
    fn test_route_request() {
        let req = RouteRequest {
            request_id: "req-001".into(),
            target_did: "did:gppn:key:bob456".into(),
            source_currency: "BRL".into(),
            destination_currency: "USD".into(),
            amount: 100_000,
            max_hops: 5,
        };
        assert_eq!(req.request_id, "req-001");
        assert_eq!(req.max_hops, 5);
    }

    #[test]
    fn test_route_response_found() {
        let resp = RouteResponse {
            request_id: "req-001".into(),
            found: true,
            path: vec![
                "did:gppn:key:alice".into(),
                "did:gppn:key:relay".into(),
                "did:gppn:key:bob".into(),
            ],
            estimated_fee: 50,
            hop_count: 2,
        };
        assert!(resp.found);
        assert_eq!(resp.path.len(), 3);
        assert_eq!(resp.hop_count, 2);
    }

    #[test]
    fn test_route_response_not_found() {
        let resp = RouteResponse {
            request_id: "req-002".into(),
            found: false,
            path: vec![],
            estimated_fee: 0,
            hop_count: 0,
        };
        assert!(!resp.found);
        assert!(resp.path.is_empty());
    }

    #[test]
    fn test_direct_message() {
        let peer_id = PeerId::random();
        let msg = DirectMessage {
            peer_id,
            data: vec![10, 20, 30],
        };
        assert_eq!(msg.peer_id, peer_id);
        assert_eq!(msg.data, vec![10, 20, 30]);
    }

    #[test]
    fn test_network_event_variants() {
        let peer_id = PeerId::random();

        // Verify we can construct all variants
        let _e1 = NetworkEvent::PeerConnected(PeerConnected {
            peer_id,
            num_connected: 1,
        });
        let _e2 = NetworkEvent::PeerDisconnected(PeerDisconnected {
            peer_id,
            num_connected: 0,
        });
        let _e3 = NetworkEvent::IncomingPaymentMessage(IncomingPaymentMessage {
            source: None,
            data: vec![],
            topic: "test".into(),
        });
        let _e4 = NetworkEvent::RouteRequest(RouteRequest {
            request_id: "r1".into(),
            target_did: "did:gppn:key:x".into(),
            source_currency: "USD".into(),
            destination_currency: "EUR".into(),
            amount: 0,
            max_hops: 1,
        });
        let _e5 = NetworkEvent::RouteResponse(RouteResponse {
            request_id: "r1".into(),
            found: false,
            path: vec![],
            estimated_fee: 0,
            hop_count: 0,
        });
        let _e6 = NetworkEvent::DirectMessage(DirectMessage {
            peer_id,
            data: vec![],
        });
    }

    #[test]
    fn test_route_request_serde_roundtrip() {
        let req = RouteRequest {
            request_id: "req-serde".into(),
            target_did: "did:gppn:key:bob".into(),
            source_currency: "BRL".into(),
            destination_currency: "USD".into(),
            amount: 999_999,
            max_hops: 10,
        };
        let json = serde_json::to_string(&req).expect("serialize failed");
        let deserialized: RouteRequest =
            serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(deserialized.request_id, req.request_id);
        assert_eq!(deserialized.amount, req.amount);
    }

    #[test]
    fn test_route_response_serde_roundtrip() {
        let resp = RouteResponse {
            request_id: "resp-serde".into(),
            found: true,
            path: vec!["a".into(), "b".into()],
            estimated_fee: 42,
            hop_count: 1,
        };
        let json = serde_json::to_string(&resp).expect("serialize failed");
        let deserialized: RouteResponse =
            serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(deserialized.request_id, resp.request_id);
        assert!(deserialized.found);
        assert_eq!(deserialized.hop_count, 1);
    }
}
