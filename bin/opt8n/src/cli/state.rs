//! State configuration tool for `opt8n`.

use super::Cli;
use alloy_consensus::Header;
use alloy_genesis::GenesisAccount;
use alloy_primitives::{Address, Bytes, B256, U64};
use alloy_provider::{
    network::{primitives::BlockTransactions, Ethereum},
    Provider, ReqwestProvider, RootProvider,
};
use color_eyre::eyre::{bail, eyre, Result};
use hashbrown::HashMap;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tracing::info;

/// The state capture tool for `opt8n`.
#[derive(Debug)]
pub struct StateCapture<'a> {
    /// The CLI configuration for `opt8n`
    pub(crate) cli: &'a Cli,
    /// The header of the prestate block.
    pub(crate) pre_header: Header,
    /// The header of the test block.
    pub(crate) header: Header,
    /// Prestate allocs
    pub(crate) allocs: DumpBlockResponse,
}

impl<'a> StateCapture<'a> {
    /// Create a new prestate configurator.
    pub(crate) fn new(cli: &'a Cli) -> Self {
        Self {
            cli,
            pre_header: Header::default(),
            header: Header::default(),
            allocs: Default::default(),
        }
    }

    /// Starts a proxy that captures `eth_sendRawTransaction` requests sent to the node.
    ///
    /// ## Returns
    /// - `Ok(L1BlockInfoTx)` - Successfully captured the prestate / test block environment.
    /// - `Err(_)` - Error capturing prestate / test block environment.
    pub(crate) async fn capture_state(&mut self) -> Result<Bytes> {
        // Set up the providers.
        let l2_rpc_url = format!("http://localhost:{}", self.cli.l2_port);
        let l2_provider: RootProvider<_, Ethereum> =
            ReqwestProvider::new_http(Url::parse(&l2_rpc_url)?);

        // Fetch the latest block number from the L2 chain.
        let latest_block_number = l2_provider.get_block_number().await?;
        let parent_block_number = latest_block_number - 1;

        // Fetch the world state at the parent block.
        let world_state = l2_provider
            .raw_request::<[U64; 1], DumpBlockResponse>(
                "debug_dumpBlock".into(),
                [U64::from(parent_block_number)],
            )
            .await?;
        self.allocs = world_state;

        // Correct storage slots. Geth's `debug_dumpBlock` does not return correct values 😞
        for (address, GenesisAccountExt { account, .. }) in self.allocs.accounts.iter_mut() {
            if let Some(storage) = &mut account.storage {
                for (slot, value) in storage.iter_mut() {
                    *value = l2_provider
                        .get_storage_at(*address, (*slot).into())
                        .block_id(parent_block_number.into())
                        .await?
                        .into();
                }
            }
        }

        // Fetch the latest and parent block.
        let latest_block = l2_provider
            .get_block_by_number(latest_block_number.into(), false)
            .await?
            .ok_or(eyre!("Latest block not found."))?;
        let parent_block = l2_provider
            .get_block_by_number(parent_block_number.into(), false)
            .await?
            .ok_or(eyre!("Parent block not found."))?;

        self.header = latest_block.header.try_into()?;
        self.pre_header = parent_block.header.try_into()?;

        let BlockTransactions::Hashes(transactions) = latest_block.transactions else {
            bail!("Could not fetch L1 info transaction.")
        };

        let l1_info_tx = transactions
            .get(0)
            .ok_or(eyre!("L1 info transaction not present"))?;
        let raw_tx = l2_provider
            .raw_request::<[B256; 1], Bytes>("debug_getRawTransaction".into(), [*l1_info_tx])
            .await?;

        info!(target: "state-capture", "Captured prestate / test block environment successfully.");

        Ok(raw_tx)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct DumpBlockResponse {
    /// The state root
    pub(crate) root: B256,
    /// The account allocs
    pub(crate) accounts: HashMap<Address, GenesisAccountExt>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct GenesisAccountExt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) root: Option<B256>,
    #[serde(flatten)]
    pub(crate) account: GenesisAccount,
}
