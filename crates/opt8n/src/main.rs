pub mod cmd;
pub mod opt8n;

use std::str::FromStr;

use anvil::{eth::EthApi, spawn, NodeConfig, NodeHandle};
use clap::Parser;
use futures::stream::StreamExt;
use op_test_vectors::execution::ExecutionFixture;
use serde::{Deserialize, Serialize};
#[derive(Parser)]
pub struct Args {}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let opt8n = Opt8n::new(None).await;
    opt8n.listen().await;
    Ok(())
}
