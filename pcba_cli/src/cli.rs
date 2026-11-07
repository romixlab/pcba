use clap::{Parser, Subcommand};

/// Opinionated PCB design CLI tool
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Do not make any changes, only print them to terminal for review (default is false)
    #[clap(long, default_value = "false", global = true)]
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
    /// Assembly drawing of a board: SVG page per side with fitted references, DNP crossed out, test points,
    /// drill map, tables (BOM, DNP, test points, drills) and a Typst document that puts them on paper
    Fab {
        /// The board, a .kicad_pcb file
        board: std::path::PathBuf,
        /// Folder for the output (default: `fab/` beside the board)
        #[arg(short, long)]
        out: Option<std::path::PathBuf>,
        /// Settings file (TOML, any field of the defaults shown by --show-settings); default: `pcba.toml`
        /// beside the board when it exists
        #[arg(long)]
        settings: Option<std::path::PathBuf>,
        /// Print the default settings as TOML and exit
        #[arg(long)]
        show_settings: bool,
    },
}
