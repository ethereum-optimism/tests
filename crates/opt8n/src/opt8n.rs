use crate::evm::to_revm_tx_env;
use alloy::{
    primitives::B256,
    rpc::types::{
        anvil::Forking,
        trace::geth::{
            GethDebugBuiltInTracerType, GethDebugTracerType, GethDebugTracingOptions, GethTrace,
            PreStateConfig, PreStateFrame,
        },
        BlockId,
    },
};

use anvil::{
    cmd::NodeArgs,
    eth::{
        pool::transactions::{PoolTransaction, TransactionPriority},
        EthApi,
    },
    NodeConfig, NodeHandle,
};
use anvil_core::{
    eth::block::Block,
    eth::transaction::{PendingTransaction, TypedTransaction},
};
use cast::traces::{GethTraceBuilder, TracingInspectorConfig};
use std::{fs, path::PathBuf, sync::Arc};

use clap::{CommandFactory, FromArgMatches, Parser};
use color_eyre::eyre::Result;
use futures::StreamExt;
use op_test_vectors::execution::{ExecutionFixture, ExecutionReceipt, ExecutionResult};
use revm::primitives::{BlobExcessGasAndPrice, TxEnv, U256};
use revm::{db::AlloyDB, primitives::BlockEnv, EvmBuilder};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
pub struct Opt8n {
    pub eth_api: EthApi,
    pub node_handle: NodeHandle,
    pub execution_fixture: ExecutionFixture,
    pub fork: Forking,
    pub output_file: PathBuf,
}

impl Opt8n {
    pub async fn new(
        node_config: Option<NodeConfig>,
        fork: Option<Forking>,
        output_file: PathBuf,
    ) -> Self {
        let node_config = node_config.unwrap_or_default().with_optimism(true);
        let (eth_api, node_handle) = anvil::spawn(node_config).await;

        Self {
            eth_api,
            node_handle,
            execution_fixture: ExecutionFixture::default(),
            fork: fork.unwrap_or_default(),
            output_file,
        }
    }

