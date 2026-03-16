// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tower Atomic transport via NeuralAPI → Songbird.
//!
//! This transport delegates all HTTP operations to **Songbird** (Pure Rust
//! TLS 1.3 + HTTP) through BiomeOS's **NeuralAPI** orchestration layer,
//! communicating over a Unix domain socket with JSON-RPC 2.0.
//!
//! **Zero C dependencies** — this is the ecoBin-compliant production path.
//!
//! ## Architecture
//!
//! ```text
//! LoamSpine ──JSON-RPC──▶ NeuralAPI ──route──▶ Songbird ──HTTP/TLS──▶ Registry
//!            (Unix sock)              (Unix sock)         (Pure Rust)
//! ```
//!
//! ## Socket Discovery
//!
//! The NeuralAPI socket is located at:
//! `$XDG_RUNTIME_DIR/biomeos/neural-api-{family_id}.sock`
//!
//! If `BIOMEOS_NEURAL_API_SOCKET` is set, it takes precedence.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::error::LoamSpineError;

use super::{DiscoveryTransport, TransportResponse};

/// NeuralAPI transport for ecoBin-compliant HTTP via Songbird.
///
/// Sends JSON-RPC 2.0 requests to NeuralAPI over a Unix domain socket.
/// NeuralAPI routes the `http.request` capability to Songbird, which
/// handles TLS 1.3 and HTTP using pure Rust (RustCrypto + custom stack).
#[derive(Debug)]
pub struct NeuralApiTransport {
    socket_path: PathBuf,
    request_id: AtomicU64,
}

impl NeuralApiTransport {
    /// Create a transport that connects to the NeuralAPI socket.
    ///
    /// Discovers the socket path from environment or uses the provided path.
    ///
    /// # Errors
    ///
    /// Returns an error if no socket path can be resolved.
    pub fn new(socket_path: Option<PathBuf>) -> Result<Self, LoamSpineError> {
        let path = socket_path.or_else(Self::socket_from_env).ok_or_else(|| {
            LoamSpineError::Network(
                "NeuralAPI socket not found: set BIOMEOS_NEURAL_API_SOCKET or \
                     XDG_RUNTIME_DIR with a valid family_id"
                    .to_string(),
            )
        })?;

        Ok(Self {
            socket_path: path,
            request_id: AtomicU64::new(1),
        })
    }

    /// Resolve socket path from environment variables.
    fn socket_from_env() -> Option<PathBuf> {
        if let Ok(explicit) = std::env::var("BIOMEOS_NEURAL_API_SOCKET") {
            return Some(PathBuf::from(explicit));
        }

        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").ok()?;
        let family_id = std::env::var("BIOMEOS_FAMILY_ID").unwrap_or_else(|_| "default".into());
        Some(PathBuf::from(format!(
            "{runtime_dir}/biomeos/neural-api-{family_id}.sock"
        )))
    }

    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Send a JSON-RPC 2.0 request over the Unix socket and read the response.
    async fn jsonrpc_call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, LoamSpineError> {
        let id = self.next_id();
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id,
        });

        let request_bytes = serde_json::to_vec(&request).map_err(|e| {
            LoamSpineError::Network(format!("Failed to serialize JSON-RPC request: {e}"))
        })?;

        let mut stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            LoamSpineError::Network(format!(
                "NeuralAPI socket connection failed at {}: {e}",
                self.socket_path.display()
            ))
        })?;

        // Write length-prefixed message (4-byte big-endian length + payload)
        let len = u32::try_from(request_bytes.len())
            .map_err(|_| LoamSpineError::Network("JSON-RPC request too large".to_string()))?;
        stream.write_all(&len.to_be_bytes()).await.map_err(|e| {
            LoamSpineError::Network(format!("Failed to write to NeuralAPI socket: {e}"))
        })?;
        stream.write_all(&request_bytes).await.map_err(|e| {
            LoamSpineError::Network(format!("Failed to write to NeuralAPI socket: {e}"))
        })?;
        stream.flush().await.map_err(|e| {
            LoamSpineError::Network(format!("Failed to flush NeuralAPI socket: {e}"))
        })?;

        // Read length-prefixed response
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await.map_err(|e| {
            LoamSpineError::Network(format!("Failed to read NeuralAPI response length: {e}"))
        })?;
        let resp_len = usize::try_from(u32::from_be_bytes(len_buf)).map_err(|_| {
            LoamSpineError::Network("NeuralAPI response length exceeds platform capacity".into())
        })?;

        let mut resp_buf = vec![0u8; resp_len];
        stream.read_exact(&mut resp_buf).await.map_err(|e| {
            LoamSpineError::Network(format!("Failed to read NeuralAPI response: {e}"))
        })?;

        let response: serde_json::Value = serde_json::from_slice(&resp_buf).map_err(|e| {
            LoamSpineError::Network(format!("Failed to parse NeuralAPI JSON-RPC response: {e}"))
        })?;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown error");
            return Err(LoamSpineError::Network(format!(
                "NeuralAPI error: {message}"
            )));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| LoamSpineError::Network("NeuralAPI response missing 'result'".into()))
    }

    /// Convert a JSON-RPC `http.request` result into a `TransportResponse`.
    fn parse_http_result(result: &serde_json::Value) -> Result<TransportResponse, LoamSpineError> {
        let status = result
            .get("status")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| LoamSpineError::Network("Missing 'status' in HTTP response".into()))?;

        let body = if let Some(b64) = result.get("body").and_then(serde_json::Value::as_str) {
            use std::io::Read;
            let decoded = base64_decode(b64)?;
            let _ = decoded.bytes(); // validate utf8 isn't required
            decoded
        } else {
            Vec::new()
        };

        Ok(TransportResponse::new(
            u16::try_from(status).unwrap_or(500),
            body,
        ))
    }
}

