// SPDX-License-Identifier: AGPL-3.0-only

//! Pure Rust JSON-RPC 2.0 server for `LoamSpine`.
//!
//! Universal, language-agnostic RPC for external clients.
//! Works with Python, JavaScript, curl, etc.
//!
//! Zero C dependencies — replaces jsonrpsee (which pulled ring/C-asm)
//! with a hand-rolled JSON-RPC dispatcher over raw HTTP/TCP.

use crate::error::ServerError;
use crate::service::LoamSpineRpcService;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tracing::{debug, info, warn};

// ============================================================================
// JSON-RPC 2.0 wire types
// ============================================================================

/// A JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Protocol version (must be "2.0").
    pub jsonrpc: String,
    /// Method name.
    pub method: String,
    /// Method parameters.
    #[serde(default)]
    pub params: serde_json::Value,
    /// Request ID (null for notifications).
    #[serde(default)]
    pub id: serde_json::Value,
}

/// A JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version.
    pub jsonrpc: String,
    /// Successful result (mutually exclusive with `error`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error (mutually exclusive with `result`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Request ID (echoed from the request).
    pub id: serde_json::Value,
}

/// A JSON-RPC 2.0 error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i32,
    /// Error message.
    pub message: String,
    /// Additional data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// Standard error codes
const PARSE_ERROR: i32 = -32700;
const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;
const LOAMSPINE_ERROR: i32 = -32000;

impl JsonRpcResponse {
    fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    fn error(id: serde_json::Value, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}

// ============================================================================
// Dispatcher
// ============================================================================

/// JSON-RPC server implementation — pure Rust, no jsonrpsee.
pub struct LoamSpineJsonRpc {
    pub(crate) service: Arc<LoamSpineRpcService>,
}

impl LoamSpineJsonRpc {
    /// Create a new JSON-RPC handler.
    #[must_use]
    pub fn new(service: LoamSpineRpcService) -> Self {
        Self {
            service: Arc::new(service),
        }
    }

    /// Create with default service.
    #[must_use]
    pub fn default_server() -> Self {
        Self::new(LoamSpineRpcService::default_service())
    }

    /// Handle a JSON-RPC request and produce a response.
    pub async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let JsonRpcRequest {
            id, method, params, ..
        } = req;
        match self.dispatch(method.as_str(), params).await {
            Ok(val) => JsonRpcResponse::success(id, val),
            Err(e) => JsonRpcResponse::error(id, e.code, e.message),
        }
    }

