use color_eyre::eyre;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Opt8nCommand {
    Anvil(String),
    Cast(String),
    Exit,
    // Help
}

impl FromStr for Opt8nCommand {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim().as_ref() {
            "exit" => Ok(Self::Exit),
            _ => Err(eyre::eyre!("Unrecognized command")),
        }
    }
}
