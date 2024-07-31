//! Contains logic to generate derivation test fixtures using L1 source block information.

use crate::cmd::blobs;
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Address, B256};
use clap::{ArgAction, Parser};
use color_eyre::{
    eyre::{ensure, eyre},
    Result,
};
use kona_derive::online::{
    AlloyChainProvider, OnlineBeaconClient, OnlineBlobProvider, SimpleSlotDerivation,
};
use kona_derive::traits::ChainProvider;
use op_test_vectors::derivation::{DerivationFixture, FixtureBlock};
use reqwest::Url;
use std::path::PathBuf;
use superchain_registry::ROLLUP_CONFIGS;
use tracing::{info, trace};

/// CLI arguments for the `from-l1` subcommand of `opd8n`.
#[derive(Parser, Clone, Debug)]
pub struct FromL1 {
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
    /// Verbosity level (0-4)
    #[arg(long, short, help = "Verbosity level (0-4)", action = ArgAction::Count)]
    pub v: u8,
}

impl FromL1 {
    /// Runs the derivation test fixture generation using the L1 source block information.
    /// This function effectively takes the L1 block info and fetches any calldata or blob
    /// data associated with this block.
    pub async fn run(&self) -> Result<()> {
        ensure!(
            self.end_block > self.start_block,
            "End block must come after the start block"
        );
        let outputs = if self.derive {
            self.derive().await?
        } else {
            self.outputs.clone()
        };
        ensure!(
            !outputs.is_empty(),
            "Must provide at least one L2 output root"
        );
        trace!(target: "from-l1", "Producing derivation fixture for L1 block range [{}, {}]", self.start_block, self.end_block);

        // Construct a sequential list of block numbers from [start_block, end_block].
        let blocks = (self.start_block..=self.end_block).collect::<Vec<_>>();

        // Construct the providers
        let l1_rpc_url =
            Url::parse(&self.rpc_url).map_err(|e| eyre!("Invalid L1 RPC URL: {}", e))?;
        let mut l1_provider = AlloyChainProvider::new_http(l1_rpc_url);
        let l1_chain_id = l1_provider.chain_id().await.map_err(|e| eyre!(e))?;
        info!(target: "from-l1", "Using L1 Chain ID: {}", l1_chain_id);
        let config = ROLLUP_CONFIGS
            .get(&l1_chain_id)
            .ok_or_else(|| eyre!("No rollup config found for chain id: {}", l1_chain_id))?;
        let batcher_address = config.batch_inbox_address;
        info!(target: "from-l1", "Using batcher address: {}", batcher_address);
        let signer = config
            .genesis
            .system_config
            .as_ref()
            .map(|sc| sc.batcher_address)
            .unwrap_or_default();
        info!(target: "from-l1", "Using signer address: {}", signer);

        let beacon_client = OnlineBeaconClient::new_http(self.beacon_url.clone());
        let mut blob_provider =
            OnlineBlobProvider::<_, SimpleSlotDerivation>::new(beacon_client, None, None);

        // Construct the derivation fixture.
        let fixture_blocks = build_fixture_blocks(
            batcher_address,
            signer,
            &blocks,
            &mut l1_provider,
            &mut blob_provider,
        )
        .await?;
        let fixture = DerivationFixture::new(fixture_blocks, outputs);
        info!(target: "from-l1", "Successfully built derivation test fixture");

        // Write the derivation fixture to the specified output location.
        let file = std::fs::File::create(&self.output)?;
        serde_json::to_writer_pretty(file, &fixture)?;
        info!(target: "from-l1", "Wrote derivation fixture to: {:?}", self.output);

        Ok(())
    }

    /// Derives the L2 output roots from the provided L1 block range.
    pub async fn derive(&self) -> Result<Vec<B256>> {
        panic!("l2 output root derivation is not supported");
    }
}

/// Constructs [FixtureBlock]s for the given L1 blocks.
pub async fn build_fixture_blocks(
    batcher_address: Address,
    signer: Address,
    blocks: &[u64],
    l1_provider: &mut AlloyChainProvider,
    blob_provider: &mut OnlineBlobProvider<OnlineBeaconClient, SimpleSlotDerivation>,
) -> Result<Vec<FixtureBlock>> {
    let mut fixtures = Vec::with_capacity(blocks.len());
    for b in blocks {
        // Fetch the block info by number.
        let block_info = l1_provider
            .block_info_by_number(*b)
            .await
            .map_err(|e| eyre!(e))?;
        let (_, txs) = l1_provider
            .block_info_and_transactions_by_hash(block_info.hash)
            .await
            .map_err(|e| eyre!(e))?;
        let mut transactions = Vec::with_capacity(txs.len());
        for tx in txs.as_slice() {
            let mut out = Vec::new();
            tx.encode_2718(&mut out);
            transactions.push(out.into());
        }

        let blobs = blobs::load(
            &block_info,
            txs.as_slice(),
            batcher_address,
            signer,
            blob_provider,
        )
        .await?;

        let fixture = FixtureBlock {
            number: *b,
            hash: block_info.hash,
            timestamp: block_info.timestamp,
            transactions,
            blobs,
        };
        fixtures.push(fixture);
    }
    Ok(fixtures)
}
