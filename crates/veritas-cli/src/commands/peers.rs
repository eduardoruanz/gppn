//! `veritas peers` — List connected peers.

use clap::Args;
use serde::Deserialize;

#[derive(Args, Debug)]
pub struct PeersArgs {
    /// API endpoint of the node.
    #[arg(short, long, default_value = "http://127.0.0.1:9001")]
    pub endpoint: String,
}

#[derive(Deserialize)]
struct PeersResponse {
    peers: Vec<PeerInfo>,
    count: usize,
}

#[derive(Deserialize)]
struct PeerInfo {
    peer_id: String,
}

pub async fn run(args: &PeersArgs) -> anyhow::Result<()> {
    let url = format!("{}/api/v1/peers", args.endpoint);
    let resp = reqwest::get(&url).await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: PeersResponse = r.json().await?;
            println!("Connected peers ({}): ", data.count);
            if data.peers.is_empty() {
                println!("  (no peers connected)");
            } else {
                for peer in &data.peers {
                    println!("  {}", peer.peer_id);
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
