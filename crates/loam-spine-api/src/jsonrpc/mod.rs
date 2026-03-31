// SPDX-License-Identifier: AGPL-3.0-or-later

//! Pure Rust JSON-RPC 2.0 server for `LoamSpine`.
//!
//! Universal, language-agnostic RPC for external clients.
//! Works with Python, JavaScript, curl, etc.
//!
//! Zero C dependencies — replaces jsonrpsee (which pulled ring/C-asm)
//! with a hand-rolled JSON-RPC dispatcher over raw HTTP/TCP.

use crate::error::ServerError;
use crate::service::LoamSpineRpcService;
use loam_spine_core::error::{DispatchOutcome, IpcErrorPhase, LoamSpineError};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tracing::{debug, error, info, warn};

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

// Standard JSON-RPC 2.0 error codes
const PARSE_ERROR: i32 = -32700;
const INVALID_REQUEST: i32 = -32600;
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

    fn error(id: serde_json::Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
            id,
        }
    }
}

// ============================================================================
// Method normalization (backward-compatible alias resolution)
// ============================================================================

/// Normalize a JSON-RPC method name to its canonical `domain.operation` form.
///
/// Maps legacy aliases (e.g., `permanent-storage.commitSession`,
/// `primal.capabilities`, `commit.session`) to the canonical semantic
/// names defined in the wateringHole Semantic Method Naming Standard v2.1.
///
/// Absorbed from barraCuda v0.3.7's `normalize_method()` pattern — a
/// single normalization step before dispatch, instead of duplicated
/// match arms.
#[must_use]
pub fn normalize_method(method: &str) -> &str {
    match method {
        "commit.session" => "session.commit",
        "permanent-storage.commitSession" => "permanence.commit_session",
        "permanent-storage.verifyCommit" => "permanence.verify_commit",
        "permanent-storage.getCommit" => "permanence.get_commit",
        "permanent-storage.healthCheck" => "permanence.health_check",
        "capability.list" | "primal.capabilities" => "capabilities.list",
        other => other,
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
    ///
    /// Routes through [`dispatch_typed`](Self::dispatch_typed) for typed
    /// protocol vs. application error separation.
    pub async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let JsonRpcRequest {
            id, method, params, ..
        } = req;
        outcome_to_response(id, self.dispatch_typed(method.as_str(), params).await)
    }

    /// Dispatch a JSON-RPC method, returning typed [`DispatchOutcome`].
    ///
    /// Separates protocol-level errors (method not found, invalid params)
    /// from application-level errors. Enables typed middleware, retry
    /// logic, and observability per the ecosystem `DispatchOutcome` pattern
    /// (rhizoCrypt / airSpring / biomeOS).
    pub async fn dispatch_typed(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> DispatchOutcome<serde_json::Value> {
        let canonical = normalize_method(method);
        match self.dispatch(canonical, params).await {
            Ok(val) => DispatchOutcome::Ok(val),
            Err(e)
                if e.code == INVALID_PARAMS
                    || e.code == METHOD_NOT_FOUND
                    || e.code == PARSE_ERROR =>
            {
                DispatchOutcome::ProtocolError(LoamSpineError::ipc(
                    IpcErrorPhase::JsonRpcError(i64::from(e.code)),
                    e.message,
                ))
            }
            Err(e) => DispatchOutcome::ApplicationError {
                code: i64::from(e.code),
                message: e.message,
            },
        }
    }

    fn dispatch<'a>(
        &'a self,
        method: &'a str,
        params: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<serde_json::Value, JsonRpcError>> + Send + 'a>,
    > {
        Box::pin(self.dispatch_inner(method, params))
    }

    async fn dispatch_inner(
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

            "session.commit" => rpc!(params, commit_session),
            "braid.commit" => rpc!(params, commit_braid),

            "slice.anchor" => rpc!(params, anchor_slice),
            "slice.checkout" => rpc!(params, checkout_slice),

            "proof.generate_inclusion" => rpc!(params, generate_inclusion_proof),
            "proof.verify_inclusion" => rpc!(params, verify_inclusion_proof),

            "permanence.commit_session" => rpc!(params, permanent_storage_commit_session),
            "permanence.verify_commit" => rpc!(params, permanent_storage_verify_commit),
            "permanence.get_commit" => rpc!(params, permanent_storage_get_commit),
            "permanence.health_check" => ser(self.service.permanence_healthy().await),

            "capabilities.list" => Ok(loam_spine_core::neural_api::capability_list()),

            "tools.list" => Ok(loam_spine_core::neural_api::mcp_tools_list()),

            "tools.call" => {
                let tool_name = params
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| JsonRpcError {
                        code: INVALID_PARAMS,
                        message: "tools.call requires 'name' string".to_string(),
                        data: None,
                    })?;
                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
                let (rpc_method, rpc_params) = loam_spine_core::neural_api::mcp_tool_to_rpc(
                    tool_name, arguments,
                )
                .ok_or_else(|| JsonRpcError {
                    code: METHOD_NOT_FOUND,
                    message: format!("unknown tool: {tool_name}"),
                    data: None,
                })?;
                let inner_result = self.dispatch(rpc_method, rpc_params).await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": inner_result.to_string() }],
                    "isError": false,
                }))
            }

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

