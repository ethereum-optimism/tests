use anvil::cmd::NodeArgs;
use clap::Parser;

use crate::opt8n::Opt8nArgs;

#[derive(Parser, Clone, Debug)]
pub struct ServerArgs {
    #[command(flatten)]
    pub opt8n_args: Opt8nArgs,
    #[command(flatten)]
    pub node_args: NodeArgs,
}

impl ServerArgs {
    pub async fn run(&self) -> color_eyre::Result<()> {
        unimplemented!()
    }
}
