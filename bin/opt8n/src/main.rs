pub mod opt8n;
use std::path::PathBuf;

use alloy::rpc::types::anvil::Forking;
use anvil::cmd::NodeArgs;
use clap::Parser;
use color_eyre::eyre;
use forge_script::ScriptArgs;
use opt8n::Opt8n;

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Clone, Debug)]
pub enum Commands {
    /// Starts a REPL for running forge, anvil, and cast commands
    #[command(visible_alias = "r")]
    Repl {
        #[command(flatten)]
        opt8n_args: Opt8nArgs,
    },
    /// Uses a forge script to generate a test vector
    #[command(visible_alias = "s")]
    Script {
        #[command(flatten)]
        opt8n_args: Opt8nArgs,
        #[command(flatten)]
        script_args: Box<ScriptArgs>,
    },
}

impl Commands {
    fn get_opt8n_args(&self) -> &Opt8nArgs {
        match self {
            Commands::Repl { opt8n_args } => opt8n_args,
            Commands::Script { opt8n_args, .. } => opt8n_args,
        }
    }
}

#[derive(Parser, Clone, Debug)]
pub struct Opt8nArgs {
    #[command(flatten)]
    pub node_args: NodeArgs,
    #[clap(short, long, help = "Output file for the execution test fixture")]
    pub output: PathBuf,
    #[clap(short, long, help = "Path to genesis state")]
    pub genesis: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    let opt8n_args = args.command.get_opt8n_args();

    let evm_options = &opt8n_args.node_args.evm_opts;
    let forking = evm_options.fork_url.as_ref().map(|fork_url| Forking {
        json_rpc_url: Some(fork_url.url.clone()),
        block_number: evm_options.fork_block_number,
    });

    let node_config = opt8n_args.node_args.clone().into_node_config();
    let mut opt8n = Opt8n::new(
        Some(node_config),
        forking,
        opt8n_args.output.clone(),
        opt8n_args.genesis.clone(),
    )
    .await?;

    match args.command {
        Commands::Repl { .. } => {
            opt8n.repl().await?;
        }
        Commands::Script {
            mut script_args, ..
        } => {
            foundry_common::shell::set_shell(foundry_common::shell::Shell::from_args(
                script_args.opts.silent,
                script_args.json,
            ))?;

            script_args.broadcast = true;
            script_args.evm_opts.sender = Some(
                opt8n
                    .node_handle
                    .genesis_accounts()
                    .last()
                    .expect("Could not get genesis account"),
            );
            script_args.unlocked = true;
            script_args.evm_opts.fork_url = Some(opt8n.node_handle.http_endpoint());

            opt8n.run_script(script_args).await?;
        }
    }

    Ok(())
}
