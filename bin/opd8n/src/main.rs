use clap::Parser;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    opd8n::Cli::parse().init_telemetry()?.run().await
}
