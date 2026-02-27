use serde::{Deserialize, Serialize};

/// Configuration for a Veritas node.
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
    /// Whether this node acts as a credential issuer.
    pub is_issuer: bool,
    /// DIDs of trusted credential issuers.
    pub trusted_issuers: Vec<String>,
    /// Log level (trace, debug, info, warn, error).
    pub log_level: String,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            name: "veritas-node".into(),
            listen_address: "0.0.0.0".into(),
            p2p_port: 9000,
            api_port: 9001,
            metrics_port: 9002,
            data_dir: "./data".into(),
            bootstrap_peers: Vec::new(),
            is_issuer: false,
            trusted_issuers: Vec::new(),
            log_level: "info".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NodeConfig::default();
        assert_eq!(config.name, "veritas-node");
        assert_eq!(config.p2p_port, 9000);
        assert_eq!(config.api_port, 9001);
        assert!(!config.is_issuer);
        assert!(config.trusted_issuers.is_empty());
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let config = NodeConfig {
            name: "test-issuer".into(),
            is_issuer: true,
            trusted_issuers: vec!["did:veritas:key:abc".into()],
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        let back: NodeConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "test-issuer");
        assert!(back.is_issuer);
        assert_eq!(back.trusted_issuers.len(), 1);
    }

    #[test]
    fn test_config_custom_ports() {
        let config = NodeConfig {
            p2p_port: 8000,
            api_port: 8001,
            metrics_port: 8002,
            ..Default::default()
        };
        assert_eq!(config.p2p_port, 8000);
        assert_eq!(config.api_port, 8001);
        assert_eq!(config.metrics_port, 8002);
    }
}
