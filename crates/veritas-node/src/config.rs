//! Node configuration loading and management.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Full configuration for the Veritas node.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VeritasConfig {
    /// P2P network settings.
    #[serde(default)]
    pub network: NetworkConfig,

    /// API server settings.
    #[serde(default)]
    pub api: ApiConfig,

    /// Storage settings.
    #[serde(default)]
    pub storage: StorageConfig,

    /// Metrics settings.
    #[serde(default)]
    pub metrics: MetricsConfig,

    /// Logging settings.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Identity settings.
    #[serde(default)]
    pub identity: IdentityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// P2P listen address.
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    /// P2P listen port.
    #[serde(default = "default_p2p_port")]
    pub port: u16,
    /// Bootstrap peer multiaddresses.
    #[serde(default)]
    pub bootstrap_peers: Vec<String>,
    /// Maximum number of connected peers.
    #[serde(default = "default_max_peers")]
    pub max_peers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API listen address.
    #[serde(default = "default_api_addr")]
    pub listen_addr: String,
    /// API port.
    #[serde(default = "default_api_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Path to the data directory.
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Whether metrics are enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Metrics listen port.
    #[serde(default = "default_metrics_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error).
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Log format (text, json).
    #[serde(default = "default_log_format")]
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdentityConfig {
    /// Path to the keypair file.
    #[serde(default)]
    pub keypair_path: Option<PathBuf>,
}

// Default value functions
fn default_listen_addr() -> String {
    "0.0.0.0".into()
}
fn default_p2p_port() -> u16 {
    9000
}
fn default_max_peers() -> usize {
    50
}
fn default_api_addr() -> String {
    "127.0.0.1".into()
}
fn default_api_port() -> u16 {
    9001
}
fn default_data_dir() -> PathBuf {
    PathBuf::from("./data")
}
fn default_true() -> bool {
    true
}
fn default_metrics_port() -> u16 {
    9002
}
fn default_log_level() -> String {
    "info".into()
}
fn default_log_format() -> String {
    "text".into()
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_listen_addr(),
            port: default_p2p_port(),
            bootstrap_peers: Vec::new(),
            max_peers: default_max_peers(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_api_addr(),
            port: default_api_port(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: default_metrics_port(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

impl VeritasConfig {
    /// Load config from a TOML file, falling back to defaults for missing fields.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            let config: VeritasConfig = toml::from_str(&contents)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save the current config to a TOML file.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let contents = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Get the full P2P listen multiaddress.
    pub fn p2p_multiaddr(&self) -> String {
        format!(
            "/ip4/{}/tcp/{}",
            self.network.listen_addr, self.network.port
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = VeritasConfig::default();
        assert_eq!(config.network.port, 9000);
        assert_eq!(config.api.port, 9001);
        assert_eq!(config.metrics.port, 9002);
        assert_eq!(config.logging.level, "info");
        assert!(config.network.bootstrap_peers.is_empty());
    }

    #[test]
    fn test_p2p_multiaddr() {
        let config = VeritasConfig::default();
        assert_eq!(config.p2p_multiaddr(), "/ip4/0.0.0.0/tcp/9000");
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = VeritasConfig::default();
        let toml_str = toml::to_string_pretty(&config).expect("serialize");
        let decoded: VeritasConfig = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(decoded.network.port, config.network.port);
        assert_eq!(decoded.api.port, config.api.port);
    }

    #[test]
    fn test_config_load_nonexistent_uses_defaults() {
        let config = VeritasConfig::load(Path::new("/nonexistent/config.toml")).unwrap();
        assert_eq!(config.network.port, 9000);
    }

    #[test]
    fn test_config_from_toml_partial() {
        let toml_str = r#"
[network]
port = 8000

[api]
port = 8001
"#;
        let config: VeritasConfig = toml::from_str(toml_str).expect("parse");
        assert_eq!(config.network.port, 8000);
        assert_eq!(config.api.port, 8001);
        // Defaults for unspecified
        assert_eq!(config.metrics.port, 9002);
    }
}
