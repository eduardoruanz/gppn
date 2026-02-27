//! `veritas start` — Start the Veritas node.

use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct StartArgs {
    /// Path to the config file.
    #[arg(short, long, default_value = "veritas.toml")]
    pub config: PathBuf,

    /// Override the P2P port.
    #[arg(long)]
    pub port: Option<u16>,

    /// Override the log level.
    #[arg(long)]
    pub log_level: Option<String>,
}

pub fn run(args: &StartArgs) -> anyhow::Result<()> {
    println!("Starting Veritas node...");
    println!("  Config: {}", args.config.display());
    if let Some(port) = args.port {
        println!("  Port override: {}", port);
    }
    if let Some(ref level) = args.log_level {
        println!("  Log level: {}", level);
    }

    println!();
    println!("Hint: Use the veritas-node binary for a full node:");
    println!("  veritas-node --config {}", args.config.display());

    Ok(())
}
