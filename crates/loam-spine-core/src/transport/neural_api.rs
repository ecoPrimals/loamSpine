// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tower Atomic transport via NeuralAPI HTTP capability routing.
//!
//! This transport delegates all HTTP operations to a capability-discovered HTTP provider
//! through the **NeuralAPI** orchestration layer,
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

use crate::error::{IpcErrorPhase, LoamSpineError};

use super::{DiscoveryTransport, TransportResponse};

/// NeuralAPI transport for ecoBin-compliant HTTP via capability-discovered provider.
///
/// Sends JSON-RPC 2.0 requests to NeuralAPI via `TransportStream`.
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
        crate::neural_api::resolve_neural_api_socket_with(
            crate::constants::env_resolution::biomeos_neural_api_socket().as_deref(),
            crate::constants::env_resolution::xdg_runtime_dir().as_deref(),
            crate::constants::env_resolution::biomeos_family_id().as_deref(),
        )
    }

    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Send a JSON-RPC 2.0 request via `TransportStream` and read the response.
    ///
    /// Platform dispatch is handled by `connect_transport` — no `#[cfg]` needed.
    async fn jsonrpc_call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, LoamSpineError> {
        let endpoint = super::endpoint_from_path(&self.socket_path);
        let mut stream = super::connect_transport(&endpoint).await?;
        super::length_prefixed_rpc_call(&mut stream, method, params, self.next_id(), "NeuralAPI")
            .await
    }

    /// Convert a JSON-RPC `http.request` result into a `TransportResponse`.
    fn parse_http_result(result: &serde_json::Value) -> Result<TransportResponse, LoamSpineError> {
        use base64::Engine as _;

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
            base64::engine::general_purpose::STANDARD
                .decode(b64)
                .map_err(|e| {
                    LoamSpineError::ipc(
                        IpcErrorPhase::InvalidJson,
                        format!("base64 decode failed: {e}"),
                    )
                })?
        } else {
            Vec::new()
        };

        Ok(TransportResponse::new(
            u16::try_from(status).unwrap_or(500),
            body,
        ))
    }
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
