//! GPPN request/response protocol types for direct peer-to-peer messaging.
//!
//! Defines the typed messages exchanged between GPPN peers over the
//! request-response protocol. Serialization is handled by the CBOR codec
//! provided by libp2p.

use libp2p::StreamProtocol;
use serde::{Deserialize, Serialize};

/// The GPPN protocol identifier.
pub const GPPN_PROTOCOL: StreamProtocol = StreamProtocol::new("/gppn/req/1.0.0");

/// Request message sent between GPPN peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GppnRequest {
    /// Request a route to a target DID.
    RouteRequest {
        request_id: String,
        target_did: String,
        source_currency: String,
        destination_currency: String,
        amount: u128,
        max_hops: u32,
    },
    /// Send a direct payment message to a peer.
    PaymentMessage {
        /// Serialized protobuf PaymentMessage bytes.
        data: Vec<u8>,
    },
    /// Ping to check peer liveness.
    Ping,
}

/// Response message sent back to the requester.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GppnResponse {
    /// Route response with path information.
    RouteResponse {
        request_id: String,
        found: bool,
        path: Vec<String>,
        estimated_fee: u128,
        hop_count: u32,
    },
    /// Acknowledgement of a payment message.
    PaymentAck {
        accepted: bool,
        reason: Option<String>,
    },
    /// Pong reply.
    Pong,
    /// Error response.
    Error { message: String },
}

/// Codec marker type (kept for re-export compatibility).
#[derive(Debug, Clone, Default)]
pub struct GppnCodec;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gppn_protocol_name() {
        assert_eq!(GPPN_PROTOCOL.as_ref(), "/gppn/req/1.0.0");
    }

    #[test]
    fn test_request_route_serde() {
        let req = GppnRequest::RouteRequest {
            request_id: "r1".into(),
            target_did: "did:gppn:key:bob".into(),
            source_currency: "BRL".into(),
            destination_currency: "USD".into(),
            amount: 100_000,
            max_hops: 5,
        };
        let json = serde_json::to_vec(&req).expect("serialize");
        let decoded: GppnRequest = serde_json::from_slice(&json).expect("deserialize");
        match decoded {
            GppnRequest::RouteRequest {
                request_id,
                max_hops,
                ..
            } => {
                assert_eq!(request_id, "r1");
                assert_eq!(max_hops, 5);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_request_payment_message_serde() {
        let req = GppnRequest::PaymentMessage {
            data: vec![1, 2, 3, 4, 5],
        };
        let json = serde_json::to_vec(&req).expect("serialize");
        let decoded: GppnRequest = serde_json::from_slice(&json).expect("deserialize");
        match decoded {
            GppnRequest::PaymentMessage { data } => {
                assert_eq!(data, vec![1, 2, 3, 4, 5]);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_request_ping_serde() {
        let req = GppnRequest::Ping;
        let json = serde_json::to_vec(&req).expect("serialize");
        let decoded: GppnRequest = serde_json::from_slice(&json).expect("deserialize");
        assert!(matches!(decoded, GppnRequest::Ping));
    }

    #[test]
    fn test_response_route_serde() {
        let resp = GppnResponse::RouteResponse {
            request_id: "r1".into(),
            found: true,
            path: vec!["a".into(), "b".into(), "c".into()],
            estimated_fee: 42,
            hop_count: 2,
        };
        let json = serde_json::to_vec(&resp).expect("serialize");
        let decoded: GppnResponse = serde_json::from_slice(&json).expect("deserialize");
        match decoded {
            GppnResponse::RouteResponse {
                found, hop_count, ..
            } => {
                assert!(found);
                assert_eq!(hop_count, 2);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_response_pong_serde() {
        let resp = GppnResponse::Pong;
        let json = serde_json::to_vec(&resp).expect("serialize");
        let decoded: GppnResponse = serde_json::from_slice(&json).expect("deserialize");
        assert!(matches!(decoded, GppnResponse::Pong));
    }

    #[test]
    fn test_response_payment_ack_serde() {
        let resp = GppnResponse::PaymentAck {
            accepted: false,
            reason: Some("insufficient balance".into()),
        };
        let json = serde_json::to_vec(&resp).expect("serialize");
        let decoded: GppnResponse = serde_json::from_slice(&json).expect("deserialize");
        match decoded {
            GppnResponse::PaymentAck { accepted, reason } => {
                assert!(!accepted);
                assert_eq!(reason.as_deref(), Some("insufficient balance"));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_response_error_serde() {
        let resp = GppnResponse::Error {
            message: "not found".into(),
        };
        let json = serde_json::to_vec(&resp).expect("serialize");
        let decoded: GppnResponse = serde_json::from_slice(&json).expect("deserialize");
        match decoded {
            GppnResponse::Error { message } => {
                assert_eq!(message, "not found");
            }
            _ => panic!("wrong variant"),
        }
    }
}
