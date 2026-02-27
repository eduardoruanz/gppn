//! `gppn send` — Send a payment via the GPPN network.

use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug)]
pub struct SendArgs {
    /// Recipient peer ID or DID.
    #[arg(short = 'r', long)]
    pub recipient: String,

    /// Amount to send (in atomic units).
    #[arg(short, long)]
    pub amount: u64,

    /// Currency code (e.g., USD, BTC, ETH).
    #[arg(short, long)]
    pub currency: String,

    /// API endpoint of the node.
    #[arg(short, long, default_value = "http://127.0.0.1:9001")]
    pub endpoint: String,
}

#[derive(Serialize)]
struct SendPaymentRequest {
    recipient: String,
    amount: u64,
    currency: String,
}

#[derive(Deserialize)]
struct SendPaymentResponse {
    pm_id: String,
    status: String,
    message: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

pub async fn run(args: &SendArgs) -> anyhow::Result<()> {
    let url = format!("{}/api/v1/payments", args.endpoint);
    let body = SendPaymentRequest {
        recipient: args.recipient.clone(),
        amount: args.amount,
        currency: args.currency.clone(),
    };

    println!("Sending payment...");
    println!("  To:       {}", args.recipient);
    println!("  Amount:   {} {}", args.amount, args.currency);
    println!("  Via:      {}", args.endpoint);
    println!();

    let client = reqwest::Client::new();
    let resp = client.post(&url).json(&body).send().await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: SendPaymentResponse = r.json().await?;
            println!("Payment dispatched!");
            println!("  PM ID:    {}", data.pm_id);
            println!("  Status:   {}", data.status);
            println!("  Message:  {}", data.message);
        }
        Ok(r) => {
            let status = r.status();
            if let Ok(err) = r.json::<ErrorResponse>().await {
                anyhow::bail!("payment failed (HTTP {}): {}", status, err.error);
            } else {
                anyhow::bail!("payment failed (HTTP {})", status);
            }
        }
        Err(e) => {
            println!("Could not reach node at {}", args.endpoint);
            println!("  Error: {}", e);
            println!();
            println!("Is the node running? Start it with: gppn-node");
        }
    }

    Ok(())
}
