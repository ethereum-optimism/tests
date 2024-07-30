//! Module to configure the verbosity level for logging and telemetry.

use tracing::Level;
use color_eyre::eyre::{eyre, Result};

/// Initializes tracing verbosity.
pub fn init_tracing(verbosity_level: u8) -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(match verbosity_level {
            0 => Level::ERROR,
            1 => Level::WARN,
            2 => Level::INFO,
            3 => Level::DEBUG,
            _ => Level::TRACE,
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber).map_err(|e| eyre!(e))
}
