//! Module containing the derivation test fixture.

use alloy::consensus::Blob;
use alloy::primitives::B256;
use anvil_core::eth::transaction::TypedTransaction;
use serde::{Deserialize, Serialize};

/// The derivation fixture is the top-level object that contains
/// everything needed to run a derivation test.
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
pub struct DerivationFixture {
    /// A list of L1 Blocks to derive from.
    pub blocks: Vec<FixtureBlock>,
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

    fn ref_blocks() -> Vec<FixtureBlock> {
        vec![
            FixtureBlock {
                number: 1,
                hash: b256!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                timestamp: 102,
                transactions: vec![],
                blobs: vec![],
            },
            FixtureBlock {
                number: 2,
                hash: b256!("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"),
                timestamp: 104,
                transactions: vec![],
                blobs: vec![],
            },
            FixtureBlock {
                number: 2,
                hash: b256!("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"),
                timestamp: 106,
                transactions: vec![],
                blobs: vec![],
            },
        ]
    }

    #[test]
    fn test_derivation_fixture() {
        let fixture_str = include_str!("./testdata/derivation_fixture.json");
        let fixture: DerivationFixture = serde_json::from_str(fixture_str).unwrap();
        assert_eq!(fixture.blocks, ref_blocks());
    }

    #[test]
    fn test_fixture_block() {
        let fixture_str = include_str!("./testdata/fixture_block.json");
        let fixture: FixtureBlock = serde_json::from_str(fixture_str).unwrap();
        assert_eq!(fixture.number, 1);
        assert_eq!(
            fixture.hash,
            b256!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
        );
        assert_eq!(fixture.timestamp, 102);
        assert_eq!(fixture.transactions.len(), 1);
        assert_eq!(fixture.blobs.len(), 0);
    }
}
