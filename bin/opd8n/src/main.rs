use clap::Parser;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    opd8n::setup()?;
    opd8n::Cli::parse().run().await
}
