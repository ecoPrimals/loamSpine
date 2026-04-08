// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON-RPC server infrastructure for TCP and Unix domain socket transports.
//!
//! Accepts both raw newline-delimited JSON and HTTP POST requests over TCP.
//! UDS transport uses newline-delimited JSON only (per `IPC_COMPLIANCE_MATRIX`).

use super::wire::{
    JsonRpcRequest, JsonRpcResponse, INVALID_REQUEST, PARSE_ERROR,
};
use super::LoamSpineJsonRpc;
use crate::error::ServerError;
use crate::service::LoamSpineRpcService;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tracing::{debug, error, info, warn};

// ============================================================================
// TCP/HTTP server
// ============================================================================

/// Server handle for graceful shutdown.
pub struct ServerHandle {
    shutdown: tokio::sync::watch::Sender<bool>,
    done: tokio::sync::watch::Receiver<bool>,
    addr: SocketAddr,
}

impl ServerHandle {
    /// Stop the server.
    pub fn stop(&self) {
        let _ = self.shutdown.send(true);
    }

    /// Wait until the server has stopped.
    pub async fn stopped(&mut self) {
        let _ = self.done.changed().await;
    }

    /// Get the actual bound address (useful when binding to port 0).
    #[must_use]
    pub const fn local_addr(&self) -> SocketAddr {
        self.addr
    }
}

/// Run the JSON-RPC server (pure Rust, no jsonrpsee).
///
/// Accepts both raw newline-delimited JSON and HTTP POST requests.
///
/// # Errors
///
/// Returns error if server fails to bind.
pub async fn run_jsonrpc_server(
    addr: SocketAddr,
    service: LoamSpineRpcService,
) -> Result<ServerHandle, ServerError> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| ServerError::Bind(e.to_string()))?;

    let bound_addr = listener
        .local_addr()
        .map_err(|e| ServerError::Bind(e.to_string()))?;

    let handler = Arc::new(LoamSpineJsonRpc::new(service));
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let (done_tx, done_rx) = tokio::sync::watch::channel(false);

    info!(
        "LoamSpine JSON-RPC server listening on http://{}",
        bound_addr
    );

    tokio::spawn(async move {
        let mut rx = shutdown_rx;
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, peer)) => {
                            debug!("JSON-RPC connection from {peer}");
                            let h = Arc::clone(&handler);
                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(h, stream).await {
                                    warn!("connection error: {e}");
                                }
                            });
                        }
                        Err(e) => {
                            warn!("accept error: {e}");
                        }
                    }
                }
                _ = rx.changed() => {
                    info!("JSON-RPC server shutting down");
                    break;
                }
            }
        }
        let _ = done_tx.send(true);
    });

    Ok(ServerHandle {
        shutdown: shutdown_tx,
        done: done_rx,
        addr: bound_addr,
    })
}

async fn handle_connection(
    handler: Arc<LoamSpineJsonRpc>,
    stream: tokio::net::TcpStream,
) -> Result<(), std::io::Error> {
    let (reader, writer) = stream.into_split();
    handle_stream(handler, reader, writer).await
}

pub(crate) async fn handle_stream<R, W>(
    handler: Arc<LoamSpineJsonRpc>,
    reader: R,
    mut writer: W,
) -> Result<(), std::io::Error>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    let mut buf_reader = BufReader::new(reader);

    let mut first_line = String::new();
    buf_reader.read_line(&mut first_line).await?;

    let is_http = first_line.starts_with("POST")
        || first_line.starts_with("GET")
        || first_line.starts_with("HTTP");

    if is_http {
        let mut content_length: usize = 0;
        let mut header_line = String::new();
        loop {
            header_line.clear();
            buf_reader.read_line(&mut header_line).await?;
            if header_line.trim().is_empty() {
                break;
            }
            if let Some(val) = header_line
                .strip_prefix("Content-Length:")
                .or_else(|| header_line.strip_prefix("content-length:"))
            {
                content_length = match val.trim().parse() {
                    Ok(len) => len,
                    Err(e) => {
                        warn!("Malformed Content-Length header {:?}: {e}", val.trim());
                        0
                    }
                };
            }
        }

        let mut body = vec![0u8; content_length];
        buf_reader.read_exact(&mut body).await?;

        let response_body = process_request(&handler, &body).await;

        if response_body.is_empty() {
            writer
                .write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n")
                .await?;
        } else {
            let http_response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                response_body.len()
            );
            writer.write_all(http_response.as_bytes()).await?;
            writer.write_all(&response_body).await?;
        }
    } else {
        let body = first_line.trim().as_bytes().to_vec();
        if !body.is_empty() {
            let response_body = process_request(&handler, &body).await;
            if !response_body.is_empty() {
                writer.write_all(&response_body).await?;
                writer.write_all(b"\n").await?;
            }
        }
    }

    writer.flush().await?;
    Ok(())
}

