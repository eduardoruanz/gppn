//! `veritas status` — Query the status of a running Veritas node.

use clap::Args;
use serde::Deserialize;

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// API endpoint of the node.
    #[arg(short, long, default_value = "http://127.0.0.1:9001")]
    pub endpoint: String,
}

#[derive(Deserialize)]
struct StatusResponse {
    version: String,
    peer_id: String,
    did: String,
    peer_count: usize,
    uptime_secs: u64,
    listening_addrs: Vec<String>,
}

pub async fn run(args: &StatusArgs) -> anyhow::Result<()> {
    let url = format!("{}/api/v1/status", args.endpoint);
    let resp = reqwest::get(&url).await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let status: StatusResponse = r.json().await?;
            println!("Node Status:");
            println!("  Version:    {}", status.version);
            println!("  DID:        {}", status.did);
            println!("  Peer ID:    {}", status.peer_id);
            println!("  Peers:      {}", status.peer_count);
            println!("  Uptime:     {}s", status.uptime_secs);
            if status.listening_addrs.is_empty() {
                println!("  Listening:  (none)");
            } else {
                for addr in &status.listening_addrs {
                    println!("  Listening:  {}", addr);
                }
            }
        }
        Ok(r) => {
            anyhow::bail!("node returned HTTP {}", r.status());
        }
        Err(e) => {
            println!("Could not reach node at {}", args.endpoint);
            println!("  Error: {}", e);
            println!();
            println!("Is the node running? Start it with: veritas-node");
        }
    }

    Ok(())
}
