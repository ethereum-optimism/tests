//! opt8n binary logic

use alloy_eips::eip2718::Encodable2718;
use alloy_eips::BlockId;
use alloy_rpc_types::trace::geth::{PreStateConfig, PreStateFrame};
use anvil::{cmd::NodeArgs, eth::EthApi, NodeConfig, NodeHandle};
use anvil_core::eth::block::Block;
use anvil_core::eth::transaction::PendingTransaction;
use cast::traces::{GethTraceBuilder, TracingInspectorConfig};
use clap::Parser;
use std::{
    error::Error,
    fs::{self, File},
    path::PathBuf,
};

use color_eyre::eyre::{ensure, eyre, Result};
use futures::StreamExt;
use op_test_vectors::execution::{ExecutionFixture, ExecutionReceipt, ExecutionResult};
use revm::{
    db::{AlloyDB, CacheDB},
    primitives::{BlobExcessGasAndPrice, BlockEnv, CfgEnv, Env, SpecId, U256},
    Database, DatabaseCommit, DatabaseRef, Evm, EvmBuilder,
};

#[derive(Parser, Clone, Debug)]
pub struct Opt8nArgs {
    #[clap(long, help = "Output file for the execution test fixture")]
    pub output: PathBuf,
    #[clap(long, help = "Path to genesis state")]
    pub genesis: Option<PathBuf>,
}

pub struct Opt8n {
    pub eth_api: EthApi,
    pub node_handle: NodeHandle,
    pub execution_fixture: ExecutionFixture,
    pub node_config: NodeConfig,
    pub output_file: PathBuf,
}

impl Opt8n {
    pub async fn new(
        node_args: Option<NodeArgs>,
        output_file: PathBuf,
        genesis: Option<PathBuf>,
    ) -> Result<Self> {
        let node_config = if let Some(node_args) = node_args {
            if node_args.evm_opts.fork_url.is_some()
                || node_args.evm_opts.fork_block_number.is_some()
            {
                return Err(eyre!(
                    "Forking is not supported in opt8n, please specify prestate with a genesis file"
                ));
            }

            Some(node_args.into_node_config())
        } else {
            None
        };

        let genesis = if let Some(genesis) = genesis.as_ref() {
            serde_json::from_reader(File::open(genesis)?)?
        } else {
            None
        };

        let node_config = node_config
            .unwrap_or_default()
            .with_optimism(true)
            .with_no_mining(true)
            .with_genesis(genesis);

        let (eth_api, node_handle) = anvil::spawn(node_config.clone()).await;
        eth_api.anvil_set_logging(false).await?;

        Ok(Self {
            eth_api,
            node_handle,
            execution_fixture: ExecutionFixture::default(),
            node_config,
            output_file,
        })
    }

    /// Updates the pre and post state allocations of the [ExecutionFixture] from Revm.
    pub fn capture_pre_post_alloc(&mut self, block: &Block) -> Result<()> {
        let revm_db = CacheDB::new(
            AlloyDB::new(
                self.node_handle.http_provider(),
                BlockId::from(block.header.number - 1),
            )
            .ok_or_else(|| eyre!("Failed to create AlloyDB"))?,
        );

        let mut evm = evm(
            block,
            self.eth_api.chain_id(),
            CacheDB::new(revm_db),
            SpecId::from(self.node_config.hardfork.unwrap_or_default()),
        );

        for tx in block.transactions.iter() {
            let pending = PendingTransaction::new(tx.clone().into())?;
            let mut buff = Vec::<u8>::with_capacity(pending.transaction.encode_2718_len());
            pending.transaction.encode_2718(&mut buff);

            let mut tx_env = pending.to_revm_tx_env();
            tx_env.optimism.enveloped_tx = Some(buff.into());
            evm.context.evm.env.tx = tx_env;

            let result = evm.transact()?;

            let db = &mut evm.context.evm.db;
            let pre_state_frame = GethTraceBuilder::new(vec![], TracingInspectorConfig::default())
                .geth_prestate_traces(
                    &result,
                    PreStateConfig {
                        diff_mode: Some(true),
                    },
                    &db,
                )?;
            db.commit(result.state);

            if let PreStateFrame::Diff(diff) = pre_state_frame {
                diff.pre.into_iter().for_each(|(account, state)| {
                    self.execution_fixture.alloc.entry(account).or_insert(state);
                });
                diff.post.into_iter().for_each(|(account, state)| {
                    self.execution_fixture.out_alloc.insert(account, state);
                });
            }
        }
        Ok(())
    }

    pub async fn mine_block(&mut self) {
        self.eth_api.mine_one().await;
    }

    /// Generates an execution fixture from a block.
    pub async fn generate_execution_fixture(&mut self, block: Block) -> Result<()> {
        self.capture_pre_post_alloc(&block)?;

        // Append block transactions and receipts to the execution fixture
        let mut receipts: Vec<ExecutionReceipt> = Vec::with_capacity(block.transactions.len());
        for tx in block.transactions.iter() {
            if let Some(receipt) = self
                .eth_api
                .backend
                .transaction_receipt(tx.transaction.hash())
                .await?
            {
                receipts.push(receipt.try_into()?);
            }
            self.execution_fixture
                .transactions
                .push(tx.transaction.clone());
        }

        let block_header = &block.header;
        let execution_result = ExecutionResult {
            state_root: block_header.state_root,
            tx_root: block_header.transactions_root,
            receipt_root: block_header.receipts_root,
            logs_bloom: block_header.logs_bloom,
            receipts,
        };

        self.execution_fixture.env = block.into();
        self.execution_fixture.result = execution_result;

        // Ensure pre and post states are different
        ensure!(
            self.execution_fixture.alloc != self.execution_fixture.out_alloc,
            "Pre and post state are the same"
        );

        // Output the execution fixture to file
        let file = fs::File::create(&self.output_file)?;
        serde_json::to_writer_pretty(file, &self.execution_fixture)?;

        Ok(())
    }
}

/// Creates a new EVM instance from a given block, chain, database, and spec id.
pub fn evm<'a, DB>(block: &Block, chain_id: u64, db: DB, spec_id: SpecId) -> Evm<'a, (), Box<DB>>
where
    DB: Database + DatabaseRef + 'a,
    <DB as Database>::Error: Error,
{
    let block_env = BlockEnv {
        number: U256::from(block.header.number),
        coinbase: block.header.beneficiary,
        timestamp: U256::from(block.header.timestamp),
        difficulty: block.header.difficulty,
        gas_limit: U256::from(block.header.gas_limit),
        prevrandao: Some(block.header.mix_hash),
        basefee: U256::from(block.header.base_fee_per_gas.unwrap_or_default()),
        blob_excess_gas_and_price: block
            .header
            .excess_blob_gas
            .map(|excess_gas| BlobExcessGasAndPrice::new(excess_gas as u64)),
    };

    let mut cfg = CfgEnv::default();
    cfg.chain_id = chain_id;
    let env = Env {
        block: block_env,
        cfg,
        ..Default::default()
    };

    let mut evm = EvmBuilder::default()
        .with_db(Box::new(db))
        .with_env(Box::new(env))
        .optimism()
        .build();
    evm.modify_spec_id(spec_id);
    evm
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    pub async fn test_update_alloc() {
        // TODO:
    }

    #[tokio::test]
    pub async fn test_dump_execution_fixture() {
        // TODO:
    }
}
