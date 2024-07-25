//! Module containing the execution test fixture.

use alloy::primitives::{Address, Bloom, B256, U256};
use alloy::rpc::types::trace::geth::AccountState;
use alloy::rpc::types::{Log, TransactionReceipt};
use anvil_core::eth::block::Block;
use anvil_core::eth::transaction::{TypedReceipt, TypedTransaction};
use color_eyre::eyre;
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
    pub transactions: Vec<TypedTransaction>,
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

impl From<Block> for ExecutionEnvironment {
    fn from(block: Block) -> Self {
        Self {
            current_coinbase: block.header.beneficiary,
            current_difficulty: block.header.difficulty,
            current_gas_limit: U256::from(block.header.gas_limit),
            previous_hash: block.header.parent_hash,
            current_number: U256::from(block.header.number),
            current_timestamp: U256::from(block.header.timestamp),
            block_hashes: None,
        }
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
    /// The logs hash.
    pub logs_hash: B256,
    /// The logs bloom.
    pub logs_bloom: Bloom,
    /// A list of execution receipts for each executed transaction.
    pub receipts: Vec<ExecutionReceipt>,
}

/// An execution receipt is the result of running a transaction in the execution environment.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionReceipt {
    /// The state root.
    pub root: B256,
    /// The hash of the transaction.
    pub transaction_hash: B256,
    /// The contract address that the transaction created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_address: Option<Address>,
    /// The gas used by the transaction.
    pub gas_used: U256,
    /// The block hash.
    pub block_hash: B256,
    /// The transaction index.
    pub transaction_index: U256,
    /// The inner log receipt.
    #[serde(flatten)]
    pub inner: TypedReceipt<Log>,
}

impl TryFrom<TransactionReceipt<TypedReceipt<Log>>> for ExecutionReceipt {
    type Error = eyre::Error;

    fn try_from(receipt: TransactionReceipt<TypedReceipt<Log>>) -> eyre::Result<Self> {
        Ok(Self {
            transaction_hash: receipt.transaction_hash,
            root: receipt
                .state_root
                .ok_or_else(|| eyre::eyre!("missing state root"))?,
            contract_address: receipt.contract_address,
            gas_used: U256::from(receipt.gas_used),
            block_hash: receipt
                .block_hash
                .ok_or_else(|| eyre::eyre!("missing block hash"))?,
            transaction_index: U256::from(
                receipt
                    .transaction_index
                    .ok_or_else(|| eyre::eyre!("missing transaction index"))?,
            ),
            inner: receipt.inner,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_exec_receipt_try_from_tx_receipt() {
        let tx_receipt_str = include_str!("./testdata/tx_receipt.json");
        let tx_receipt: TransactionReceipt<TypedReceipt<Log>> =
            serde_json::from_str(tx_receipt_str).expect("failed to parse tx receipt");
        let exec_receipt = ExecutionReceipt::try_from(tx_receipt.clone())
            .expect("failed to convert tx receipt to exec receipt");
        assert_eq!(exec_receipt.transaction_hash, tx_receipt.transaction_hash);
        assert_eq!(exec_receipt.root, tx_receipt.state_root.unwrap());
        assert_eq!(exec_receipt.contract_address, tx_receipt.contract_address);
        assert_eq!(exec_receipt.gas_used, U256::from(tx_receipt.gas_used));
        assert_eq!(exec_receipt.block_hash, tx_receipt.block_hash.unwrap());
        assert_eq!(
            exec_receipt.transaction_index,
            U256::from(tx_receipt.transaction_index.unwrap())
        );
        assert_eq!(exec_receipt.inner, tx_receipt.inner);
    }

    #[test]
    fn test_exec_receipt_try_from_missing_root() {
        let tx_receipt_str = include_str!("./testdata/tx_receipt.json");
        let mut tx_receipt: TransactionReceipt<TypedReceipt<Log>> =
            serde_json::from_str(tx_receipt_str).expect("failed to parse tx receipt");
        tx_receipt.state_root = None;
        let exec_receipt = ExecutionReceipt::try_from(tx_receipt);
        assert!(exec_receipt.is_err());
    }

    #[test]
    fn test_exec_receipt_try_from_missing_block_hash() {
        let tx_receipt_str = include_str!("./testdata/tx_receipt.json");
        let mut tx_receipt: TransactionReceipt<TypedReceipt<Log>> =
            serde_json::from_str(tx_receipt_str).expect("failed to parse tx receipt");
        tx_receipt.block_hash = None;
        let exec_receipt = ExecutionReceipt::try_from(tx_receipt);
        assert!(exec_receipt.is_err());
    }

    #[test]
    fn test_exec_receipt_try_from_missing_tx_index() {
        let tx_receipt_str = include_str!("./testdata/tx_receipt.json");
        let mut tx_receipt: TransactionReceipt<TypedReceipt<Log>> =
            serde_json::from_str(tx_receipt_str).expect("failed to parse tx receipt");
        tx_receipt.transaction_index = None;
        let exec_receipt = ExecutionReceipt::try_from(tx_receipt);
        assert!(exec_receipt.is_err());
    }
}
