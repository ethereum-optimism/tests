//! Module containing the execution test fixture.

use alloy_primitives::{Address, Bloom, B256, U256};
use alloy_rpc_types::{trace::geth::AccountState, Block};

use color_eyre::eyre::{self};
use op_alloy_rpc_types::{OpTransactionReceipt, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The execution fixture is the top-level object that contains
/// everything needed to run an execution test.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionFixture {
    /// The execution environment sets up the current block context.
    pub env: ExecutionEnvironment,
    /// The initial state of the accounts before running the transactions, also called the
    /// "pre-state".
    pub alloc: HashMap<Address, AccountState>,
    /// The expected state of the accounts after running the transactions, also called the
    /// "post-state".
    pub out_alloc: HashMap<Address, AccountState>,
    /// Transactions to execute.
    #[serde(rename = "txs")]
    pub transactions: Vec<Transaction>,
    /// The expected result after executing transactions.
    pub result: ExecutionResult,
}

/// The execution environment is the initial state of the execution context.
/// It's used to set the execution environment current block information.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionEnvironment {
    /// The current block coinbase.
    pub current_coinbase: Address,
    /// The current block difficulty.
    pub current_difficulty: U256,
    /// The current block gas limit.
    pub current_gas_limit: U256,
    /// The previous block hash.
    pub previous_hash: B256,
    /// The current block number.
    pub current_number: U256,
    /// The current block timestamp.
    pub current_timestamp: U256,
    /// The block hashes of the previous blocks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hashes: Option<HashMap<U256, B256>>,
}

impl TryFrom<Block> for ExecutionEnvironment {
    type Error = eyre::Error;

    fn try_from(block: Block) -> Result<Self, Self::Error> {
        let block_number = block
            .header
            .number
            .ok_or_else(|| eyre::eyre!("missing block number"))?;

        Ok(Self {
            current_coinbase: block.header.miner,
            current_difficulty: block.header.difficulty,
            current_gas_limit: U256::from(block.header.gas_limit),
            previous_hash: block.header.parent_hash,
            current_number: U256::from(block_number),
            current_timestamp: U256::from(block.header.timestamp),
            block_hashes: None,
        })
    }
}

/// The execution result is the expected result after running the transactions
/// in the execution environment over the pre-state.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResult {
    /// The state root.
    pub state_root: B256,
    /// The transaction root.
    pub tx_root: B256,
    /// The receipt root.
    pub receipt_root: B256,
    /// The logs bloom.
    pub logs_bloom: Bloom,
    /// A list of execution receipts for each executed transaction.
    pub receipts: Vec<OpTransactionReceipt>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_rpc_types::Header;
    use serde_json::Value;

    #[test]
    fn test_serialize_execution_environment() {
        let expected_env = include_str!("./testdata/environment.json");
        let env = serde_json::from_str::<ExecutionEnvironment>(expected_env)
            .expect("failed to parse environment");
        let serialized_env = serde_json::to_string(&env).expect("failed to serialize environment");
        let serialized_value = serde_json::from_str::<Value>(&serialized_env)
            .expect("failed to parse serialized environment");
        let expected_value = serde_json::from_str::<Value>(expected_env)
            .expect("failed to parse expected environment");
        assert_eq!(serialized_value, expected_value);
    }

    #[test]
    fn test_serialize_execution_result() {
        let expected_result = include_str!("./testdata/result.json");
        let execution_result = serde_json::from_str::<ExecutionResult>(expected_result)
            .expect("failed to parse result");
        let serialized_result =
            serde_json::to_string(&execution_result).expect("failed to serialize result");
        let serialized_value = serde_json::from_str::<Value>(&serialized_result)
            .expect("failed to parse serialized result");
        let expected_value = serde_json::from_str::<Value>(expected_result)
            .expect("failed to parse expected result");
        assert_eq!(serialized_value, expected_value);
    }

    #[test]
    fn test_execution_environment_try_from_missing_block() {
        let block = Block {
            header: Header {
                number: None,
                ..Default::default()
            },
            ..Default::default()
        };

        let execution_environment = ExecutionEnvironment::try_from(block);
        assert!(execution_environment.is_err());
    }
}
