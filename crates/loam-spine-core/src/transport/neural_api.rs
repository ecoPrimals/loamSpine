// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tower Atomic transport via NeuralAPI HTTP capability routing.
//!
//! This transport delegates all HTTP operations to a capability-discovered HTTP provider
//! through BiomeOS's **NeuralAPI** orchestration layer,
//! communicating over a Unix domain socket with JSON-RPC 2.0.
//!
//! **Zero C dependencies** — this is the ecoBin-compliant production path.
//!
//! ## Architecture
//!
//! ```text
//! LoamSpine ──JSON-RPC──▶ NeuralAPI ──route──▶ HTTP provider ──HTTP/TLS──▶ Registry
//!            (Unix sock)              (Unix sock)         (TLS + HTTP)
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

use crate::error::{IpcErrorPhase, LoamSpineError};

use super::{DiscoveryTransport, TransportResponse};

/// NeuralAPI transport for ecoBin-compliant HTTP via capability-discovered provider.
///
/// Sends JSON-RPC 2.0 requests to NeuralAPI over a Unix domain socket.
/// NeuralAPI routes the `http.request` capability to a provider that
/// handles TLS 1.3 and HTTP.
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
            LoamSpineError::ipc(
                IpcErrorPhase::Connect,
                "NeuralAPI socket not found: set BIOMEOS_NEURAL_API_SOCKET or \
                     XDG_RUNTIME_DIR with a valid family_id",
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
        let dir = crate::primal_names::BIOMEOS_SOCKET_DIR;
        Some(PathBuf::from(format!(
            "{runtime_dir}/{dir}/neural-api-{family_id}.sock"
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
            LoamSpineError::ipc(
                IpcErrorPhase::Serialization,
                format!("Failed to serialize JSON-RPC request: {e}"),
            )
        })?;

        let mut stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::Connect,
                format!(
                    "NeuralAPI socket connection failed at {}: {e}",
                    self.socket_path.display()
                ),
            )
        })?;

        let len = u32::try_from(request_bytes.len())
            .map_err(|_| LoamSpineError::ipc(IpcErrorPhase::Write, "JSON-RPC request too large"))?;
        stream.write_all(&len.to_be_bytes()).await.map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::Write,
                format!("Failed to write to NeuralAPI socket: {e}"),
            )
        })?;
        stream.write_all(&request_bytes).await.map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::Write,
                format!("Failed to write to NeuralAPI socket: {e}"),
            )
        })?;
        stream.flush().await.map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::Write,
                format!("Failed to flush NeuralAPI socket: {e}"),
            )
        })?;

        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await.map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::Read,
                format!("Failed to read NeuralAPI response length: {e}"),
            )
        })?;
        let resp_len = usize::try_from(u32::from_be_bytes(len_buf)).map_err(|_| {
            LoamSpineError::ipc(
                IpcErrorPhase::Read,
                "NeuralAPI response length exceeds platform capacity",
            )
        })?;

        let mut resp_buf = vec![0u8; resp_len];
        stream.read_exact(&mut resp_buf).await.map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::Read,
                format!("Failed to read NeuralAPI response: {e}"),
            )
        })?;

        let response: serde_json::Value = serde_json::from_slice(&resp_buf).map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::InvalidJson,
                format!("Failed to parse NeuralAPI JSON-RPC response: {e}"),
            )
        })?;

        if let Some((code, message)) = crate::error::extract_rpc_error(&response) {
            return Err(LoamSpineError::ipc(
                IpcErrorPhase::JsonRpcError(code),
                format!("NeuralAPI error: {message}"),
            ));
        }

        response.get("result").cloned().ok_or_else(|| {
            LoamSpineError::ipc(
                IpcErrorPhase::NoResult,
                "NeuralAPI response missing 'result'",
            )
        })
    }

    /// Convert a JSON-RPC `http.request` result into a `TransportResponse`.
    fn parse_http_result(result: &serde_json::Value) -> Result<TransportResponse, LoamSpineError> {
        let status = result
            .get("status")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                LoamSpineError::ipc(
                    IpcErrorPhase::InvalidJson,
                    "Missing 'status' in HTTP response",
                )
            })?;

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
            _ => Err(LoamSpineError::ipc(
                IpcErrorPhase::InvalidJson,
                format!("Invalid base64 character: {c}"),
            )),
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
                            LoamSpineError::ipc(IpcErrorPhase::Serialization, format!("POST body serialization failed: {e}"))
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
                out.push(char::from(byte));
            }
            _ => {
                out.push('%');
                out.push(char::from(b"0123456789ABCDEF"[usize::from(byte >> 4)]));
                out.push(char::from(b"0123456789ABCDEF"[usize::from(byte & 0xF)]));
            }
        }
    }
    out
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests use unwrap/expect for conciseness"
)]
#[path = "neural_api_tests.rs"]
mod tests;
