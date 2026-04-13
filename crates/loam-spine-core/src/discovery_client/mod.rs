// SPDX-License-Identifier: AGPL-3.0-or-later

//! Service registry client for universal service discovery.
//!
//! This module provides integration with any RFC 2782 compliant service registry
//! (the "universal adapter") for discovering other primals' capabilities at runtime
//! without hardcoding.
//!
//! ## Transport Layer
//!
//! `DiscoveryClient` is transport-agnostic. The backing HTTP implementation is
//! selected at construction time via [`DiscoveryTransport`]:
//!
//! | Constructor | Transport | Feature | ecoBin? |
//! |---|---|---|---|
//! | [`connect`] | Tower Atomic (NeuralAPI → HTTP provider) | `tower-atomic` | **Yes** |
//! | `connect_http` | ureq (pure Rust HTTP) | `discovery-http` | **Yes** |
//! | [`connect_with_transport`] | Caller-provided | — | Depends |
//!
//! [`connect`]: DiscoveryClient::connect
//! [`connect_with_transport`]: DiscoveryClient::connect_with_transport
//! [`DiscoveryTransport`]: crate::transport::DiscoveryTransport
//!
//! ## Philosophy
//!
//! - **Zero hardcoding**: No primal names in code
//! - **Runtime discovery**: Find capabilities when needed
//! - **O(n) complexity**: Each primal connects to a registry, not to each other
//! - **Infant learning**: Start with zero knowledge, discover everything
//!
//! ## Example
//!
//! ```rust,no_run
//! use loam_spine_core::discovery_client::DiscoveryClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to the service registry (auto-selects best transport)
//! let client = DiscoveryClient::connect("http://registry.local:8082").await?;
//!
//! // Discover signing capability
//! let services = client.discover_capability("signing").await?;
//! for service in services {
//!     println!("Found signing service: {} at {}", service.name, service.endpoint);
//! }
//!
//! // Advertise our capabilities
//! client.advertise_self("http://localhost:9001", "http://localhost:8080").await?;
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::error::{IpcErrorPhase, LoamSpineError, LoamSpineResult};
use crate::resilience::{
    CircuitBreaker, CircuitBreakerConfig, ResilientAdapter, RetryPolicy, RetryPolicyConfig,
};
use crate::transport::DiscoveryTransport;

/// Service registry discovery client.
///
/// This client connects to any RFC 2782 compliant service registry to discover
/// other primals' capabilities and advertise LoamSpine's own capabilities.
///
/// Compatible registries include any system exposing `/health`, `/discover`,
/// `/register`, `/heartbeat`, and `/deregister` HTTP endpoints.
#[derive(Clone, Debug)]
pub struct DiscoveryClient {
    /// Registry endpoint (Arc for O(1) clone in resilient adapters).
    endpoint: Arc<str>,
    /// Pluggable HTTP transport.
    transport: Arc<dyn DiscoveryTransport>,
}

/// A discovered service.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveredService {
    /// Service name.
    pub name: String,
    /// Service endpoint (URL).
    pub endpoint: String,
    /// Service capabilities.
    pub capabilities: Vec<String>,
    /// Service health status.
    #[serde(default)]
    pub healthy: bool,
    /// Service metadata.
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

/// Service advertisement payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ServiceAdvertisement {
    name: String,
    primary_role: String,
    capabilities: Vec<String>,
    endpoints: Vec<ServiceEndpoint>,
    metadata: std::collections::HashMap<String, String>,
}

/// Service endpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ServiceEndpoint {
    protocol: String,
    address: String,
    port: u16,
    health_check: Option<String>,
}

