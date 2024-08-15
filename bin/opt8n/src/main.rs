pub mod cmd;
pub mod opt8n;

use std::path::PathBuf;

use anvil::cmd::NodeArgs;
use clap::Parser;
use cmd::repl::ReplArgs;
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
    Repl(ReplArgs),
    // /// Uses a forge script to generate a test vector
    // Script {
    //     #[command(flatten)]
    //     opt8n_args: Opt8nArgs,
    //     #[command(flatten)]
    //     script_args: Box<ScriptArgs>,
    // },
    // Server {
    //     #[command(flatten)]
    //     opt8n_args: Opt8nArgs,
    // },
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
    let command = Args::parse().command;

    match command {
        Commands::Repl(cmd) => cmd.run().await?,
    }

    // match args.command {
    //     Commands::Script {
    //         mut script_args, ..
    //     } => {
    //         foundry_common::shell::set_shell(foundry_common::shell::Shell::from_args(
    //             script_args.opts.silent,
    //             script_args.json,
    //         ))?;

    //         script_args.broadcast = true;
    //         script_args.evm_opts.sender = Some(
    //             opt8n
    //                 .node_handle
    //                 .genesis_accounts()
    //                 .last()
    //                 .expect("Could not get genesis account"),
    //         );
    //         script_args.unlocked = true;
    //         script_args.evm_opts.fork_url = Some(opt8n.node_handle.http_endpoint());

    //         opt8n.run_script(script_args).await?;
    //     }
    // }

    Ok(())
}
