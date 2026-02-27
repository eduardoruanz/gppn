//! GPPN CLI — Command-line interface for the Global Payment Protocol Network.
//!
//! Subcommands: init, start, status, send, peers.

mod commands;

use clap::{Parser, Subcommand};

/// GPPN — The universal language of money.
#[derive(Parser, Debug)]
#[command(name = "gppn", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a new GPPN node configuration.
    Init(commands::init::InitArgs),
    /// Start the GPPN node.
    Start(commands::start::StartArgs),
    /// Query the status of a running node.
    Status(commands::status::StatusArgs),
    /// Send a payment via the GPPN network.
    Send(commands::send::SendArgs),
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
        Commands::Send(args) => commands::send::run(args).await,
        Commands::Peers(args) => commands::peers::run(args).await,
    }
}
