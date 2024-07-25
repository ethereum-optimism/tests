//! Module containing the derivation test fixture.

use serde::{Deserialize, Serialize};
use alloy::primitives::B256;
use alloy::consensus::Blob;
use anvil_core::eth::transaction::TypedTransaction;

/// The derivation fixture is the top-level object that contains
/// everything needed to run a derivation test.
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
pub struct DerivationFixture {
    /// A list of chain transitions.
    pub transitions: Vec<Transition>,
    /// A list of L1 Blocks to derive from.
    pub blocks: Vec<FixtureBlock>,
}

/// A transition is a change in the chain state from one block to another.
/// Since previous blocks can be further in the past than block N-1, we
/// encompass reorgs.
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
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
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
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
    pub blobs: Vec<Box<Blob>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::b256;

    fn ref_transitions() -> Vec<Transition> {
        vec![
            Transition {
                previous_block: 1,
                current_block: 2,
                block_hash: b256!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            },
            Transition {
                previous_block: 2,
                current_block: 3,
                block_hash: b256!("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"),
            },
            Transition {
                previous_block: 3,
                current_block: 4,
                block_hash: b256!("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"),
            },
        ]
    }

    fn ref_blocks() -> Vec<FixtureBlock> {
        vec![
            FixtureBlock {
                number: 1,
                hash: b256!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                timestamp: 102,
                transactions: vec![],
                blobs: vec![],
            }
        ]
    }

    #[test]
    fn test_derivation_fixture() {
        let fixture_str = include_str!("./testdata/derivation_fixture.json");
        let fixture: DerivationFixture = serde_json::from_str(fixture_str).unwrap();
        assert_eq!(fixture.transitions, ref_transitions());
        assert_eq!(fixture.blocks, ref_blocks());
    }

    #[test]
    fn test_fixture_block() {
        let fixture_str = include_str!("./testdata/fixture_block.json");
        let fixture: FixtureBlock = serde_json::from_str(fixture_str).unwrap();
        assert_eq!(fixture.number, 1);
        assert_eq!(fixture.hash, b256!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"));
        assert_eq!(fixture.timestamp, 102);
        assert_eq!(fixture.transactions.len(), 1);
        assert_eq!(fixture.blobs.len(), 0);
    }

    #[test]
    fn test_transitions() {
        let transitions_str = include_str!("./testdata/transitions.json");
        let transitions: Vec<Transition> = serde_json::from_str(transitions_str).unwrap();
        assert_eq!(transitions, ref_transitions());
    }
}
