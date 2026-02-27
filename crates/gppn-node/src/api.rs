//! HTTP API server for the GPPN node.
//!
//! Provides REST endpoints for node status, peer listing, and payment submission.

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::commands::{NodeCommand, SendPaymentResponse};
use crate::state::NodeState;

// --- Response types ---

#[derive(Serialize)]
pub struct StatusResponse {
    pub version: String,
    pub peer_id: String,
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

#[derive(Deserialize)]
pub struct SendPaymentRequest {
    pub recipient: String,
    pub amount: u64,
    pub currency: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// --- Handlers ---

async fn handle_status(State(state): State<Arc<NodeState>>) -> Json<StatusResponse> {
    Json(StatusResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        peer_id: state.peer_id.to_string(),
        peer_count: state.peer_count(),
        uptime_secs: state.start_time.elapsed().as_secs(),
        listening_addrs: state.listening_addrs(),
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

async fn handle_send_payment(
    State(state): State<Arc<NodeState>>,
    Json(req): Json<SendPaymentRequest>,
) -> Result<Json<SendPaymentResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();

    let cmd = NodeCommand::SendPayment {
        recipient: req.recipient,
        amount: req.amount,
        currency: req.currency,
        reply: reply_tx,
    };

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
        Ok(Err(e)) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )),
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
        .route("/api/v1/status", get(handle_status))
        .route("/api/v1/peers", get(handle_peers))
        .route("/api/v1/payments", post(handle_send_payment))
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
