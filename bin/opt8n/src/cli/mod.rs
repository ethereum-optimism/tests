//! The CLI for `opt8n`

use alloy_primitives::Address;
use clap::{ArgAction, Parser};
use color_eyre::eyre::{eyre, Result};
use std::path::PathBuf;
use t8n::T8n;
use tokio::sync::broadcast::{self};
use tracing::Level;

pub(crate) mod deposits;
pub(crate) mod state;
pub(crate) mod t8n;

/// The root CLI for `opt8n`
#[derive(Parser, Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Cli {
    /// Verbosity level (0-2)
    #[arg(long, short, action = ArgAction::Count)]
    pub(crate) v: u8,
    /// The port of the L1 execution layer node.
    #[arg(long)]
    pub(crate) l1_port: u16,
    /// The port of the L2 execution layer node.
    #[arg(long)]
    pub(crate) l2_port: u16,
    /// The path to the L2 genesis file.
    #[arg(long)]
    pub(crate) l2_genesis: PathBuf,
    /// The address of the OptimismPortal contract on the L1 chain.
    #[arg(long, short)]
    pub(crate) optimism_portal_address: Address,
    /// The output file to write the [ExecutionFixture] to.
    ///
    /// [ExecutionFixture]: op_test_vectors::execution::ExecutionFixture
    #[arg(long)]
    pub(crate) output: Option<PathBuf>,
}

impl Cli {
    /// Initializes telemtry for the application.
    pub(crate) fn init_telemetry(self) -> Result<Self> {
        color_eyre::install()?;
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(match self.v {
                0 => Level::INFO,
                1 => Level::DEBUG,
                _ => Level::TRACE,
            })
            .finish();
        tracing::subscriber::set_global_default(subscriber).map_err(|e| eyre!(e))?;
        Ok(self)
    }

    /// Parse the CLI arguments and run the command
    pub(crate) async fn run(self) -> Result<()> {
        // Set the interrupt handler.
        let (sender, _) = broadcast::channel(256);
        let sender_ctrlc = sender.clone();
        ctrlc::set_handler(move || {
            let _ = sender_ctrlc.send(());
        })?;

        T8n::new(&self, sender).run().await
    }
}
