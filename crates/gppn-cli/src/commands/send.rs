//! `gppn send` — Send a payment via the GPPN network.

use clap::Args;

#[derive(Args, Debug)]
pub struct SendArgs {
    /// Recipient DID (e.g., did:gppn:key:abc123).
    #[arg(short, long)]
    pub to: String,

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

pub fn run(args: &SendArgs) -> anyhow::Result<()> {
    println!("Sending payment...");
    println!("  To:       {}", args.to);
    println!("  Amount:   {} {}", args.amount, args.currency);
    println!("  Via:      {}", args.endpoint);
    println!();

    // In a real implementation, this would:
    // 1. Connect to the node's JSON-RPC API
    // 2. Create a PaymentMessage
    // 3. Submit it and wait for routing + settlement
    println!("Payment submitted. (stub — connect a running node for real transactions)");

    Ok(())
}
