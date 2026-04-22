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
/// Provides backpressure under composition load (trio IPC, ecosystem pipeline graphs).
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
/// the BTSP handshake (delegated to the BTSP capability provider) before JSON-RPC is exposed.
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
        std::fs::create_dir_all(parent).map_err(|e| ServerError::Bind {
            context: "cannot create socket directory".into(),
            source: e,
        })?;
    }

    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| ServerError::Bind {
            context: "cannot remove stale socket".into(),
            source: e,
        })?;
    }

    let listener = tokio::net::UnixListener::bind(&path).map_err(|e| ServerError::Bind {
        context: format!("UDS bind at {}", path.display()),
        source: e,
    })?;

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

/// Handle a single UDS connection with wire-level protocol auto-detection.
///
/// Always peeks the first byte to determine wire format, regardless of
/// whether static BTSP is configured:
///
/// 1. **`{` → line-based**: read the full first line. If it contains
///    `"protocol":"btsp"`, route to NDJSON BTSP handshake (primalSpring-
///    compatible). Otherwise, dispatch as JSON-RPC.
/// 2. **Non-`{` + BTSP configured**: length-prefixed BTSP handshake
///    (Phase 2 binary framing), then JSON-RPC.
/// 3. **Non-`{` + no BTSP**: unexpected binary data, close.
async fn handle_uds_connection(
    handler: Arc<LoamSpineJsonRpc>,
    stream: tokio::net::UnixStream,
    btsp_config: Option<Arc<loam_spine_core::btsp::BtspHandshakeConfig>>,
) -> Result<(), std::io::Error> {
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = tokio::io::BufReader::new(reader);

    let first_byte = {
        let buf = tokio::io::AsyncBufReadExt::fill_buf(&mut buf_reader).await?;
        if buf.is_empty() {
            return Ok(());
        }
        buf[0]
    };

    if first_byte == b'{' {
        let mut first_line = String::new();
        tokio::io::AsyncBufReadExt::read_line(&mut buf_reader, &mut first_line).await?;

        if first_line.trim().is_empty() {
            return Ok(());
        }

        if is_btsp_ndjson(&first_line) {
            let provider_path = btsp_config
                .as_ref()
                .map(|c| c.provider_socket.clone())
                .or_else(resolve_btsp_provider);

            if let Some(provider_path) = provider_path {
                match loam_spine_core::btsp::perform_ndjson_server_handshake(
                    &mut buf_reader,
                    &mut writer,
                    &provider_path,
                    &first_line,
                )
                .await
                {
                    Ok(session) => {
                        debug!(
                            "BTSP NDJSON authenticated: session={}, cipher={}",
                            session.session_id, session.cipher
                        );
                    }
                    Err(e) => {
                        warn!("BTSP NDJSON handshake failed: {e}");
                        return Ok(());
                    }
                }
                super::server::handle_stream(handler, buf_reader, writer).await
            } else {
                warn!(
                    "BTSP NDJSON handshake requested but no provider available; \
                     set BTSP_PROVIDER_SOCKET or BIOMEOS_FAMILY_ID"
                );
                Ok(())
            }
        } else {
            super::server::handle_stream_with_first_line(handler, buf_reader, writer, &first_line)
                .await
        }
    } else if let Some(ref btsp) = btsp_config {
        match loam_spine_core::btsp::perform_server_handshake(
            &mut buf_reader,
            &mut writer,
            &btsp.provider_socket,
        )
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
        super::server::handle_stream(handler, buf_reader, writer).await
    } else {
        debug!("UDS connection starts with non-JSON byte and no BTSP config; closing");
        Ok(())
    }
}

/// Check whether a first line looks like a BTSP NDJSON `ClientHello`.
fn is_btsp_ndjson(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('{') && trimmed.contains("\"protocol\"") && trimmed.contains("\"btsp\"")
}

/// Resolve the BTSP provider socket from environment for NDJSON auto-detect.
fn resolve_btsp_provider() -> Option<std::path::PathBuf> {
    if let Ok(path) = std::env::var("BTSP_PROVIDER_SOCKET") {
        return Some(std::path::PathBuf::from(path));
    }
    let config = loam_spine_core::btsp::BtspHandshakeConfig::from_env()?;
    Some(config.provider_socket)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn btsp_ndjson_detection_positive() {
        assert!(is_btsp_ndjson(
            r#"{"protocol":"btsp","version":1,"client_ephemeral_pub":"key"}"#
        ));
    }

    #[test]
    fn btsp_ndjson_detection_positive_with_newline() {
        assert!(is_btsp_ndjson(
            "{\"protocol\":\"btsp\",\"version\":1,\"client_ephemeral_pub\":\"key\"}\n"
        ));
    }

    #[test]
    fn btsp_ndjson_detection_negative_jsonrpc() {
        assert!(!is_btsp_ndjson(
            r#"{"jsonrpc":"2.0","method":"health.check","id":1}"#
        ));
    }

    #[test]
    fn btsp_ndjson_detection_negative_http() {
        assert!(!is_btsp_ndjson("POST /jsonrpc HTTP/1.1"));
    }

    #[test]
    fn btsp_ndjson_detection_negative_empty() {
        assert!(!is_btsp_ndjson(""));
        assert!(!is_btsp_ndjson("  \n"));
    }
}
