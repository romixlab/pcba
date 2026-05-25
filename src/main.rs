use crate::cli::Commands;
use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod format;
mod util;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    match cli.command {
        Commands::New { .. } => commands::new::new(cli.dry_run)?,
    }
    Ok(())
}
