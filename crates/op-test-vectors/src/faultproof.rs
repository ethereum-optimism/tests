//! Module containing the fault proof test fixture.

use alloy_primitives::{BlockNumber, Bytes, ChainId, B256, U256};
use serde::{Deserialize, Serialize};
use hashbrown::HashMap;

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
  pub witness_data: HashMap<U256, Bytes>,
}

/// The fault proof inputs are the inputs to the fault proof test.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FaultProofInputs {
  /// The L1 head block hash.
  pub l1_head: B256,
  /// The L2 head block hash.
  pub l2_head: B256,
  /// The claimed L2 output root to validate.
  pub l2_claim: B256,
  /// The agreed L2 output root to start derivation from.
  pub l2_output_root: B256,
  /// The L2 block number that the claim is from.
  pub l2_block_number: BlockNumber,
  /// The L2 chain ID that the claim is from.
  pub l2_chain_id: ChainId,
}

/// The fault proof status is the result of executing the fault proof program.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
pub enum FaultProofStatus {
  /// The claim is valid.
  #[default]
  Valid,
  /// The claim is invalid.
  Invalid,
  /// Executing the program resulted in a panic.
  Panic,
  /// The status is unknown.
  Unknown
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_serialize_fault_proof_status() {
    let statuses = vec![
      FaultProofStatus::Valid,
      FaultProofStatus::Invalid,
      FaultProofStatus::Panic,
      FaultProofStatus::Unknown,
    ];

    for status in statuses {
      let serialized_status = serde_json::to_string(&status).expect("failed to serialize status");
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
      l2_chain_id: 42,
    };

    let serialized_inputs = serde_json::to_string(&inputs).expect("failed to serialize inputs");
    let deserialized_inputs = serde_json::from_str::<FaultProofInputs>(&serialized_inputs)
      .expect("failed to deserialize inputs");
    assert_eq!(inputs, deserialized_inputs);
  }

  #[test]
  fn test_serialize_fault_proof_fixture() {
    let mut witness_data = HashMap::new();
    witness_data.insert(U256::from(1), Bytes::from([1; 32]));
    witness_data.insert(U256::from(2), Bytes::from([2; 32]));

    let fixture = FaultProofFixture {
      inputs: FaultProofInputs {
        l1_head: B256::from([1; 32]),
        l2_head: B256::from([2; 32]),
        l2_claim: B256::from([3; 32]),
        l2_output_root: B256::from([4; 32]),
        l2_block_number: 1337,
        l2_chain_id: 42,
      },
      expected_status: FaultProofStatus::Valid,
      witness_data: witness_data,
    };

    let serialized_fixture = serde_json::to_string(&fixture).expect("failed to serialize fixture");
    let deserialized_fixture = serde_json::from_str::<FaultProofFixture>(&serialized_fixture)
      .expect("failed to deserialize fixture");
    assert_eq!(fixture, deserialized_fixture);
  }
}