/// Convert a [`DispatchOutcome`] into a [`JsonRpcResponse`].
///
/// Protocol errors (method not found, invalid params) carry their
/// original JSON-RPC error code via [`IpcErrorPhase::JsonRpcError`].
/// Application errors use the code embedded in the outcome.
fn outcome_to_response(
    id: serde_json::Value,
    outcome: DispatchOutcome<serde_json::Value>,
) -> JsonRpcResponse {
    match outcome {
        DispatchOutcome::Ok(val) => JsonRpcResponse::success(id, val),
        DispatchOutcome::ApplicationError { code, message } => {
            JsonRpcResponse::error(id, i32::try_from(code).unwrap_or(LOAMSPINE_ERROR), message)
        }
        DispatchOutcome::ProtocolError(ref err) => {
            let (code, message) = match err {
                LoamSpineError::Ipc {
                    phase: IpcErrorPhase::JsonRpcError(c),
                    message,
                } => (
                    i32::try_from(*c).unwrap_or(LOAMSPINE_ERROR),
                    message.clone(),
                ),
                other => (LOAMSPINE_ERROR, other.to_string()),
            };
            JsonRpcResponse::error(id, code, message)
        }
    }
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
    let (reader, writer) = stream.into_split();
    handle_stream(handler, reader, writer).await
}

async fn handle_stream<R, W>(
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
fn serialize_response(response: &JsonRpcResponse) -> Vec<u8> {
    serde_json::to_vec(response).unwrap_or_else(|e| {
        error!("Failed to serialize JSON-RPC response: {e}");
        br#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"internal serialization error"},"id":null}"#.to_vec()
    })
}

/// Serialize a batch of JSON-RPC responses with the same fallback.
fn serialize_response_batch(responses: &[JsonRpcResponse]) -> Vec<u8> {
    serde_json::to_vec(responses).unwrap_or_else(|e| {
        error!("Failed to serialize JSON-RPC batch response: {e}");
        br#"[{"jsonrpc":"2.0","error":{"code":-32603,"message":"internal serialization error"},"id":null}]"#.to_vec()
    })
}

/// JSON-RPC 2.0: a request is a notification when the `id` member is absent
/// or null.  Per spec section 4.1, notifications MUST NOT receive a response.
fn is_notification(value: &serde_json::Value) -> bool {
    value.get("id").is_none_or(serde_json::Value::is_null)
}

pub(crate) async fn process_request(handler: &LoamSpineJsonRpc, body: &[u8]) -> Vec<u8> {
    // Parse as generic JSON first so we can inspect structure and detect
    // notifications (missing/null `id`) before committing to a type.
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

        // JSON-RPC 2.0 spec: "jsonrpc" MUST be exactly "2.0"
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

    // Remove stale socket from a previous run
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

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests;
#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests_permanence_cert;
#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests_protocol;
#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests_validation;
