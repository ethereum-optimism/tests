use anvil::cmd::NodeArgs;
use axum::extract::State;
use axum::Router;
use clap::Parser;
use futures::future::Ready;
use hyper::service::{make_servce, service_fn, Service};
use hyper::{body::Body, client::conn::http1, Method, Request, Response};
use std::sync::Arc;
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::TcpListener; // <-- Ensure this line is present

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
        let mut opt8n = Arc::new(
            Opt8n::new(
                Some(self.node_args.clone()),
                self.opt8n_args.output.clone(),
                self.opt8n_args.genesis.clone(),
            )
            .await?,
        );

        let router = axum::Router::new()
            .route("/dump_fixture", axum::routing::post(dump_fixture_handler))
            .with_state(opt8n);
        //TODO: add fallback route

        let addr: SocketAddr = ([127, 0, 0, 1], 0).into();
        let listener = TcpListener::bind(addr).await?;

        let bound_server = axum::Server::from_tcp(listener)?;
        bound_server.serve(router.into_make_service()).await?;

        Ok(())
    }
}

async fn dump_fixture_handler() -> &'static str {
    "Fixture dumped!"
}

async fn fallback_handler(State(opt8n): State<Arc<Opt8n>>, req: Request<Body>) -> Response<Body> {
    // TODO: handle the request
}
