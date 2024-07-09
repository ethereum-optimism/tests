use anvil::{eth::EthApi, NodeConfig, NodeHandle};
use futures::StreamExt;
use op_test_vectors::execution::ExecutionFixture;

use crate::cmd::Opt8nCommand;

pub struct Opt8n {
    pub eth_api: EthApi,
    pub node_handle: NodeHandle,
    pub execution_fixture: ExecutionFixture,
}

impl Opt8n {
    pub async fn new(node_config: Option<NodeConfig>) -> Self {
        let node_config = node_config.unwrap_or_default().with_optimism(true);
        let (eth_api, node_handle) = anvil::spawn(node_config).await;

        Self {
            eth_api,
            node_handle,
            execution_fixture: ExecutionFixture::default(),
        }
    }

    pub async fn listen(&mut self) {
        let mut new_blocks = self.eth_api.backend.new_block_notifications();

        loop {
            tokio::select! {
                command = self.receive_command() => {
                    if command == Opt8nCommand::Exit {
                        break;
                    }
                    self.execute(command);
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
