pub mod cmd;
pub mod opt8n;

use std::path::PathBuf;

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
    #[command(flatten)]
    pub node_args: NodeArgs,
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
    Script {
        #[command(flatten)]
        opt8n_args: Opt8nArgs,
        #[command(flatten)]
        script_args: Box<ScriptArgs>,
    },
    Server {
        #[command(flatten)]
        opt8n_args: Opt8nArgs,
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
    #[clap(long, help = "Output file for the execution test fixture")]
    pub output: PathBuf,
    #[clap(long, help = "Path to genesis state")]
    pub genesis: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    let node_args = args.node_args.clone();
    let opt8n_args = args.command.get_opt8n_args();

    if node_args.evm_opts.fork_url.is_some() || node_args.evm_opts.fork_block_number.is_some() {
        return Err(eyre::eyre!(
            "Forking is not supported in opt8n, please specify prestate with a genesis file"
        ));
    }

    let node_config = node_args.clone().into_node_config();
    let mut opt8n = Opt8n::new(
        Some(node_config),
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