    async fn dispatch(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, JsonRpcError> {
        macro_rules! rpc {
            ($params:expr, $method:ident) => {{
                let req = deser($params)?;
                let res = self.service.$method(req).await.map_err(app_err)?;
                ser(res)
            }};
        }

        match method {
            "spine.create" => rpc!(params, create_spine),
            "spine.get" => rpc!(params, get_spine),
            "spine.seal" => rpc!(params, seal_spine),

            "entry.append" => rpc!(params, append_entry),
            "entry.get" => rpc!(params, get_entry),
            "entry.get_tip" => rpc!(params, get_tip),

            "certificate.mint" => rpc!(params, mint_certificate),
            "certificate.transfer" => rpc!(params, transfer_certificate),
            "certificate.loan" => rpc!(params, loan_certificate),
            "certificate.return" => rpc!(params, return_certificate),
            "certificate.get" => rpc!(params, get_certificate),

            "health.check" => rpc!(params, health_check),
            "health.liveness" => ser(self.service.liveness().await),
            "health.readiness" => {
                let probe = self.service.readiness().await.map_err(app_err)?;
                ser(probe)
            }

            "session.commit" | "commit.session" => rpc!(params, commit_session),
            "braid.commit" => rpc!(params, commit_braid),

            "slice.anchor" => rpc!(params, anchor_slice),
            "slice.checkout" => rpc!(params, checkout_slice),

            "proof.generate_inclusion" => rpc!(params, generate_inclusion_proof),
            "proof.verify_inclusion" => rpc!(params, verify_inclusion_proof),

            "permanence.commit_session" | "permanent-storage.commitSession" => {
                rpc!(params, permanent_storage_commit_session)
            }
            "permanence.verify_commit" | "permanent-storage.verifyCommit" => {
                rpc!(params, permanent_storage_verify_commit)
            }
            "permanence.get_commit" | "permanent-storage.getCommit" => {
                rpc!(params, permanent_storage_get_commit)
            }
            "permanence.health_check" | "permanent-storage.healthCheck" => {
                ser(self.service.permanence_healthy().await)
            }

            "capability.list" => Ok(loam_spine_core::neural_api::capability_list()),

            _ => Err(JsonRpcError {
                code: METHOD_NOT_FOUND,
                message: format!("method not found: {method}"),
                data: None,
            }),
        }
    }
}

fn app_err(e: impl std::fmt::Display) -> JsonRpcError {
    JsonRpcError {
        code: LOAMSPINE_ERROR,
        message: e.to_string(),
        data: None,
    }
}

fn deser<T: serde::de::DeserializeOwned>(params: serde_json::Value) -> Result<T, JsonRpcError> {
    serde_json::from_value(params).map_err(|e| JsonRpcError {
        code: INVALID_PARAMS,
        message: format!("invalid params: {e}"),
        data: None,
    })
}

fn ser<T: serde::Serialize>(val: T) -> Result<serde_json::Value, JsonRpcError> {
    serde_json::to_value(val).map_err(|e| JsonRpcError {
        code: LOAMSPINE_ERROR,
        message: format!("serialization error: {e}"),
        data: None,
    })
}

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
    let (reader, mut writer) = stream.into_split();
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
                content_length = val.trim().parse().unwrap_or(0);
            }
        }

        let mut body = vec![0u8; content_length];
        buf_reader.read_exact(&mut body).await?;

        let response_body = process_request(&handler, &body).await;

        let http_response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            response_body.len()
        );
        writer.write_all(http_response.as_bytes()).await?;
        writer.write_all(&response_body).await?;
    } else {
        let body = first_line.trim().as_bytes().to_vec();
        if !body.is_empty() {
            let response_body = process_request(&handler, &body).await;
            writer.write_all(&response_body).await?;
            writer.write_all(b"\n").await?;
        }
    }

    writer.flush().await?;
    Ok(())
}

pub(crate) async fn process_request(handler: &LoamSpineJsonRpc, body: &[u8]) -> Vec<u8> {
    // Try single request first
    if let Ok(request) = serde_json::from_slice::<JsonRpcRequest>(body) {
        let response = handler.handle_request(request).await;
        return serde_json::to_vec(&response).unwrap_or_default();
    }

    // Try batch request
    if let Ok(batch) = serde_json::from_slice::<Vec<serde_json::Value>>(body) {
        if batch.is_empty() {
            let err = JsonRpcResponse::error(
                serde_json::Value::Null,
                PARSE_ERROR,
                "empty batch".to_string(),
            );
            return serde_json::to_vec(&err).unwrap_or_default();
        }
        let mut responses = Vec::with_capacity(batch.len());
        for item in batch {
            let is_notification =
                item.get("id").is_none() || item.get("id").is_some_and(serde_json::Value::is_null);
            match serde_json::from_value::<JsonRpcRequest>(item) {
                Ok(req) => {
                    let resp = handler.handle_request(req).await;
                    if !is_notification {
                        responses.push(resp);
                    }
                }
                Err(e) => {
                    responses.push(JsonRpcResponse::error(
                        serde_json::Value::Null,
                        PARSE_ERROR,
                        format!("parse error in batch element: {e}"),
                    ));
                }
            }
        }
        if responses.is_empty() {
            return Vec::new();
        }
        return serde_json::to_vec(&responses).unwrap_or_default();
    }

    // Neither single nor batch
    let err = JsonRpcResponse::error(
        serde_json::Value::Null,
        PARSE_ERROR,
        "parse error: expected JSON-RPC request or batch array".to_string(),
    );
    serde_json::to_vec(&err).unwrap_or_default()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests;
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests_validation;
