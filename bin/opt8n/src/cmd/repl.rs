use anvil::cmd::NodeArgs;
use clap::Parser;
use color_eyre::eyre::eyre;

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
