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
    #[command(flatten)]
    pub node_args: NodeArgs,
    #[clap(short, long, help = "Output file for the execution test fixture")]
    pub output: PathBuf,
    #[clap(short, long, help = "A path to a Genesis state")]
    pub genesis: Option<PathBuf>,
}

#[derive(Parser, Clone, Debug)]
pub enum Commands {
    /// Starts a REPL for running forge, anvil, and cast commands
    #[command(visible_alias = "r")]
    Repl {},
    /// Uses a forge script to generate a test vector
    #[command(visible_alias = "s")]
    Script {
        #[command(flatten)]
        script_args: ScriptArgs,
    },
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let evm_options = &args.node_args.evm_opts;
    let forking = evm_options.fork_url.as_ref().map(|fork_url| Forking {
        json_rpc_url: Some(fork_url.url.clone()),
        block_number: evm_options.fork_block_number,
    });

    let node_config = args.node_args.into_node_config();
    let mut opt8n = Opt8n::new(Some(node_config), forking, args.output, args.genesis).await;

    match args.command {
        Commands::Repl {} => {
            opt8n.repl().await?;
        }
        Commands::Script {
            script_args: _script_args,
        } => {
            // TODO: Run foundry script, pass the opt8n anvil instance endpoint to the script

            // foundry_common::shell::set_shell(foundry_common::shell::Shell::from_args(
            //     cmd.opts.silent,
            //     cmd.json,
            // ))?;
            // utils::block_on(cmd.run_script())

            opt8n.mine_block().await;
        }
    }

    Ok(())
}
