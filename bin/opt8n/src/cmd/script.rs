use anvil::cmd::NodeArgs;
use clap::Parser;

use crate::opt8n::{Opt8n, Opt8nArgs};

#[derive(Parser, Clone, Debug)]
pub struct ScriptArgs {
    #[command(flatten)]
    opt8n_args: Opt8nArgs,
    #[command(flatten)]
    inner: forge_script::ScriptArgs,
    #[command(flatten)]
    pub node_args: NodeArgs,
}

impl ScriptArgs {
    pub async fn run(mut self) -> color_eyre::Result<()> {
        let opt8n = Opt8n::new(
            Some(self.node_args.clone()),
            self.opt8n_args.output.clone(),
            self.opt8n_args.genesis.clone(),
        )
        .await?;

        foundry_common::shell::set_shell(foundry_common::shell::Shell::from_args(
            self.inner.opts.silent,
            self.inner.json,
        ))?;

        self.inner.broadcast = true;
        self.inner.evm_opts.sender = Some(
            opt8n
                .node_handle
                .genesis_accounts()
                .last()
                .expect("Could not get genesis account"),
        );
        self.inner.unlocked = true;
        self.inner.evm_opts.fork_url = Some(opt8n.node_handle.http_endpoint());

        opt8n.run_script(Box::new(self.inner)).await?;

        Ok(())
    }
}
