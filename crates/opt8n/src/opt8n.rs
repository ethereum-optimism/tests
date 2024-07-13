use std::{hash::Hash, sync::Arc};

use crate::cmd::Opt8nCommand;
use alloy::{
    primitives::B256,
    rpc::types::{
        anvil::Forking,
        trace::geth::{GethDebugTracingOptions, GethTrace, PreStateConfig, PreStateFrame},
    },
};
use anvil::{
    eth::{
        pool::transactions::{PoolTransaction, TransactionPriority},
        EthApi,
    },
    NodeConfig, NodeHandle,
};
use anvil_core::eth::transaction::{PendingTransaction, TypedTransaction};
use futures::StreamExt;
use op_test_vectors::execution::{ExecutionFixture, ExecutionReceipt, ExecutionResult};

pub struct Opt8n {
    pub eth_api: EthApi,
    pub node_handle: NodeHandle,
    pub execution_fixture: ExecutionFixture,
    pub fork: Forking,
}

impl Opt8n {
    pub async fn new(node_config: Option<NodeConfig>, fork: Option<Forking>) -> Self {
        let node_config = node_config.unwrap_or_default().with_optimism(true);
        let (eth_api, node_handle) = anvil::spawn(node_config).await;

        Self {
            eth_api,
            node_handle,
            execution_fixture: ExecutionFixture::default(),
            fork: fork.unwrap_or_default(),
        }
    }

    pub async fn listen(&mut self) {
        // TODO: I might have to update this to use alloy if the relevent methods are not available
        let mut new_blocks = self.eth_api.backend.new_block_notifications();
        loop {
            tokio::select! {
                command = self.receive_command() => {
                    // TODO: Update to save fixture cmd
                    if command == Opt8nCommand::Exit {
                        // Reset the fork
                        let _ = self.eth_api.backend.reset_fork(self.fork.clone()).await;
                        let pool_txs = self.execution_fixture.transactions.iter().cloned().map(|tx| {
                            Arc::new(PoolTransaction {
                                pending_transaction: PendingTransaction::new(tx).expect("Failed to create pending transaction"),
                                requires: vec![],
                                provides: vec![],
                                priority: TransactionPriority(1) // TODO: revisit these fields
                            })
                        }).collect::<Vec<Arc<_>>>();

                        let mined_block = self.eth_api.backend.mine_block(pool_txs).await;
                        if let Some(block) = self.eth_api.backend.get_block(mined_block.block_number) {

                            // TODO: collect into futures ordered
                            let mut receipts: Vec<ExecutionReceipt> = vec![];
                            for tx in &block.transactions {
                                if let Some(receipt) = self.eth_api.backend.transaction_receipt(tx.transaction.hash()).await.expect("Failed to get receipt") {
                                    receipts.push(receipt.into());
                                }
                            }

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
                        break;
                    }
                    self.execute(command);
                }

                new_block = new_blocks.next() => {
                    if let Some(new_block) = new_block {
                        if let Some(block) = self.eth_api.backend.get_block_by_hash(new_block.hash) {
                            let transactions = block.transactions.into_iter().map(|tx| tx.transaction).collect::<Vec<_>>();
                            self.update_alloc(&transactions).await;

                            self.execution_fixture.transactions.extend(transactions);

                        }
                    }
                }
            }
        }
    }

    /// Updates the pre and post state allocations of the [ExecutionFixture].
    async fn update_alloc(&mut self, transactions: &Vec<TypedTransaction>) {
        // TODO: Make this concurrent
        for transaction in transactions {
            if let Ok(GethTrace::PreStateTracer(PreStateFrame::Diff(frame))) = self
                .eth_api
                .backend
                .debug_trace_transaction(
                    transaction.hash(),
                    GethDebugTracingOptions::default().with_prestate_config(PreStateConfig {
                        diff_mode: Some(true),
                    }),
                )
                .await
            {
                frame.pre.into_iter().for_each(|(address, account)| {
                    self.execution_fixture
                        .alloc
                        .entry(address)
                        .or_insert(account);
                });

                frame.post.into_iter().for_each(|(address, account)| {
                    self.execution_fixture.out_alloc.insert(address, account);
                });
            }
        }
    }

    pub async fn receive_command(&self) -> Opt8nCommand {
        todo!()
    }

    pub fn execute(&self, command: Opt8nCommand) {
        match command {
            Opt8nCommand::Cast(_) => todo!(),
            _ => unreachable!("Unrecognized command"),
        }
    }
}
