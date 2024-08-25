//! The `opt8n` application.

use super::{deposits::DepositCapture, state::StateCapture, Cli};
use crate::generator::STF;
use alloy_genesis::Genesis;
use alloy_primitives::Bytes;
use color_eyre::{eyre::ensure, owo_colors::OwoColorize, Result};
use futures::{future::BoxFuture, FutureExt};
use inquire::Select;
use std::{fmt::Display, path::PathBuf};
use tokio::sync::broadcast::Sender;
use tracing::info;

/// The `opt8n` application
#[derive(Debug)]
pub(crate) struct T8n<'a> {
    /// The CLI for `opt8n`
    pub(crate) cli: &'a Cli,
    /// The ctrl+c signal sender.
    pub(crate) interrupt: Sender<()>,
    /// The prestate configurator for `opt8n`
    pub(crate) state_cfg: StateCapture<'a>,
    /// The RLP encoded deposit transactions to send in the test.
    pub(crate) deposits: DepositCapture<'a>,
    /// The RLP encoded transactions to send in the test.
    pub(crate) transactions: Vec<Bytes>,
}

impl<'a> T8n<'a> {
    /// Create a new [T8n] with the given [Cli] and interrupt channel.
    pub(crate) fn new(cli: &'a Cli, interrupt: Sender<()>) -> Self {
        Self {
            cli,
            interrupt: interrupt.clone(),
            state_cfg: StateCapture::new(&cli),
            deposits: DepositCapture::new(&cli, interrupt),
            transactions: Default::default(),
        }
    }

    /// Run the `opt8n` application
    pub(crate) fn run(&mut self) -> BoxFuture<'_, Result<()>> {
        async move {
            let options = [
                MainMenuOption::SetupEnvironment,
                MainMenuOption::CaptureDeposits,
                MainMenuOption::CaptureTransactions,
                MainMenuOption::GenerateFixture,
                MainMenuOption::Exit,
            ];

            let choice = Select::new("What would you like to do?", options.to_vec()).prompt()?;
            match choice {
                MainMenuOption::SetupEnvironment => {
                    // Capture prestate and test block environment.
                    let l1_info_encoded = self.state_cfg.capture_state().await?;
                    self.deposits.transactions.push(l1_info_encoded);

                    // Return to the main menu.
                    self.run().await?;
                }
                MainMenuOption::CaptureDeposits => {
                    // Capture deposit transactions sent to the OptimismPortal on L1.
                    self.deposits.capture_deposits().await?;

                    // Return to the main menu.
                    self.run().await?;
                }
                MainMenuOption::CaptureTransactions => {
                    // Capture the transactions sent to the L2 node.
                    let transactions = crate::proxy::capture_transactions(
                        3000,
                        self.cli.l2_port,
                        self.interrupt.subscribe(),
                    )
                    .await?;

                    // Update the test vector transactions.
                    self.transactions = transactions;

                    // Return to the main menu.
                    self.run().await?;
                }
                MainMenuOption::GenerateFixture => {
                    let genesis: Genesis =
                        serde_json::from_slice(&std::fs::read(&self.cli.l2_genesis)?)?;

                    let mut stf = STF::new(genesis, self.state_cfg.allocs.clone())?;

                    // Sanity check that the pre-state root is correct before proceeding.
                    ensure!(
                        stf.state_root()? == self.state_cfg.pre_header.state_root,
                        "Pre-state root mismatch; Fatal error."
                    );

                    // Chain together all transactions for the test block.
                    let transactions = self
                        .deposits
                        .transactions
                        .iter()
                        .chain(&self.transactions)
                        .cloned()
                        .collect::<Vec<_>>();

                    // Execute the test block to create the fixture.
                    let fixture = stf.execute(self.state_cfg.header.clone(), transactions)?;

                    // Write the fixture to disk.
                    let out_path = self
                        .cli
                        .output
                        .clone()
                        .unwrap_or_else(|| PathBuf::from("fixture.json"));
                    std::fs::write(&out_path, serde_json::to_string_pretty(&fixture)?)?;

                    info!(target: "opt8n", "Execution fixture written to disk @ {}.", out_path.display().green());
                }
                MainMenuOption::Exit => { /* Fall to exit */ }
            }

            Ok(())
        }
        .boxed()
    }
}

/// The main menu options for `opt8n`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MainMenuOption {
    SetupEnvironment,
    CaptureDeposits,
    CaptureTransactions,
    GenerateFixture,
    Exit,
}

impl Display for MainMenuOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetupEnvironment => write!(
                f,
                "Setup environment (capture prestate and test block environment)"
            ),
            Self::CaptureDeposits => write!(f, "Set deposit transactions"),
            Self::CaptureTransactions => write!(f, "Set user-space transactions"),
            Self::GenerateFixture => write!(f, "Generate execution fixture"),
            Self::Exit => write!(f, "Exit"),
        }
    }
}
