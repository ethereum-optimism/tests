use clap::Parser;
use color_eyre::eyre::Result;
use std::path::PathBuf;
use alloy::primitives::{Address, B256};

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
    FromL2 {
        /// Path to the L2 info file
        #[clap(short, long, help = "Path to the L2 block info file")]
        input: PathBuf,
    },
    /// Pulls in the batch calldata or blob data for a given L1 Block
    FromL1 {
        #[command(flatten)]
        l1_args: L1Args,
    },
}

#[derive(Parser, Clone, Debug)]
pub struct L1Args {
    /// A list of L1 blocks to fetch data from.
    #[clap(short, long, help = "L1 block number")]
    pub blocks: Vec<u64>,
    /// A list of L2 output root hashes to assert against.
    #[clap(short, long, help = "L2 output root hashes")]
    pub outputs: Vec<B256>,
    /// An RPC URL to fetch L1 block data from.
    #[clap(long, help = "RPC url to fetch L1 block data from")]
    pub rpc_url: String,
    /// A beacon client to fetch blob data from.
    #[clap(long, help = "Beacon client url to fetch blob data from")]
    pub beacon_url: String,
    /// Optionally derive l1 output roots by running derivation over
    /// the provided l1 block range.
    #[clap(long, help = "Derive L1 output roots")]
    pub derive: bool,
    /// The output file for the test fixture.
    #[clap(long, help = "Output file for the test fixture")]
    pub output: PathBuf,
    /// The batcher address.
    #[clap(long, help = "The batcher address to check against")]
    pub batcher_address: Address,
    /// The signer address.
    #[clap(long, help = "The signer to check against")]
    pub signer: Address,
}

impl Cli {
    /// Parse the CLI arguments and run the command
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::FromL2 { input } => Self::load(input).await,
            Commands::FromL1 { l1_args } => crate::from_l1::run(l1_args).await,
        }
    }

    /// Loads the L2 input file and generates the test fixtures
    async fn load(input: PathBuf) -> Result<()> {
        println!("Loading L2 input from file: {:?}", input);
        Ok(())
    }
}
