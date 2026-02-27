//! `gppn init` — Initialize a new GPPN node configuration.

use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Directory to initialize (defaults to current directory).
    #[arg(default_value = ".")]
    pub dir: PathBuf,
}

pub fn run(args: &InitArgs) -> anyhow::Result<()> {
    let config_path = args.dir.join("gppn.toml");

    if config_path.exists() {
        anyhow::bail!("configuration file already exists at {}", config_path.display());
    }

    std::fs::create_dir_all(&args.dir)?;

    let default_config = r#"# GPPN Node Configuration

[network]
listen_addr = "0.0.0.0"
port = 9000
max_peers = 50
bootstrap_peers = []

[api]
listen_addr = "127.0.0.1"
port = 9001

[storage]
data_dir = "./data"

[metrics]
enabled = true
port = 9002

[logging]
level = "info"
format = "text"
"#;

    std::fs::write(&config_path, default_config)?;
    println!("Initialized GPPN node at {}", config_path.display());
    println!("Edit gppn.toml to customize your configuration.");
    println!("Run 'gppn start' to start the node.");

    // Create data directory
    let data_dir = args.dir.join("data");
    std::fs::create_dir_all(&data_dir)?;

    Ok(())
}
