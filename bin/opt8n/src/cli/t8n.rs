//! The `opt8n` application.

use super::{deposits::DepositCapture, l1_info::L1InfoConfigurator, state::StateCapture, Cli};
use alloy_primitives::Bytes;
use futures::{future::BoxFuture, FutureExt};
use inquire::Select;
use std::fmt::Display;
use tokio::sync::broadcast::Sender;
use color_eyre::Result;

/// The `opt8n` application
#[derive(Debug)]
pub(crate) struct T8n<'a> {
    /// The CLI for `opt8n`
    pub(crate) cli: &'a Cli,
    /// The ctrl+c signal sender.
    pub(crate) interrupt: Sender<()>,
    /// The prestate configurator for `opt8n`
    pub(crate) state_cfg: StateCapture<'a>,
    /// The L1 info system transaction.
    pub(crate) l1_info: L1InfoConfigurator,
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
            l1_info: Default::default(),
            deposits: DepositCapture::new(&cli, interrupt),
            transactions: Default::default(),
        }
    }

    /// Run the `opt8n` application
    pub(crate) fn run(&mut self) -> BoxFuture<'_, Result<()>> {
        async move {
            let options = [
                MainMenuOption::SetupEnvironment,
                MainMenuOption::ModifyL1Info,
                MainMenuOption::CaptureDeposits,
                MainMenuOption::ModifyBlock,
                MainMenuOption::GenerateFixture,
                MainMenuOption::Exit,
            ];

            let choice = Select::new("What would you like to do?", options.to_vec()).prompt()?;
            match choice {
                MainMenuOption::SetupEnvironment => {
                    // Capture prestate and test block environment.
                    let l1_info = self.state_cfg.capture_state().await?;
                    self.l1_info.tx = l1_info;

                    // Return to the main menu.
                    self.run().await?;
                }
                MainMenuOption::ModifyL1Info => {
                    // Show the L1 info configuration menu.
                    self.l1_info.show_configuration_menu()?;

                    // Return to the main menu.
                    self.run().await?;
                }
                MainMenuOption::CaptureDeposits => {
                    // Capture deposit transactions sent to the OptimismPortal on L1.
                    self.deposits.capture_deposits().await?;

                    // Return to the main menu.
                    self.run().await?;
                }
                MainMenuOption::ModifyBlock => {
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
                    dbg!(&self.state_cfg);
                    dbg!(&self.l1_info);
                    dbg!(&self.deposits);
                    dbg!(&self.transactions);

                    // let fixture = ExecutionFixture {
                    //     env: ExecutionEnvironment {
                    //         current_coinbase: self.env.coinbase,
                    //         current_difficulty: self.env.difficulty,
                    //         current_gas_limit: self.env.gas_limit,
                    //         previous_hash: todo!(),
                    //         current_number: self.env.number,
                    //         current_timestamp: self.env.timestamp,
                    //         block_hashes: Default::default(), // TODO
                    //     },
                    //     alloc: todo!(),
                    //     out_alloc: todo!(),
                    //     transactions: todo!(),
                    //     result: todo!(),
                    // };
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
    ModifyL1Info,
    CaptureDeposits,
    ModifyBlock,
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
            Self::ModifyL1Info => write!(f, "Modify L1 information system transaction"),
            Self::CaptureDeposits => write!(f, "Set deposit transactions"),
            Self::ModifyBlock => write!(f, "Set user-space transactions"),
            Self::GenerateFixture => write!(f, "Generate execution fixture"),
            Self::Exit => write!(f, "Exit"),
        }
    }
}
