use std::str::FromStr;

use serde::{Deserialize, Serialize};

// #[derive(Parser, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
// #[clap(rename_all = "kebab_case", infer_subcommands = true, multicall = true)]
// pub enum Opt8nCommand {
//     #[command(visible_alias = "a")]
//     Anvil {
//         #[arg(index = 1, allow_hyphen_values = true)]
//         args: Vec<String>,
//     },
//     #[command(visible_alias = "c")]
//     Cast {
//         #[arg(index = 1, allow_hyphen_values = true)]
//         args: Vec<String>,
//     },
//     #[command(visible_alias = "e")]
//     Exit,
// }

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Opt8nCommand {
    Anvil(String),
    Cast(String),
    Exit,
    // TODO: rename
    Dump,
}

impl FromStr for Opt8nCommand {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //TODO: check if command starts with anvil or cast etc

        match s.to_lowercase().trim().as_ref() {
            "dump" => Ok(Self::Dump),
            //TODO: match anvil or just parse or something

            // TODO: same for cast
            "exit" => Ok(Self::Exit),
            _ => Err(color_eyre::eyre::eyre!("Unrecognized command")),
        }
    }
}
