// SPDX-License-Identifier: AGPL-3.0-or-later

//! IPC framing helpers — deduplicated wire formats for JSON-RPC over streams.
//!
//! Two framing modes are used across the ecosystem:
//!
//! | Mode | Format | Used by |
//! |------|--------|---------|
//! | NDJSON | `{json}\n` | BTSP provider, crypto provider |
//! | Length-prefixed | `[4B BE len][json]` | NeuralAPI, primal.announce |
//!
//! These helpers are generic over `AsyncRead + AsyncWrite` so they work
//! with both `TransportStream` (production) and raw streams (tests).

use std::time::Duration;

use serde::de::DeserializeOwned;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

use crate::error::{IpcErrorPhase, LoamSpineError};

/// Default read timeout for IPC responses.
pub const DEFAULT_IPC_TIMEOUT: Duration = Duration::from_secs(10);

// ──────────────────────────────────────────────────────────────────────────────
// NDJSON framing (newline-delimited JSON)
// ──────────────────────────────────────────────────────────────────────────────

/// Write a JSON-RPC request as a newline-delimited JSON line.
///
/// Serializes `request` to JSON, appends `\n`, and flushes.
///
/// # Errors
///
/// Returns `LoamSpineError::Ipc` on serialization or I/O failure.
pub async fn write_ndjson_request(
    writer: &mut (impl AsyncWriteExt + Unpin),
    request: &serde_json::Value,
    context: &str,
) -> Result<(), LoamSpineError> {
    let bytes = serde_json::to_vec(request).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Serialization,
            format!("{context} request serialize: {e}"),
        )
    })?;
    writer
        .write_all(&bytes)
        .await
        .map_err(|e| LoamSpineError::ipc(IpcErrorPhase::Write, format!("{context} write: {e}")))?;
    writer.write_all(b"\n").await.map_err(|e| {
        LoamSpineError::ipc(IpcErrorPhase::Write, format!("{context} newline: {e}"))
    })?;
    writer
        .flush()
        .await
        .map_err(|e| LoamSpineError::ipc(IpcErrorPhase::Write, format!("{context} flush: {e}")))?;
    Ok(())
}

/// Read an NDJSON response line with timeout, parse as JSON-RPC result.
///
/// Reads one line from a `BufReader`, deserializes the JSON, extracts
/// `result` (checking for `error` first), and deserializes into `R`.
///
/// # Errors
///
/// Returns `LoamSpineError::Ipc` on timeout, I/O, parse, or JSON-RPC error.
pub async fn read_ndjson_response<R: DeserializeOwned>(
    reader: &mut BufReader<impl AsyncReadExt + Unpin>,
    timeout: Duration,
    method: &str,
    context: &str,
) -> Result<R, LoamSpineError> {
    let mut line = String::new();
    tokio::time::timeout(timeout, reader.read_line(&mut line))
        .await
        .map_err(|_| {
            LoamSpineError::ipc(
                IpcErrorPhase::Read,
                format!("{context} {method}: read timeout ({timeout:?})"),
            )
        })?
        .map_err(|e| LoamSpineError::ipc(IpcErrorPhase::Read, format!("{context} read: {e}")))?;

    let response: serde_json::Value = serde_json::from_str(line.trim()).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::InvalidJson,
            format!("{context} {method} response parse: {e}"),
        )
    })?;

    parse_jsonrpc_result(&response, method, context)
}

/// Stateless NDJSON JSON-RPC call over a transport stream.
///
/// Sends a single NDJSON request, reads the response, and returns the
/// parsed result. The stream is consumed (single-use connection pattern).
///
/// # Errors
///
/// Returns `LoamSpineError::Ipc` on I/O, serialization, or JSON-RPC error.
pub async fn ndjson_rpc_call<R: DeserializeOwned>(
    stream: super::stream::TransportStream,
    method: &str,
    params: serde_json::Value,
    request_id: u64,
    timeout: Duration,
    context: &str,
) -> Result<R, LoamSpineError> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": request_id,
    });

    let (reader, mut writer) = stream.split();
    write_ndjson_request(&mut writer, &request, context).await?;

    let mut buf_reader = BufReader::new(reader);
    read_ndjson_response(&mut buf_reader, timeout, method, context).await
}

// ──────────────────────────────────────────────────────────────────────────────
// Length-prefixed framing ([4B BE len][json])
// ──────────────────────────────────────────────────────────────────────────────

