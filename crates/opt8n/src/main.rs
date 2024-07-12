pub mod cmd;
pub mod opt8n;

use clap::Parser;
use color_eyre::eyre;

use crate::opt8n::Opt8n;
#[derive(Parser)]
pub struct Args {}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _args = Args::parse();
    let mut opt8n = Opt8n::new(None, None).await;
    opt8n.listen().await;
    Ok(())
}
