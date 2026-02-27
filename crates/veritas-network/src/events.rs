//! Network event types for the Veritas P2P identity layer.
//!
//! These events are emitted by the network layer to the application layer
//! so that higher-level components can react to network activity.

use libp2p::PeerId;
use serde::{Deserialize, Serialize};

/// High-level events emitted by the Veritas network layer.
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// A new peer connected to this node.
    PeerConnected(PeerConnected),

    /// A peer disconnected from this node.
    PeerDisconnected(PeerDisconnected),

    /// An incoming credential was received via GossipSub.
    IncomingCredential(IncomingCredential),

    /// A proof request was received from a peer.
    ProofRequestEvent(ProofRequestEvent),

    /// A proof response was received from a peer.
    ProofResponseEvent(ProofResponseEvent),

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

/// A credential received over the gossipsub network.
#[derive(Debug, Clone)]
pub struct IncomingCredential {
    /// The PeerId of the gossipsub source (propagating peer), if available.
    pub source: Option<PeerId>,
    /// Raw serialized credential bytes.
    pub data: Vec<u8>,
    /// The gossipsub topic the message arrived on.
    pub topic: String,
}

/// A proof request received from a peer via direct messaging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofRequestEvent {
    /// Unique request identifier.
    pub request_id: String,
    /// Type of proof requested (e.g., "AgeProof", "ResidencyProof").
    pub proof_type: String,
    /// Serialized requirements.
    pub requirements: Vec<u8>,
    /// DID of the requesting verifier.
    pub verifier_did: String,
}

/// A proof response received from a peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofResponseEvent {
    /// The request identifier this is responding to.
    pub request_id: String,
    /// Whether the proof is valid.
    pub valid: bool,
    /// Serialized proof data.
    pub proof_data: Vec<u8>,
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
    fn test_incoming_credential() {
        let source = PeerId::random();
        let msg = IncomingCredential {
            source: Some(source),
            data: vec![1, 2, 3, 4],
            topic: "veritas/credentials/v1".into(),
        };
        assert_eq!(msg.source, Some(source));
        assert_eq!(msg.data.len(), 4);
        assert_eq!(msg.topic, "veritas/credentials/v1");
    }

    #[test]
    fn test_proof_request_event() {
        let req = ProofRequestEvent {
            request_id: "req-001".into(),
            proof_type: "AgeProof".into(),
            requirements: vec![18],
            verifier_did: "did:veritas:key:verifier".into(),
        };
        assert_eq!(req.request_id, "req-001");
        assert_eq!(req.proof_type, "AgeProof");
    }

    #[test]
    fn test_proof_response_event() {
        let resp = ProofResponseEvent {
            request_id: "req-001".into(),
            valid: true,
            proof_data: vec![1, 2, 3],
        };
        assert!(resp.valid);
        assert_eq!(resp.proof_data.len(), 3);
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

        let _e1 = NetworkEvent::PeerConnected(PeerConnected {
            peer_id,
            num_connected: 1,
        });
        let _e2 = NetworkEvent::PeerDisconnected(PeerDisconnected {
            peer_id,
            num_connected: 0,
        });
        let _e3 = NetworkEvent::IncomingCredential(IncomingCredential {
            source: None,
            data: vec![],
            topic: "test".into(),
        });
        let _e4 = NetworkEvent::ProofRequestEvent(ProofRequestEvent {
            request_id: "r1".into(),
            proof_type: "AgeProof".into(),
            requirements: vec![],
            verifier_did: "did:veritas:key:v".into(),
        });
        let _e5 = NetworkEvent::ProofResponseEvent(ProofResponseEvent {
            request_id: "r1".into(),
            valid: false,
            proof_data: vec![],
        });
        let _e6 = NetworkEvent::DirectMessage(DirectMessage {
            peer_id,
            data: vec![],
        });
    }

    #[test]
    fn test_proof_request_serde_roundtrip() {
        let req = ProofRequestEvent {
            request_id: "req-serde".into(),
            proof_type: "ResidencyProof".into(),
            requirements: vec![1, 2, 3],
            verifier_did: "did:veritas:key:bob".into(),
        };
        let json = serde_json::to_string(&req).expect("serialize failed");
        let deserialized: ProofRequestEvent =
            serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(deserialized.request_id, req.request_id);
        assert_eq!(deserialized.proof_type, req.proof_type);
    }

    #[test]
    fn test_proof_response_serde_roundtrip() {
        let resp = ProofResponseEvent {
            request_id: "resp-serde".into(),
            valid: true,
            proof_data: vec![10, 20],
        };
        let json = serde_json::to_string(&resp).expect("serialize failed");
        let deserialized: ProofResponseEvent =
            serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(deserialized.request_id, resp.request_id);
        assert!(deserialized.valid);
    }
}