/// Write a length-prefixed payload: `[4 bytes big-endian length][payload]`.
///
/// # Errors
///
/// Returns `LoamSpineError::Ipc` if the payload exceeds `u32::MAX` or on I/O failure.
pub async fn write_length_prefixed(
    writer: &mut (impl AsyncWriteExt + Unpin),
    payload: &[u8],
    context: &str,
) -> Result<(), LoamSpineError> {
    let len = u32::try_from(payload.len()).map_err(|_| {
        LoamSpineError::ipc(
            IpcErrorPhase::Serialization,
            format!("{context}: payload too large"),
        )
    })?;
    writer.write_all(&len.to_be_bytes()).await.map_err(|e| {
        LoamSpineError::ipc(IpcErrorPhase::Write, format!("{context} length write: {e}"))
    })?;
    writer.write_all(payload).await.map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Write,
            format!("{context} payload write: {e}"),
        )
    })?;
    writer
        .flush()
        .await
        .map_err(|e| LoamSpineError::ipc(IpcErrorPhase::Write, format!("{context} flush: {e}")))?;
    Ok(())
}

/// Read a length-prefixed payload: `[4 bytes big-endian length][payload]`.
///
/// # Errors
///
/// Returns `LoamSpineError::Ipc` on I/O failure or length overflow.
pub async fn read_length_prefixed(
    reader: &mut (impl AsyncReadExt + Unpin),
    context: &str,
) -> Result<Vec<u8>, LoamSpineError> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await.map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Read,
            format!("{context} response length read: {e}"),
        )
    })?;
    let resp_len = usize::try_from(u32::from_be_bytes(len_buf)).map_err(|_| {
        LoamSpineError::ipc(
            IpcErrorPhase::Read,
            format!("{context}: response length exceeds platform capacity"),
        )
    })?;
    let mut buf = vec![0u8; resp_len];
    reader.read_exact(&mut buf).await.map_err(|e| {
        LoamSpineError::ipc(IpcErrorPhase::Read, format!("{context} response read: {e}"))
    })?;
    Ok(buf)
}

/// Stateless length-prefixed JSON-RPC call.
///
/// Sends a JSON-RPC request with 4-byte big-endian length prefix,
/// reads the length-prefixed response, and returns the parsed `result`.
/// The stream is consumed (single-use connection pattern).
///
/// # Errors
///
/// Returns `LoamSpineError::Ipc` on I/O, serialization, or JSON-RPC error.
pub async fn length_prefixed_rpc_call(
    stream: &mut super::stream::TransportStream,
    method: &str,
    params: serde_json::Value,
    request_id: u64,
    context: &str,
) -> Result<serde_json::Value, LoamSpineError> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": request_id,
    });
    let request_bytes = serde_json::to_vec(&request).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::Serialization,
            format!("{context}: request serialize: {e}"),
        )
    })?;

    write_length_prefixed(stream, &request_bytes, context).await?;
    let resp_buf = read_length_prefixed(stream, context).await?;

    let response: serde_json::Value = serde_json::from_slice(&resp_buf).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::InvalidJson,
            format!("{context} {method} response parse: {e}"),
        )
    })?;

    if let Some((code, message)) = crate::error::extract_rpc_error(&response) {
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::JsonRpcError(code),
            format!("{context} {method}: {message}"),
        ));
    }

    response.get("result").cloned().ok_or_else(|| {
        LoamSpineError::ipc(
            IpcErrorPhase::NoResult,
            format!("{context} {method}: missing result field"),
        )
    })
}

// ──────────────────────────────────────────────────────────────────────────────
// Shared JSON-RPC parsing
// ──────────────────────────────────────────────────────────────────────────────

