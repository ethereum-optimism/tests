pub mod cmd;
pub mod opt8n;

use crate::cmd::script::ScriptArgs;
use clap::Parser;
use cmd::repl::ReplArgs;
use cmd::server::ServerArgs;
use color_eyre::eyre;

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Clone, Debug)]
pub enum Commands {
    Repl(ReplArgs),
    Script(ScriptArgs),
    Server(ServerArgs),
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let command = Args::parse().command;

    match command {
        Commands::Repl(cmd) => cmd.run().await?,
        Commands::Script(cmd) => cmd.run().await?,
        Commands::Server(cmd) => cmd.run().await?,
    }

    Ok(())
}
