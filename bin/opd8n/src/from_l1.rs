//! Contains logic to generate derivation test fixtures using L1 source block information.

use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Address, B256};
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
use superchain_registry::ROLLUP_CONFIGS;
use tracing::{info, trace};

use crate::cli::L1Args;

/// Runs the derivation test fixture generation using the L1 source block information.
/// This function effectively takes the L1 block info and fetches any calldata or blob
/// data associated with this block.
pub async fn run(args: L1Args) -> Result<()> {
    ensure!(
        args.end_block > args.start_block,
        "End block must come after the start block"
    );
    let outputs = if args.derive {
        derive(&args).await?
    } else {
        args.outputs
    };
    ensure!(
        !outputs.is_empty(),
        "Must provide at least one L2 output root"
    );
    trace!(target: "from-l1", "Producing derivation fixture for L1 block range [{}, {}]", args.start_block, args.end_block);

    // Construct a sequential list of block numbers from [start_block, end_block].
    let blocks = (args.start_block..=args.end_block).collect::<Vec<_>>();

    // Construct the providers
    let l1_rpc_url = Url::parse(&args.rpc_url).map_err(|e| eyre!("Invalid L1 RPC URL: {}", e))?;
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

    let beacon_client = OnlineBeaconClient::new_http(args.beacon_url);
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
    let file = std::fs::File::create(&args.output)?;
    serde_json::to_writer_pretty(file, &fixture)?;
    info!(target: "from-l1", "Wrote derivation fixture to: {:?}", args.output);

    Ok(())
}

/// Derives the L2 output roots from the provided L1 block range.
pub async fn derive(_: &L1Args) -> Result<Vec<B256>> {
    panic!("l2 output root derivation is not supported");
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

        let blobs = crate::blobs::load(
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
