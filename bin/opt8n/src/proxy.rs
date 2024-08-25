//! Contains the logic for capturing transactions sent to a node.

use alloy_primitives::{keccak256, Bytes};
use axum::{
    body::{self, Body},
    extract::State,
    http::{Request, Response},
    response::IntoResponse,
    routing::any,
    Router,
};
use color_eyre::{owo_colors::OwoColorize, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast::Receiver, Mutex};
use tracing::{error, info};

/// 200MB max return size.
const MAX_BODY_SIZE: usize = 200_000_000;

/// Starts a proxy server that captures transactions sent to the node via `eth_sendRawTransaction`.
pub(crate) async fn capture_transactions(
    port: u16,
    forward_port: u16,
    receiver: Receiver<()>,
) -> Result<Vec<Bytes>> {
    // Initialize the proxy and the exit signal channel.
    let proxy = TransactionCaptureProxy::new(port, forward_port);

    // Spawn the proxy server on a separate thread.
    let proxy_handle = tokio::task::spawn(proxy.capture(receiver));

    // Wait for the user to exit the transaction capture proxy.
    let (transactions,) = tokio::try_join!(proxy_handle)?;
    transactions
}

/// A proxy server that captures transactions sent to the node via `eth_sendRawTransaction`.
#[derive(Debug, Clone)]
pub(crate) struct TransactionCaptureProxy {
    /// The port the proxy server listens on.
    port: u16,
    /// The port the proxy server forwards requests to.
    forward_port: u16,
    /// The RLP encoded transactions captured by the proxy server.
    transactions: Vec<Bytes>,
}

impl TransactionCaptureProxy {
    /// Creates a new `TransactionCaptureProxy` instance.
    pub(crate) fn new(port: u16, forward_port: u16) -> Self {
        Self {
            port,
            forward_port,
            transactions: Vec::new(),
        }
    }

    /// Starts the proxy server and captures transactions sent to the node. When the server exits, the captured
    /// transactions are returned.
    pub(crate) async fn capture(mut self, mut signal: Receiver<()>) -> Result<Vec<Bytes>> {
        // Print startup information.
        info!(target: "capture-proxy", "Starting proxy server @ http://localhost:{}. Forwarding requests to http://localhost:{}", self.port.green(), self.forward_port.green());
        info!(target: "capture-proxy", "Waiting for transactions to be sent to the relay via `eth_sendRawTransaction`...");
        info!(target: "capture-proxy", "To finalize, press {}.", "`Ctrl + C`".cyan());

        // Setup server.
        let state = Arc::new(Mutex::new(self.clone()));
        let app = Router::new()
            .route("/", any(Self::proxy_handler))
            .with_state(state.clone());
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                signal.recv().await.ok();
                info!(target: "capture-proxy", "Shutting down the proxy server.");
            })
            .await?;

        self.transactions = state.lock().await.transactions.clone();

        // Return the captured transactions.
        info!(target: "capture-proxy", "Captured {} transactions successfully.", self.transactions.len());
        Ok(self.transactions)
    }

    /// Proxies the request to the original server listening on the configured port.
    async fn proxy_handler(
        State(state): State<Arc<Mutex<TransactionCaptureProxy>>>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        let mut state = state.lock().await;

        // Forward the request to the original server listening on `state.port`
        let uri_string = format!("http://127.0.0.1:{}{}", state.forward_port, req.uri());

        // Create a reqwest client
        let client = Client::new();

        // Build the new request for the upstream server
        let mut forwarded_req = client
            .request(req.method().clone(), &uri_string)
            .headers(req.headers().clone());

        if let Ok(body_bytes) = body::to_bytes(req.into_body(), MAX_BODY_SIZE).await {
            let req =
                serde_json::from_slice::<JsonRpcRequest<serde_json::Value>>(&body_bytes).unwrap();

            // Capture `eth_sendRawTransaction` requests and persist the RLP-encoded transactions.
            if req.method == "eth_sendRawTransaction" {
                let value: [Bytes; 1] = serde_json::from_value(req.params.unwrap()).unwrap();
                state.transactions.push(value[0].clone());

                info!(target: "capture-proxy", "🕸️ Captured transaction (Hash: {})", keccak256(&value[0]).cyan());
            }

            // Add body if method is POST, PUT, etc.
            forwarded_req = forwarded_req.body(body_bytes);
        }

        // Drop the lock on the state.
        drop(state);

        // Forward the request.
        match forwarded_req.send().await {
            Ok(res) => {
                let status = res.status();
                let body = res.bytes().await.unwrap_or_default();

                Response::builder()
                    .status(status)
                    .body(Body::from(body))
                    .unwrap()
            }
            Err(err) => {
                error!(target: "capture-proxy", "Request forwarding failed: {:?}", err);
                Response::builder()
                    .status(500)
                    .body(Body::from("Internal Server Error"))
                    .unwrap()
            }
        }
    }
}

/// An opaque JSON-RPC request.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct JsonRpcRequest<T> {
    /// The JSON-RPC method.
    method: String,
    /// The JSON-RPC params.
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<T>,
}
