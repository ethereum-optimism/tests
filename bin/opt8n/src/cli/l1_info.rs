//! L1 info transaction configuration tool for `opt8n`.

use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use inquire::{Editor, Select};
use kona_primitives::{L1BlockInfoBedrock, L1BlockInfoEcotone, L1BlockInfoTx};
use std::{ffi::OsStr, fmt::Display};

/// The L1 info transaction configurator for `opt8n`.
#[derive(Debug)]
pub struct L1InfoConfigurator {
    /// The L1 info transaction data.
    pub(crate) tx: L1BlockInfoTx,
}

impl Default for L1InfoConfigurator {
    fn default() -> Self {
        Self {
            tx: L1BlockInfoTx::Ecotone(L1BlockInfoEcotone::default()),
        }
    }
}

impl L1InfoConfigurator {
    /// Show the configuration menu for the prestate.
    pub(crate) fn show_configuration_menu(&mut self) -> Result<()> {
        let select = Select::new(
            "What would you like to configure?",
            [L1InfoConfigOption::TxKind, L1InfoConfigOption::Tx].to_vec(),
        )
        .prompt()?;

        match select {
            L1InfoConfigOption::TxKind => self.configure_tx_kind(),
            L1InfoConfigOption::Tx => self.configure_tx(),
        }
    }

    /// Opens a prompt to configure the chain state.
    pub(crate) fn configure_tx_kind(&mut self) -> Result<()> {
        let select = Select::new(
            "Which version of the L1 info transaction are you sending?",
            [L1InfoTxKind::Bedrock, L1InfoTxKind::Ecotone].to_vec(),
        )
        .prompt()?;

        // Update the tx kind.
        match select {
            L1InfoTxKind::Bedrock => {
                self.tx = L1BlockInfoTx::Bedrock(L1BlockInfoBedrock::default())
            }
            L1InfoTxKind::Ecotone => {
                self.tx = L1BlockInfoTx::Ecotone(L1BlockInfoEcotone::default())
            }
        }

        // Return to the L1 info tx configuration menu.
        self.show_configuration_menu()
    }

    /// Opens a prompt to configure the L1 info transaction data.
    pub(crate) fn configure_tx(&mut self) -> Result<()> {
        // Serialize the tx data to a JSON string.
        let serialized_tx_data = serde_json::to_string_pretty(&self.tx)?;

        // Prompt the user to edit the block environment.
        let editor_command = option_env!("EDITOR").unwrap_or("vim");
        let editor = Editor::new("Edit the L1 info transaction data")
            .with_file_extension(".json")
            .with_editor_command(OsStr::new(editor_command))
            .with_predefined_text(serialized_tx_data.as_str())
            .prompt()?;

        // Deserialize the block environment from the editor.
        let deserialized_tx_data = serde_json::from_str(&editor);
        if let Ok(tx_data) = deserialized_tx_data {
            self.tx = tx_data;
        } else {
            eprintln!("Failed to deserialize L1 info transaction data. Using previous value.");
        }

        Ok(())
    }
}

/// The L1 info transaction configuration options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum L1InfoConfigOption {
    TxKind,
    Tx,
}

impl Display for L1InfoConfigOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            L1InfoConfigOption::TxKind => write!(
                f,
                "L1 info transaction kind ({} | {})",
                "BEDROCK".green(),
                "ECOTONE".green()
            ),
            L1InfoConfigOption::Tx => write!(f, "Modify L1 info transaction"),
        }
    }
}

/// The L1 info transaction kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum L1InfoTxKind {
    Bedrock,
    Ecotone,
}

impl Display for L1InfoTxKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            L1InfoTxKind::Bedrock => write!(f, "Bedrock"),
            L1InfoTxKind::Ecotone => write!(f, "Ecotone"),
        }
    }
}
