use clap::{Parser, Subcommand};

/// Opinionated PCB design CLI tool
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Do not make any changes, only print them to terminal for review (default is false)
    #[clap(long, default_value = "false")]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new PCBA folder structure
    New {
        /// Create full folder structure (default is simplified)
        #[arg(short, long)]
        full: bool,

        /// Use mx backend to retrieve next unique board serial (not a public tool yet)
        #[arg(long)]
        mx: bool,
    },
}
