//! `veritas identity` — Show the node's identity (DID and peer ID).

use clap::Args;
use serde::Deserialize;

#[derive(Args, Debug)]
pub struct IdentityArgs {
    /// API endpoint of the node.
    #[arg(short, long, default_value = "http://127.0.0.1:9001")]
    pub endpoint: String,
}

#[derive(Deserialize)]
struct IdentityResponse {
    did: String,
    peer_id: String,
}

pub async fn run(args: &IdentityArgs) -> anyhow::Result<()> {
    let url = format!("{}/api/v1/identity", args.endpoint);
    let resp = reqwest::get(&url).await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: IdentityResponse = r.json().await?;
            println!("Node Identity:");
            println!("  DID:      {}", data.did);
            println!("  Peer ID:  {}", data.peer_id);
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
