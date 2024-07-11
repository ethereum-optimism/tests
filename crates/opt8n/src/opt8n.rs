use anvil::cmd::NodeArgs;
use anvil::{eth::EthApi, NodeConfig, NodeHandle};
use clap::{CommandFactory, FromArgMatches};
use color_eyre::eyre::Result;
use futures::StreamExt;
use op_test_vectors::execution::{ExecutionFixture, ExecutionReceipt, ExecutionResult};
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;

use crate::cli::Opt8nCommand;

pub struct Opt8n {
    pub eth_api: EthApi,
    pub node_handle: NodeHandle,
    pub execution_fixture: ExecutionFixture,
}

impl Opt8n {
    pub async fn new(node_config: NodeConfig) -> Self {
        let (eth_api, node_handle) = anvil::spawn(node_config).await;

        Self {
            eth_api,
            node_handle,
            execution_fixture: ExecutionFixture::default(),
        }
    }

    pub async fn listen(&mut self) {
        // TODO: I might ahve to update this to use alloy if the relevent methods are not available
        let mut new_blocks = self.eth_api.backend.new_block_notifications();

        loop {
            tokio::select! {
                command = self.receive_command() => {
                    match command {
                        Ok(Opt8nCommand::Exit) => break,
                        Ok(command) => self.execute(command).await.unwrap(),
                        Err(e) => eprintln!("Error: {:?}", e),
                    }
                }

                new_block = new_blocks.next() => {
                    if let Some(new_block) = new_block {
                        if let Some(block) = self.eth_api.backend.get_block_by_hash(new_block.hash) {
                            let transactions = block.transactions.into_iter().map(|tx| tx.transaction).collect::<Vec<_>>();

                            // TODO: get receipts
                            let receipts: Vec<ExecutionReceipt> = vec![];

                            self.execution_fixture.transactions.extend(transactions);

                            let block_header = block.header;
                            let execution_result = ExecutionResult{
                                state_root: block_header.state_root,
                                tx_root: block_header.transactions_root,
                                receipt_root: block_header.receipts_root,
                                logs_hash: todo!("logs_hash"),
                                logs_bloom: block_header.logs_bloom,
                                receipts,
                            };

                            self.execution_fixture.result = execution_result;
                        }
                    }
                }
            }
        }
    }

    pub async fn receive_command(&self) -> Result<Opt8nCommand> {
        let line = BufReader::new(tokio::io::stdin())
            .lines()
            .next_line()
            .await?
            .unwrap();
        let words = shellwords::split(&line)?;
        println!("Received command: {:?}", words);
        let matches = Opt8nCommand::command().try_get_matches_from(words)?;
        Ok(Opt8nCommand::from_arg_matches(&matches)?)
    }

    pub async fn execute(&self, command: Opt8nCommand) -> Result<()> {
        match command {
            Opt8nCommand::Anvil { mut args } => {
                args.insert(0, "anvil".to_string());
                println!("Args: {:?}", args);
                let command = NodeArgs::command_for_update();
                let matches = command.try_get_matches_from(args)?;
                let node_args = NodeArgs::from_arg_matches(&matches)?;
                node_args.run().await?;
            }
            Opt8nCommand::Cast { .. } => {}
            Opt8nCommand::Exit => unreachable!(),
        }
        Ok(())
    }
}
