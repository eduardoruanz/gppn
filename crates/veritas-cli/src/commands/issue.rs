//! `veritas issue` — Issue a verifiable credential.

use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug)]
pub struct IssueArgs {
    /// Subject DID to issue the credential to.
    #[arg(short, long)]
    pub subject: String,

    /// Credential type(s), comma-separated.
    #[arg(short = 't', long, value_delimiter = ',')]
    pub credential_type: Vec<String>,

    /// Claims as JSON string.
    #[arg(short, long)]
    pub claims: String,

    /// API endpoint of the node.
    #[arg(short, long, default_value = "http://127.0.0.1:9001")]
    pub endpoint: String,
}

#[derive(Serialize)]
struct IssueRequest {
    subject_did: String,
    credential_type: Vec<String>,
    claims: serde_json::Value,
}

#[derive(Deserialize)]
struct CredentialResponse {
    credential_id: String,
    issuer: String,
    subject: String,
    status: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

pub async fn run(args: &IssueArgs) -> anyhow::Result<()> {
    let claims: serde_json::Value = serde_json::from_str(&args.claims)
        .map_err(|e| anyhow::anyhow!("invalid claims JSON: {}", e))?;

    let url = format!("{}/api/v1/credentials/issue", args.endpoint);
    let body = IssueRequest {
        subject_did: args.subject.clone(),
        credential_type: args.credential_type.clone(),
        claims,
    };

    let client = reqwest::Client::new();
    let resp = client.post(&url).json(&body).send().await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: CredentialResponse = r.json().await?;
            println!("Credential issued!");
            println!("  ID:       {}", data.credential_id);
            println!("  Issuer:   {}", data.issuer);
            println!("  Subject:  {}", data.subject);
            println!("  Status:   {}", data.status);
        }
        Ok(r) => {
            let status = r.status();
            if let Ok(err) = r.json::<ErrorResponse>().await {
                anyhow::bail!("issuance failed (HTTP {}): {}", status, err.error);
            } else {
                anyhow::bail!("issuance failed (HTTP {})", status);
            }
        }
        Err(e) => {
            println!("Could not reach node at {}", args.endpoint);
            println!("  Error: {}", e);
        }
    }

    Ok(())
}
