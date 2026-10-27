mod commands;

use clap::Parser;
use commands::Commands;

#[derive(Parser)]
#[command(name = "cargo-lean", version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Validate and manage Cargo.toml package exclude lists")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.command.run()
}
