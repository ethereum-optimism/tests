use std::collections::HashMap;

use alloy::primitives::{Address, Bloom, B256, U256};
use alloy::rpc::types::trace::geth::AccountState;
use op_alloy_consensus::{OpReceiptEnvelope, OpTypedTransaction};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ExecutionFixture {
    pub env: ExecutionEnvironment,
    pub alloc: HashMap<Address, AccountState>,
    #[serde(rename = "txs")]
    pub transactions: Vec<OpTypedTransaction>,
    pub result: ExecutionResult,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionEnvironment {
    pub current_coinbase: Address,
    pub current_difficulty: U256,
    pub current_gas_limit: U256,
    pub previous_hash: B256,
    pub current_number: U256,
    pub current_timestamp: U256,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hashes: Option<HashMap<U256, B256>>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResult {
    pub state_root: B256,
    pub tx_root: B256,
    pub receipt_root: B256,
    pub logs_hash: B256,
    pub logs_bloom: Bloom,
    pub receipts: Vec<ExecutionReceipt>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionReceipt {
    pub root: B256,
    pub transaction_hash: B256,
    pub contract_address: Address,
    pub gas_used: U256,
    pub block_hash: B256,
    pub transaction_index: U256,
    #[serde(flatten)]
    pub op_receipt: OpReceiptEnvelope,
}

#[cfg(test)]
mod tests {

    use super::ExecutionEnvironment;
    use crate::execution::ExecutionResult;
    use color_eyre::eyre;
    use serde_json::Value;

    #[test]
    fn test_serialize_execution_environment() -> eyre::Result<()> {
        let expected_env = r#"  
        {
            "currentCoinbase" : "0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba",
            "currentDifficulty" : "0x20000",
            "currentGasLimit" : "0x5f5e100",
            "currentNumber" : "0x1",
            "currentTimestamp" : "0x3e8",
            "previousHash" : "0xe729de3fec21e30bea3d56adb01ed14bc107273c2775f9355afb10f594a10d9e",
            "blockHashes" : {
                "0x0" : "0xe729de3fec21e30bea3d56adb01ed14bc107273c2775f9355afb10f594a10d9e"
            }
         }
        "#;

        let env = serde_json::from_str::<ExecutionEnvironment>(expected_env)?;
        let serialized_env = serde_json::to_string(&env)?;

        assert_eq!(
            serde_json::from_str::<Value>(expected_env)?,
            serde_json::from_str::<Value>(&serialized_env)?
        );

        Ok(())
    }

    #[test]
    fn test_serialize_execution_result() -> eyre::Result<()> {
        let expected_result = r#"
        {
            "stateRoot": "0x1c99b01120e7a2fa1301b3505f20100e72362e5ac3f96854420e56ba8984d716",
            "txRoot": "0xb5eee60b45801179cbde3781b9a5dee9b3111554618c9cda3d6f7e351fd41e0b",
            "receiptRoot": "0x86ceb80cb6bef8fe4ac0f1c99409f67cb2554c4432f374e399b94884eb3e6562",
            "logsHash": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
            "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "receipts": [
               {
                   "root": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                   "status": "0x1",
                   "type": "0x0",
                   "cumulativeGasUsed": "0xa878",
                   "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                   "logs": [],
                   "transactionHash": "0x4e6549e2276d1bc256b2a56ead2d9705a51a8bf54e3775fbd2e98c91fb0e4494",
                   "contractAddress": "0x0000000000000000000000000000000000000000",
                   "gasUsed": "0xa878",
                   "blockHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                   "transactionIndex": "0x0"
               }
            ]
          }
        "#;

        let execution_result = serde_json::from_str::<ExecutionResult>(expected_result)?;
        let serialized_result = serde_json::to_string(&execution_result)?;

        assert_eq!(
            serde_json::from_str::<Value>(expected_result)?,
            serde_json::from_str::<Value>(&serialized_result)?
        );

        Ok(())
    }
}
