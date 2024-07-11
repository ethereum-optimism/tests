use crate::opt8n::Opt8n;
use clap::{Parser, Subcommand};
use color_eyre::eyre;
use forge_script::ScriptArgs;
use std::fmt::Debug;

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Clone, Debug)]
#[clap(rename_all = "kebab_case", infer_subcommands = true)]
pub enum Command {
    /// Uses a forge script to generate a test vector
    #[command(visible_alias = "s")]
    Script {
        #[command(flatten)]
        script_args: ScriptArgs,
    },

    /// Starts a REPL for running forge, anvil, and cast commands
    #[command(visible_alias = "r")]
    Repl {},
}

impl Cli {
    pub async fn run(&self) -> eyre::Result<()> {
        match &self.command {
            Command::Script { script_args } => {
                println!("Running script: {}", script_args.path);
                Ok(())
            }
            Command::Repl { .. } => {
                println!("Starting REPL");
                let mut opt8n = Opt8n::new(None).await;
                opt8n.listen().await;
                Ok(())
            }
        }
    }
}
