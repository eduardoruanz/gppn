//! `veritas attest` — Attest trust in a DID.

use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug)]
pub struct AttestArgs {
    /// Subject DID to attest trust for.
    #[arg(short, long)]
    pub subject: String,

    /// Trust score (0.0 to 1.0).
    #[arg(long)]
    pub score: f64,

    /// Trust category (e.g., identity, kyc, behavior).
    #[arg(short, long, default_value = "identity")]
    pub category: String,

    /// API endpoint of the node.
    #[arg(short, long, default_value = "http://127.0.0.1:9001")]
    pub endpoint: String,
}

#[derive(Serialize)]
struct AttestRequest {
    subject_did: String,
    score: f64,
    category: String,
}

#[derive(Deserialize)]
struct TrustResponse {
    subject_did: String,
    score: f64,
    status: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

pub async fn run(args: &AttestArgs) -> anyhow::Result<()> {
    if !(0.0..=1.0).contains(&args.score) {
        anyhow::bail!("score must be between 0.0 and 1.0");
    }

    let url = format!("{}/api/v1/trust/attest", args.endpoint);
    let body = AttestRequest {
        subject_did: args.subject.clone(),
        score: args.score,
        category: args.category.clone(),
    };

    let client = reqwest::Client::new();
    let resp = client.post(&url).json(&body).send().await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: TrustResponse = r.json().await?;
            println!("Trust attested!");
            println!("  Subject:  {}", data.subject_did);
            println!("  Score:    {:.2}", data.score);
            println!("  Status:   {}", data.status);
        }
        Ok(r) => {
            let status = r.status();
            if let Ok(err) = r.json::<ErrorResponse>().await {
                anyhow::bail!("attestation failed (HTTP {}): {}", status, err.error);
            } else {
                anyhow::bail!("attestation failed (HTTP {})", status);
            }
        }
        Err(e) => {
            println!("Could not reach node at {}", args.endpoint);
            println!("  Error: {}", e);
        }
    }

    Ok(())
}