/// Serialize a JSON-RPC response, logging and returning a hard-coded
/// internal error if serialization itself fails.
pub(crate) fn serialize_response(response: &JsonRpcResponse) -> Vec<u8> {
    serde_json::to_vec(response).unwrap_or_else(|e| {
        error!("Failed to serialize JSON-RPC response: {e}");
        br#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"internal serialization error"},"id":null}"#.to_vec()
    })
}

/// Serialize a batch of JSON-RPC responses with the same fallback.
pub(crate) fn serialize_response_batch(responses: &[JsonRpcResponse]) -> Vec<u8> {
    serde_json::to_vec(responses).unwrap_or_else(|e| {
        error!("Failed to serialize JSON-RPC batch response: {e}");
        br#"[{"jsonrpc":"2.0","error":{"code":-32603,"message":"internal serialization error"},"id":null}]"#.to_vec()
    })
}

/// JSON-RPC 2.0: a request is a notification when the `id` member is absent
/// or null.  Per spec section 4.1, notifications MUST NOT receive a response.
pub(crate) fn is_notification(value: &serde_json::Value) -> bool {
    value.get("id").is_none_or(serde_json::Value::is_null)
}

pub(crate) async fn process_request(handler: &LoamSpineJsonRpc, body: &[u8]) -> Vec<u8> {
    let parsed: serde_json::Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => {
            return serialize_response(&JsonRpcResponse::error(
                serde_json::Value::Null,
                PARSE_ERROR,
                "parse error: invalid JSON",
            ));
        }
    };

    // Single request (JSON object)
    if parsed.is_object() {
        let notification = is_notification(&parsed);

        let request: JsonRpcRequest = match serde_json::from_value(parsed) {
            Ok(r) => r,
            Err(e) => {
                if notification {
                    return Vec::new();
                }
                return serialize_response(&JsonRpcResponse::error(
                    serde_json::Value::Null,
                    INVALID_REQUEST,
                    format!("invalid request: {e}"),
                ));
            }
        };

        if request.jsonrpc != "2.0" {
            if notification {
                return Vec::new();
            }
            return serialize_response(&JsonRpcResponse::error(
                request.id,
                INVALID_REQUEST,
                "jsonrpc version must be \"2.0\"",
            ));
        }

        let response = handler.handle_request(request).await;
        if notification {
            return Vec::new();
        }
        return serialize_response(&response);
    }

    // Batch request (JSON array)
    if let serde_json::Value::Array(batch) = parsed {
        if batch.is_empty() {
            return serialize_response(&JsonRpcResponse::error(
                serde_json::Value::Null,
                INVALID_REQUEST,
                "empty batch",
            ));
        }
        let mut responses = Vec::with_capacity(batch.len());
        for item in batch {
            let notification = is_notification(&item);
            match serde_json::from_value::<JsonRpcRequest>(item) {
                Ok(req) => {
                    if req.jsonrpc == "2.0" {
                        let resp = handler.handle_request(req).await;
                        if !notification {
                            responses.push(resp);
                        }
                    } else if !notification {
                        responses.push(JsonRpcResponse::error(
                            req.id,
                            INVALID_REQUEST,
                            "jsonrpc version must be \"2.0\"",
                        ));
                    }
                }
                Err(e) => {
                    if !notification {
                        responses.push(JsonRpcResponse::error(
                            serde_json::Value::Null,
                            INVALID_REQUEST,
                            format!("invalid request in batch: {e}"),
                        ));
                    }
                }
            }
        }
        if responses.is_empty() {
            return Vec::new();
        }
        return serialize_response_batch(&responses);
    }

    // Neither object nor array
    serialize_response(&JsonRpcResponse::error(
        serde_json::Value::Null,
        PARSE_ERROR,
        "expected JSON-RPC request object or batch array",
    ))
}

// ============================================================================
// Unix Domain Socket server (IPC_COMPLIANCE_MATRIX requirement)
// ============================================================================

/// Server handle for a UDS JSON-RPC listener.
#[cfg(unix)]
pub struct UdsServerHandle {
    shutdown: tokio::sync::watch::Sender<bool>,
    done: tokio::sync::watch::Receiver<bool>,
    path: std::path::PathBuf,
}

#[cfg(unix)]
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

#[cfg(unix)]
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
/// # Errors
///
/// Returns error if the socket cannot be bound.
#[cfg(unix)]
pub async fn run_jsonrpc_uds_server(
    path: impl Into<std::path::PathBuf>,
    service: LoamSpineRpcService,
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

    info!(
        "LoamSpine JSON-RPC UDS server listening on {}",
        path.display()
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
                            tokio::spawn(async move {
                                let (reader, writer) = stream.into_split();
                                if let Err(e) = handle_stream(h, reader, writer).await {
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
