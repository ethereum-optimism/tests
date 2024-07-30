//! CLI for the opd8n tool

use alloy_primitives::B256;
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
    FromL2 {
        /// Path to the L2 info file
        #[clap(short, long, help = "Path to the L2 block info file")]
        input: PathBuf,
    },
    /// Pulls in the batch calldata or blob data for a given L1 Block
    FromL1 {
        /// Arguments for the L1 command
        #[command(flatten)]
        l1_args: L1Args,
    },
    /// Gets the L2 block info including the l1 origin for the l2 block number.
    #[command(visible_alias = "i")]
    Info {
        /// The L2 Chain ID
        #[clap(long, help = "L2 chain ID")]
        l2_chain_id: u64,
        /// The L2 block number to get info for
        #[clap(long, help = "L2 block number")]
        l2_block: u64,
        /// The rpc url to fetch L2 block info from.
        #[clap(long, help = "RPC url to fetch L2 block info from")]
        rpc_url: String,
    },
}

/// CLI arguments for the `from-l1` subcommand of `opd8n`.
#[derive(Parser, Clone, Debug)]
pub struct L1Args {
    /// The L1 block number to start from
    #[clap(short, long, help = "Starting L1 block number")]
    pub start_block: u64,
    /// The L1 block number to end at
    #[clap(short, long, help = "Ending L1 block number")]
    pub end_block: u64,
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
}

impl Cli {
    /// Parse the CLI arguments and run the command
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::FromL2 { input } => Self::load(input).await,
            Commands::FromL1 { l1_args } => crate::from_l1::run(l1_args).await,
            Commands::Info {
                l2_chain_id,
                l2_block,
                rpc_url,
            } => crate::info::run(l2_chain_id, l2_block, rpc_url).await,
        }
    }

    /// Loads the L2 input file and generates the test fixtures
    async fn load(input: PathBuf) -> Result<()> {
        println!("Loading L2 input from file: {:?}", input);
        Ok(())
    }
}
