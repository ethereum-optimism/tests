pub mod cli;
pub mod cmd;
pub mod opt8n;

use clap::Parser;
use color_eyre::eyre;

use crate::cli::Cli;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    cli.run().await?;
    Ok(())
}
