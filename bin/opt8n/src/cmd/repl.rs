use anvil::cmd::NodeArgs;
use clap::{CommandFactory, FromArgMatches, Parser};
use color_eyre::eyre::eyre;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::opt8n::{Opt8n, Opt8nArgs};

#[derive(Parser, Clone, Debug)]
pub struct ReplArgs {
    #[command(flatten)]
    opt8n_args: Opt8nArgs,
    #[command(flatten)]
    pub node_args: NodeArgs,
}

impl ReplArgs {
    pub async fn run(&self) -> color_eyre::Result<()> {
        let mut opt8n = Opt8n::new(
            Some(self.node_args.clone()),
            self.opt8n_args.output.clone(),
            self.opt8n_args.genesis.clone(),
        )
        .await?;

        opt8n.repl().await?;

        Ok(())
    }
}

#[derive(Parser, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[clap(rename_all = "snake_case", infer_subcommands = true, multicall = true)]
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
    RpcEndpoint,
    // TODO: implement clear
    // TODO: implement reset
    #[command(visible_alias = "e")]
    Exit,
}

/// Listens for commands, and new blocks from the block stream.
pub async fn repl(opt8n: &mut Opt8n) -> color_eyre::Result<()> {
    let mut new_blocks = opt8n.eth_api.backend.new_block_notifications();

    loop {
        tokio::select! {
            command = receive_command() => {
                match command {
                    Ok(ReplCommand::Exit) => break,
                    Ok(command) => execute(opt8n, command).await?,
                    Err(e) => eprintln!("Error: {:?}", e),
                }
            }

            new_block = new_blocks.next() => {
                if let Some(new_block) = new_block {
                    if let Some(block) = opt8n.eth_api.backend.get_block_by_hash(new_block.hash) {
                        opt8n.generate_execution_fixture(block).await?;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn receive_command() -> color_eyre::Result<ReplCommand> {
    let line = BufReader::new(tokio::io::stdin())
        .lines()
        .next_line()
        .await?
        .unwrap();
    let words = shellwords::split(&line)?;

    let matches = ReplCommand::command().try_get_matches_from(words)?;
    Ok(ReplCommand::from_arg_matches(&matches)?)
}

async fn execute(opt8n: &mut Opt8n, command: ReplCommand) -> color_eyre::Result<()> {
    match command {
        ReplCommand::Dump => {
            opt8n.mine_block().await;
        }
        ReplCommand::Anvil { mut args } => {
            args.insert(0, "anvil".to_string());
            let command = NodeArgs::command_for_update();
            let matches = command.try_get_matches_from(args)?;
            let node_args = NodeArgs::from_arg_matches(&matches)?;
            node_args.run().await?;
        }
        ReplCommand::Cast { .. } => {}
        ReplCommand::RpcEndpoint => {
            println!("{}", opt8n.node_handle.http_endpoint());
        }
        ReplCommand::Exit => unreachable!(),
    }
    Ok(())
}
