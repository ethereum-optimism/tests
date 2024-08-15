use anvil::cmd::NodeArgs;
use clap::Parser;
use color_eyre::eyre::eyre;

use crate::{opt8n::Opt8n, Opt8nArgs};

#[derive(Parser, Clone, Debug)]
pub struct ScriptArgs {
    #[command(flatten)]
    opt8n_args: Opt8nArgs,
    #[command(flatten)]
    node_args: NodeArgs,
}
