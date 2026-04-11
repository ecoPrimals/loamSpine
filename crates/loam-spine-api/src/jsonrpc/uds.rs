// SPDX-License-Identifier: AGPL-3.0-or-later

//! Unix Domain Socket JSON-RPC server transport.
//!
//! Binds a UDS listener and serves newline-delimited JSON-RPC requests.
//! Supports optional BTSP handshake gating: when `BtspHandshakeConfig` is
//! provided, every connection must complete the 4-step handshake before
//! JSON-RPC methods are exposed.

use super::LoamSpineJsonRpc;
use crate::error::ServerError;
use crate::service::LoamSpineRpcService;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

/// Maximum concurrent UDS connections.
///
/// Provides backpressure under composition load (trio IPC, biomeOS graphs).
/// Connections beyond this limit wait until a slot opens rather than being
/// rejected, which prevents transient overload from breaking long-lived
/// trio partner connections.
const UDS_MAX_CONCURRENT_CONNECTIONS: usize = 256;

/// Server handle for a UDS JSON-RPC listener.
pub struct UdsServerHandle {
    shutdown: tokio::sync::watch::Sender<bool>,
    done: tokio::sync::watch::Receiver<bool>,
    path: std::path::PathBuf,
}

impl UdsServerHandle {
    /// Stop the UDS server.
    pub fn stop(&self) {
        let _ = self.shutdown.send(true);
    }

    /// Wait until the server has stopped.
    pub async fn stopped(&mut self) {
        let _ = self.done.changed().await;
    }

    /// Get the socket path.
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for UdsServerHandle {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Run a JSON-RPC server on a Unix domain socket.
///
/// Binds at the given path, accepting newline-delimited JSON-RPC requests.
/// Creates the parent directory if it does not exist and removes any stale
/// socket file from a previous run.
///
/// When `btsp_config` is `Some`, every incoming connection must complete
/// the BTSP handshake (delegated to `BearDog`) before JSON-RPC is exposed.
/// When `None`, raw newline-delimited JSON-RPC is accepted (development mode).
///
/// # Errors
///
/// Returns error if the socket cannot be bound.
pub async fn run_jsonrpc_uds_server(
    path: impl Into<std::path::PathBuf>,
    service: LoamSpineRpcService,
    btsp_config: Option<loam_spine_core::btsp::BtspHandshakeConfig>,
) -> Result<UdsServerHandle, ServerError> {
    let path = path.into();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ServerError::Bind(format!("cannot create socket directory: {e}")))?;
    }

    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| ServerError::Bind(format!("cannot remove stale socket: {e}")))?;
    }

    let listener = tokio::net::UnixListener::bind(&path)
        .map_err(|e| ServerError::Bind(format!("UDS bind at {}: {e}", path.display())))?;

    let handler = Arc::new(LoamSpineJsonRpc::new(service));
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let (done_tx, done_rx) = tokio::sync::watch::channel(false);
    let btsp_config = btsp_config.map(Arc::new);
    let semaphore = Arc::new(Semaphore::new(UDS_MAX_CONCURRENT_CONNECTIONS));

    info!(
        "LoamSpine JSON-RPC UDS server listening on {} (BTSP {}, max_conns={})",
        path.display(),
        if btsp_config.is_some() {
            "required"
        } else {
            "off"
        },
        UDS_MAX_CONCURRENT_CONNECTIONS,
    );

    tokio::spawn(async move {
        let mut rx = shutdown_rx;
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _peer)) => {
                            debug!("JSON-RPC UDS connection accepted");
                            let h = Arc::clone(&handler);
                            let btsp = btsp_config.clone();
                            let permit = Arc::clone(&semaphore);
                            tokio::spawn(async move {
                                let Ok(_permit) = permit.acquire().await else {
                                    return;
                                };
                                if let Err(e) = handle_uds_connection(h, stream, btsp).await {
                                    warn!("UDS connection error: {e}");
                                }
                            });
                        }
                        Err(e) => {
                            warn!("UDS accept error: {e}");
                        }
                    }
                }
                _ = rx.changed() => {
                    info!("JSON-RPC UDS server shutting down");
                    break;
                }
            }
        }
        let _ = done_tx.send(true);
    });

    Ok(UdsServerHandle {
        shutdown: shutdown_tx,
        done: done_rx,
        path,
    })
}

/// Handle a single UDS connection: BTSP handshake (if configured), then JSON-RPC.
async fn handle_uds_connection(
    handler: Arc<LoamSpineJsonRpc>,
    mut stream: tokio::net::UnixStream,
    btsp_config: Option<Arc<loam_spine_core::btsp::BtspHandshakeConfig>>,
) -> Result<(), std::io::Error> {
    if let Some(ref btsp) = btsp_config {
        match loam_spine_core::btsp::perform_server_handshake(&mut stream, &btsp.beardog_socket)
            .await
        {
            Ok(session) => {
                debug!(
                    "BTSP authenticated: session={}, cipher={}",
                    session.session_id, session.cipher
                );
            }
            Err(e) => {
                warn!("BTSP handshake failed, refusing connection: {e}");
                return Ok(());
            }
        }
    }

    let (reader, writer) = stream.into_split();
    super::server::handle_stream(handler, reader, writer).await
}
