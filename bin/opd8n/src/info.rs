//! Info Module

use color_eyre::eyre::{eyre, Result};
use kona_derive::online::AlloyL2ChainProvider;
use kona_derive::traits::L2ChainProvider;
use reqwest::Url;
use std::sync::Arc;
use superchain_registry::ROLLUP_CONFIGS;

/// Runs the info subcommand.
pub async fn run(l2_chain_id: u64, block: u64, rpc_url: String) -> Result<()> {
    let url = Url::parse(&rpc_url).map_err(|e| eyre!("Invalid RPC URL: {}", e))?;
    let rollup_config = ROLLUP_CONFIGS
        .get(&l2_chain_id)
        .ok_or_else(|| eyre!("No rollup config found for chain id: {}", l2_chain_id))?;
    let rollup_config = Arc::new(rollup_config.clone());
    let mut provider = AlloyL2ChainProvider::new_http(url, rollup_config);
    let info = provider
        .l2_block_info_by_number(block)
        .await
        .map_err(|e| eyre!("Failed to fetch block info: {}", e))?;
    println!("{:#?}", info);
    Ok(())
}