/// Minimal base64 decoder (avoids pulling in a base64 crate).
///
/// NeuralAPI encodes HTTP response bodies as standard base64. This handles
/// the common case; production deployments should validate against the
/// NeuralAPI response specification.
fn base64_decode(input: &str) -> Result<Vec<u8>, LoamSpineError> {
    fn sextet(c: u8) -> Result<u32, LoamSpineError> {
        match c {
            b'A'..=b'Z' => Ok(u32::from(c - b'A')),
            b'a'..=b'z' => Ok(u32::from(c - b'a') + 26),
            b'0'..=b'9' => Ok(u32::from(c - b'0') + 52),
            b'+' => Ok(62),
            b'/' => Ok(63),
            _ => Err(LoamSpineError::Network(format!(
                "Invalid base64 character: {c}"
            ))),
        }
    }

    let input = input.as_bytes();
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;

    for &byte in input {
        if byte == b'=' || byte == b'\n' || byte == b'\r' {
            continue;
        }
        buf = (buf << 6) | sextet(byte)?;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            // bits ∈ [0,6] after subtraction ⇒ (buf >> bits) ∈ [0,255], fits u8.
            let byte = u8::try_from(buf >> bits).unwrap_or(0);
            out.push(byte);
            buf &= (1 << bits) - 1;
        }
    }

    Ok(out)
}

impl DiscoveryTransport for NeuralApiTransport {
    fn get<'a>(
        &'a self,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>> {
        Box::pin(async move {
            let result = self
                .jsonrpc_call(
                    "http.request",
                    serde_json::json!({
                        "method": "GET",
                        "url": url,
                    }),
                )
                .await?;
            Self::parse_http_result(&result)
        })
    }

    fn get_with_query<'a>(
        &'a self,
        url: &'a str,
        query: &'a [(&'a str, &'a str)],
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>> {
        Box::pin(async move {
            let mut full_url = url.to_string();
            if !query.is_empty() {
                full_url.push('?');
                for (i, (key, value)) in query.iter().enumerate() {
                    if i > 0 {
                        full_url.push('&');
                    }
                    full_url.push_str(&urlencoding_encode(key));
                    full_url.push('=');
                    full_url.push_str(&urlencoding_encode(value));
                }
            }
            let result = self
                .jsonrpc_call(
                    "http.request",
                    serde_json::json!({
                        "method": "GET",
                        "url": full_url,
                    }),
                )
                .await?;
            Self::parse_http_result(&result)
        })
    }

    fn post_json<'a>(
        &'a self,
        url: &'a str,
        body: &'a serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>> {
        Box::pin(async move {
            let result = self
                .jsonrpc_call(
                    "http.request",
                    serde_json::json!({
                        "method": "POST",
                        "url": url,
                        "headers": { "Content-Type": "application/json" },
                        "body": serde_json::to_string(body).map_err(|e| {
                            LoamSpineError::Serialization(format!("POST body serialization failed: {e}"))
                        })?,
                    }),
                )
                .await?;
            Self::parse_http_result(&result)
        })
    }
}

