use clap::Parser;
use color_eyre::eyre::Result;

pub mod blobs;
pub mod cli;
pub mod from_l1;
pub mod info;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    cli::Cli::parse().run().await
}