impl DiscoveryClient {
    /// Connect to a service registry using the best available transport.
    ///
    /// Transport selection order:
    /// 1. **Tower Atomic** (feature `tower-atomic`) — ecoBin compliant, zero C deps
    /// 2. **ureq HTTP** (feature `discovery-http`) — pure Rust fallback
    ///
    /// Verifies the registry is reachable via its `/health` endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error if no transport is available or the registry is unreachable.
    #[cfg_attr(
        not(any(feature = "tower-atomic", feature = "discovery-http")),
        expect(
            clippy::unused_async,
            reason = "async required when transport features are enabled"
        )
    )]
    pub async fn connect(endpoint: impl Into<String>) -> LoamSpineResult<Self> {
        let endpoint: Arc<str> = Arc::from(endpoint.into());

        // Try Tower Atomic first (ecoBin)
        #[cfg(feature = "tower-atomic")]
        {
            match crate::transport::NeuralApiTransport::new(None) {
                Ok(transport) => {
                    let client = Self {
                        endpoint: Arc::clone(&endpoint),
                        transport: Arc::new(transport),
                    };
                    client.health_check().await?;
                    return Ok(client);
                }
                Err(e) => {
                    tracing::debug!("Tower Atomic transport unavailable, falling back: {e}");
                }
            }
        }

        // Fall back to ureq HTTP
        #[cfg(feature = "discovery-http")]
        {
            let transport = crate::transport::HttpTransport::new()?;
            let client = Self {
                endpoint: Arc::clone(&endpoint),
                transport: Arc::new(transport),
            };
            client.health_check().await?;
            return Ok(client);
        }

        #[cfg_attr(
            any(feature = "tower-atomic", feature = "discovery-http"),
            expect(unreachable_code, reason = "transport features make this dead code")
        )]
        Err(LoamSpineError::Network(format!(
            "No discovery transport available for {endpoint}. \
             Enable feature 'tower-atomic' (recommended) or 'discovery-http'."
        )))
    }

    /// Connect using the ureq HTTP transport explicitly.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry is unreachable.
    #[cfg(feature = "discovery-http")]
    pub async fn connect_http(endpoint: impl Into<String>) -> LoamSpineResult<Self> {
        let endpoint: Arc<str> = Arc::from(endpoint.into());
        let transport = crate::transport::HttpTransport::new()?;
        let client = Self {
            endpoint,
            transport: Arc::new(transport),
        };
        client.health_check().await?;
        Ok(client)
    }

    /// Connect with a caller-provided transport.
    ///
    /// Verifies the registry is reachable via its `/health` endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry is unreachable.
    pub async fn connect_with_transport(
        endpoint: impl Into<String>,
        transport: Arc<dyn DiscoveryTransport>,
    ) -> LoamSpineResult<Self> {
        let endpoint: Arc<str> = Arc::from(endpoint.into());
        let client = Self {
            endpoint,
            transport,
        };
        client.health_check().await?;
        Ok(client)
    }

    /// Verify the registry is reachable.
    pub(crate) async fn health_check(&self) -> LoamSpineResult<()> {
        let health_url = format!(
            "{}{}",
            self.endpoint,
            crate::constants::protocol::HEALTH_PATH
        );
        self.transport.get(&health_url).await.map_err(|e| {
            LoamSpineError::CapabilityUnavailable(format!(
                "Service registry unavailable at {}: {e}",
                self.endpoint
            ))
        })?;
        Ok(())
    }

    /// Discover services by capability.
    ///
    /// # Errors
    ///
    /// Returns an error if the discovery request fails.
    pub async fn discover_capability(
        &self,
        capability: &str,
    ) -> LoamSpineResult<Vec<DiscoveredService>> {
        let url = format!(
            "{}{}",
            self.endpoint,
            crate::constants::registry::DISCOVER_PATH
        );
        let response = self
            .transport
            .get_with_query(&url, &[("capability", capability)])
            .await
            .map_err(|e| {
                LoamSpineError::ipc(
                    IpcErrorPhase::Connect,
                    format!("Discovery request failed: {e}"),
                )
            })?;

        if !response.is_success() {
            return Err(LoamSpineError::ipc(
                IpcErrorPhase::HttpStatus(response.status),
                format!("Discovery failed with status: {}", response.status),
            ));
        }

        response.json()
    }

    /// Discover all available services.
    ///
    /// # Errors
    ///
    /// Returns an error if the discovery request fails.
    pub async fn discover_all(&self) -> LoamSpineResult<Vec<DiscoveredService>> {
        let url = format!(
            "{}{}",
            self.endpoint,
            crate::constants::registry::DISCOVER_PATH
        );
        let response = self.transport.get(&url).await.map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::Connect,
                format!("Discovery request failed: {e}"),
            )
        })?;

        if !response.is_success() {
            return Err(LoamSpineError::ipc(
                IpcErrorPhase::HttpStatus(response.status),
                format!("Discovery failed with status: {}", response.status),
            ));
        }

        response.json()
    }

    /// Advertise LoamSpine's capabilities to the service registry.
    ///
    /// # Errors
    ///
    /// Returns an error if the advertisement fails.
    pub async fn advertise_self(
        &self,
        tarpc_endpoint: &str,
        jsonrpc_endpoint: &str,
    ) -> LoamSpineResult<()> {
        let tarpc_port = extract_port(tarpc_endpoint)
            .unwrap_or_else(crate::constants::env_resolution::tarpc_port);
        let jsonrpc_port = extract_port(jsonrpc_endpoint)
            .unwrap_or_else(crate::constants::env_resolution::jsonrpc_port);

        let advertisement = ServiceAdvertisement {
            name: crate::neural_api::PRIMAL_NAME.to_string(),
            primary_role: crate::capabilities::identifiers::loamspine::PERMANENT_LEDGER.to_string(),
            capabilities: crate::capabilities::identifiers::loamspine::ADVERTISED
                .iter()
                .map(|&s| s.to_string())
                .collect(),
            endpoints: vec![
                ServiceEndpoint {
                    protocol: crate::constants::protocol::TARPC.to_string(),
                    address: tarpc_endpoint.to_string(),
                    port: tarpc_port,
                    health_check: None,
                },
                ServiceEndpoint {
                    protocol: crate::constants::protocol::JSONRPC.to_string(),
                    address: jsonrpc_endpoint.to_string(),
                    port: jsonrpc_port,
                    health_check: Some(crate::constants::protocol::HEALTH_PATH.to_string()),
                },
            ],
            metadata: [
                ("version", env!("CARGO_PKG_VERSION")),
                ("language", crate::constants::metadata::LANGUAGE),
                ("rpc_style", crate::constants::metadata::RPC_STYLE),
                (
                    "storage_backend",
                    crate::constants::metadata::STORAGE_BACKEND,
                ),
                ("zero_copy", "true"),
                ("unsafe_code", "false"),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        };

        let body = serde_json::to_value(&advertisement).map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::Serialization,
                format!("Failed to serialize advertisement: {e}"),
            )
        })?;

        let url = format!(
            "{}{}",
            self.endpoint,
            crate::constants::registry::REGISTER_PATH
        );
        let response = self.transport.post_json(&url, &body).await.map_err(|e| {
            LoamSpineError::ipc(IpcErrorPhase::Connect, format!("Advertisement failed: {e}"))
        })?;

        if !response.is_success() {
            return Err(LoamSpineError::ipc(
                IpcErrorPhase::HttpStatus(response.status),
                format!("Advertisement failed with status: {}", response.status),
            ));
        }

        Ok(())
    }

    /// Heartbeat to keep advertisement alive.
    ///
    /// # Errors
    ///
    /// Returns an error if the heartbeat fails.
    pub async fn heartbeat(&self) -> LoamSpineResult<()> {
        let url = format!(
            "{}{}",
            self.endpoint,
            crate::constants::registry::HEARTBEAT_PATH
        );
        let body = serde_json::json!({ "name": crate::neural_api::PRIMAL_NAME });
        let response = self.transport.post_json(&url, &body).await.map_err(|e| {
            LoamSpineError::ipc(IpcErrorPhase::Connect, format!("Heartbeat failed: {e}"))
        })?;

        if !response.is_success() {
            return Err(LoamSpineError::ipc(
                IpcErrorPhase::HttpStatus(response.status),
                format!("Heartbeat failed with status: {}", response.status),
            ));
        }

        Ok(())
    }

    /// Deregister from the service registry on shutdown.
    ///
    /// # Errors
    ///
    /// Returns an error if the deregistration fails.
    pub async fn deregister(&self) -> LoamSpineResult<()> {
        let url = format!(
            "{}{}",
            self.endpoint,
            crate::constants::registry::DEREGISTER_PATH
        );
        let body = serde_json::json!({ "name": crate::neural_api::PRIMAL_NAME });
        let response = self.transport.post_json(&url, &body).await.map_err(|e| {
            LoamSpineError::ipc(IpcErrorPhase::Connect, format!("Deregister failed: {e}"))
        })?;

        if !response.is_success() {
            return Err(LoamSpineError::ipc(
                IpcErrorPhase::HttpStatus(response.status),
                format!("Deregister failed with status: {}", response.status),
            ));
        }

        Ok(())
    }

    /// Get the registry endpoint.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Create a client with a mock transport for testing (bypasses health check).
    #[cfg(test)]
    #[must_use]
    pub fn for_testing(endpoint: impl Into<String>) -> Self {
        let endpoint: Arc<str> = Arc::from(endpoint.into());
        Self {
            transport: Arc::new(crate::transport::mock::MockTransport::new(&*endpoint)),
            endpoint,
        }
    }

    /// Create a client with a success transport for testing (all operations succeed).
    ///
    /// Used to exercise success paths (advertise, heartbeat, deregister) in lifecycle tests.
    #[cfg(test)]
    #[must_use]
    pub fn for_testing_success(endpoint: impl Into<String>) -> Self {
        Self {
            transport: Arc::new(crate::transport::mock::SuccessTransport::new()),
            endpoint: Arc::from(endpoint.into()),
        }
    }

    /// Create a client with a custom transport for testing (bypasses health check).
    ///
    /// Used to test error paths (non-success status, invalid JSON) without a real backend.
    #[cfg(test)]
    #[must_use]
    pub fn for_testing_with_transport(
        endpoint: impl Into<String>,
        transport: Arc<dyn DiscoveryTransport>,
    ) -> Self {
        Self {
            endpoint: Arc::from(endpoint.into()),
            transport,
        }
    }

    /// Wrap this client with retry and circuit-breaker resilience.
    ///
    /// Returns a [`ResilientDiscoveryClient`] that applies retry with exponential
    /// backoff and circuit-breaking to all discovery operations.
    #[must_use]
    pub fn with_resilience(
        self,
        circuit_config: CircuitBreakerConfig,
        retry_config: RetryPolicyConfig,
    ) -> ResilientDiscoveryClient {
        ResilientDiscoveryClient::new(self, circuit_config, retry_config)
    }
}

