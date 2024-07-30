use clap::Parser;
use color_eyre::eyre::Result;

pub mod cli;
pub mod from_l1;
pub mod blobs;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    cli::Cli::parse().run().await
}
