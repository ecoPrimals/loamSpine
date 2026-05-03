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
use loam_spine_core::btsp::{CIPHER_CHACHA20_POLY1305, SessionKeys};
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
                let session = match loam_spine_core::btsp::perform_ndjson_server_handshake(
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
                        session
                    }
                    Err(e) => {
                        warn!("BTSP NDJSON handshake failed: {e}");
                        return Ok(());
                    }
                };
                handle_post_handshake(&handler, &mut buf_reader, &mut writer, session).await
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
        let session = match loam_spine_core::btsp::perform_server_handshake(
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
                session
            }
            Err(e) => {
                warn!("BTSP handshake failed, refusing connection: {e}");
                return Ok(());
            }
        };
        handle_post_handshake(&handler, &mut buf_reader, &mut writer, session).await
    } else {
        debug!("UDS connection starts with non-JSON byte and no BTSP config; closing");
        Ok(())
    }
}

/// Post-handshake path: read the first JSON-RPC line, which may be
/// `btsp.negotiate`. If the negotiate selects `chacha20-poly1305`, derive
/// session keys and switch to encrypted framing for all subsequent messages.
/// Otherwise, continue in plaintext.
pub(crate) async fn handle_post_handshake<R, W>(
    handler: &LoamSpineJsonRpc,
    buf_reader: &mut tokio::io::BufReader<R>,
    writer: &mut W,
    session: loam_spine_core::btsp::BtspSession,
) -> Result<(), std::io::Error>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    if session.handshake_key.is_none() {
        return super::server::handle_stream_buffered(handler, buf_reader, writer).await;
    }
    let handshake_key = session.handshake_key.unwrap_or([0u8; 32]);

    handler
        .service()
        .register_btsp_session(session.session_id.clone(), handshake_key)
        .await;

    let mut first_line = String::new();
    let n = buf_reader.read_line(&mut first_line).await?;
    if n == 0 {
        return Ok(());
    }

    let trimmed = first_line.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    let client_nonce = extract_negotiate_client_nonce(trimmed);

    let resp_bytes = super::server::process_request(handler, trimmed.as_bytes()).await;
    if !resp_bytes.is_empty() {
        writer.write_all(&resp_bytes).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }

    if let Some(ref client_nonce) = client_nonce
        && let Some(keys) = try_derive_phase3_keys(&resp_bytes, &handshake_key, client_nonce)?
    {
        debug!("BTSP Phase 3: switching to encrypted framing");
        return handle_encrypted_stream(handler, buf_reader, writer, keys).await;
    }

    let mut line = String::new();
    loop {
        line.clear();
        let n = buf_reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }
        if let Some(resp) = super::server::process_ndjson_line(handler, &line).await {
            writer.write_all(&resp).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
        }
    }
    Ok(())
}

/// If the line is a `btsp.negotiate` request, extract the `client_nonce`.
pub(crate) fn extract_negotiate_client_nonce(line: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    let parsed: serde_json::Value = serde_json::from_str(line).ok()?;
    if parsed.get("method")?.as_str()? != "btsp.negotiate" {
        return None;
    }
    let nonce_b64 = parsed.get("params")?.get("client_nonce")?.as_str()?;
    base64::engine::general_purpose::STANDARD
        .decode(nonce_b64)
        .ok()
}

/// After the `btsp.negotiate` JSON-RPC response has been generated,
/// extract the `server_nonce` from it and derive `SessionKeys`.
/// Returns `None` if the cipher was `null` or parsing failed.
pub(crate) fn try_derive_phase3_keys(
    response_bytes: &[u8],
    handshake_key: &[u8; 32],
    client_nonce: &[u8],
) -> Result<Option<SessionKeys>, std::io::Error> {
    use base64::Engine;

    let resp: serde_json::Value = match serde_json::from_slice(response_bytes) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };

    let Some(result) = resp.get("result") else {
        return Ok(None);
    };

    let cipher = result.get("cipher").and_then(|c| c.as_str());
    if cipher != Some(CIPHER_CHACHA20_POLY1305) {
        return Ok(None);
    }

    let Some(server_nonce_b64) = result.get("server_nonce").and_then(|n| n.as_str()) else {
        return Ok(None);
    };

    let server_nonce = base64::engine::general_purpose::STANDARD
        .decode(server_nonce_b64)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let keys = SessionKeys::derive(handshake_key, client_nonce, &server_nonce, true)
        .map_err(std::io::Error::other)?;

    Ok(Some(keys))
}

/// Frame-encrypted message loop (Phase 3 transport).
///
/// Reads length-prefixed encrypted frames, decrypts, dispatches as JSON-RPC,
/// encrypts the response, and writes it back as a length-prefixed frame.
/// Frame format: `[4B big-endian length][12B nonce][ciphertext + 16B Poly1305 tag]`.
///
/// An 8 MiB guard prevents amplification from oversized frames.
pub(crate) async fn handle_encrypted_stream<R, W>(
    handler: &LoamSpineJsonRpc,
    reader: &mut tokio::io::BufReader<R>,
    writer: &mut W,
    keys: SessionKeys,
) -> Result<(), std::io::Error>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    use loam_spine_core::btsp::{read_encrypted_frame, write_encrypted_frame};

    loop {
        let plaintext = match read_encrypted_frame(reader, &keys).await {
            Ok(pt) => pt,
            Err(e) => {
                let msg = e.to_string();
                let lower = msg.to_ascii_lowercase();
                if lower.contains("unexpected eof")
                    || lower.contains("unexpectedeof")
                    || lower.contains("end of file")
                {
                    debug!("BTSP Phase 3: client closed encrypted connection");
                    return Ok(());
                }
                warn!("BTSP Phase 3 read error: {e}");
                return Err(std::io::Error::other(msg));
            }
        };

        let response = super::server::process_request(handler, &plaintext).await;
        if response.is_empty() {
            continue;
        }

        write_encrypted_frame(writer, &keys, &response)
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?;
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
