//! `veritas resolve` — Resolve a DID to its document.

use clap::Args;
use serde::Deserialize;

#[derive(Args, Debug)]
pub struct ResolveArgs {
    /// The DID to resolve.
    pub did: String,

    /// API endpoint of the node.
    #[arg(short, long, default_value = "http://127.0.0.1:9001")]
    pub endpoint: String,
}

#[derive(Deserialize)]
struct DidResponse {
    did: String,
    document: serde_json::Value,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

pub async fn run(args: &ResolveArgs) -> anyhow::Result<()> {
    let url = format!("{}/api/v1/identity/did/{}", args.endpoint, args.did);
    let resp = reqwest::get(&url).await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: DidResponse = r.json().await?;
            println!("DID: {}", data.did);
            println!(
                "Document:\n{}",
                serde_json::to_string_pretty(&data.document)?
            );
        }
        Ok(r) => {
            let status = r.status();
            if let Ok(err) = r.json::<ErrorResponse>().await {
                anyhow::bail!("resolve failed (HTTP {}): {}", status, err.error);
            } else {
                anyhow::bail!("resolve failed (HTTP {})", status);
            }
        }
        Err(e) => {
            println!("Could not reach node at {}", args.endpoint);
            println!("  Error: {}", e);
        }
    }

    Ok(())
}
