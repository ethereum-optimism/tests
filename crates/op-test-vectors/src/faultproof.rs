//! Module containing the fault proof test fixture.

use alloy_primitives::{BlockHash, BlockNumber, Bytes, B256};
use hashbrown::HashMap;
use kona_primitives::RollupConfig;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// The fault proof fixture is the top-level object that contains
/// everything needed to run a fault proof test.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FaultProofFixture {
    /// The inputs to the fault proof test.
    pub inputs: FaultProofInputs,
    /// The expected status of the fault proof test.
    pub expected_status: FaultProofStatus,
    /// The witness data for the fault proof test.
    pub witness_data: HashMap<B256, Bytes>,
}

/// The fault proof inputs are the inputs to the fault proof test.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FaultProofInputs {
    /// The L1 head block hash.
    pub l1_head: BlockHash,
    /// The L2 head block hash.
    pub l2_head: BlockHash,
    /// The claimed L2 output root to validate.
    pub l2_claim: B256,
    /// The agreed L2 output root to start derivation from.
    pub l2_output_root: B256,
    /// The L2 block number that the claim is from.
    pub l2_block_number: BlockNumber,
    /// The configuration of the l2 chain.
    pub rollup_config: RollupConfig,
}

/// The fault proof status is the result of executing the fault proof program.
#[derive(Serialize_repr, Deserialize_repr, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum FaultProofStatus {
    /// The claim is valid.
    #[default]
    Valid = 0,
    /// The claim is invalid.
    Invalid = 1,
    /// Executing the program resulted in a panic.
    Panic = 2,
    /// The program has not exited.
    Unfinished = 3,
    /// The status is unknown.
    Unknown,
}

impl TryFrom<u8> for FaultProofStatus {
        type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FaultProofStatus::Valid),
            1 => Ok(FaultProofStatus::Invalid),
            2 => Ok(FaultProofStatus::Panic),
            3 => Ok(FaultProofStatus::Unfinished),
            _ => Ok(FaultProofStatus::Unknown),
        }
    }
}

impl From<FaultProofStatus> for u8 {
    fn from(status: FaultProofStatus) -> u8 {
        status as u8
    }
}

#[cfg(test)]
mod tests {
    use kona_primitives::BASE_MAINNET_CONFIG;

    use super::*;

    #[test]
    fn test_serialize_fault_proof_status() {
        let statuses = vec![
            FaultProofStatus::Valid,
            FaultProofStatus::Invalid,
            FaultProofStatus::Panic,
            FaultProofStatus::Unfinished,
            FaultProofStatus::Unknown,
        ];

        for status in statuses {
            let serialized_status =
                serde_json::to_string(&status).expect("failed to serialize status");
            let deserialized_status = serde_json::from_str::<FaultProofStatus>(&serialized_status)
                .expect("failed to deserialize status");
            assert_eq!(status, deserialized_status);
        }
    }

    #[test]
    fn test_serialize_fault_proof_inputs() {
        let inputs = FaultProofInputs {
            l1_head: B256::from([1; 32]),
            l2_head: B256::from([2; 32]),
            l2_claim: B256::from([3; 32]),
            l2_output_root: B256::from([4; 32]),
            l2_block_number: 1337,
            rollup_config: BASE_MAINNET_CONFIG,
        };

        let serialized_inputs = serde_json::to_string(&inputs).expect("failed to serialize inputs");
        let deserialized_inputs = serde_json::from_str::<FaultProofInputs>(&serialized_inputs)
            .expect("failed to deserialize inputs");
        assert_eq!(inputs, deserialized_inputs);
    }

    #[test]
    fn test_serialize_fault_proof_fixture() {
        let mut witness_data = HashMap::new();
        witness_data.insert(B256::random(), Bytes::from([1; 32]));
        witness_data.insert(B256::random(), Bytes::from([2; 32]));

        let fixture = FaultProofFixture {
            inputs: FaultProofInputs {
                l1_head: B256::from([1; 32]),
                l2_head: B256::from([2; 32]),
                l2_claim: B256::from([3; 32]),
                l2_output_root: B256::from([4; 32]),
                l2_block_number: 1337,
                rollup_config: BASE_MAINNET_CONFIG,
            },
            expected_status: FaultProofStatus::Valid,
            witness_data,
        };

        let serialized_fixture =
            serde_json::to_string(&fixture).expect("failed to serialize fixture");
        let deserialized_fixture = serde_json::from_str::<FaultProofFixture>(&serialized_fixture)
            .expect("failed to deserialize fixture");
        assert_eq!(fixture, deserialized_fixture);
    }
}
