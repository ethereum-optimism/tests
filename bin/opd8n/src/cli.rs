use clap::Parser;
use color_eyre::eyre::Result;
use std::path::PathBuf;

/// Main opd8n CLI
#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Subcommands for the CLI
    #[command(subcommand)]
    pub command: Commands,
}

/// Subcommands for the CLI
#[derive(Parser, Clone, Debug)]
pub enum Commands {
    /// Loads L2 input from a file
    #[command(visible_alias = "l")]
    Load {
        /// Path to the L2 info file
        #[clap(short, long, help = "Path to the L2 block info file")]
        input: PathBuf,
    },
    // TODO: Add another subcommand that provides an interactive method for generating the
    // derivation test fixtures
}

impl Cli {
    /// Parse the CLI arguments and run the command
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::Load { input } => Self::load(input).await,
        }
    }

    /// Loads the L2 input file and generates the test fixtures
    async fn load(input: PathBuf) -> Result<()> {
        println!("Loading L2 input from file: {:?}", input);
        Ok(())
    }
}
