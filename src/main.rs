use clap::Parser;

pub mod url;
pub mod commands;

use commands::*;

// TODO: use tracing library

fn main() -> anyhow::Result<()> {
    Cli::parse().command.execute()
}
