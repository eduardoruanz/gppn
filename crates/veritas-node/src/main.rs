//! Veritas Full Node — entry point.
//!
//! Starts the Veritas full node with configuration from a TOML file or defaults.

// Public APIs for node internals — used by tests and external consumers.
#![allow(dead_code)]

mod api;
mod commands;
mod config;
mod node;
mod state;
mod storage;

use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

use config::VeritasConfig;
use node::VeritasFullNode;

/// Veritas Full Node
#[derive(Parser, Debug)]
#[command(name = "veritas-node", version, about = "Veritas Full Node")]
struct Args {
    /// Path to the configuration file (TOML).
    #[arg(short, long, default_value = "veritas.toml")]
    config: PathBuf,

    /// Override the P2P listen port.
    #[arg(long)]
    port: Option<u16>,

    /// Override the API port.
    #[arg(long)]
    api_port: Option<u16>,

    /// Override the data directory.
    #[arg(long)]
    data_dir: Option<PathBuf>,

    /// Override the log level (trace, debug, info, warn, error).
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Generate a default config file and exit.
    #[arg(long)]
    init: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&args.log_level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .init();

    // Handle --init flag
    if args.init {
        let config = VeritasConfig::default();
        config.save(&args.config)?;
        tracing::info!(path = %args.config.display(), "wrote default config");
        return Ok(());
    }

    // Load configuration
    let mut config = VeritasConfig::load(&args.config)?;

    // Apply CLI overrides
    if let Some(port) = args.port {
        config.network.port = port;
    }
    if let Some(api_port) = args.api_port {
        config.api.port = api_port;
    }
    if let Some(ref data_dir) = args.data_dir {
        config.storage.data_dir = data_dir.clone();
    }
    config.logging.level = args.log_level;

    tracing::info!("Veritas Full Node v{}", env!("CARGO_PKG_VERSION"));

    // Create and start the node
    let mut node = VeritasFullNode::new(config)?;
    node.start().await?;

    // Set up graceful shutdown on SIGINT/SIGTERM
    let shutdown = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl-c");
        tracing::info!("received shutdown signal");
    };

    tokio::select! {
        result = node.run() => {
            if let Err(e) = result {
                tracing::error!(error = %e, "node event loop error");
            }
        }
        _ = shutdown => {
            tracing::info!("initiating graceful shutdown");
        }
    }

    node.shutdown().await?;
    tracing::info!("Veritas node exited cleanly");
    Ok(())
}
