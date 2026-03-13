// SPDX-License-Identifier: AGPL-3.0-only

//! HTTP transport abstraction for service discovery.
//!
//! Decouples [`DiscoveryClient`](crate::discovery_client::DiscoveryClient) from
//! any specific HTTP library, enabling both the legacy `reqwest` path and the
//! ecoBin-compliant **Tower Atomic** path (Songbird via NeuralAPI).
//!
//! ## Transport Hierarchy
//!
//! | Transport | Feature | C deps? | Use case |
//! |---|---|---|---|
//! | `NeuralApiTransport` | `tower-atomic` | **None** | Production (ecoBin) |
//! | `HttpTransport` | `discovery-http` | `ring` | Development / fallback |
//!
//! ## Example
//!
//! ```rust,no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use loam_spine_core::transport::TransportResponse;
//!
//! let response = TransportResponse {
//!     status: 200,
//!     body: b"ok".to_vec(),
//! };
//! assert!(response.is_success());
//! # Ok(())
//! # }
//! ```

use std::future::Future;
use std::pin::Pin;

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
#[derive(Debug, Clone)]
pub struct TransportResponse {
    /// HTTP status code.
    pub status: u16,
    /// Raw response body.
    pub body: Vec<u8>,
}

impl TransportResponse {
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
        serde_json::from_slice(&self.body)
            .map_err(|e| LoamSpineError::Network(format!("Failed to parse response JSON: {e}")))
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// reqwest-backed transport (feature = "discovery-http")
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
pub(crate) mod mock;

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn transport_response_success_range() {
        for code in 200..300_u16 {
            let r = TransportResponse {
                status: code,
                body: Vec::new(),
            };
            assert!(r.is_success(), "status {code} should be success");
        }
        for code in [100, 301, 400, 404, 500] {
            let r = TransportResponse {
                status: code,
                body: Vec::new(),
            };
            assert!(!r.is_success(), "status {code} should not be success");
        }
    }

    #[test]
    fn transport_response_json_parse() {
        let body = serde_json::to_vec(&serde_json::json!({"key": "value"})).unwrap();
        let r = TransportResponse { status: 200, body };
        let parsed: serde_json::Value = r.json().unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn transport_response_json_parse_error() {
        let r = TransportResponse {
            status: 200,
            body: b"not json".to_vec(),
        };
        let result: Result<serde_json::Value, _> = r.json();
        assert!(result.is_err());
    }

    #[test]
    fn transport_response_clone() {
        let r = TransportResponse {
            status: 200,
            body: b"test".to_vec(),
        };
        let cloned = r.clone();
        assert_eq!(cloned.status, r.status);
        assert_eq!(cloned.body, r.body);
    }

    #[test]
    fn transport_response_debug() {
        let r = TransportResponse {
            status: 200,
            body: Vec::new(),
        };
        let debug = format!("{r:?}");
        assert!(debug.contains("200"));
    }
}
