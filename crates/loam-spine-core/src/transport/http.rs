// SPDX-License-Identifier: AGPL-3.0-only

//! Direct HTTP transport backed by `reqwest`.
//!
//! **Feature-gated** behind `discovery-http`. This transport pulls `ring`
//! (C/asm) via `reqwest → hyper-rustls → rustls → ring` and is therefore
//! **not ecoBin compliant**. Use only for development or when the Tower Atomic
//! stack is unavailable.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use crate::error::LoamSpineError;

use super::{DiscoveryTransport, TransportResponse};

/// Direct HTTP transport using `reqwest`.
///
/// Wraps a [`reqwest::Client`] behind the [`DiscoveryTransport`] trait so that
/// [`DiscoveryClient`](crate::discovery_client::DiscoveryClient) can use it
/// interchangeably with the Tower Atomic transport.
#[derive(Clone, Debug)]
pub struct HttpTransport {
    client: reqwest::Client,
}

impl HttpTransport {
    /// Create a new HTTP transport with default settings (30 s timeout).
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client cannot be constructed.
    pub fn new() -> Result<Self, LoamSpineError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| LoamSpineError::Network(format!("Failed to create HTTP client: {e}")))?;
        Ok(Self { client })
    }
}

impl DiscoveryTransport for HttpTransport {
    fn get<'a>(
        &'a self,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>> {
        Box::pin(async move {
            let resp = self
                .client
                .get(url)
                .send()
                .await
                .map_err(|e| LoamSpineError::Network(format!("GET {url} failed: {e}")))?;
            let status = resp.status().as_u16();
            let body = resp
                .bytes()
                .await
                .map_err(|e| LoamSpineError::Network(format!("reading response body: {e}")))?
                .to_vec();
            Ok(TransportResponse { status, body })
        })
    }

    fn get_with_query<'a>(
        &'a self,
        url: &'a str,
        query: &'a [(&'a str, &'a str)],
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>> {
        Box::pin(async move {
            let resp = self
                .client
                .get(url)
                .query(query)
                .send()
                .await
                .map_err(|e| LoamSpineError::Network(format!("GET {url} failed: {e}")))?;
            let status = resp.status().as_u16();
            let body = resp
                .bytes()
                .await
                .map_err(|e| LoamSpineError::Network(format!("reading response body: {e}")))?
                .to_vec();
            Ok(TransportResponse { status, body })
        })
    }

    fn post_json<'a>(
        &'a self,
        url: &'a str,
        body: &'a serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<TransportResponse, LoamSpineError>> + Send + 'a>> {
        Box::pin(async move {
            let resp = self
                .client
                .post(url)
                .json(body)
                .send()
                .await
                .map_err(|e| LoamSpineError::Network(format!("POST {url} failed: {e}")))?;
            let status = resp.status().as_u16();
            let body_bytes = resp
                .bytes()
                .await
                .map_err(|e| LoamSpineError::Network(format!("reading response body: {e}")))?
                .to_vec();
            Ok(TransportResponse {
                status,
                body: body_bytes,
            })
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
        let transport = HttpTransport::new().unwrap();
        let _cloned = transport.clone();
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
