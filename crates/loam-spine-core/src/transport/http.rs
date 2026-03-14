// SPDX-License-Identifier: AGPL-3.0-only

//! Direct HTTP transport backed by `ureq` (pure Rust, no TLS, no ring).
//!
//! **Feature-gated** behind `discovery-http`. This transport is fully ecoBin
//! compliant — zero C dependencies. For HTTPS, route through the
//! BearDog/Songbird TLS stack via Tower Atomic.

use std::future::Future;
use std::io::Read;
use std::pin::Pin;

use crate::error::LoamSpineError;

use super::{DiscoveryTransport, TransportResponse};

/// Direct HTTP transport using `ureq` (pure Rust, synchronous under the hood).
///
/// Wraps [`ureq::Agent`] behind the [`DiscoveryTransport`] trait so that
/// [`DiscoveryClient`](crate::discovery_client::DiscoveryClient) can use it
/// interchangeably with the Tower Atomic transport.
///
/// Since `ureq` is blocking, calls are dispatched to `tokio::task::spawn_blocking`.
#[derive(Clone, Debug)]
pub struct HttpTransport {
    agent: ureq::Agent,
}

impl HttpTransport {
    /// Create a new HTTP transport with default settings (30 s timeout).
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client cannot be constructed.
    pub fn new() -> Result<Self, LoamSpineError> {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(5))
            .timeout_read(std::time::Duration::from_secs(30))
            .build();
        Ok(Self { agent })
    }
}

impl DiscoveryTransport for HttpTransport {
    fn get<'a>(
        &'a self,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>> {
        let url = url.to_string();
        let agent = self.agent.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let resp = agent
                    .get(&url)
                    .call()
                    .map_err(|e| LoamSpineError::Network(format!("GET {url} failed: {e}")))?;
                let status = resp.status();
                let mut body = Vec::new();
                resp.into_reader()
                    .read_to_end(&mut body)
                    .map_err(|e| LoamSpineError::Network(format!("reading response body: {e}")))?;
                Ok(TransportResponse::new(status, body))
            })
            .await
            .map_err(|e| LoamSpineError::Network(format!("spawn_blocking: {e}")))?
        })
    }

    fn get_with_query<'a>(
        &'a self,
        url: &'a str,
        query: &'a [(&'a str, &'a str)],
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>> {
        let url = url.to_string();
        let query_owned: Vec<(String, String)> = query
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let agent = self.agent.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let mut req = agent.get(&url);
                for (k, v) in &query_owned {
                    req = req.query(k, v);
                }
                let resp = req
                    .call()
                    .map_err(|e| LoamSpineError::Network(format!("GET {url} failed: {e}")))?;
                let status = resp.status();
                let mut body = Vec::new();
                resp.into_reader()
                    .read_to_end(&mut body)
                    .map_err(|e| LoamSpineError::Network(format!("reading response body: {e}")))?;
                Ok(TransportResponse::new(status, body))
            })
            .await
            .map_err(|e| LoamSpineError::Network(format!("spawn_blocking: {e}")))?
        })
    }

    fn post_json<'a>(
        &'a self,
        url: &'a str,
        body: &'a serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>> {
        let url = url.to_string();
        let body_str = body.to_string();
        let agent = self.agent.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let resp = agent
                    .post(&url)
                    .set("Content-Type", "application/json")
                    .send_string(&body_str)
                    .map_err(|e| LoamSpineError::Network(format!("POST {url} failed: {e}")))?;
                let status = resp.status();
                let mut body = Vec::new();
                resp.into_reader()
                    .read_to_end(&mut body)
                    .map_err(|e| LoamSpineError::Network(format!("reading response body: {e}")))?;
                Ok(TransportResponse::new(status, body))
            })
            .await
            .map_err(|e| LoamSpineError::Network(format!("spawn_blocking: {e}")))?
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn http_transport_creation() {
        let transport = HttpTransport::new();
        assert!(transport.is_ok());
    }

    #[test]
    fn http_transport_debug() {
        let transport = HttpTransport::new().unwrap();
        let debug = format!("{transport:?}");
        assert!(debug.contains("HttpTransport"));
    }

    #[test]
    fn http_transport_clone() {
        let original = HttpTransport::new().unwrap();
        #[allow(clippy::redundant_clone)]
        let _cloned = original.clone();
    }

    #[tokio::test]
    async fn http_transport_get_unreachable() {
        let transport = HttpTransport::new().unwrap();
        let result = transport.get("http://127.0.0.1:1/health").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn http_transport_post_unreachable() {
        let transport = HttpTransport::new().unwrap();
        let body = serde_json::json!({"name": "test"});
        let result = transport
            .post_json("http://127.0.0.1:1/register", &body)
            .await;
        assert!(result.is_err());
    }
}
