//! From L2 Subcommand

use clap::{ArgAction, Parser};
use color_eyre::eyre::Result;
use std::path::PathBuf;

/// CLI arguments for the `from-l2` subcommand of `opd8n`.
#[derive(Parser, Clone, Debug)]
pub struct FromL2 {
    /// Path to the L2 info file
    #[clap(short, long, help = "Path to the L2 block info file")]
    input: PathBuf,
    /// Verbosity level (0-4)
    #[arg(long, short, help = "Verbosity level (0-4)", action = ArgAction::Count)]
    pub v: u8,
}

impl FromL2 {
    /// Runs the from-l2 subcommand.
    pub async fn run(&self) -> Result<()> {
        unimplemented!()
    }
}
