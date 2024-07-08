pub struct Opt8n {
    pub eth_api: EthApi,
    pub node_handle: NodeHandle,
    pub execution_fixture: ExecutionFixture,
}

impl Opt8n {
    pub async fn new(node_config: Option<NodeConfig>) -> Self {
        let node_config = node_config.unwrap_or_default().with_optimism(true);
        let (eth_api, node_handle) = spawn(node_config).await;

        Self {
            eth_api,
            node_handle,
            execution_fixture: ExecutionFixture::default(),
        }
    }

    pub async fn listen(&self) {
        let new_blocks = self.eth_api.backend.new_block_notifications();

        let x = new_blocks.next().await;

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
                            self.execution_fixture.transactions = block.transactions;
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
