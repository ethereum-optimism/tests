//! State configuration tool for `opt8n`.

use super::Cli;
use alloy_consensus::Header;
use alloy_primitives::{Address, Bytes, B256, U256, U64};
use alloy_provider::{
    network::{primitives::BlockTransactions, Ethereum},
    Provider, ReqwestProvider, RootProvider,
};
use color_eyre::eyre::{bail, eyre, Result};
use hashbrown::HashMap;
use kona_primitives::L1BlockInfoTx;
use reqwest::Url;
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Serialize,
};
use std::fmt;

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
    pub(crate) async fn capture_state(&mut self) -> Result<L1BlockInfoTx> {
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

        // Fetch the latest and parent block.
        let latest_block = l2_provider
            .get_block_by_number(latest_block_number.into(), true)
            .await?
            .ok_or(eyre!("Latest block not found."))?;
        let parent_block = l2_provider
            .get_block_by_number(parent_block_number.into(), false)
            .await?
            .ok_or(eyre!("Parent block not found."))?;

        self.header = latest_block.header.try_into()?;
        self.pre_header = parent_block.header.try_into()?;

        let BlockTransactions::Full(transactions) = latest_block.transactions else {
            bail!("Could not fetch L1 info transaction.")
        };
        let l1_info_data = transactions
            .get(0)
            .ok_or(eyre!("L1 info transaction not present"))?
            .input
            .clone();
        let l1_info_tx = L1BlockInfoTx::decode_calldata(l1_info_data.as_ref())
            .map_err(|e| eyre!("Error decoding L1 info tx: {e}"))?;

        Ok(l1_info_tx)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct DumpBlockResponse {
    /// The state root
    pub(crate) root: B256,
    /// The account allocs
    pub(crate) accounts: HashMap<Address, AccountAlloc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AccountAlloc {
    /// The storage root of the account.
    pub(crate) root: B256,
    /// Account balance.
    pub(crate) balance: U256,
    /// Account nonce.
    pub(crate) nonce: u64,
    /// code hash.
    pub(crate) code_hash: B256,
    /// bytecode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) code: Option<Bytes>,
    /// The complete storage of the account.
    #[serde(default = "HashMap::default", deserialize_with = "storage_deserialize")]
    pub(crate) storage: HashMap<U256, U256>,
}

/// Custom deserialization function for the storage hashmap. Accounts for trimmed [U256] hex strings.
fn storage_deserialize<'de, D>(deserializer: D) -> Result<HashMap<U256, U256>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct MapVisitor;

    impl<'de> Visitor<'de> for MapVisitor {
        type Value = HashMap<U256, U256>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));

            while let Some((key, value)) = access.next_entry::<U256, String>()? {
                // Pad the hex string to 32 bytes (64 characters)
                let padded_hex = format!("{:0>64}", value);
                let u256_value =
                    U256::from_str_radix(&padded_hex, 16).map_err(serde::de::Error::custom)?;

                map.insert(key, u256_value);
            }

            Ok(map)
        }
    }

    deserializer.deserialize_map(MapVisitor)
}