    /// Listens for commands, and new blocks from the block stream.
    pub async fn repl(&mut self) -> Result<()> {
        let mut new_blocks = self.eth_api.backend.new_block_notifications();
        loop {
            tokio::select! {
                command = self.receive_command() => {
                    match command {
                        Ok(ReplCommand::Exit) => break,
                        Ok(command) => self.execute(command).await?,
                        Err(e) => eprintln!("Error: {:?}", e),
                    }
                }

                new_block = new_blocks.next() => {
                    if let Some(new_block) = new_block {
                        if let Some(block) = self.eth_api.backend.get_block_by_hash(new_block.hash) {
                            let transactions = block.transactions.into_iter().map(|tx| tx.transaction).collect::<Vec<_>>();
                            self.execution_fixture.transactions.extend(transactions);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn receive_command(&self) -> Result<ReplCommand> {
        let line = BufReader::new(tokio::io::stdin())
            .lines()
            .next_line()
            .await?
            .unwrap();
        let words = shellwords::split(&line)?;

        let matches = ReplCommand::command().try_get_matches_from(words)?;
        Ok(ReplCommand::from_arg_matches(&matches)?)
    }

    async fn execute(&mut self, command: ReplCommand) -> Result<()> {
        match command {
            ReplCommand::Dump => self.dump_execution_fixture().await?,
            ReplCommand::Anvil { mut args } => {
                args.insert(0, "anvil".to_string());
                let command = NodeArgs::command_for_update();
                let matches = command.try_get_matches_from(args)?;
                let node_args = NodeArgs::from_arg_matches(&matches)?;
                node_args.run().await?;
            }
            ReplCommand::Cast { .. } => {}
            ReplCommand::Exit => unreachable!(),
        }
        Ok(())
    }

    /// Updates the pre and post state allocations of the [ExecutionFixture].
    pub fn update_alloc(&mut self, block: &Block) -> Result<()> {
        let revm_db = AlloyDB::new(
            self.node_handle.http_provider(),
            BlockId::from(block.header.number - 1),
        );

        let blob_excess_gas_and_price = if let Some(excess_gas) = block.header.excess_blob_gas {
            Some(BlobExcessGasAndPrice::new(excess_gas as u64))
        } else {
            None
        };

        let block_env = BlockEnv {
            number: U256::from(block.header.number),
            coinbase: block.header.beneficiary,
            timestamp: U256::from(block.header.timestamp),
            difficulty: block.header.difficulty,
            gas_limit: U256::from(block.header.gas_limit),
            prevrandao: None,
            basefee: U256::from(block.header.base_fee_per_gas.unwrap_or_default()),
            blob_excess_gas_and_price,
        };

        let mut evm = EvmBuilder::default()
            .with_ref_db(Box::new(revm_db))
            .with_block_env(block_env)
            .optimism()
            .build();
        println!("Block number aa: {}", block.header.number);
        println!("Transactions length: {}", block.transactions.len());
        for tx in block.transactions.iter() {
            let tx_env = to_revm_tx_env(tx.transaction.clone())?;
            evm.context.evm.env.tx = tx_env;
            let result = evm.transact()?;
            println!("{:?}", result);
            let db = evm.context.evm.db.0.clone();
            let pre_state_frame = GethTraceBuilder::new(vec![], TracingInspectorConfig::default())
                .geth_prestate_traces(
                    &result,
                    PreStateConfig {
                        diff_mode: Some(true),
                    },
                    db,
                )?;

            println!("Pre state: {:?}", pre_state_frame);

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

    pub async fn dump_execution_fixture(&mut self) -> Result<()> {
        println!("txs: {:?}", self.execution_fixture.transactions.len());
        // Reset the fork
        let _ = self.eth_api.backend.reset_fork(self.fork.clone()).await;
        let pool_txs = self
            .execution_fixture
            .transactions
            .iter()
            .cloned()
            .map(|tx| {
                let gas_price = tx.gas_price();
                let pending_tx = PendingTransaction::new(tx).expect("Failed to create pending tx");
                Arc::new(PoolTransaction {
                    pending_transaction: pending_tx,
                    requires: vec![],
                    provides: vec![],
                    priority: TransactionPriority(gas_price),
                })
            })
            .collect::<Vec<Arc<_>>>();

        println!("pool txs: {:?}", pool_txs);

        let mined_block = self.eth_api.backend.mine_block(pool_txs).await;

        dbg!("mined block", &mined_block);

        if let Some(block) = self.eth_api.backend.get_block(mined_block.block_number) {
            println!("block: {:?}", block);

            // TODO: collect into futures ordered
            let mut receipts: Vec<ExecutionReceipt> = vec![];
            // TODO: This could be done in 1 loop instead of 2
            let ordered_txs = block
                .transactions
                .iter()
                .cloned()
                .map(|tx| tx.transaction)
                .collect::<Vec<_>>();

            for tx in &ordered_txs {
                if let Some(receipt) = self.eth_api.backend.transaction_receipt(tx.hash()).await? {
                    receipts.push(receipt.into());
                }
            }

            self.update_alloc(&block)?;

            let block_header = &block.header;
            let execution_result = ExecutionResult {
                state_root: block_header.state_root,
                tx_root: block_header.transactions_root,
                receipt_root: block_header.receipts_root,
                // TODO: Update logs hash
                logs_hash: B256::default(),
                logs_bloom: block_header.logs_bloom,
                receipts,
            };

            self.execution_fixture.env = block.into();
            self.execution_fixture.result = execution_result;
        }

        // Output the execution fixture to file
        let file = fs::File::create(&self.output_file)?;
        serde_json::to_writer_pretty(file, &self.execution_fixture)?;

        Ok(())
    }
}

#[derive(Parser, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[clap(rename_all = "kebab_case", infer_subcommands = true, multicall = true)]
pub enum ReplCommand {
    #[command(visible_alias = "a")]
    Anvil {
        #[arg(index = 1, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(visible_alias = "c")]
    Cast {
        #[arg(index = 1, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Dump,
    // TODO: implement clear
    // TODO: implement reset
    #[command(visible_alias = "e")]
    Exit,
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
