//! `veritas verify` — Verify a verifiable credential.

use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// Credential JSON (as string or path to file).
    #[arg(short, long)]
    pub credential: String,

    /// API endpoint of the node.
    #[arg(short, long, default_value = "http://127.0.0.1:9001")]
    pub endpoint: String,
}

#[derive(Serialize)]
struct VerifyRequest {
    credential: serde_json::Value,
}

#[derive(Deserialize)]
struct VerifyResponse {
    valid: bool,
    checks: Vec<VerifyCheck>,
}

#[derive(Deserialize)]
struct VerifyCheck {
    name: String,
    passed: bool,
    detail: Option<String>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

pub async fn run(args: &VerifyArgs) -> anyhow::Result<()> {
    // Try reading as file first, then as inline JSON
    let json_str = if std::path::Path::new(&args.credential).exists() {
        std::fs::read_to_string(&args.credential)?
    } else {
        args.credential.clone()
    };

    let credential: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| anyhow::anyhow!("invalid credential JSON: {}", e))?;

    let url = format!("{}/api/v1/credentials/verify", args.endpoint);
    let body = VerifyRequest { credential };

    let client = reqwest::Client::new();
    let resp = client.post(&url).json(&body).send().await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: VerifyResponse = r.json().await?;
            if data.valid {
                println!("Credential is VALID");
            } else {
                println!("Credential is INVALID");
            }
            println!();
            for check in &data.checks {
                let icon = if check.passed { "PASS" } else { "FAIL" };
                print!("  [{}] {}", icon, check.name);
                if let Some(ref detail) = check.detail {
                    print!(" — {}", detail);
                }
                println!();
            }
        }
        Ok(r) => {
            let status = r.status();
            if let Ok(err) = r.json::<ErrorResponse>().await {
                anyhow::bail!("verification failed (HTTP {}): {}", status, err.error);
            } else {
                anyhow::bail!("verification failed (HTTP {})", status);
            }
        }
        Err(e) => {
            println!("Could not reach node at {}", args.endpoint);
            println!("  Error: {}", e);
        }
    }

    Ok(())
}
