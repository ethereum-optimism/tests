//! Module containing the derivation test fixture.

use serde::{Deserialize, Serialize};
use alloy::primitives::B256;
use alloy::consensus::Blob;
use anvil_core::eth::transaction::TypedTransaction;

/// The derivation fixture is the top-level object that contains
/// everything needed to run a derivation test.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DerivationFixture {
    /// A list of chain transitions.
    pub transitions: Vec<Transition>,
    /// A list of L1 Blocks to derive from.
    pub blocks: Vec<FixtureBlock>,
}

/// A transition is a change in the chain state from one block to another.
/// Since previous blocks can be further in the past than block N-1, we
/// encompass reorgs.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Transition {
    /// The previous block number.
    pub previous_block: u64,
    /// The current block number.
    pub current_block: u64,
    /// The current block hash.
    pub block_hash: B256,
}

/// A fixture block is a minimal block with associated data including blobs
/// to derive from.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct FixtureBlock {
    /// The block number.
    pub number: u64,
    /// The block hash.
    pub hash: B256,
    /// The block timestamp.
    pub timestamp: u64,
    /// Block Transactions.
    /// EIP-2718 encoded raw transactions
    pub transactions: Vec<TypedTransaction>,
    /// Blobs for this block.
    pub blobs: Vec<Blob>,
}