/// Discovery client with retry and circuit-breaker resilience.
///
/// Wraps [`DiscoveryClient`] and applies [`ResilientAdapter`] to all operations,
/// protecting against transient failures and cascading outages.
#[derive(Clone, Debug)]
pub struct ResilientDiscoveryClient {
    inner: DiscoveryClient,
    adapter: Arc<ResilientAdapter>,
}

impl ResilientDiscoveryClient {
    /// Create a resilient wrapper around a discovery client.
    #[must_use]
    pub fn new(
        inner: DiscoveryClient,
        circuit_config: CircuitBreakerConfig,
        retry_config: RetryPolicyConfig,
    ) -> Self {
        let circuit_breaker = Arc::new(CircuitBreaker::new(circuit_config));
        let retry_policy = RetryPolicy::new(retry_config);
        let adapter = Arc::new(ResilientAdapter::new(circuit_breaker, retry_policy));
        Self { inner, adapter }
    }

    /// Discover services by capability (with retry and circuit-breaker).
    ///
    /// # Errors
    ///
    /// Returns error if discovery fails or circuit breaker is open.
    pub async fn discover_capability(
        &self,
        capability: &str,
    ) -> LoamSpineResult<Vec<DiscoveredService>> {
        let cap: Arc<str> = Arc::from(capability);
        let client = self.inner.clone();
        self.adapter
            .execute(move || {
                let c = Arc::clone(&cap);
                let cl = client.clone();
                async move { cl.discover_capability(&c).await }
            })
            .await
    }

