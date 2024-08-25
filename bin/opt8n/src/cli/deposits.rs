//! Deposit capture tool for `opt8n`.

use super::Cli;
use alloy_consensus::{Eip658Value, Receipt};
use alloy_primitives::{b256, keccak256, Address, Bytes, Log, B256};
use alloy_provider::{network::Ethereum, Provider, ReqwestProvider, RootProvider};
use color_eyre::eyre::{eyre, Result};
use kona_primitives::{decode_deposit, DEPOSIT_EVENT_ABI_HASH};
use reqwest::Url;
use tokio::sync::broadcast::Sender;
use tracing::info;

/// keccak256("t8n")
const T8N_L1_SOURCE_HASH: B256 =
    b256!("60b82a553424b66356596123f87f40cc9a453c02816a147e9418b49a373fa41a");

/// The deposit capture proxy for `opt8n`.
#[derive(Debug)]
pub struct DepositCapture<'a> {
    /// The CLI configuration for `opt8n`
    pub(crate) cli: &'a Cli,
    /// The interrupt receiver for the deposit configurator.
    pub(crate) interrupt: Sender<()>,
    /// The RLP-encoded transactions to apply to the genesis state to create the deposit.
    pub(crate) transactions: Vec<Bytes>,
}

impl<'a> DepositCapture<'a> {
    /// Create a new deposit configurator.
    pub(crate) fn new(cli: &'a Cli, interrupt: Sender<()>) -> Self {
        Self {
            cli,
            interrupt,
            transactions: Vec::new(),
        }
    }

    /// Opens a proxy to the L1 chain and captures the deposit transactions.
    pub(crate) async fn capture_deposits(&mut self) -> Result<()> {
        info!(
            target: "opt8n", "Capturing deposit transactions sent to the L1 chain's `OptimismPortal`..."
        );

        // Capture the deposit transactions sent to the L1 node.
        let transactions =
            crate::proxy::capture_transactions(3000, self.cli.l1_port, self.interrupt.subscribe())
                .await?;

        // Fetch all receipts for the deposit transactions from the L1.
        let l1_rpc_url = format!("http://localhost:{}", self.cli.l1_port);
        let l1_provider: RootProvider<_, Ethereum> =
            ReqwestProvider::new_http(Url::parse(&l1_rpc_url)?);
        let mut receipts = Vec::new();
        for tx in transactions {
            let tx_hash = keccak256(tx.as_ref());

            // Fetch RPC receipt.
            let rpc_receipt = l1_provider
                .get_transaction_receipt(tx_hash)
                .await?
                .ok_or(eyre!(
                    "Transaction receipt not found for transaction hash: {:?}",
                    tx_hash
                ))?;
            let receipt_rpc_logs = rpc_receipt.inner.as_receipt().ok_or(eyre!(
                "Receipt not found for transaction hash: {:?}",
                tx_hash
            ))?;

            // Convert to consensus receipt.
            let receipt: Receipt<Log> = Receipt {
                status: receipt_rpc_logs.status,
                cumulative_gas_used: receipt_rpc_logs.cumulative_gas_used,
                logs: receipt_rpc_logs
                    .logs
                    .iter()
                    .map(|l| Log {
                        address: l.address(),
                        data: l.data().clone(),
                    })
                    .collect(),
            };

            receipts.push(receipt);
        }

        // Derive the deposit transactions from the OptimismPortal `depositTransaction` receipts.
        let deposits = Self::derive_deposits(
            T8N_L1_SOURCE_HASH,
            receipts,
            self.cli.optimism_portal_address,
        )?;

        self.transactions = deposits;
        Ok(())
    }

    /// Derive the deposit transactions from the OptimismPortal `depositTransaction` receipts.
    fn derive_deposits(
        block_hash: B256,
        receipts: Vec<Receipt>,
        deposit_contract: Address,
    ) -> Result<Vec<Bytes>> {
        let mut global_index = 0;
        let mut res = Vec::new();
        for r in receipts.iter() {
            if Eip658Value::Eip658(false) == r.status {
                continue;
            }
            for l in r.logs.iter() {
                let curr_index = global_index;
                global_index += 1;
                if !l
                    .data
                    .topics()
                    .first()
                    .map_or(false, |i| *i == DEPOSIT_EVENT_ABI_HASH)
                {
                    continue;
                }
                if l.address != deposit_contract {
                    continue;
                }
                let decoded = decode_deposit(block_hash, curr_index, l).map_err(|e| eyre!(e))?;
                res.push(decoded.0);
            }
        }
        Ok(res)
    }
}
