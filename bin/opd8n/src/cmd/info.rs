//! Info Module

use clap::{ArgAction, Parser};
use color_eyre::eyre::{eyre, Result};
use kona_derive::online::AlloyL2ChainProvider;
use kona_derive::traits::L2ChainProvider;
use reqwest::Url;
use std::sync::Arc;
use superchain_registry::ROLLUP_CONFIGS;

/// CLI arguments for the `info` subcommand of `opd8n`.
#[derive(Parser, Clone, Debug)]
pub struct Info {
    /// The L2 Chain ID
    #[clap(long, help = "L2 chain ID")]
    l2_chain_id: u64,
    /// The L2 block number to get info for
    #[clap(long, help = "L2 block number")]
    l2_block: u64,
    /// The rpc url to fetch L2 block info from.
    #[clap(long, help = "RPC url to fetch L2 block info from")]
    rpc_url: String,
    /// Verbosity level (0-4)
    #[arg(long, short, help = "Verbosity level (0-4)", action = ArgAction::Count)]
    pub v: u8,
}

impl Info {
    /// Runs the info subcommand.
    pub async fn run(&self) -> Result<()> {
        let url = Url::parse(&self.rpc_url).map_err(|e| eyre!("Invalid RPC URL: {}", e))?;
        let rollup_config = ROLLUP_CONFIGS
            .get(&self.l2_chain_id)
            .ok_or_else(|| eyre!("No rollup config found for chain id: {}", self.l2_chain_id))?;
        let rollup_config = Arc::new(rollup_config.clone());
        let mut provider = AlloyL2ChainProvider::new_http(url, rollup_config);
        let info = provider
            .l2_block_info_by_number(self.l2_block)
            .await
            .map_err(|e| eyre!("Failed to fetch block info: {}", e))?;
        println!("{:#?}", info);
        Ok(())
    }
}
