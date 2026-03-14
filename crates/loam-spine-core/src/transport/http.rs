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

#[cfg(all(test, feature = "discovery-http"))]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

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
    async fn http_transport_get_invalid_url() {
        let transport = HttpTransport::new().unwrap();
        let result = transport.get("not-a-valid-url").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn http_transport_get_with_query_unreachable() {
        let transport = HttpTransport::new().unwrap();
        let result = transport
            .get_with_query(
                "http://127.0.0.1:1/health",
                &[("foo", "bar"), ("baz", "qux")],
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn http_transport_get_with_query_empty_params() {
        let transport = HttpTransport::new().unwrap();
        let result = transport.get_with_query("http://127.0.0.1:1/", &[]).await;
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

    #[tokio::test]
    async fn http_transport_post_invalid_url() {
        let transport = HttpTransport::new().unwrap();
        let body = serde_json::json!({});
        let result = transport.post_json("not-a-url", &body).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn http_transport_post_empty_json() {
        let transport = HttpTransport::new().unwrap();
        let body = serde_json::json!({});
        let result = transport.post_json("http://127.0.0.1:1/", &body).await;
        assert!(result.is_err());
    }

    /// Spawns a minimal HTTP server that responds with the given status and body.
    fn spawn_mini_server(status: u16, body: &'static str) -> (u16, thread::JoinHandle<()>) {
        let reason = match status {
            200 => "OK",
            201 => "Created",
            404 => "Not Found",
            _ => "OK",
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let response = format!(
                "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}",
                status,
                reason,
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        });
        (port, handle)
    }

    #[tokio::test]
    async fn http_transport_get_success_transport_response() {
        let (port, _handle) = spawn_mini_server(200, "ok");
        std::thread::sleep(std::time::Duration::from_millis(50));

        let transport = HttpTransport::new().unwrap();
        let url = format!("http://127.0.0.1:{port}/");
        let result = transport.get(&url).await.unwrap();

        assert_eq!(result.status, 200);
        assert_eq!(result.body.as_ref(), b"ok");
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn http_transport_get_with_query_success() {
        let (port, _handle) = spawn_mini_server(200, r#"{"query":"received"}"#);
        std::thread::sleep(std::time::Duration::from_millis(50));

        let transport = HttpTransport::new().unwrap();
        let url = format!("http://127.0.0.1:{port}/search");
        let result = transport
            .get_with_query(&url, &[("q", "test"), ("limit", "10")])
            .await
            .unwrap();

        assert_eq!(result.status, 200);
        let parsed: serde_json::Value = result.json().unwrap();
        assert_eq!(parsed["query"], "received");
    }

    #[tokio::test]
    async fn http_transport_post_json_success() {
        let (port, _handle) = spawn_mini_server(201, r#"{"id":"created"}"#);
        std::thread::sleep(std::time::Duration::from_millis(50));

        let transport = HttpTransport::new().unwrap();
        let url = format!("http://127.0.0.1:{port}/register");
        let body = serde_json::json!({"name": "test-service"});
        let result = transport.post_json(&url, &body).await.unwrap();

        assert_eq!(result.status, 201);
        let parsed: serde_json::Value = result.json().unwrap();
        assert_eq!(parsed["id"], "created");
    }

    #[tokio::test]
    async fn http_transport_get_non_success_status() {
        let (port, _handle) = spawn_mini_server(404, "Not Found");
        std::thread::sleep(std::time::Duration::from_millis(50));

        let transport = HttpTransport::new().unwrap();
        let url = format!("http://127.0.0.1:{port}/");
        let result = transport.get(&url).await;

        assert!(
            result.is_err(),
            "non-2xx HTTP status should produce a Network error"
        );
        let err = result.unwrap_err().to_string();
        assert!(err.contains("404") || err.contains("Not Found") || err.contains("failed"));
    }
}
