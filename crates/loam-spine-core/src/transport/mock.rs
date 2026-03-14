// SPDX-License-Identifier: AGPL-3.0-only

//! Mock transport for unit testing.
//!
//! Always returns a network error. Used by `DiscoveryClient::for_testing` so
//! that tests compile and run without pulling in `ureq` or a live socket.

use std::future::Future;
use std::pin::Pin;

use crate::error::LoamSpineError;

use super::{DiscoveryTransport, TransportResponse};

/// Transport that always fails with a descriptive network error.
///
/// Enables testing of [`DiscoveryClient`](crate::discovery_client::DiscoveryClient)
/// error paths without a real backend or live socket.
///
/// Re-exported from the crate root when the `testing` feature is enabled.
#[derive(Clone, Debug)]
pub struct MockTransport {
    endpoint_hint: String,
}

impl MockTransport {
    /// Create a new mock transport that reports the given endpoint in errors.
    #[must_use]
    pub fn new(endpoint_hint: impl Into<String>) -> Self {
        Self {
            endpoint_hint: endpoint_hint.into(),
        }
    }
}

impl DiscoveryTransport for MockTransport {
    fn get<'a>(
        &'a self,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>> {
        let hint = self.endpoint_hint.clone();
        Box::pin(async move {
            Err(LoamSpineError::Network(format!(
                "MockTransport: GET {url} (endpoint: {hint})"
            )))
        })
    }

    fn get_with_query<'a>(
        &'a self,
        url: &'a str,
        _query: &'a [(&'a str, &'a str)],
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>> {
        let hint = self.endpoint_hint.clone();
        Box::pin(async move {
            Err(LoamSpineError::Network(format!(
                "MockTransport: GET {url} (endpoint: {hint})"
            )))
        })
    }

    fn post_json<'a>(
        &'a self,
        url: &'a str,
        _body: &'a serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>> {
        let hint = self.endpoint_hint.clone();
        Box::pin(async move {
            Err(LoamSpineError::Network(format!(
                "MockTransport: POST {url} (endpoint: {hint})"
            )))
        })
    }
}

/// Transport that always succeeds with 200 OK.
///
/// Used to test success paths (advertise, heartbeat, deregister) on
/// [`DiscoveryClient`](crate::discovery_client::DiscoveryClient)
/// without a real discovery server.
#[cfg(any(test, feature = "testing"))]
#[derive(Clone, Debug)]
pub struct SuccessTransport;

#[cfg(any(test, feature = "testing"))]
impl SuccessTransport {
    /// Create a new success transport.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[cfg(any(test, feature = "testing"))]
impl Default for SuccessTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(any(test, feature = "testing"))]
impl DiscoveryTransport for SuccessTransport {
    fn get<'a>(
        &'a self,
        _url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>> {
        Box::pin(async move { Ok(TransportResponse::from_static(200, b"{}")) })
    }

    fn get_with_query<'a>(
        &'a self,
        _url: &'a str,
        _query: &'a [(&'a str, &'a str)],
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>> {
        Box::pin(async move { Ok(TransportResponse::from_static(200, b"[]")) })
    }

    fn post_json<'a>(
        &'a self,
        _url: &'a str,
        _body: &'a serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>> {
        Box::pin(async move { Ok(TransportResponse::from_static(200, b"{}")) })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn mock_transport_debug() {
        let t = MockTransport::new("http://test:8082");
        let debug = format!("{t:?}");
        assert!(debug.contains("MockTransport"));
        assert!(debug.contains("test:8082"));
    }

    #[test]
    fn mock_transport_clone() {
        let t = MockTransport::new("http://test:8082");
        let cloned = t.clone();
        assert_eq!(t.endpoint_hint, cloned.endpoint_hint);
    }

    #[tokio::test]
    async fn mock_transport_get_fails() {
        let t = MockTransport::new("http://test:8082");
        let result = t.get("http://test:8082/health").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("MockTransport"));
    }

    #[tokio::test]
    async fn mock_transport_post_fails() {
        let t = MockTransport::new("http://test:8082");
        let body = serde_json::json!({"test": true});
        let result = t.post_json("http://test:8082/register", &body).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn mock_transport_get_with_query_fails() {
        let t = MockTransport::new("http://test:8082");
        let result = t
            .get_with_query("http://test:8082/discover", &[("capability", "signing")])
            .await;
        assert!(result.is_err());
    }
}
