pub mod cli;
pub mod opt8n;

use clap::FromArgMatches;
use color_eyre::eyre;

use crate::cli::Cli;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    // let command = Cli::default_command();
    // let matches = command.get_matches();
    // let cli = Cli::from_arg_matches(&matches)?;
    // cli.run().await?;
    Ok(())
}
