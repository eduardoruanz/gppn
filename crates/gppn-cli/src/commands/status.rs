//! `gppn status` — Query the status of a running GPPN node.

use clap::Args;

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// API endpoint of the node.
    #[arg(short, long, default_value = "http://127.0.0.1:9001")]
    pub endpoint: String,
}

pub fn run(args: &StatusArgs) -> anyhow::Result<()> {
    println!("Querying node status at {}...", args.endpoint);
    println!();

    // In a real implementation, this would make an HTTP/JSON-RPC call
    // to the running node's API endpoint.
    println!("Node Status:");
    println!("  Endpoint: {}", args.endpoint);
    println!("  Status:   (node not running or endpoint unreachable)");
    println!();
    println!("Start a node with 'gppn start' or 'gppn-node' first.");

    Ok(())
}
