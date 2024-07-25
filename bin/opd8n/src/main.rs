use clap::Parser;
use color_eyre::eyre::Result;

pub mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    cli::Cli::parse().run().await
}