/// Minimal percent-encoding for query parameters (avoids extra crate).
fn urlencoding_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                out.push('%');
                out.push(char::from(b"0123456789ABCDEF"[(byte >> 4) as usize]));
                out.push(char::from(b"0123456789ABCDEF"[(byte & 0xF) as usize]));
            }
        }
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn neural_api_transport_requires_socket() {
        // Without any env vars pointing to a socket, creation should still succeed
        // if we provide an explicit path
        let transport = NeuralApiTransport::new(Some(PathBuf::from("/tmp/test.sock")));
        assert!(transport.is_ok());
    }

    #[test]
    fn neural_api_transport_debug() {
        let transport = NeuralApiTransport::new(Some(PathBuf::from("/tmp/test.sock"))).unwrap();
        let debug = format!("{transport:?}");
        assert!(debug.contains("NeuralApiTransport"));
        assert!(debug.contains("test.sock"));
    }

    #[test]
    fn base64_decode_roundtrip() {
        let encoded = "SGVsbG8gV29ybGQ=";
        let decoded = base64_decode(encoded).unwrap();
        assert_eq!(decoded, b"Hello World");
    }

    #[test]
    fn base64_decode_empty() {
        let decoded = base64_decode("").unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn base64_decode_no_padding() {
        let decoded = base64_decode("YQ").unwrap();
        assert_eq!(decoded, b"a");
    }

    #[test]
    fn urlencoding_basic() {
        assert_eq!(urlencoding_encode("hello"), "hello");
        assert_eq!(urlencoding_encode("hello world"), "hello%20world");
        assert_eq!(urlencoding_encode("a=b&c"), "a%3Db%26c");
    }

    #[test]
    fn urlencoding_safe_chars() {
        assert_eq!(urlencoding_encode("a-b_c.d~e"), "a-b_c.d~e");
    }

    #[test]
    fn parse_http_result_success() {
        let result = serde_json::json!({
            "status": 200,
            "body": "eyJrZXkiOiJ2YWx1ZSJ9", // {"key":"value"} in base64
        });
        let resp = NeuralApiTransport::parse_http_result(&result).unwrap();
        assert_eq!(resp.status, 200);
        assert!(resp.is_success());
        let parsed: serde_json::Value = resp.json().unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn parse_http_result_no_body() {
        let result = serde_json::json!({ "status": 204 });
        let resp = NeuralApiTransport::parse_http_result(&result).unwrap();
        assert_eq!(resp.status, 204);
        assert!(resp.body.is_empty());
    }

    #[test]
    fn parse_http_result_missing_status() {
        let result = serde_json::json!({ "body": "dGVzdA==" });
        let resp = NeuralApiTransport::parse_http_result(&result);
        assert!(resp.is_err());
    }

    #[tokio::test]
    async fn neural_api_get_fails_no_socket() {
        let transport =
            NeuralApiTransport::new(Some(PathBuf::from("/tmp/nonexistent-neural-api-test.sock")))
                .unwrap();
        let result = transport.get("http://registry:8082/health").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("NeuralAPI") || err.contains("socket"),
            "error should mention NeuralAPI: {err}"
        );
    }

    #[tokio::test]
    async fn neural_api_post_fails_no_socket() {
        let transport =
            NeuralApiTransport::new(Some(PathBuf::from("/tmp/nonexistent-neural-api-test.sock")))
                .unwrap();
        let body = serde_json::json!({"name": "test"});
        let result = transport
            .post_json("http://registry:8082/register", &body)
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn request_id_increments() {
        let transport = NeuralApiTransport::new(Some(PathBuf::from("/tmp/test.sock"))).unwrap();
        let id1 = transport.next_id();
        let id2 = transport.next_id();
        assert_eq!(id2, id1 + 1);
    }

    #[test]
    fn base64_decode_invalid_character() {
        let result = base64_decode("SGVsbG8!!!!");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("base64") || err.contains("Invalid"));
    }

    #[test]
    fn base64_decode_with_newlines() {
        let decoded = base64_decode("SGVs\nbG8=\r\n").unwrap();
        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn base64_decode_standard_vectors() {
        assert_eq!(base64_decode("").unwrap(), b"");
        assert_eq!(base64_decode("Zg==").unwrap(), b"f");
        assert_eq!(base64_decode("Zm8=").unwrap(), b"fo");
        assert_eq!(base64_decode("Zm9v").unwrap(), b"foo");
        assert_eq!(base64_decode("Zm9vYg==").unwrap(), b"foob");
        assert_eq!(base64_decode("Zm9vYmE=").unwrap(), b"fooba");
        assert_eq!(base64_decode("Zm9vYmFy").unwrap(), b"foobar");
    }

    #[test]
    fn parse_http_result_status_overflow() {
        let result = serde_json::json!({
            "status": 99999,
            "body": "dGVzdA==",
        });
        let resp = NeuralApiTransport::parse_http_result(&result).unwrap();
        assert_eq!(
            resp.status, 500,
            "overflowing status should fallback to 500"
        );
    }

    #[test]
    fn parse_http_result_invalid_base64_body() {
        let result = serde_json::json!({
            "status": 200,
            "body": "not!!valid!!base64",
        });
        let resp = NeuralApiTransport::parse_http_result(&result);
        assert!(resp.is_err(), "invalid base64 should produce an error");
    }

    #[test]
    fn neural_api_transport_from_env_fallback() {
        let transport = NeuralApiTransport::new(None);
        // Without any env vars this may or may not succeed depending on XDG_RUNTIME_DIR
        // The important thing is it doesn't panic
        let _ = transport;
    }

    /// Helper: spawn a mock NeuralAPI socket that responds to `http.request`
    fn spawn_mock_transport_server(
        socket_path: &std::path::Path,
        response: &serde_json::Value,
    ) -> tokio::task::JoinHandle<()> {
        let listener = tokio::net::UnixListener::bind(socket_path).unwrap();
        let resp_bytes = serde_json::to_vec(&response).unwrap();

        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut len_buf = [0u8; 4];
                let _ = stream.read_exact(&mut len_buf).await;
                let req_len = u32::from_be_bytes(len_buf) as usize;
                let mut req_buf = vec![0u8; req_len];
                let _ = stream.read_exact(&mut req_buf).await;

                let len = u32::try_from(resp_bytes.len())
                    .unwrap_or(u32::MAX)
                    .to_be_bytes();
                let _ = stream.write_all(&len).await;
                let _ = stream.write_all(&resp_bytes).await;
                let _ = stream.flush().await;
            }
        })
    }

    #[tokio::test]
    async fn jsonrpc_call_success_via_mock_socket() {
        let tmp = tempfile::tempdir().unwrap();
        let sock = tmp.path().join("neural-transport-test.sock");

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "status": 200,
                "body": "eyJvayI6dHJ1ZX0="  // {"ok":true}
            },
            "id": 1
        });
        let handle = spawn_mock_transport_server(&sock, &response);

        let transport = NeuralApiTransport::new(Some(sock)).unwrap();
        let result = transport.get("http://registry:8082/health").await;

        assert!(result.is_ok(), "GET should succeed: {:?}", result.err());
        let resp = result.unwrap();
        assert_eq!(resp.status, 200);
        assert!(!resp.body.is_empty());

        handle.abort();
    }

    #[tokio::test]
    async fn jsonrpc_call_returns_error_on_rpc_error() {
        let tmp = tempfile::tempdir().unwrap();
        let sock = tmp.path().join("neural-transport-err.sock");

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": -32601, "message": "method not found" },
            "id": 1
        });
        let handle = spawn_mock_transport_server(&sock, &response);

        let transport = NeuralApiTransport::new(Some(sock)).unwrap();
        let result = transport.get("http://registry:8082/health").await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("method not found"), "error: {err}");

        handle.abort();
    }

    #[tokio::test]
    async fn get_with_query_builds_url_and_calls() {
        let tmp = tempfile::tempdir().unwrap();
        let sock = tmp.path().join("neural-query-test.sock");

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "status": 200,
                "body": "e30="  // {}
            },
            "id": 1
        });
        let handle = spawn_mock_transport_server(&sock, &response);

        let transport = NeuralApiTransport::new(Some(sock)).unwrap();
        let result = transport
            .get_with_query(
                "http://registry:8082/discover",
                &[("capability", "signing"), ("format", "json")],
            )
            .await;

        assert!(
            result.is_ok(),
            "get_with_query should succeed: {:?}",
            result.err()
        );
        let resp = result.unwrap();
        assert_eq!(resp.status, 200);

        handle.abort();
    }

    #[tokio::test]
    async fn post_json_via_mock_socket() {
        let tmp = tempfile::tempdir().unwrap();
        let sock = tmp.path().join("neural-post-test.sock");

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "status": 201,
                "body": "eyJjcmVhdGVkIjp0cnVlfQ=="  // {"created":true}
            },
            "id": 1
        });
        let handle = spawn_mock_transport_server(&sock, &response);

        let transport = NeuralApiTransport::new(Some(sock)).unwrap();
        let body = serde_json::json!({"name": "test-service"});
        let result = transport
            .post_json("http://registry:8082/register", &body)
            .await;

        assert!(result.is_ok(), "POST should succeed: {:?}", result.err());
        let resp = result.unwrap();
        assert_eq!(resp.status, 201);

        handle.abort();
    }

    #[tokio::test]
    async fn jsonrpc_call_missing_result_field() {
        let tmp = tempfile::tempdir().unwrap();
        let sock = tmp.path().join("neural-noresult.sock");

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1
        });
        let handle = spawn_mock_transport_server(&sock, &response);

        let transport = NeuralApiTransport::new(Some(sock)).unwrap();
        let result = transport.get("http://registry:8082/health").await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("missing") || err.contains("result"),
            "error: {err}"
        );

        handle.abort();
    }
}
