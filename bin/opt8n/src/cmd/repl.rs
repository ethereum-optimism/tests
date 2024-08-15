use anvil::cmd::NodeArgs;
use clap::Parser;
use color_eyre::eyre::eyre;

use crate::{opt8n::Opt8n, Opt8nArgs};

#[derive(Parser, Clone, Debug)]
pub struct ReplArgs {
    #[command(flatten)]
    opt8n_args: Opt8nArgs,
    #[command(flatten)]
    pub node_args: NodeArgs,
}

impl ReplArgs {
    pub async fn run(&self) -> color_eyre::Result<()> {
        if self.node_args.evm_opts.fork_url.is_some()
            || self.node_args.evm_opts.fork_block_number.is_some()
        {
            return Err(eyre!(
                "Forking is not supported in opt8n, please specify prestate with a genesis file"
            ));
        }

        let node_config = self.node_args.clone().into_node_config();
        let mut opt8n = Opt8n::new(
            Some(node_config),
            self.opt8n_args.output.clone(),
            self.opt8n_args.genesis.clone(),
        )
        .await?;

        opt8n.repl().await?;

        Ok(())
    }
}
