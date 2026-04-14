// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP provider JSON-RPC client for handshake delegation.
//!
//! All BTSP cryptographic operations are delegated to the handshake provider
//! via newline-delimited JSON-RPC over UDS, per `PRIMAL_IPC_PROTOCOL.md` v3.1.
//! LoamSpine never performs crypto directly — it is a consumer of the provider's
//! `btsp.session.*` capability domain.

use std::path::Path;

use tokio::io::AsyncWriteExt;

use crate::error::{IpcErrorPhase, LoamSpineError};

/// Send a JSON-RPC request to the BTSP provider over UDS and return the result.
///
/// Accepts a pre-serialized `serde_json::Value` so the future is `Send`.
pub(crate) async fn provider_call<R: serde::de::DeserializeOwned>(
    socket: &Path,
    method: &str,
    params: serde_json::Value,
    request_id: u64,
) -> Result<R, LoamSpineError> {
    let request_bytes = serialize_request(method, &params, request_id)?;
    let response = provider_roundtrip(socket, method, &request_bytes).await?;
    parse_response(&response, method)
}

/// Build the JSON-RPC request bytes from a pre-built params `Value`.
fn serialize_request(
    method: &str,
    params: &serde_json::Value,
    request_id: u64,
) -> Result<Vec<u8>, LoamSpineError> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": request_id,
    });
    serde_json::to_vec(&request).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Serialization,
            format!("BTSP provider request serialize: {e}"),
        )
    })
}

/// Connect to the BTSP provider UDS, send request bytes, return response.
async fn provider_roundtrip(
    socket: &Path,
    method: &str,
    request_bytes: &[u8],
) -> Result<serde_json::Value, LoamSpineError> {
    let stream = tokio::net::UnixStream::connect(socket).await.map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Connect,
            format!("BTSP provider socket {} unreachable: {e}", socket.display()),
        )
    })?;

    let (reader, mut writer) = stream.into_split();

    writer.write_all(request_bytes).await.map_err(|e| {
        LoamSpineError::ipc(IpcErrorPhase::Write, format!("BTSP provider write: {e}"))
    })?;
    writer.write_all(b"\n").await.map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Write,
            format!("BTSP provider write newline: {e}"),
        )
    })?;
    writer.flush().await.map_err(|e| {
        LoamSpineError::ipc(IpcErrorPhase::Write, format!("BTSP provider flush: {e}"))
    })?;
    writer.shutdown().await.map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Write,
            format!("BTSP provider shutdown write: {e}"),
        )
    })?;

    let mut response_line = String::new();
    let mut buf_reader = tokio::io::BufReader::new(reader);
    tokio::io::AsyncBufReadExt::read_line(&mut buf_reader, &mut response_line)
        .await
        .map_err(|e| {
            LoamSpineError::ipc(IpcErrorPhase::Read, format!("BTSP provider read: {e}"))
        })?;

    serde_json::from_str(response_line.trim()).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::InvalidJson,
            format!("BTSP provider {method} response parse: {e}"),
        )
    })
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
