//! The [ExecutionFixture] generator for `opt8n`.

use alloy_consensus::Header;
use alloy_primitives::B256;
use alloy_rlp::Decodable;
use color_eyre::{eyre::eyre, Result};
use reth_chainspec::OP_MAINNET;
use reth_evm::execute::{BlockExecutionInput, BlockExecutionOutput, Executor, ProviderError};
use reth_evm_optimism::{OpBlockExecutor, OptimismEvmConfig};
use reth_primitives::{Block, BlockWithSenders, Bytes, TransactionSigned, U256};
use revm::db::{CacheDB, DbAccount, EmptyDBTyped, State};

/// The database for [STF].
type STFDB = CacheDB<EmptyDBTyped<ProviderError>>;

/// The execution fixture generator for `opt8n`.
pub(crate) struct STF {
    pub(crate) db: STFDB,
}

impl STF {
    /// Create a new execution fixture generator.
    pub(crate) fn new(db: STFDB) -> Self {
        Self { db }
    }

    /// Grab the block executor with the current state.
    pub(crate) fn executor(&mut self) -> OpBlockExecutor<OptimismEvmConfig, &mut STFDB> {
        let state = State::builder()
            .with_database(&mut self.db)
            .with_bundle_update()
            .build();

        // TODO: Custom EVMConfig
        OpBlockExecutor::new(OP_MAINNET.clone(), OptimismEvmConfig::default(), state)
    }

    pub(crate) fn execute(&mut self, header: Header, transactions: Vec<Bytes>) -> Result<()> {
        let mut executor = self.executor();

        // header fields that matter:
        // block_env.number = U256::from(header.number);
        // block_env.coinbase = header.beneficiary;
        // block_env.timestamp = U256::from(header.timestamp);
        // if after_merge {
        //     block_env.prevrandao = Some(header.mix_hash);
        //     block_env.difficulty = U256::ZERO;
        // } else {
        //     block_env.difficulty = header.difficulty;
        //     block_env.prevrandao = None;
        // }
        // block_env.basefee = U256::from(header.base_fee_per_gas.unwrap_or_default());
        // block_env.gas_limit = U256::from(header.gas_limit);
        //
        // // EIP-4844 excess blob gas of this block, introduced in Cancun
        // if let Some(excess_blob_gas) = header.excess_blob_gas {
        //     block_env.set_blob_excess_gas_and_price(excess_blob_gas);
        // }
        let block = Block {
            header: reth_primitives::Header {
                parent_hash: header.parent_hash,
                ommers_hash: header.ommers_hash,
                beneficiary: header.beneficiary,
                state_root: header.state_root,
                transactions_root: header.transactions_root,
                receipts_root: header.receipts_root,
                withdrawals_root: header.withdrawals_root,
                logs_bloom: header.logs_bloom,
                difficulty: header.difficulty,
                number: header.number,
                gas_limit: header.gas_limit as u64,
                gas_used: header.gas_used as u64,
                timestamp: header.timestamp,
                mix_hash: header.mix_hash,
                nonce: u64::from_be_bytes(*header.nonce),
                base_fee_per_gas: header.base_fee_per_gas.map(|x| x as u64),
                blob_gas_used: header.blob_gas_used.map(|x| x as u64),
                excess_blob_gas: header.excess_blob_gas.map(|x| x as u64),
                parent_beacon_block_root: header.parent_beacon_block_root,
                requests_root: header.requests_root,
                extra_data: header.extra_data,
            },
            body: transactions
                .into_iter()
                .map(|tx| {
                    TransactionSigned::decode(&mut tx.as_ref())
                        .map_err(|e| eyre!("Error decoding transaction: {e}"))
                })
                .collect::<Result<Vec<TransactionSigned>>>()?,
            ..Default::default()
        };
        let senders = block
            .body
            .iter()
            .map(|tx| tx.recover_signer().ok_or(eyre!("Error recovering signer")))
            .collect::<Result<Vec<_>>>()?;

        // Execute the block.
        let block_with_senders = BlockWithSenders::new(block, senders)
            .ok_or(eyre!("Error creating block with senders"))?;
        let execution_input = BlockExecutionInput::new(&block_with_senders, U256::ZERO);
        let BlockExecutionOutput {
            state,
            receipts,
            gas_used,
            ..
        } = executor.execute(execution_input)?;

        // Flush the bundle state updates to the in-memory database.
        for (address, account) in state.state {
            let mut info = account.info.unwrap_or_default();
            self.db.insert_contract(&mut info);
            self.db.accounts.insert(
                address,
                DbAccount {
                    info,
                    storage: account
                        .storage
                        .iter()
                        .map(|(k, v)| (*k, v.present_value))
                        .collect(),
                    ..Default::default()
                },
            );
        }

        // Compute the state root

        Ok(())
    }

    /// Computes the state root from the data available in [Self::db].
    pub(crate) fn state_root(&self) -> B256 {
        todo!()
    }
}
