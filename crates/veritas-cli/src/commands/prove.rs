//! `veritas prove` — Generate a zero-knowledge proof.

use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug)]
pub struct ProveArgs {
    /// Proof type: age, residency, kyc_level.
    #[arg(short = 't', long)]
    pub proof_type: String,

    /// Proof parameters as JSON string.
    #[arg(short, long)]
    pub params: String,

    /// API endpoint of the node.
    #[arg(short, long, default_value = "http://127.0.0.1:9001")]
    pub endpoint: String,
}

#[derive(Serialize)]
struct ProofRequest {
    proof_type: String,
    params: serde_json::Value,
}

#[derive(Deserialize)]
struct ProofResponse {
    proof_type: String,
    proof_json: String,
    status: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

pub async fn run(args: &ProveArgs) -> anyhow::Result<()> {
    let params: serde_json::Value = serde_json::from_str(&args.params)
        .map_err(|e| anyhow::anyhow!("invalid params JSON: {}", e))?;

    let url = format!("{}/api/v1/proofs/generate", args.endpoint);
    let body = ProofRequest {
        proof_type: args.proof_type.clone(),
        params,
    };

    let client = reqwest::Client::new();
    let resp = client.post(&url).json(&body).send().await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: ProofResponse = r.json().await?;
            println!("Proof generated!");
            println!("  Type:   {}", data.proof_type);
            println!("  Status: {}", data.status);
            println!("  Proof:");
            // Pretty-print the proof JSON
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data.proof_json) {
                println!("{}", serde_json::to_string_pretty(&v)?);
            } else {
                println!("{}", data.proof_json);
            }
        }
        Ok(r) => {
            let status = r.status();
            if let Ok(err) = r.json::<ErrorResponse>().await {
                anyhow::bail!("proof generation failed (HTTP {}): {}", status, err.error);
            } else {
                anyhow::bail!("proof generation failed (HTTP {})", status);
            }
        }
        Err(e) => {
            println!("Could not reach node at {}", args.endpoint);
            println!("  Error: {}", e);
        }
    }

    Ok(())
}
