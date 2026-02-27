//! Commands dispatched from the HTTP API to the node event loop.

use serde::Serialize;
use tokio::sync::oneshot;

/// A command sent from the HTTP API to the node's main event loop.
pub enum NodeCommand {
    /// Send a payment to a recipient.
    SendPayment {
        recipient: String,
        amount: u64,
        currency: String,
        reply: oneshot::Sender<Result<SendPaymentResponse, String>>,
    },
}

/// Response returned after dispatching a payment.
#[derive(Debug, Clone, Serialize)]
pub struct SendPaymentResponse {
    pub pm_id: String,
    pub status: String,
    pub message: String,
}
