//! `gppn peers` — List connected peers.

use clap::Args;

#[derive(Args, Debug)]
pub struct PeersArgs {
    /// API endpoint of the node.
    #[arg(short, long, default_value = "http://127.0.0.1:9001")]
    pub endpoint: String,
}

pub fn run(args: &PeersArgs) -> anyhow::Result<()> {
    println!("Connected peers at {}:", args.endpoint);
    println!();

    // In a real implementation, this would query the node's API
    println!("  (no peers — node not running or endpoint unreachable)");
    println!();
    println!("Start a node with 'gppn start' first.");

    Ok(())
}
