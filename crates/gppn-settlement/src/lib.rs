//! GPPN Settlement Layer
//!
//! Provides the settlement abstraction, HTLC engine, and pluggable adapters
//! for settling payments across heterogeneous rails (blockchains, fiat systems,
//! internal ledgers).

pub mod error;
pub mod types;
pub mod traits;
pub mod htlc;
pub mod manager;
pub mod adapters;

pub use error::SettlementError;
pub use types::{SettlementId, SettlementStatus, SettlementReceipt};
pub use traits::ISettlement;
pub use htlc::{Htlc, HtlcManager, HtlcStatus};
pub use manager::SettlementManager;
