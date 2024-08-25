use clap::Parser;

mod cli;
mod generator;
mod proxy;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    cli::Cli::parse().init_telemetry()?.run().await
}