/// Parse a JSON-RPC response: check for `error`, extract and deserialize `result`.
fn parse_jsonrpc_result<R: DeserializeOwned>(
    response: &serde_json::Value,
    method: &str,
    context: &str,
) -> Result<R, LoamSpineError> {
    if let Some(err) = response.get("error") {
        let code = err
            .get("code")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(-1);
        let msg = err
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown error");
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::JsonRpcError(code),
            format!("{context} {method}: {msg}"),
        ));
    }

    let result = response.get("result").ok_or_else(|| {
        LoamSpineError::ipc(
            IpcErrorPhase::NoResult,
            format!("{context} {method}: missing result field"),
        )
    })?;

    R::deserialize(result).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::InvalidJson,
            format!("{context} {method} result deserialize: {e}"),
        )
    })
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn ndjson_roundtrip_via_tcp() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut buf_reader = BufReader::new(reader);
            let mut line = String::new();
            buf_reader.read_line(&mut line).await.unwrap();
            let req: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
            let id = req.get("id").cloned().unwrap_or_default();
            let resp = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": { "value": 42 }
            });
            let mut bytes = serde_json::to_vec(&resp).unwrap();
            bytes.push(b'\n');
            writer.write_all(&bytes).await.unwrap();
            writer.flush().await.unwrap();
        });

        let ep = super::super::TransportEndpoint::tcp("127.0.0.1", addr.port());
        let stream = super::super::stream::connect_transport(&ep).await.unwrap();

        let result: serde_json::Value = ndjson_rpc_call(
            stream,
            "test.method",
            serde_json::json!({}),
            1,
            DEFAULT_IPC_TIMEOUT,
            "test",
        )
        .await
        .unwrap();

        assert_eq!(result["value"], 42);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn length_prefixed_roundtrip_via_tcp() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut len_buf = [0u8; 4];
            stream.read_exact(&mut len_buf).await.unwrap();
            let req_len = u32::from_be_bytes(len_buf) as usize;
            let mut req_buf = vec![0u8; req_len];
            stream.read_exact(&mut req_buf).await.unwrap();
            let req: serde_json::Value = serde_json::from_slice(&req_buf).unwrap();
            let id = req.get("id").cloned().unwrap_or_default();
            let resp = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": { "ok": true }
            });
            let resp_bytes = serde_json::to_vec(&resp).unwrap();
            let len = u32::try_from(resp_bytes.len()).unwrap().to_be_bytes();
            stream.write_all(&len).await.unwrap();
            stream.write_all(&resp_bytes).await.unwrap();
            stream.flush().await.unwrap();
        });

        let ep = super::super::TransportEndpoint::tcp("127.0.0.1", addr.port());
        let mut stream = super::super::stream::connect_transport(&ep).await.unwrap();

        let result =
            length_prefixed_rpc_call(&mut stream, "test.method", serde_json::json!({}), 1, "test")
                .await
                .unwrap();

        assert_eq!(result["ok"], true);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn ndjson_error_response() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut buf_reader = BufReader::new(reader);
            let mut line = String::new();
            buf_reader.read_line(&mut line).await.unwrap();
            let resp = serde_json::json!({
                "jsonrpc": "2.0", "id": 1,
                "error": { "code": -32601, "message": "method not found" }
            });
            let mut bytes = serde_json::to_vec(&resp).unwrap();
            bytes.push(b'\n');
            writer.write_all(&bytes).await.unwrap();
            writer.flush().await.unwrap();
        });

        let ep = super::super::TransportEndpoint::tcp("127.0.0.1", addr.port());
        let stream = super::super::stream::connect_transport(&ep).await.unwrap();

        let result: Result<serde_json::Value, _> = ndjson_rpc_call(
            stream,
            "bad.method",
            serde_json::json!({}),
            1,
            DEFAULT_IPC_TIMEOUT,
            "test",
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("method not found"));
        server.await.unwrap();
    }

    #[test]
    fn parse_jsonrpc_result_ok() {
        let resp = serde_json::json!({
            "jsonrpc": "2.0", "id": 1,
            "result": { "key": "val" }
        });
        let r: serde_json::Value = parse_jsonrpc_result(&resp, "test", "ctx").unwrap();
        assert_eq!(r["key"], "val");
    }

    #[test]
    fn parse_jsonrpc_result_error() {
        let resp = serde_json::json!({
            "jsonrpc": "2.0", "id": 1,
            "error": { "code": -1, "message": "fail" }
        });
        let r: Result<serde_json::Value, _> = parse_jsonrpc_result(&resp, "test", "ctx");
        assert!(r.is_err());
    }

    #[test]
    fn parse_jsonrpc_result_missing() {
        let resp = serde_json::json!({ "jsonrpc": "2.0", "id": 1 });
        let r: Result<serde_json::Value, _> = parse_jsonrpc_result(&resp, "test", "ctx");
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("missing result"));
    }

    #[tokio::test]
    async fn length_prefixed_zero_length_frame() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut len_buf = [0u8; 4];
            stream.read_exact(&mut len_buf).await.unwrap();
            let req_len = u32::from_be_bytes(len_buf) as usize;
            let mut req_buf = vec![0u8; req_len];
            stream.read_exact(&mut req_buf).await.unwrap();
            // respond with zero-length frame
            stream.write_all(&0u32.to_be_bytes()).await.unwrap();
            stream.flush().await.unwrap();
        });

        let ep = super::super::TransportEndpoint::tcp("127.0.0.1", addr.port());
        let mut stream = super::super::stream::connect_transport(&ep).await.unwrap();

        let result: Result<serde_json::Value, _> =
            length_prefixed_rpc_call(&mut stream, "test.zero", serde_json::json!({}), 1, "test")
                .await;

        assert!(result.is_err());
        server.await.unwrap();
    }

    #[tokio::test]
    async fn length_prefixed_server_disconnect() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut len_buf = [0u8; 4];
            stream.read_exact(&mut len_buf).await.unwrap();
            let req_len = u32::from_be_bytes(len_buf) as usize;
            let mut req_buf = vec![0u8; req_len];
            stream.read_exact(&mut req_buf).await.unwrap();
            drop(stream);
        });

        let ep = super::super::TransportEndpoint::tcp("127.0.0.1", addr.port());
        let mut stream = super::super::stream::connect_transport(&ep).await.unwrap();

        let result: Result<serde_json::Value, _> =
            length_prefixed_rpc_call(&mut stream, "test.dc", serde_json::json!({}), 1, "test")
                .await;

        assert!(result.is_err());
        server.await.unwrap();
    }

    #[tokio::test]
    async fn ndjson_server_sends_empty_line_then_response() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut buf_reader = BufReader::new(reader);
            let mut line = String::new();
            buf_reader.read_line(&mut line).await.unwrap();
            let req: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
            let id = req.get("id").cloned().unwrap_or_default();
            let resp = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": "ok"
            });
            let mut bytes = serde_json::to_vec(&resp).unwrap();
            bytes.push(b'\n');
            writer.write_all(&bytes).await.unwrap();
            writer.flush().await.unwrap();
        });

        let ep = super::super::TransportEndpoint::tcp("127.0.0.1", addr.port());
        let stream = super::super::stream::connect_transport(&ep).await.unwrap();

        let result: String = ndjson_rpc_call(
            stream,
            "test.echo",
            serde_json::json!({}),
            1,
            DEFAULT_IPC_TIMEOUT,
            "test",
        )
        .await
        .unwrap();

        assert_eq!(result, "ok");
        server.await.unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn ndjson_roundtrip_via_uds() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("test.sock");
        let sock_path_str = sock_path.display().to_string();

        let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut buf_reader = BufReader::new(reader);
            let mut line = String::new();
            buf_reader.read_line(&mut line).await.unwrap();
            let req: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
            let id = req.get("id").cloned().unwrap_or_default();
            let resp = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": { "transport": "uds" }
            });
            let mut bytes = serde_json::to_vec(&resp).unwrap();
            bytes.push(b'\n');
            writer.write_all(&bytes).await.unwrap();
            writer.flush().await.unwrap();
        });

        let ep = super::super::TransportEndpoint::Uds {
            path: sock_path_str,
        };
        let stream = super::super::stream::connect_transport(&ep).await.unwrap();

        let result: serde_json::Value = ndjson_rpc_call(
            stream,
            "test.uds",
            serde_json::json!({}),
            1,
            DEFAULT_IPC_TIMEOUT,
            "uds-test",
        )
        .await
        .unwrap();

        assert_eq!(result["transport"], "uds");
        server.await.unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn length_prefixed_roundtrip_via_uds() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("test_lp.sock");
        let sock_path_str = sock_path.display().to_string();

        let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut len_buf = [0u8; 4];
            stream.read_exact(&mut len_buf).await.unwrap();
            let req_len = u32::from_be_bytes(len_buf) as usize;
            let mut req_buf = vec![0u8; req_len];
            stream.read_exact(&mut req_buf).await.unwrap();
            let req: serde_json::Value = serde_json::from_slice(&req_buf).unwrap();
            let id = req.get("id").cloned().unwrap_or_default();
            let resp = serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": { "via": "uds_lp" }
            });
            let resp_bytes = serde_json::to_vec(&resp).unwrap();
            let len = u32::try_from(resp_bytes.len()).unwrap().to_be_bytes();
            stream.write_all(&len).await.unwrap();
            stream.write_all(&resp_bytes).await.unwrap();
            stream.flush().await.unwrap();
        });

        let ep = super::super::TransportEndpoint::Uds {
            path: sock_path_str,
        };
        let mut stream = super::super::stream::connect_transport(&ep).await.unwrap();

        let result: serde_json::Value =
            length_prefixed_rpc_call(&mut stream, "test.lp_uds", serde_json::json!({}), 1, "test")
                .await
                .unwrap();

        assert_eq!(result["via"], "uds_lp");
        server.await.unwrap();
    }
}
