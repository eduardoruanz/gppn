//! Veritas request/response protocol types for direct peer-to-peer messaging.
//!
//! Defines the typed messages exchanged between Veritas peers over the
//! request-response protocol. Serialization is handled by the CBOR codec
//! provided by libp2p.

use libp2p::StreamProtocol;
use serde::{Deserialize, Serialize};

/// The Veritas protocol identifier.
pub const VERITAS_PROTOCOL: StreamProtocol = StreamProtocol::new("/veritas/req/1.0.0");

/// Request message sent between Veritas peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VeritasRequest {
    /// Request a credential from an issuer.
    CredentialRequest {
        request_id: String,
        subject_did: String,
        credential_type: String,
        claims: Vec<u8>,
    },
    /// Request a proof from a holder.
    ProofRequest {
        request_id: String,
        proof_type: String,
        requirements: Vec<u8>,
    },
    /// Resolve a DID document.
    DidResolve { did: String },
    /// Submit a trust attestation.
    TrustAttestation {
        subject_did: String,
        score: f64,
        category: String,
    },
    /// Ping to check peer liveness.
    Ping,
}

/// Response message sent back to the requester.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VeritasResponse {
    /// Credential response from issuer.
    CredentialResponse {
        request_id: String,
        issued: bool,
        credential_data: Vec<u8>,
        reason: Option<String>,
    },
    /// Proof response from holder.
    ProofResponse {
        request_id: String,
        valid: bool,
        proof_data: Vec<u8>,
    },
    /// DID document response.
    DidDocument {
        did: String,
        document_data: Vec<u8>,
        found: bool,
    },
    /// Trust update acknowledgement.
    TrustUpdate {
        accepted: bool,
        new_score: Option<f64>,
    },
    /// Pong reply.
    Pong,
    /// Error response.
    Error { message: String },
}

/// Codec marker type (kept for re-export compatibility).
#[derive(Debug, Clone, Default)]
pub struct VeritasCodec;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_name() {
        assert_eq!(VERITAS_PROTOCOL.as_ref(), "/veritas/req/1.0.0");
    }

    #[test]
    fn test_request_credential_serde() {
        let req = VeritasRequest::CredentialRequest {
            request_id: "cr1".into(),
            subject_did: "did:veritas:key:bob".into(),
            credential_type: "KycBasic".into(),
            claims: vec![1, 2, 3],
        };
        let json = serde_json::to_vec(&req).expect("serialize");
        let decoded: VeritasRequest = serde_json::from_slice(&json).expect("deserialize");
        match decoded {
            VeritasRequest::CredentialRequest {
                request_id,
                credential_type,
                ..
            } => {
                assert_eq!(request_id, "cr1");
                assert_eq!(credential_type, "KycBasic");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_request_proof_serde() {
        let req = VeritasRequest::ProofRequest {
            request_id: "pr1".into(),
            proof_type: "AgeProof".into(),
            requirements: vec![18],
        };
        let json = serde_json::to_vec(&req).expect("serialize");
        let decoded: VeritasRequest = serde_json::from_slice(&json).expect("deserialize");
        match decoded {
            VeritasRequest::ProofRequest {
                request_id,
                proof_type,
                ..
            } => {
                assert_eq!(request_id, "pr1");
                assert_eq!(proof_type, "AgeProof");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_request_did_resolve_serde() {
        let req = VeritasRequest::DidResolve {
            did: "did:veritas:key:abc".into(),
        };
        let json = serde_json::to_vec(&req).expect("serialize");
        let decoded: VeritasRequest = serde_json::from_slice(&json).expect("deserialize");
        match decoded {
            VeritasRequest::DidResolve { did } => {
                assert_eq!(did, "did:veritas:key:abc");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_request_trust_attestation_serde() {
        let req = VeritasRequest::TrustAttestation {
            subject_did: "did:veritas:key:bob".into(),
            score: 0.85,
            category: "identity".into(),
        };
        let json = serde_json::to_vec(&req).expect("serialize");
        let decoded: VeritasRequest = serde_json::from_slice(&json).expect("deserialize");
        match decoded {
            VeritasRequest::TrustAttestation { score, .. } => {
                assert!((score - 0.85).abs() < f64::EPSILON);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_request_ping_serde() {
        let req = VeritasRequest::Ping;
        let json = serde_json::to_vec(&req).expect("serialize");
        let decoded: VeritasRequest = serde_json::from_slice(&json).expect("deserialize");
        assert!(matches!(decoded, VeritasRequest::Ping));
    }

    #[test]
    fn test_response_credential_serde() {
        let resp = VeritasResponse::CredentialResponse {
            request_id: "cr1".into(),
            issued: true,
            credential_data: vec![10, 20, 30],
            reason: None,
        };
        let json = serde_json::to_vec(&resp).expect("serialize");
        let decoded: VeritasResponse = serde_json::from_slice(&json).expect("deserialize");
        match decoded {
            VeritasResponse::CredentialResponse { issued, .. } => {
                assert!(issued);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_response_proof_serde() {
        let resp = VeritasResponse::ProofResponse {
            request_id: "pr1".into(),
            valid: true,
            proof_data: vec![1, 2, 3],
        };
        let json = serde_json::to_vec(&resp).expect("serialize");
        let decoded: VeritasResponse = serde_json::from_slice(&json).expect("deserialize");
        match decoded {
            VeritasResponse::ProofResponse { valid, .. } => {
                assert!(valid);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_response_pong_serde() {
        let resp = VeritasResponse::Pong;
        let json = serde_json::to_vec(&resp).expect("serialize");
        let decoded: VeritasResponse = serde_json::from_slice(&json).expect("deserialize");
        assert!(matches!(decoded, VeritasResponse::Pong));
    }

    #[test]
    fn test_response_error_serde() {
        let resp = VeritasResponse::Error {
            message: "not found".into(),
        };
        let json = serde_json::to_vec(&resp).expect("serialize");
        let decoded: VeritasResponse = serde_json::from_slice(&json).expect("deserialize");
        match decoded {
            VeritasResponse::Error { message } => {
                assert_eq!(message, "not found");
            }
            _ => panic!("wrong variant"),
        }
    }
}
