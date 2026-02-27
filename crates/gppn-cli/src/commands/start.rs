//! `gppn start` — Start the GPPN node.

use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct StartArgs {
    /// Path to the config file.
    #[arg(short, long, default_value = "gppn.toml")]
    pub config: PathBuf,

    /// Override the P2P port.
    #[arg(long)]
    pub port: Option<u16>,

    /// Override the log level.
    #[arg(long)]
    pub log_level: Option<String>,
}

pub fn run(args: &StartArgs) -> anyhow::Result<()> {
    println!("Starting GPPN node...");
    println!("  Config: {}", args.config.display());
    if let Some(port) = args.port {
        println!("  Port override: {}", port);
    }
    if let Some(ref level) = args.log_level {
        println!("  Log level: {}", level);
    }

    // In a real implementation, this would:
    // 1. Load config from the TOML file
    // 2. Create a GppnFullNode
    // 3. Start it and run the event loop
    // For now, delegate to `gppn-node` binary
    println!();
    println!("Hint: Use the gppn-node binary for a full node:");
    println!("  gppn-node --config {}", args.config.display());

    Ok(())
}
