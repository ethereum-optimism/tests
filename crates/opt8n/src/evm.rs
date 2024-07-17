use alloy::primitives::SignatureError;
use anvil_core::eth::transaction::TypedTransaction;
use revm::primitives::{OptimismFields, TxEnv, TxKind, U256};

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

    let op_fields: OptimismFields = match &tx {
        TypedTransaction::Deposit(fields) => OptimismFields {
            enveloped_tx: None,
            source_hash: Some(fields.source_hash),
            is_system_transaction: Some(fields.is_system_tx),
            mint: Some(fields.mint.to()),
        },
        _ => OptimismFields::default(),
    };
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
        access_list: tx.essentials().access_list.flattened(),
        gas_priority_fee: tx.essentials().max_priority_fee_per_gas,
        blob_hashes: tx.essentials().blob_versioned_hashes.unwrap_or_default(),
        optimism: op_fields,
    })
}
