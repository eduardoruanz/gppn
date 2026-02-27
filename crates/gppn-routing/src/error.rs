use gppn_core::Did;

/// Errors that can occur within the routing layer.
#[derive(Debug, thiserror::Error)]
pub enum RoutingError {
    #[error("no route found from {from} to {to}")]
    NoRouteFound { from: Did, to: Did },

    #[error("insufficient liquidity: required {required}, available {available}")]
    InsufficientLiquidity { required: u128, available: u128 },

    #[error("route expired: hop {hop_index} last updated {last_updated}")]
    RouteExpired {
        hop_index: usize,
        last_updated: String,
    },

    #[error("maximum hop count exceeded: {max_hops}")]
    MaxHopsExceeded { max_hops: u32 },

    #[error("currency not supported: {currency} at hop {hop_index}")]
    CurrencyNotSupported {
        currency: String,
        hop_index: usize,
    },

    #[error("invalid route entry: {reason}")]
    InvalidRouteEntry { reason: String },

    #[error("duplicate peer in route: {peer_id}")]
    DuplicatePeer { peer_id: String },

    #[error("trust score below threshold: {score} < {threshold}")]
    TrustBelowThreshold { score: f64, threshold: f64 },

    #[error("scoring weights must sum to 1.0, got {sum}")]
    InvalidScoringWeights { sum: f64 },

    #[error("route table is empty")]
    EmptyRoutingTable,
}
