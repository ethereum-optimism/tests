use std::collections::HashMap;

use alloy::primitives::{Address, Bloom, B256, U256};
use alloy::rpc::types::trace::geth::AccountState;
use op_alloy_consensus::{OpReceiptEnvelope, OpTypedTransaction};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionFixture {
    pub env: ExecutionEnvironment,
    pub alloc: HashMap<Address, AccountState>,
    pub txs: Vec<OpTypedTransaction>,
    pub result: ExecutionResult,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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
    use alloy::{
        primitives::{Address, Bloom, B256},
        rpc::types::ReceiptWithBloom,
    };
    use op_alloy_consensus::OpReceiptEnvelope;
    use serde_json::Value;

    use super::ExecutionEnvironment;
    use crate::execution::{ExecutionFixture, ExecutionReceipt, ExecutionResult};

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

        let env = serde_json::from_str::<ExecutionEnvironment>(&expected_env)?;
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

    #[test]
    fn test_serialize_execution_fixture() -> eyre::Result<()> {
        let expected_fixture = r#"
        {
            "env":  {
                "currentCoinbase" : "0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba",
                "currentDifficulty" : "0x20000",
                "currentGasLimit" : "0x5f5e100",
                "currentNumber" : "0x1",
                "currentTimestamp" : "0x3e8",
                "previousHash" : "0xe729de3fec21e30bea3d56adb01ed14bc107273c2775f9355afb10f594a10d9e",
                "blockHashes" : {
                    "0x0" : "0xe729de3fec21e30bea3d56adb01ed14bc107273c2775f9355afb10f594a10d9e"
                }
             },
            "alloc": {
                "0x8a0a19589531694250d570040a0c4b74576919b8": {
                    "nonce": 0,
                    "balance": "0x0de0b6b3a7640000",
                    "code": "0x600060006000600060007310000000000000000000000000000000000000015af1600155600060006000600060007310000000000000000000000000000000000000025af16002553d600060003e600051600355",
                    "storage": {
                        "0x01": "0x0100",
                        "0x02": "0x0100",
                        "0x03": "0x0100"
                    }
                },
                "0x1000000000000000000000000000000000000001": {
                    "nonce": 0,
                    "balance": "0x29a2241af62c0000",
                    "code": "0x6103e8ff",
                    "storage": {}
                },
                "0x1000000000000000000000000000000000000002": {
                    "nonce": 0,
                    "balance": "0x4563918244f40000",
                    "code": "0x600060006000600060647310000000000000000000000000000000000000015af1600f0160005260206000fd",
                    "storage": {}
                },
                "0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b": {
                    "nonce": 0,
                    "balance": "0x6124fee993bc0000",
                    "code": "0x",
                    "storage": {}
                }
            },
            "txs": [
                {
                  "gas": "0x4ef00",
                  "maxPriorityFeePerGas": "0x2",
                  "maxFeePerGas": "0x12A05F200",
                  "chainId": "0x1",
                  "input": "0x",
                  "nonce": 0,
                  "to": "0x000000000000000000000000000000000000aaaa",
                  "value": "0x0",
                  "type" : "0x2",
                  "accessList": [
                    {"address": "0x000000000000000000000000000000000000aaaa",
                      "storageKeys": [
                        "0x0000000000000000000000000000000000000000000000000000000000000000"
                      ]
                    }
                  ],
                  "v": "0x0",
                  "r": "0x0",
                  "s": "0x0",
                },
                {
                  "gas": "0x4ef00",
                  "gasPrice": "0x12A05F200",
                  "chainId": "0x1",
                  "input": "0x",
                  "nonce": 1,
                  "to": "0x000000000000000000000000000000000000aaaa",
                  "value": "0x0",
                  "v": "0x0",
                  "r": "0x0",
                  "s": "0x0",
                }
              ],
            "result": {
                "stateRoot": "0x1c99b01120e7a2fa1301b3505f20100e72362e5ac3f96854420e56ba8984d716",
                "txRoot": "0xb5eee60b45801179cbde3781b9a5dee9b3111554618c9cda3d6f7e351fd41e0b",
                "receiptRoot": "0x86ceb80cb6bef8fe4ac0f1c99409f67cb2554c4432f374e399b94884eb3e6562",
                "logsHash": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                "receipts": [
                   {
                       "root": "0x0",
                       "status": "0x1",
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
        }
        "#;

        let fixture = serde_json::from_str::<ExecutionFixture>(expected_fixture)?;

        let serialized_fixture = serde_json::to_string(&fixture)?;

        assert_eq!(
            serde_json::from_str::<Value>(expected_fixture)?,
            serde_json::from_str::<Value>(&serialized_fixture)?
        );

        Ok(())
    }
}
