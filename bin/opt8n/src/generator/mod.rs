//! The reference state transition function.

use crate::cli::state::{DumpBlockResponse, GenesisAccountExt};
use alloy_consensus::{constants::KECCAK_EMPTY, Header};
use alloy_genesis::{Genesis, GenesisAccount};
use alloy_primitives::B256;
use alloy_rlp::{Decodable, Encodable};
use alloy_trie::{HashBuilder, HashMap, Nibbles};
use color_eyre::{eyre::eyre, owo_colors::OwoColorize, Result};
use itertools::Itertools;
use kona_mpt::TrieAccount;
use op_test_vectors::execution::{ExecutionEnvironment, ExecutionFixture, ExecutionResult};
use reth_chainspec::ChainSpec;
use reth_evm::execute::{
    BlockExecutionInput, BlockExecutionOutput, ExecutionOutcome, Executor, ProviderError,
};
use reth_evm_optimism::{OpBlockExecutor, OptimismEvmConfig};
use reth_primitives::{
    keccak256, Address, Block, BlockWithSenders, Bytes, TransactionSigned, U256,
};
use revm::{
    db::{AccountState, CacheDB, DbAccount, EmptyDBTyped, State},
    primitives::{AccountInfo, Bytecode},
};
use std::{collections::BTreeMap, sync::Arc};
use tracing::info;

/// The database for [StateTransition].
type StfDb = CacheDB<EmptyDBTyped<ProviderError>>;

/// The execution fixture generator for `opt8n`.
pub(crate) struct StateTransition {
    /// Inner cache database.
    pub(crate) db: StfDb,
    /// The [ChainSpec] for the devnet chain.
    pub(crate) chain_spec: Arc<ChainSpec>,
}

impl StateTransition {
    /// Create a new execution fixture generator.
    pub(crate) fn new(genesis: Genesis, state_dump: DumpBlockResponse) -> Result<Self> {
        let mut db = CacheDB::default();

        // Insert the genesis allocs
        genesis.alloc.iter().for_each(|(k, v)| {
            let mut info = AccountInfo {
                nonce: v.nonce.unwrap_or_default(),
                balance: v.balance,
                code: v.code.clone().map(Bytecode::new_raw),
                code_hash: KECCAK_EMPTY,
            };

            db.insert_contract(&mut info);
            db.accounts.insert(
                *k,
                DbAccount {
                    info: info.clone(),
                    storage: v
                        .storage
                        .as_ref()
                        .unwrap_or(&BTreeMap::new())
                        .iter()
                        .map(|(k, v)| ((*k).into(), (*v).into()))
                        .collect(),
                    account_state: AccountState::None,
                },
            );
        });

        // Overlay the state dump on top of the genesis allocs.
        state_dump.accounts.iter().for_each(|(k, v)| {
            let GenesisAccountExt { account, .. } = v;

            let mut info = AccountInfo {
                nonce: account.nonce.unwrap_or_default(),
                balance: account.balance,
                code: account.code.clone().map(Bytecode::new_raw),
                code_hash: KECCAK_EMPTY,
            };
            db.insert_contract(&mut info);

            let entry = db.accounts.entry(*k).or_default();
            entry.info = info;
            if let Some(storage) = &account.storage {
                storage.iter().for_each(|(k, v)| {
                    entry.storage.insert((*k).into(), (*v).into());
                });
            }
        });

        Ok(Self {
            db,
            chain_spec: Arc::new(ChainSpec::from(genesis)),
        })
    }

    /// Grab the block executor with the current state.
    pub(crate) fn executor(&mut self) -> OpBlockExecutor<OptimismEvmConfig, StfDb> {
        // Construct an ephemeral state with the current database.
        let state = State::builder()
            .with_database(self.db.clone())
            .with_bundle_update()
            .build();

        // TODO: Custom EVMConfig
        OpBlockExecutor::new(self.chain_spec.clone(), OptimismEvmConfig::default(), state)
    }

    pub(crate) fn execute(
        &mut self,
        prev_header: Header,
        header: Header,
        encoded_txs: Vec<Bytes>,
    ) -> Result<ExecutionFixture> {
        let executor = self.executor();

        let txs = encoded_txs
            .iter()
            .cloned()
            .map(|tx| {
                TransactionSigned::decode(&mut tx.as_ref())
                    .map_err(|e| eyre!("Error decoding transaction: {e}"))
            })
            .collect::<Result<Vec<TransactionSigned>>>()?;
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
            body: txs.clone(),
            ..Default::default()
        };
        let senders = block
            .body
            .iter()
            .map(|tx| tx.recover_signer().ok_or(eyre!("Error recovering signer")))
            .collect::<Result<Vec<_>>>()?;

        // Execute the block.
        info!(target: "stf", "Executing block with {} transactions...", txs.len().cyan());
        let block_with_senders = BlockWithSenders::new(block, senders)
            .ok_or(eyre!("Error creating block with senders"))?;
        let execution_input = BlockExecutionInput::new(&block_with_senders, header.difficulty);
        let BlockExecutionOutput {
            state, receipts, ..
        } = executor.execute(execution_input)?;
        info!(target: "stf", "✅ Block successfully executed.");

        // Flush the bundle state updates to the in-memory database.
        let alloc_db = self.db.clone();
        for (address, account) in &state.state {
            if account.status.is_not_modified() {
                continue;
            }

            let mut info = account.info.clone().unwrap_or_default();
            self.db.insert_contract(&mut info);

            let db_account = self.db.accounts.entry(*address).or_default();
            if account.is_info_changed() {
                db_account.info = info;
            }

            // Insert all storage slots into the account storage trie.
            db_account.storage.extend(
                account
                    .storage
                    .iter()
                    .map(|(k, v)| (*k, v.present_value))
                    .collect::<HashMap<U256, U256>>(),
            );
        }

