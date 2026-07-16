// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP provider JSON-RPC client for handshake delegation.
//!
//! All BTSP cryptographic operations are delegated to the handshake provider
//! via newline-delimited JSON-RPC over `TransportStream`, per
//! `PRIMAL_IPC_PROTOCOL.md` v3.1. LoamSpine never performs crypto directly —
//! it is a consumer of the provider's `btsp.session.*` capability domain.
//!
//! Per SOURDOUGH BTSP Relay Pattern §3: a single persistent connection is used
//! for all relay calls within one handshake. Write-half shutdown is never called
//! during relay — only flush after each request line.

use std::path::Path;

#[cfg(test)]
use crate::error::IpcErrorPhase;
use crate::error::LoamSpineError;
use crate::transport::stream::TransportStream;
use crate::transport::{DEFAULT_IPC_TIMEOUT, read_ndjson_response, write_ndjson_request};

/// Persistent connection to the BTSP provider.
///
/// Holds a single transport connection reused across all relay calls within
/// one handshake (create → verify → negotiate). Per SOURDOUGH §3: do not
/// reconnect per call; use `flush()`, never `shutdown()`.
///
/// Platform dispatch is handled by `connect_transport` — UDS on Unix,
/// TCP or Named Pipe (future) on Windows.
pub(crate) struct ProviderConn {
    reader: tokio::io::BufReader<tokio::io::ReadHalf<TransportStream>>,
    writer: tokio::io::WriteHalf<TransportStream>,
}

impl ProviderConn {
    /// Connect to the BTSP provider socket.
    pub(crate) async fn connect(socket: &Path) -> Result<Self, LoamSpineError> {
        let endpoint = crate::transport::endpoint_from_path(socket);
        let stream = crate::transport::connect_transport(&endpoint).await?;
        let (reader, writer) = stream.split();
        Ok(Self {
            reader: tokio::io::BufReader::new(reader),
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
        write_ndjson_request(&mut self.writer, &request, "BTSP provider").await?;
        read_ndjson_response(
            &mut self.reader,
            DEFAULT_IPC_TIMEOUT,
            method,
            "BTSP provider",
        )
        .await
    }
}

/// Test-accessible wrapper for `parse_response`.
#[cfg(test)]
pub(crate) fn parse_response_for_test<R: serde::de::DeserializeOwned>(
    response: &serde_json::Value,
    method: &str,
) -> Result<R, LoamSpineError> {
    parse_response(response, method)
}

/// Parse a BTSP provider JSON-RPC response value into the expected result type.
#[cfg(test)]
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
