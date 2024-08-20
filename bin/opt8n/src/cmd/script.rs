use anvil::{cmd::NodeArgs, Hardfork};
use clap::{Parser, ValueHint};
use color_eyre::eyre::eyre;
use futures::StreamExt;

use crate::opt8n::{Opt8n, Opt8nArgs};

#[derive(Parser, Clone, Debug)]
pub struct ScriptArgs {
    #[command(flatten)]
    opt8n_args: Opt8nArgs,
    // #[command(flatten)]
    // inner: forge_script::ScriptArgs,
    #[arg(value_hint = ValueHint::FilePath)]
    pub path: String,
    #[command(flatten)]
    pub node_args: NodeArgs,
}

impl ScriptArgs {
    pub async fn run(mut self) -> color_eyre::Result<()> {
        let opt8n = Opt8n::new(
            Some(self.node_args.clone()),
            self.opt8n_args.output.clone(),
            self.opt8n_args.genesis.clone(),
        )
        .await?;

        let mut script_args = forge_script::ScriptArgs::default();
        script_args.path = self.path.clone();

        foundry_common::shell::set_shell(foundry_common::shell::Shell::from_args(
            script_args.opts.silent,
            script_args.json,
        ))?;

        script_args.broadcast = true;
        script_args.evm_opts.sender = Some(
            opt8n
                .node_handle
                .genesis_accounts()
                .last()
                .expect("Could not get genesis account"),
        );
        script_args.unlocked = true;
        script_args.evm_opts.fork_url = Some(opt8n.node_handle.http_endpoint());

        run_script(opt8n, Box::new(script_args)).await?;

        Ok(())
    }
}

/// Run a Forge script with the given arguments, and generate an execution fixture
/// from the broadcasted transactions.
pub async fn run_script(
    opt8n: Opt8n,
    script_args: Box<forge_script::ScriptArgs>,
) -> color_eyre::Result<()> {
    let mut new_blocks = opt8n.eth_api.backend.new_block_notifications();

    // Run the forge script and broadcast the transactions to the anvil node
    let mut opt8n = broadcast_transactions(opt8n, script_args).await?;

    // Mine the block and generate the execution fixture
    opt8n.mine_block().await;

    let block = new_blocks.next().await.ok_or(eyre!("No new block"))?;
    if let Some(block) = opt8n.eth_api.backend.get_block_by_hash(block.hash) {
        opt8n.generate_execution_fixture(block).await?;
    }

    Ok(())
}

async fn broadcast_transactions(
    opt8n: Opt8n,
    script_args: Box<forge_script::ScriptArgs>,
) -> color_eyre::Result<Opt8n> {
    // Run the script, compile the transactions and broadcast to the anvil instance
    let compiled = script_args.preprocess().await?.compile()?;

    let pre_simulation = compiled
        .link()
        .await?
        .prepare_execution()
        .await?
        .execute()
        .await?
        .prepare_simulation()
        .await?;

    let bundled = pre_simulation.fill_metadata().await?.bundle().await?;

    let tx_count = bundled
        .sequence
        .sequences()
        .iter()
        .fold(0, |sum, sequence| sum + sequence.transactions.len());

    // TODO: break into function
    let broadcast = bundled.broadcast();

    let pending_transactions = tokio::task::spawn(async move {
        loop {
            let pending_tx_count = opt8n
                .eth_api
                .txpool_content()
                .await
                .expect("Failed to get txpool content")
                .pending
                .len();

            if pending_tx_count == tx_count {
                return opt8n;
            }
        }
    });

    let opt8n = tokio::select! {
        _ = broadcast => {
            // TODO: Gracefully handle this error
            return Err(eyre!("Script failed early"));
        },
        opt8n = pending_transactions => {
            opt8n?
        }
    };

    Ok(opt8n)
}
