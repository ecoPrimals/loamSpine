// SPDX-License-Identifier: AGPL-3.0-or-later

//! HTTP transport abstraction for service discovery.
//!
//! Decouples [`DiscoveryClient`](crate::discovery_client::DiscoveryClient) from
//! any specific HTTP library, enabling both the `ureq` path and the
//! ecoBin-compliant **Tower Atomic** path (Songbird via NeuralAPI).
//!
//! ## Transport Hierarchy
//!
//! | Transport | Feature | C deps? | Use case |
//! |---|---|---|---|
//! | `NeuralApiTransport` | `tower-atomic` | **None** | Production (ecoBin) |
//! | `HttpTransport` | `discovery-http` | **None** | Pure Rust HTTP fallback |
//!
//! ## Example
//!
//! ```rust,no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use loam_spine_core::transport::TransportResponse;
//!
//! let response = TransportResponse::new(200, b"ok".to_vec());
//! assert!(response.is_success());
//! # Ok(())
//! # }
//! ```

use std::future::Future;
use std::pin::Pin;

use bytes::Bytes;
use serde::de::DeserializeOwned;

use crate::error::LoamSpineError;

// ──────────────────────────────────────────────────────────────────────────────
// Transport trait
// ──────────────────────────────────────────────────────────────────────────────

/// Object-safe HTTP transport for service discovery.
///
/// Every method returns a boxed future so the trait is usable through
/// `Arc<dyn DiscoveryTransport>` (required by [`DiscoveryClient`]).
///
/// [`DiscoveryClient`]: crate::discovery_client::DiscoveryClient
pub trait DiscoveryTransport: Send + Sync + std::fmt::Debug {
    /// `GET url` — returns status and response body.
    fn get<'a>(
        &'a self,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>>;

    /// `GET url?key=val&…` — returns status and response body.
    fn get_with_query<'a>(
        &'a self,
        url: &'a str,
        query: &'a [(&'a str, &'a str)],
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>>;

    /// `POST url` with a JSON body — returns status and response body.
    fn post_json<'a>(
        &'a self,
        url: &'a str,
        body: &'a serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>>;
}

// ──────────────────────────────────────────────────────────────────────────────
// TransportResponse
// ──────────────────────────────────────────────────────────────────────────────

/// Simplified HTTP response returned by every [`DiscoveryTransport`].
///
/// Uses [`Bytes`] for the response body so clones are O(1) reference-count
/// bumps and downstream consumers can slice without copying.
#[derive(Debug, Clone)]
pub struct TransportResponse {
    /// HTTP status code.
    pub status: u16,
    /// Raw response body (zero-copy via `Bytes`).
    pub body: Bytes,
}

impl TransportResponse {
    /// Construct a response from a status and owned byte vector.
    ///
    /// Converts the `Vec<u8>` into `Bytes` without copying.
    #[must_use]
    pub fn new(status: u16, body: Vec<u8>) -> Self {
        Self {
            status,
            body: Bytes::from(body),
        }
    }

    /// Construct from a status and pre-existing `Bytes` (true zero-copy).
    #[must_use]
    pub const fn from_bytes(status: u16, body: Bytes) -> Self {
        Self { status, body }
    }

    /// Construct from a static byte slice (compile-time zero-copy).
    #[must_use]
    #[expect(
        clippy::missing_const_for_fn,
        reason = "Bytes::from_static is not const-stable"
    )]
    pub fn from_static(status: u16, body: &'static [u8]) -> Self {
        Self {
            status,
            body: Bytes::from_static(body),
        }
    }

    /// Whether the status code is in the 2xx range.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Deserialise the body as JSON.
    ///
    /// # Errors
    ///
    /// Returns a network error if deserialization fails.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, LoamSpineError> {
        serde_json::from_slice(&self.body).map_err(|e| {
            LoamSpineError::ipc(
                crate::error::IpcErrorPhase::InvalidJson,
                format!("Failed to parse response JSON: {e}"),
            )
        })
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// ureq-backed transport (feature = "discovery-http")
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "discovery-http")]
pub mod http;

#[cfg(feature = "discovery-http")]
pub use http::HttpTransport;

// ──────────────────────────────────────────────────────────────────────────────
// Tower Atomic / NeuralAPI transport (feature = "tower-atomic")
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "tower-atomic")]
pub mod neural_api;

#[cfg(feature = "tower-atomic")]
pub use neural_api::NeuralApiTransport;

// ──────────────────────────────────────────────────────────────────────────────
// Mock transport for testing (always available so unit tests compile
// without either feature)
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(any(test, feature = "testing"))]
pub mod mock;

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests {
    use super::*;

    #[test]
    fn transport_response_success_range() {
        for code in 200..300_u16 {
            let r = TransportResponse::new(code, Vec::new());
            assert!(r.is_success(), "status {code} should be success");
        }
        for code in [100, 301, 400, 404, 500] {
            let r = TransportResponse::new(code, Vec::new());
            assert!(!r.is_success(), "status {code} should not be success");
        }
    }

    #[test]
    fn transport_response_json_parse() {
        let body = serde_json::to_vec(&serde_json::json!({"key": "value"})).unwrap();
        let r = TransportResponse::new(200, body);
        let parsed: serde_json::Value = r.json().unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn transport_response_json_parse_error() {
        let r = TransportResponse::new(200, b"not json".to_vec());
        let result: Result<serde_json::Value, _> = r.json();
        assert!(result.is_err());
    }

    #[test]
    fn transport_response_clone() {
        let r = TransportResponse::new(200, b"test".to_vec());
        let cloned = r.clone();
        assert_eq!(cloned.status, r.status);
        assert_eq!(cloned.body, r.body);
    }

    #[test]
    fn transport_response_debug() {
        let r = TransportResponse::new(200, Vec::new());
        let debug = format!("{r:?}");
        assert!(debug.contains("200"));
    }

    #[test]
    fn transport_response_from_bytes_zero_copy() {
        let body = bytes::Bytes::from_static(b"zero-copy payload");
        let r = TransportResponse::from_bytes(200, body);
        assert_eq!(r.status, 200);
        assert_eq!(&r.body[..], b"zero-copy payload");
    }
}
