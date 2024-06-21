use std::collections::BTreeMap;

use alloy::primitives::{Address, Bytes, B256};
use alloy::rpc::types::trace::geth::AccountState;
use alloy::rpc::types::{ReceiptEnvelope, Transaction};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionTestVector {
    pub pre_state: BTreeMap<Address, AccountState>,
    pub transactions: Vec<Transaction>,
    pub post_state: ExecutionPostState,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionPostState {
    pub state_root: B256,
    pub tx_root: B256,
    pub receipt_root: B256,
    pub logs_hash: B256,
    pub logs_bloom: Bytes,
    pub receipts: Vec<ReceiptEnvelope>,
}
