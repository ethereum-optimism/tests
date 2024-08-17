use anvil::cmd::NodeArgs;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use clap::Parser;
use color_eyre::eyre::eyre;
use futures::StreamExt;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::opt8n::{Opt8n, Opt8nArgs};

#[derive(Parser, Clone, Debug)]
pub struct ServerArgs {
    #[command(flatten)]
    pub opt8n_args: Opt8nArgs,
    #[command(flatten)]
    pub node_args: NodeArgs,
}

impl ServerArgs {
    pub async fn run(&self) -> color_eyre::Result<()> {
        let mut opt8n = Opt8n::new(
            Some(self.node_args.clone()),
            self.opt8n_args.output.clone(),
            self.opt8n_args.genesis.clone(),
        )
        .await?;

        let opt8n = Arc::new(Mutex::new(opt8n));

        let router = axum::Router::new()
            .route("/dump_fixture", axum::routing::post(dump_execution_fixture))
            .fallback(fallback_handler)
            .with_state(opt8n);

        let addr: SocketAddr = ([127, 0, 0, 1], 0).into();
        let listener = TcpListener::bind(addr).await?;

        axum::serve(listener, router.into_make_service()).await?;

        Ok(())
    }
}

async fn dump_execution_fixture(State(opt8n): State<Arc<Mutex<Opt8n>>>) -> Result<(), ServerError> {
    let mut opt8n = opt8n.lock().await;

    let mut new_blocks = opt8n.eth_api.backend.new_block_notifications();

    opt8n.mine_block().await;

    let block = new_blocks
        .next()
        .await
        .ok_or(eyre!("No new block"))
        .map_err(ServerError::Opt8nError)?;
    if let Some(block) = opt8n.eth_api.backend.get_block_by_hash(block.hash) {
        opt8n
            .generate_execution_fixture(block)
            .await
            .map_err(ServerError::Opt8nError)?;
    }

    Ok(())
}

async fn fallback_handler(
    State(opt8n): State<Arc<Mutex<Opt8n>>>,
    req: Request<Body>,
) -> Result<(), ServerError> {
    let opt8n = opt8n.lock().await;

    // TODO: Forward request to the ETH api

    Ok(())
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Opt8n error: {0}")]
    Opt8nError(color_eyre::Report),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let message = match self {
            ServerError::Opt8nError(err) => err.to_string(),
        };

        let body = Body::from(message);

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
