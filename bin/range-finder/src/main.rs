use clap::Parser;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    range_finder::Cli::parse().init_telemetry()?.run().await
}
