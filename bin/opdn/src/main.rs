use clap::Parser;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    opdn::Cli::parse().init_telemetry()?.run().await
}
