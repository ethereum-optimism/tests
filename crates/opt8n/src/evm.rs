use alloy::primitives::SignatureError;
use anvil_core::eth::transaction::TypedTransaction;
use revm::primitives::{TxEnv, TxKind, U256};

pub fn to_revm_tx_env(tx: TypedTransaction) -> Result<TxEnv, SignatureError> {
    let transact_to = if let Some(to) = tx.to() {
        TxKind::Call(to)
    } else {
        TxKind::Create
    };

    let max_fee_per_blob_gas = if let Some(max_fee_per_blob_gas) = tx.max_fee_per_blob_gas() {
        Some(U256::from(max_fee_per_blob_gas))
    } else {
        None
    };

    let from = tx.recover()?;

    Ok(TxEnv {
        gas_price: U256::from(tx.gas_price()),
        gas_limit: tx.gas_limit() as u64,
        value: U256::from(tx.value()),
        nonce: Some(tx.nonce()),
        caller: from,
        transact_to,
        data: tx.data().clone(),
        chain_id: tx.chain_id(),
        max_fee_per_blob_gas,
        ..Default::default()
    })
}
