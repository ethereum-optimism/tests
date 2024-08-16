//! Utilities

use kona_derive::types::{L2ExecutionPayloadEnvelope, L2PayloadAttributes, RawTransaction};

/// Converts an [L2ExecutionPayloadEnvelope] to an [L2PayloadAttributes].
pub fn to_payload_attributes(payload: L2ExecutionPayloadEnvelope) -> L2PayloadAttributes {
    L2PayloadAttributes {
        timestamp: payload.execution_payload.timestamp,
        prev_randao: payload.execution_payload.prev_randao,
        fee_recipient: payload.execution_payload.fee_recipient,
        withdrawals: payload.execution_payload.withdrawals.clone(),
        parent_beacon_block_root: payload.parent_beacon_block_root,
        gas_limit: Some(payload.execution_payload.gas_limit as u64),
        transactions: payload
            .execution_payload
            .transactions
            .into_iter()
            .map(RawTransaction)
            .collect(),
        no_tx_pool: true,
    }
}
