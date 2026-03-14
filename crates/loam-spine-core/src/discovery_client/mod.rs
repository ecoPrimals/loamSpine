// SPDX-License-Identifier: AGPL-3.0-only

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
//! | [`connect`] | Tower Atomic (NeuralAPI → Songbird) | `tower-atomic` | **Yes** |
//! | `connect_http` | reqwest (rustls → ring) | `discovery-http` | No |
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

use crate::error::{LoamSpineError, LoamSpineResult};
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
    /// Registry endpoint.
    endpoint: String,
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
    /// 2. **reqwest HTTP** (feature `discovery-http`) — development fallback
    ///
    /// Verifies the registry is reachable via its `/health` endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error if no transport is available or the registry is unreachable.
    #[allow(clippy::unused_async)] // async is required when transport features are enabled
    pub async fn connect(endpoint: impl Into<String>) -> LoamSpineResult<Self> {
        let endpoint = endpoint.into();

        // Try Tower Atomic first (ecoBin)
        #[cfg(feature = "tower-atomic")]
        {
            match crate::transport::NeuralApiTransport::new(None) {
                Ok(transport) => {
                    let client = Self {
                        endpoint: endpoint.clone(),
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

        // Fall back to reqwest HTTP
        #[cfg(feature = "discovery-http")]
        {
            let transport = crate::transport::HttpTransport::new()?;
            let client = Self {
                endpoint: endpoint.clone(),
                transport: Arc::new(transport),
            };
            client.health_check().await?;
            return Ok(client);
        }

        #[allow(unreachable_code)]
        Err(LoamSpineError::Network(format!(
            "No discovery transport available for {endpoint}. \
             Enable feature 'tower-atomic' (recommended) or 'discovery-http'."
        )))
    }

    /// Connect using the reqwest HTTP transport explicitly.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry is unreachable.
    #[cfg(feature = "discovery-http")]
    pub async fn connect_http(endpoint: impl Into<String>) -> LoamSpineResult<Self> {
        let endpoint = endpoint.into();
        let transport = crate::transport::HttpTransport::new()?;
        let client = Self {
            endpoint: endpoint.clone(),
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
        let endpoint = endpoint.into();
        let client = Self {
            endpoint: endpoint.clone(),
            transport,
        };
        client.health_check().await?;
        Ok(client)
    }

    /// Verify the registry is reachable.
    pub(crate) async fn health_check(&self) -> LoamSpineResult<()> {
        let health_url = format!("{}/health", self.endpoint);
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
        let url = format!("{}/discover", self.endpoint);
        let response = self
            .transport
            .get_with_query(&url, &[("capability", capability)])
            .await
            .map_err(|e| LoamSpineError::Network(format!("Discovery request failed: {e}")))?;

        if !response.is_success() {
            return Err(LoamSpineError::Network(format!(
                "Discovery failed with status: {}",
                response.status
            )));
        }

        response.json()
    }

    /// Discover all available services.
    ///
    /// # Errors
    ///
    /// Returns an error if the discovery request fails.
    pub async fn discover_all(&self) -> LoamSpineResult<Vec<DiscoveredService>> {
        let url = format!("{}/discover", self.endpoint);
        let response = self
            .transport
            .get(&url)
            .await
            .map_err(|e| LoamSpineError::Network(format!("Discovery request failed: {e}")))?;

        if !response.is_success() {
            return Err(LoamSpineError::Network(format!(
                "Discovery failed with status: {}",
                response.status
            )));
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
        let tarpc_port =
            extract_port(tarpc_endpoint).unwrap_or(crate::constants::DEFAULT_TARPC_PORT);
        let jsonrpc_port =
            extract_port(jsonrpc_endpoint).unwrap_or(crate::constants::DEFAULT_JSONRPC_PORT);

        let advertisement = ServiceAdvertisement {
            name: crate::neural_api::PRIMAL_NAME.to_string(),
            primary_role: "permanence".to_string(),
            capabilities: vec![
                "permanence".to_string(),
                "selective-memory".to_string(),
                "spine-management".to_string(),
                "certificate-management".to_string(),
                "inclusion-proofs".to_string(),
                "backup".to_string(),
                "restore".to_string(),
            ],
            endpoints: vec![
                ServiceEndpoint {
                    protocol: "tarpc".to_string(),
                    address: tarpc_endpoint.to_string(),
                    port: tarpc_port,
                    health_check: None,
                },
                ServiceEndpoint {
                    protocol: "jsonrpc".to_string(),
                    address: jsonrpc_endpoint.to_string(),
                    port: jsonrpc_port,
                    health_check: Some("/health".to_string()),
                },
            ],
            metadata: [
                ("version".to_string(), env!("CARGO_PKG_VERSION").to_string()),
                ("language".to_string(), "rust".to_string()),
                ("rpc_style".to_string(), "pure-rust".to_string()),
                ("storage_backend".to_string(), "sled".to_string()),
                ("zero_copy".to_string(), "true".to_string()),
                ("unsafe_code".to_string(), "false".to_string()),
            ]
            .into_iter()
            .collect(),
        };

        let body = serde_json::to_value(&advertisement).map_err(|e| {
            LoamSpineError::Network(format!("Failed to serialize advertisement: {e}"))
        })?;

        let url = format!("{}/register", self.endpoint);
        let response = self
            .transport
            .post_json(&url, &body)
            .await
            .map_err(|e| LoamSpineError::Network(format!("Advertisement failed: {e}")))?;

        if !response.is_success() {
            return Err(LoamSpineError::Network(format!(
                "Advertisement failed with status: {}",
                response.status
            )));
        }

        Ok(())
    }

    /// Backward-compatible alias for [`Self::advertise_self`].
    ///
    /// # Errors
    ///
    /// Returns an error if the advertisement fails.
    #[deprecated(since = "0.9.0", note = "Use advertise_self instead")]
    pub async fn advertise_loamspine(
        &self,
        tarpc_endpoint: &str,
        jsonrpc_endpoint: &str,
    ) -> LoamSpineResult<()> {
        self.advertise_self(tarpc_endpoint, jsonrpc_endpoint).await
    }

    /// Heartbeat to keep advertisement alive.
    ///
    /// # Errors
    ///
    /// Returns an error if the heartbeat fails.
    pub async fn heartbeat(&self) -> LoamSpineResult<()> {
        let url = format!("{}/heartbeat", self.endpoint);
        let body = serde_json::json!({ "name": crate::neural_api::PRIMAL_NAME });
        let response = self
            .transport
            .post_json(&url, &body)
            .await
            .map_err(|e| LoamSpineError::Network(format!("Heartbeat failed: {e}")))?;

        if !response.is_success() {
            return Err(LoamSpineError::Network(format!(
                "Heartbeat failed with status: {}",
                response.status
            )));
        }

        Ok(())
    }

    /// Deregister from the service registry on shutdown.
    ///
    /// # Errors
    ///
    /// Returns an error if the deregistration fails.
    pub async fn deregister(&self) -> LoamSpineResult<()> {
        let url = format!("{}/deregister", self.endpoint);
        let body = serde_json::json!({ "name": crate::neural_api::PRIMAL_NAME });
        let response = self
            .transport
            .post_json(&url, &body)
            .await
            .map_err(|e| LoamSpineError::Network(format!("Deregister failed: {e}")))?;

        if !response.is_success() {
            return Err(LoamSpineError::Network(format!(
                "Deregister failed with status: {}",
                response.status
            )));
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
        let endpoint = endpoint.into();
        Self {
            transport: Arc::new(crate::transport::mock::MockTransport::new(&endpoint)),
            endpoint,
        }
    }

    /// Create a client with a success transport for testing (all operations succeed).
    ///
    /// Used to exercise success paths (advertise, heartbeat, deregister) in lifecycle tests.
    #[cfg(test)]
    #[must_use]
    pub fn for_testing_success(endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        Self {
            transport: Arc::new(crate::transport::mock::SuccessTransport::new()),
            endpoint,
        }
    }
}

/// Extract the port from a URL string using the `url` crate (pure Rust).
fn extract_port(url_str: &str) -> Option<u16> {
    url::Url::parse(url_str).ok().and_then(|u| u.port())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
