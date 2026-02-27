//! HTTP API server for the Veritas node.
//!
//! Provides REST endpoints for node status, peer listing, credential operations,
//! proof generation, DID resolution, and trust attestation.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::commands::{
    CredentialResponse, DidResponse, NodeCommand, ProofResponse, TrustResponse, VerifyResponse,
};
use crate::state::NodeState;

// --- Response types ---

#[derive(Serialize)]
pub struct StatusResponse {
    pub version: String,
    pub peer_id: String,
    pub did: String,
    pub peer_count: usize,
    pub uptime_secs: u64,
    pub listening_addrs: Vec<String>,
}

#[derive(Serialize)]
pub struct PeerInfo {
    pub peer_id: String,
}

#[derive(Serialize)]
pub struct PeersResponse {
    pub peers: Vec<PeerInfo>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct IdentityResponse {
    pub did: String,
    pub peer_id: String,
}

#[derive(Deserialize)]
pub struct IssueCredentialRequest {
    pub subject_did: String,
    pub credential_type: Vec<String>,
    pub claims: serde_json::Value,
}

#[derive(Deserialize)]
pub struct VerifyCredentialRequest {
    pub credential: serde_json::Value,
}

#[derive(Deserialize)]
pub struct GenerateProofRequest {
    pub proof_type: String,
    pub params: serde_json::Value,
}

#[derive(Deserialize)]
pub struct AttestTrustRequest {
    pub subject_did: String,
    pub score: f64,
    pub category: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
}

// --- Handlers ---

async fn handle_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
    })
}

async fn handle_status(State(state): State<Arc<NodeState>>) -> Json<StatusResponse> {
    Json(StatusResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        peer_id: state.peer_id.to_string(),
        did: state.did.clone(),
        peer_count: state.peer_count(),
        uptime_secs: state.start_time.elapsed().as_secs(),
        listening_addrs: state.listening_addrs(),
    })
}

async fn handle_identity(State(state): State<Arc<NodeState>>) -> Json<IdentityResponse> {
    Json(IdentityResponse {
        did: state.did.clone(),
        peer_id: state.peer_id.to_string(),
    })
}

async fn handle_peers(State(state): State<Arc<NodeState>>) -> Json<PeersResponse> {
    let peers = state.connected_peers();
    let peer_infos: Vec<PeerInfo> = peers
        .iter()
        .map(|p| PeerInfo {
            peer_id: p.to_string(),
        })
        .collect();
    let count = peer_infos.len();
    Json(PeersResponse {
        peers: peer_infos,
        count,
    })
}

async fn handle_issue_credential(
    State(state): State<Arc<NodeState>>,
    Json(req): Json<IssueCredentialRequest>,
) -> Result<Json<CredentialResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();

    let cmd = NodeCommand::IssueCredential {
        subject_did: req.subject_did,
        credential_type: req.credential_type,
        claims: req.claims,
        reply: reply_tx,
    };

    send_command_and_await(&state, cmd, reply_rx).await
}

async fn handle_verify_credential(
    State(state): State<Arc<NodeState>>,
    Json(req): Json<VerifyCredentialRequest>,
) -> Result<Json<VerifyResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();

    let credential_json = serde_json::to_string(&req.credential).unwrap_or_default();

    let cmd = NodeCommand::VerifyCredential {
        credential_json,
        reply: reply_tx,
    };

    send_command_and_await(&state, cmd, reply_rx).await
}

async fn handle_generate_proof(
    State(state): State<Arc<NodeState>>,
    Json(req): Json<GenerateProofRequest>,
) -> Result<Json<ProofResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();

    let cmd = NodeCommand::GenerateProof {
        proof_type: req.proof_type,
        params: req.params,
        reply: reply_tx,
    };

    send_command_and_await(&state, cmd, reply_rx).await
}

async fn handle_resolve_did(
    State(state): State<Arc<NodeState>>,
    Path(did): Path<String>,
) -> Result<Json<DidResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();

    let cmd = NodeCommand::ResolveDid {
        did,
        reply: reply_tx,
    };

    send_command_and_await(&state, cmd, reply_rx).await
}

async fn handle_attest_trust(
    State(state): State<Arc<NodeState>>,
    Json(req): Json<AttestTrustRequest>,
) -> Result<Json<TrustResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();

    let cmd = NodeCommand::AttestTrust {
        subject_did: req.subject_did,
        score: req.score,
        category: req.category,
        reply: reply_tx,
    };

    send_command_and_await(&state, cmd, reply_rx).await
}

/// Helper to send a command and await the reply.
async fn send_command_and_await<T: Serialize>(
    state: &Arc<NodeState>,
    cmd: NodeCommand,
    reply_rx: tokio::sync::oneshot::Receiver<Result<T, String>>,
) -> Result<Json<T>, (StatusCode, Json<ErrorResponse>)> {
    state.command_tx.send(cmd).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "node event loop not running".into(),
            }),
        )
    })?;

    match reply_rx.await {
        Ok(Ok(resp)) => Ok(Json(resp)),
        Ok(Err(e)) => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e }))),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "event loop dropped the reply channel".into(),
            }),
        )),
    }
}

// --- Server ---

pub fn build_router(state: Arc<NodeState>) -> Router {
    Router::new()
        .route("/api/v1/health", get(handle_health))
        .route("/api/v1/status", get(handle_status))
        .route("/api/v1/identity", get(handle_identity))
        .route("/api/v1/identity/did/{did}", get(handle_resolve_did))
        .route("/api/v1/peers", get(handle_peers))
        .route("/api/v1/credentials/issue", post(handle_issue_credential))
        .route("/api/v1/credentials/verify", post(handle_verify_credential))
        .route("/api/v1/proofs/generate", post(handle_generate_proof))
        .route("/api/v1/trust/attest", post(handle_attest_trust))
        .with_state(state)
}

pub async fn start_api_server(
    listen_addr: SocketAddr,
    state: Arc<NodeState>,
) -> anyhow::Result<()> {
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    tracing::info!(%listen_addr, "HTTP API server started");
    axum::serve(listener, app).await?;
    Ok(())
}
