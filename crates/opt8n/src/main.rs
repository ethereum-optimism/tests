pub mod opt8n;

use alloy::rpc::types::anvil::Forking;
use anvil::cmd::NodeArgs;
use clap::{FromArgMatches, Parser};
use color_eyre::eyre;
use forge_script::ScriptArgs;
use opt8n::{ForkChoice, Opt8n};

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
    #[command(flatten)]
    pub fork_url: ForkChoice,
    #[command(flatten)]
    pub node_args: NodeArgs,
}

#[derive(Parser, Clone, Debug)]
pub enum Commands {
    /// Uses a forge script to generate a test vector
    #[command(visible_alias = "s")]
    Script {
        #[command(flatten)]
        script_args: Box<ScriptArgs>,
    },

    /// Starts a REPL for running forge, anvil, and cast commands
    #[command(visible_alias = "r")]
    Repl {},
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    match args.command {
        Commands::Script { script_args } => {
            todo!("Implement script")
        }

        Commands::Repl {} => {
            let node_config = args.node_args.into_node_config();
            let forking = args.fork_url.into();
            let mut opt8n = Opt8n::new(Some(node_config), Some(forking)).await;
            opt8n.repl().await?;
        }
    }

    Ok(())
}
