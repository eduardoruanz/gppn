//! Commands dispatched from the HTTP API to the node event loop.

use serde::Serialize;
use tokio::sync::oneshot;

/// A command sent from the HTTP API to the node's main event loop.
pub enum NodeCommand {
    /// Issue a credential to a subject.
    IssueCredential {
        subject_did: String,
        credential_type: Vec<String>,
        claims: serde_json::Value,
        reply: oneshot::Sender<Result<CredentialResponse, String>>,
    },
    /// Verify a credential.
    VerifyCredential {
        credential_json: String,
        reply: oneshot::Sender<Result<VerifyResponse, String>>,
    },
    /// Generate a ZK proof.
    GenerateProof {
        proof_type: String,
        params: serde_json::Value,
        reply: oneshot::Sender<Result<ProofResponse, String>>,
    },
    /// Resolve a DID to its document.
    ResolveDid {
        did: String,
        reply: oneshot::Sender<Result<DidResponse, String>>,
    },
    /// Attest trust in a subject.
    AttestTrust {
        subject_did: String,
        score: f64,
        category: String,
        reply: oneshot::Sender<Result<TrustResponse, String>>,
    },
}

/// Response after issuing a credential.
#[derive(Debug, Clone, Serialize)]
pub struct CredentialResponse {
    pub credential_id: String,
    pub issuer: String,
    pub subject: String,
    pub status: String,
}

/// Response after verifying a credential.
#[derive(Debug, Clone, Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub checks: Vec<VerifyCheck>,
}

/// Individual verification check result.
#[derive(Debug, Clone, Serialize)]
pub struct VerifyCheck {
    pub name: String,
    pub passed: bool,
    pub detail: Option<String>,
}

/// Response after generating a proof.
#[derive(Debug, Clone, Serialize)]
pub struct ProofResponse {
    pub proof_type: String,
    pub proof_json: String,
    pub status: String,
}

/// Response after resolving a DID.
#[derive(Debug, Clone, Serialize)]
pub struct DidResponse {
    pub did: String,
    pub document: serde_json::Value,
}

/// Response after attesting trust.
#[derive(Debug, Clone, Serialize)]
pub struct TrustResponse {
    pub subject_did: String,
    pub score: f64,
    pub status: String,
}