    /// Discover all available services (with retry and circuit-breaker).
    ///
    /// # Errors
    ///
    /// Returns error if discovery fails or circuit breaker is open.
    pub async fn discover_all(&self) -> LoamSpineResult<Vec<DiscoveredService>> {
        let client = self.inner.clone();
        self.adapter
            .execute(move || {
                let cl = client.clone();
                async move { cl.discover_all().await }
            })
            .await
    }

    /// Advertise LoamSpine's capabilities (with retry and circuit-breaker).
    ///
    /// # Errors
    ///
    /// Returns error if advertisement fails or circuit breaker is open.
    pub async fn advertise_self(
        &self,
        tarpc_endpoint: &str,
        jsonrpc_endpoint: &str,
    ) -> LoamSpineResult<()> {
        let tarpc: Arc<str> = Arc::from(tarpc_endpoint);
        let jsonrpc: Arc<str> = Arc::from(jsonrpc_endpoint);
        let client = self.inner.clone();
        self.adapter
            .execute(move || {
                let cl = client.clone();
                let t = Arc::clone(&tarpc);
                let j = Arc::clone(&jsonrpc);
                async move { cl.advertise_self(&t, &j).await }
            })
            .await
    }

    /// Heartbeat (with retry and circuit-breaker).
    ///
    /// # Errors
    ///
    /// Returns error if heartbeat fails or circuit breaker is open.
    pub async fn heartbeat(&self) -> LoamSpineResult<()> {
        let client = self.inner.clone();
        self.adapter
            .execute(move || {
                let cl = client.clone();
                async move { cl.heartbeat().await }
            })
            .await
    }

    /// Deregister (with retry and circuit-breaker).
    ///
    /// # Errors
    ///
    /// Returns error if deregistration fails or circuit breaker is open.
    pub async fn deregister(&self) -> LoamSpineResult<()> {
        let client = self.inner.clone();
        self.adapter
            .execute(move || {
                let cl = client.clone();
                async move { cl.deregister().await }
            })
            .await
    }

    /// Get the registry endpoint.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        self.inner.endpoint()
    }
}

/// Extract the port from a URL string using the `url` crate (pure Rust).
fn extract_port(url_str: &str) -> Option<u16> {
    url::Url::parse(url_str).ok().and_then(|u| u.port())
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests;