        // Compute the execution fixture results.
        let root = self.state_root()?;
        let execution_outcome =
            ExecutionOutcome::new(state, receipts.clone().into(), header.number, Vec::new());
        let receipts_root = execution_outcome
            .optimism_receipts_root_slow(header.number, self.chain_spec.as_ref(), header.timestamp)
            .expect("Number is in range");
        let logs_bloom = execution_outcome
            .block_logs_bloom(header.number)
            .expect("Number is in range");
        let transactions_root = reth_primitives::proofs::calculate_transaction_root(&txs);

        // Log the execution fixture results.
        let ind = "~>".magenta().italic().to_string();
        info!(target: "stf", "{} State root: {}", ind, root.cyan());
        info!(target: "stf", "{} Transactions root: {}", ind, transactions_root.cyan());
        info!(target: "stf", "{} Receipts root: {}", ind, receipts_root.cyan());
        info!(target: "stf", "{} Logs bloom: {}", ind, logs_bloom.cyan());

        Ok(ExecutionFixture {
            env: ExecutionEnvironment {
                genesis: {
                    let mut genesis = self.chain_spec.genesis.clone();
                    // strip allocs; they are not needed in the fixture
                    genesis.alloc.clear();
                    genesis
                },
                previous_header: prev_header,
                current_coinbase: header.beneficiary,
                current_difficulty: header.mix_hash.into(),
                current_gas_limit: U256::from(header.gas_limit),
                current_number: U256::from(header.number),
                current_timestamp: U256::from(header.timestamp),
                parent_beacon_block_root: header.parent_beacon_block_root,
                block_hashes: Some(
                    [(U256::from(header.number - 1), header.parent_hash)]
                        .iter()
                        .cloned()
                        .collect::<HashMap<_, _>>(),
                ),
            },
            transactions: encoded_txs,
            result: ExecutionResult {
                state_root: root,
                tx_root: transactions_root,
                receipt_root: receipts_root,
                logs_bloom,
                receipts: receipts
                    .iter()
                    .map(|r| {
                        let mut buf = Vec::with_capacity(r.length());
                        r.encode(&mut buf);
                        buf.into()
                    })
                    .collect::<Vec<_>>(),
            },
            alloc: Self::gen_allocs(alloc_db),
        })
    }

    /// Computes the state root from the data available in [Self::db].
    pub(crate) fn state_root(&self) -> Result<B256> {
        // First, generate all account tries.
        let mut trie_accounts = HashMap::new();
        for (address, account_state) in self.db.accounts.iter() {
            let mut hb = HashBuilder::default();

            // Sort the storage by the hash of the slot, and filter out any storage slots with zero values.
            let sorted_storage = account_state
                .storage
                .iter()
                .filter(|(_, v)| **v != U256::ZERO)
                .sorted_by_key(|(k, _)| keccak256(k.to_be_bytes::<32>()));

            // Insert all non-zero storage slots into the account storage trie, in-order.
            for (slot, value) in sorted_storage {
                let slot_nibbles = Nibbles::unpack(keccak256(slot.to_be_bytes::<32>()));
                let mut value_buf = Vec::with_capacity(value.length());
                value.encode(&mut value_buf);
                hb.add_leaf(slot_nibbles, &value_buf);
            }

            // Compute the root of the account storage trie.
            let storage_root = hb.root();

            // Construct the trie account.
            let trie_account = TrieAccount {
                nonce: account_state.info.nonce,
                balance: account_state.info.balance,
                storage_root,
                code_hash: account_state
                    .info
                    .code
                    .as_ref()
                    .map(|code| match code {
                        Bytecode::LegacyRaw(code) => keccak256(code),
                        Bytecode::LegacyAnalyzed(code) => keccak256(code.bytecode()),
                        _ => panic!("Unsupported bytecode type"),
                    })
                    .unwrap_or(KECCAK_EMPTY),
            };

            trie_accounts.insert(address, trie_account);
        }

        let mut hb = HashBuilder::default();

        // Sort the accounts by the hash of the address.
        let sorted_accounts = trie_accounts.iter().sorted_by_key(|(k, _)| keccak256(k));

        // Insert all accounts into the state trie, in-order.
        for (address, _) in sorted_accounts {
            let trie_account = trie_accounts
                .get(address)
                .ok_or(eyre!("Missing trie account"))?;

            let address_nibbles = Nibbles::unpack(keccak256(address));
            let mut account_buffer = Vec::with_capacity(trie_account.length());
            trie_account.encode(&mut account_buffer);

            hb.add_leaf(address_nibbles, &account_buffer);
        }

        Ok(hb.root())
    }

    /// Transforms a [StfDb] into a map of [Address]es to [GenesisAccount]s.
    fn gen_allocs(db: StfDb) -> HashMap<Address, GenesisAccount> {
        db.accounts
            .iter()
            .map(|(address, account)| {
                let mut storage = BTreeMap::new();
                for (slot, value) in &account.storage {
                    storage.insert((*slot).into(), (*value).into());
                }

                (
                    *address,
                    GenesisAccount {
                        balance: account.info.balance,
                        nonce: Some(account.info.nonce),
                        code: account.info.code.as_ref().map(|code| match code {
                            Bytecode::LegacyRaw(code) => code.clone(),
                            Bytecode::LegacyAnalyzed(code) => code.bytecode().clone(),
                            _ => panic!("Unsupported bytecode type"),
                        }),
                        storage: Some(storage),
                        private_key: None,
                    },
                )
            })
            .collect()
    }
}
