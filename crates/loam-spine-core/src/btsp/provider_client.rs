// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP provider JSON-RPC client for handshake delegation.
//!
//! All BTSP cryptographic operations are delegated to the handshake provider
//! via newline-delimited JSON-RPC over UDS, per `PRIMAL_IPC_PROTOCOL.md` v3.1.
//! LoamSpine never performs crypto directly — it is a consumer of the provider's
//! `btsp.session.*` capability domain.
//!
//! Per SOURDOUGH BTSP Relay Pattern §3: a single persistent connection is used
//! for all relay calls within one handshake. Write-half shutdown is never called
//! during relay — only flush after each request line.

use std::path::Path;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};

use crate::error::{IpcErrorPhase, LoamSpineError};

/// Timeout for reading a single JSON-RPC response from the BTSP provider.
const PROVIDER_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// Persistent connection to the BTSP provider (BearDog).
///
/// Holds a single UDS connection reused across all relay calls within one
/// handshake (create → verify → negotiate). Per SOURDOUGH §3: do not
/// reconnect per call; use `flush()`, never `shutdown()`.
pub(crate) struct ProviderConn {
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
}

impl ProviderConn {
    /// Connect to the BTSP provider socket.
    pub(crate) async fn connect(socket: &Path) -> Result<Self, LoamSpineError> {
        let stream = tokio::net::UnixStream::connect(socket).await.map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::Connect,
                format!("BTSP provider socket {} unreachable: {e}", socket.display()),
            )
        })?;
        let (reader, writer) = stream.into_split();
        Ok(Self {
            reader: BufReader::new(reader),
            writer,
        })
    }

    /// Send a JSON-RPC request and read the response on the existing connection.
    ///
    /// Uses `flush()` only (never `shutdown()`). Reads with a timeout to
    /// prevent hangs if the provider drops the connection.
    pub(crate) async fn call<R: serde::de::DeserializeOwned>(
        &mut self,
        method: &str,
        params: serde_json::Value,
        request_id: u64,
    ) -> Result<R, LoamSpineError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": request_id,
        });
        let request_bytes = serde_json::to_vec(&request).map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::Serialization,
                format!("BTSP provider request serialize: {e}"),
            )
        })?;

        self.writer.write_all(&request_bytes).await.map_err(|e| {
            LoamSpineError::ipc(IpcErrorPhase::Write, format!("BTSP provider write: {e}"))
        })?;
        self.writer.write_all(b"\n").await.map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::Write,
                format!("BTSP provider write newline: {e}"),
            )
        })?;
        self.writer.flush().await.map_err(|e| {
            LoamSpineError::ipc(IpcErrorPhase::Write, format!("BTSP provider flush: {e}"))
        })?;

        let mut response_line = String::new();
        tokio::time::timeout(
            PROVIDER_READ_TIMEOUT,
            self.reader.read_line(&mut response_line),
        )
        .await
        .map_err(|_| {
            LoamSpineError::ipc(
                IpcErrorPhase::Read,
                format!("BTSP provider {method}: read timeout ({PROVIDER_READ_TIMEOUT:?})"),
            )
        })?
        .map_err(|e| {
            LoamSpineError::ipc(IpcErrorPhase::Read, format!("BTSP provider read: {e}"))
        })?;

        let response: serde_json::Value =
            serde_json::from_str(response_line.trim()).map_err(|e| {
                LoamSpineError::ipc(
                    IpcErrorPhase::InvalidJson,
                    format!("BTSP provider {method} response parse: {e}"),
                )
            })?;

        parse_response(&response, method)
    }
}

/// Parse a BTSP provider JSON-RPC response value into the expected result type.
fn parse_response<R: serde::de::DeserializeOwned>(
    response: &serde_json::Value,
    method: &str,
) -> Result<R, LoamSpineError> {
    if let Some(err) = response.get("error") {
        let code = err
            .get("code")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(-1);
        let msg = err
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown BTSP provider error");
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::JsonRpcError(code),
            format!("BTSP provider {method}: {msg}"),
        ));
    }

    let result = response.get("result").ok_or_else(|| {
        LoamSpineError::ipc(
            IpcErrorPhase::NoResult,
            format!("BTSP provider {method}: missing result field"),
        )
    })?;

    R::deserialize(result).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::InvalidJson,
            format!("BTSP provider {method} result deserialize: {e}"),
        )
    })
}
