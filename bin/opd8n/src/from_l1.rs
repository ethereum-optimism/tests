//! Contains logic to generate derivation test fixtures using L1 source block information.

use color_eyre::{eyre::{eyre, ensure}, Result};
use tracing::{info, trace};
use alloy::primitives::{Address, B256};
use reqwest::Url;
use alloy_eips::eip2718::Encodable2718;
use op_test_vectors::derivation::{FixtureBlock, DerivationFixture};
use kona_derive::traits::ChainProvider;
use kona_derive::online::{AlloyChainProvider, OnlineBeaconClient, OnlineBlobProvider, SimpleSlotDerivation};

use crate::cli::L1Args;

/// Runs the derivation test fixture generation using the L1 source block information.
/// This function effectively takes the L1 block info and fetches any calldata or blob
/// data associated with this block.
pub async fn run(args: L1Args) -> Result<()> {
    ensure!(!args.blocks.is_empty(), "Must provide at least one L1 block number");
    let outputs = if args.derive { derive(&args).await? } else { args.outputs };
    ensure!(!outputs.is_empty(), "Must provide at least one L2 output root");
    let min = args.blocks.iter().min().ok_or_else(|| eyre!("No minimum block number"))?;
    let max = args.blocks.iter().max().ok_or_else(|| eyre!("No maximum block number"))?;
    trace!(target: "from-l1", "Producing derivation fixture for L1 block range [{}, {}]", min, max);

    // Construct the providers
    let l1_rpc_url = Url::parse(&args.rpc_url).map_err(|e| eyre!("Invalid L1 RPC URL: {}", e))?;
    let mut l1_provider = AlloyChainProvider::new_http(l1_rpc_url);
    let beacon_client = OnlineBeaconClient::new_http(args.beacon_url);
    let mut blob_provider =
        OnlineBlobProvider::<_, SimpleSlotDerivation>::new(beacon_client, None, None);

    // Construct the derivation fixture.
    let fixture_blocks = build_fixture_blocks(args.batcher_address, args.signer, &args.blocks, &mut l1_provider, &mut blob_provider).await?;
    let fixture = DerivationFixture::new(fixture_blocks, outputs);

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
pub async fn build_fixture_blocks(batcher_address: Address, signer: Address, blocks: &[u64], l1_provider: &mut AlloyChainProvider, blob_provider: &mut OnlineBlobProvider<OnlineBeaconClient, SimpleSlotDerivation>) -> Result<Vec<FixtureBlock>> {
    let mut fixtures = Vec::with_capacity(blocks.len());
    for b in blocks {
        // Fetch the block info by number.
        let block_info = l1_provider.block_info_by_number(*b).await.map_err(|e| eyre!(e))?;
        let (_, txs) = l1_provider.block_info_and_transactions_by_hash(block_info.hash).await.map_err(|e| eyre!(e))?;
        let mut transactions = Vec::with_capacity(txs.len());
        for tx in txs {
            let mut out = Vec::new();
            tx.encode_2718(&mut out);
            transactions.push(out.into());
        }

        let blobs = crate::blobs::load(&block_info, &txs, batcher_address, signer, blob_provider).await?;

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
