use serde::{Deserialize, Serialize};

/// Configuration for a GPPN node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node display name.
    pub name: String,
    /// Listen address for P2P networking.
    pub listen_address: String,
    /// Port for P2P networking.
    pub p2p_port: u16,
    /// Port for the local API server.
    pub api_port: u16,
    /// Port for Prometheus metrics.
    pub metrics_port: u16,
    /// Path to the data directory.
    pub data_dir: String,
    /// Bootstrap peers to connect to on startup.
    pub bootstrap_peers: Vec<String>,
    /// Default TTL for payment messages (seconds).
    pub default_ttl: u32,
    /// Maximum number of hops for routing.
    pub max_hops: u32,
    /// Log level (trace, debug, info, warn, error).
    pub log_level: String,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            name: "gppn-node".into(),
            listen_address: "0.0.0.0".into(),
            p2p_port: 9000,
            api_port: 9001,
            metrics_port: 9002,
            data_dir: "./data".into(),
            bootstrap_peers: Vec::new(),
            default_ttl: 300,
            max_hops: 10,
            log_level: "info".into(),
        }
    }
}
