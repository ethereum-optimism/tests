use std::str::FromStr;

use anvil::{eth::EthApi, spawn, NodeConfig, NodeHandle};
use clap::Parser;
use op_test_vectors::execution::ExecutionFixture;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
pub struct Args {}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let opt8n = Opt8n::new(None).await;
    opt8n.listen().await;

    Ok(())
}

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
        loop {
            // TODO: this should catch any blocks or transactions advanced by the node and then add them to transactions

            tokio::select! {

                command = self.receive_command() => {
                    if command == Opt8nCommand::Exit {
                        break;
                    }
                    self.execute(command);
                }
                // // TODO: Listen for new block changes
                // self.eth_api.backend => {}


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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Opt8nCommand {
    Anvil(String),
    Cast(String),
    Exit,
    // Help
}

impl FromStr for Opt8nCommand {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim().as_ref() {
            "exit" => Ok(Self::Exit),
            _ => Err(eyre::eyre!("Unrecognized command")),
        }
    }
}
