//! Veritas CLI — Command-line interface for decentralized identity.
//!
//! Subcommands: init, start, status, identity, resolve, issue, verify,
//! prove, trust, attest, peers.

mod commands;

use clap::{Parser, Subcommand};

/// Veritas — Decentralized identity protocol.
#[derive(Parser, Debug)]
#[command(name = "veritas", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a new Veritas node configuration.
    Init(commands::init::InitArgs),
    /// Start the Veritas node.
    Start(commands::start::StartArgs),
    /// Query the status of a running node.
    Status(commands::status::StatusArgs),
    /// Show the node's identity (DID and peer ID).
    Identity(commands::identity::IdentityArgs),
    /// Resolve a DID to its document.
    Resolve(commands::resolve::ResolveArgs),
    /// Issue a verifiable credential.
    Issue(commands::issue::IssueArgs),
    /// Verify a verifiable credential.
    Verify(commands::verify::VerifyArgs),
    /// Generate a zero-knowledge proof.
    Prove(commands::prove::ProveArgs),
    /// Attest trust in a DID.
    Attest(commands::attest::AttestArgs),
    /// List connected peers.
    Peers(commands::peers::PeersArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init(args) => commands::init::run(args),
        Commands::Start(args) => commands::start::run(args),
        Commands::Status(args) => commands::status::run(args).await,
        Commands::Identity(args) => commands::identity::run(args).await,
        Commands::Resolve(args) => commands::resolve::run(args).await,
        Commands::Issue(args) => commands::issue::run(args).await,
        Commands::Verify(args) => commands::verify::run(args).await,
        Commands::Prove(args) => commands::prove::run(args).await,
        Commands::Attest(args) => commands::attest::run(args).await,
        Commands::Peers(args) => commands::peers::run(args).await,
    }
}
