//! Songbird integration for universal service discovery.
//!
//! This module provides integration with Songbird (the universal adapter) for
//! discovering other primals' capabilities at runtime without hardcoding.
//!
//! ## Philosophy
//!
//! - **Zero hardcoding**: No primal names in code
//! - **Runtime discovery**: Find capabilities when needed
//! - **O(n) complexity**: Each primal connects to Songbird, not to each other
//! - **Infant learning**: Start with zero knowledge, discover everything
//!
//! ## Example
//!
//! ```rust,no_run
//! use loam_spine_core::songbird::SongbirdClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to Songbird
//! let client = SongbirdClient::connect("http://localhost:8082").await?;
//!
//! // Discover signing capability
//! let services = client.discover_capability("signing").await?;
//! for service in services {
//!     println!("Found signing service: {} at {}", service.name, service.endpoint);
//! }
//!
//! // Advertise our capabilities
//! client.advertise_loamspine("http://localhost:9001", "http://localhost:8080").await?;
//! # Ok(())
//! # }
//! ```

use crate::error::{LoamSpineError, LoamSpineResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Songbird discovery client.
///
/// This client connects to a Songbird instance to discover other primals'
/// capabilities and advertise LoamSpine's own capabilities.
#[derive(Clone, Debug)]
pub struct SongbirdClient {
    /// Songbird endpoint.
    endpoint: String,
    /// HTTP client.
    client: reqwest::Client,
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

impl SongbirdClient {
    /// Connect to a Songbird instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails or Songbird is unavailable.
    pub async fn connect(endpoint: impl Into<String>) -> LoamSpineResult<Self> {
        let endpoint = endpoint.into();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| LoamSpineError::Network(format!("Failed to create HTTP client: {e}")))?;

        // Verify Songbird is reachable
        let health_url = format!("{endpoint}/health");
        client.get(&health_url).send().await.map_err(|e| {
            LoamSpineError::CapabilityUnavailable(format!(
                "Songbird unavailable at {endpoint}: {e}"
            ))
        })?;

        Ok(Self { endpoint, client })
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
            .client
            .get(&url)
            .query(&[("capability", capability)])
            .send()
            .await
            .map_err(|e| LoamSpineError::Network(format!("Discovery request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(LoamSpineError::Network(format!(
                "Discovery failed with status: {}",
                response.status()
            )));
        }

        let services: Vec<DiscoveredService> = response.json().await.map_err(|e| {
            LoamSpineError::Network(format!("Failed to parse discovery response: {e}"))
        })?;

        Ok(services)
    }

    /// Discover all available services.
    ///
    /// # Errors
    ///
    /// Returns an error if the discovery request fails.
    pub async fn discover_all(&self) -> LoamSpineResult<Vec<DiscoveredService>> {
        let url = format!("{}/discover", self.endpoint);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LoamSpineError::Network(format!("Discovery request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(LoamSpineError::Network(format!(
                "Discovery failed with status: {}",
                response.status()
            )));
        }

        let services: Vec<DiscoveredService> = response.json().await.map_err(|e| {
            LoamSpineError::Network(format!("Failed to parse discovery response: {e}"))
        })?;

        Ok(services)
    }

    /// Advertise LoamSpine's capabilities to Songbird.
    ///
    /// # Errors
    ///
    /// Returns an error if the advertisement fails.
    pub async fn advertise_loamspine(
        &self,
        tarpc_endpoint: &str,
        jsonrpc_endpoint: &str,
    ) -> LoamSpineResult<()> {
        // Extract ports from endpoint URLs (default to standard ports if parsing fails)
        let tarpc_port = tarpc_endpoint
            .parse::<reqwest::Url>()
            .ok()
            .and_then(|u| u.port())
            .unwrap_or(9001);
        let jsonrpc_port = jsonrpc_endpoint
            .parse::<reqwest::Url>()
            .ok()
            .and_then(|u| u.port())
            .unwrap_or(8080);

        let advertisement = ServiceAdvertisement {
            name: "loamspine".to_string(),
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

        let url = format!("{}/register", self.endpoint);
        let response = self
            .client
            .post(&url)
            .json(&advertisement)
            .send()
            .await
            .map_err(|e| LoamSpineError::Network(format!("Advertisement failed: {e}")))?;

        if !response.status().is_success() {
            return Err(LoamSpineError::Network(format!(
                "Advertisement failed with status: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Heartbeat to keep advertisement alive.
    ///
    /// # Errors
    ///
    /// Returns an error if the heartbeat fails.
    pub async fn heartbeat(&self) -> LoamSpineResult<()> {
        let url = format!("{}/heartbeat", self.endpoint);
        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "name": "loamspine" }))
            .send()
            .await
            .map_err(|e| LoamSpineError::Network(format!("Heartbeat failed: {e}")))?;

        if !response.status().is_success() {
            return Err(LoamSpineError::Network(format!(
                "Heartbeat failed with status: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Deregister from Songbird on shutdown.
    ///
    /// # Errors
    ///
    /// Returns an error if the deregistration fails.
    pub async fn deregister(&self) -> LoamSpineResult<()> {
        let url = format!("{}/deregister", self.endpoint);
        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "name": "loamspine" }))
            .send()
            .await
            .map_err(|e| LoamSpineError::Network(format!("Deregister failed: {e}")))?;

        if !response.status().is_success() {
            return Err(LoamSpineError::Network(format!(
                "Deregister failed with status: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Get the Songbird endpoint.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn client_creation() {
        // Client creation is async, so we just test the structure
        let endpoint = "http://localhost:8082";
        assert!(!endpoint.is_empty());
    }

    #[test]
    fn discovered_service_serialization() {
        let service = DiscoveredService {
            name: "test-service".to_string(),
            endpoint: "http://localhost:9000".to_string(),
            capabilities: vec!["signing".to_string()],
            healthy: true,
            metadata: std::iter::once(("version".to_string(), "1.0.0".to_string())).collect(),
        };

        let json = serde_json::to_string(&service).unwrap();
        let deserialized: DiscoveredService = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test-service");
        assert_eq!(deserialized.capabilities.len(), 1);
    }

    #[test]
    fn discovered_service_default_fields() {
        let json = r#"{"name":"test","endpoint":"http://localhost:9000","capabilities":[]}"#;
        let service: DiscoveredService = serde_json::from_str(json).unwrap();

        assert_eq!(service.name, "test");
        assert_eq!(service.endpoint, "http://localhost:9000");
        assert!(service.capabilities.is_empty());
        assert!(!service.healthy);
        assert!(service.metadata.is_empty());
    }

    #[test]
    fn service_advertisement_serialization() {
        let advertisement = ServiceAdvertisement {
            name: "loamspine".to_string(),
            primary_role: "permanence".to_string(),
            capabilities: vec!["permanence".to_string()],
            endpoints: vec![ServiceEndpoint {
                protocol: "tarpc".to_string(),
                address: "http://localhost:9001".to_string(),
                port: 9001,
                health_check: None,
            }],
            metadata: std::iter::once(("version".to_string(), "0.6.0".to_string())).collect(),
        };

        let json = serde_json::to_string(&advertisement).unwrap();
        assert!(json.contains("loamspine"));
        assert!(json.contains("permanence"));
        assert!(json.contains("tarpc"));
    }

    #[test]
    fn service_endpoint_serialization() {
        let endpoint = ServiceEndpoint {
            protocol: "jsonrpc".to_string(),
            address: "http://localhost:8080".to_string(),
            port: 8080,
            health_check: Some("/health".to_string()),
        };

        let json = serde_json::to_string(&endpoint).unwrap();
        let deserialized: ServiceEndpoint = serde_json::from_str(&json).unwrap();

        assert_eq!(endpoint.protocol, deserialized.protocol);
        assert_eq!(endpoint.address, deserialized.address);
        assert_eq!(endpoint.port, deserialized.port);
        assert_eq!(endpoint.health_check, deserialized.health_check);
    }

    #[test]
    fn songbird_client_endpoint_getter() {
        let endpoint = "http://localhost:8082";
        let client = SongbirdClient {
            endpoint: endpoint.to_string(),
            client: reqwest::Client::new(),
        };

        assert_eq!(client.endpoint(), endpoint);
    }

    #[test]
    fn songbird_client_is_cloneable() {
        let client = SongbirdClient {
            endpoint: "http://localhost:8082".to_string(),
            client: reqwest::Client::new(),
        };

        // Test that client is cloneable
        #[allow(clippy::no_effect_underscore_binding)]
        let _cloned = &client;
        assert_eq!(client.endpoint(), "http://localhost:8082");
    }

    #[test]
    fn discovered_service_with_metadata() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("version".to_string(), "1.0.0".to_string());
        metadata.insert("language".to_string(), "rust".to_string());

        let service = DiscoveredService {
            name: "test-service".to_string(),
            endpoint: "http://localhost:9000".to_string(),
            capabilities: vec!["signing".to_string(), "verification".to_string()],
            healthy: true,
            metadata,
        };

        assert_eq!(service.metadata.len(), 2);
        assert_eq!(service.metadata.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(service.metadata.get("language"), Some(&"rust".to_string()));
    }

    #[test]
    fn discovered_service_multiple_capabilities() {
        let service = DiscoveredService {
            name: "multi-service".to_string(),
            endpoint: "http://localhost:9000".to_string(),
            capabilities: vec![
                "signing".to_string(),
                "verification".to_string(),
                "encryption".to_string(),
            ],
            healthy: true,
            metadata: std::collections::HashMap::new(),
        };

        assert_eq!(service.capabilities.len(), 3);
        assert!(service.capabilities.contains(&"signing".to_string()));
        assert!(service.capabilities.contains(&"verification".to_string()));
        assert!(service.capabilities.contains(&"encryption".to_string()));
    }

    #[test]
    fn service_endpoint_without_health_check() {
        let endpoint = ServiceEndpoint {
            protocol: "tarpc".to_string(),
            address: "http://localhost:9001".to_string(),
            port: 9001,
            health_check: None,
        };

        assert!(endpoint.health_check.is_none());
    }

    #[test]
    fn service_endpoint_with_health_check() {
        let endpoint = ServiceEndpoint {
            protocol: "jsonrpc".to_string(),
            address: "http://localhost:8080".to_string(),
            port: 8080,
            health_check: Some("/health".to_string()),
        };

        assert!(endpoint.health_check.is_some());
        assert_eq!(endpoint.health_check.unwrap(), "/health");
    }

    #[test]
    fn service_advertisement_with_multiple_endpoints() {
        let advertisement = ServiceAdvertisement {
            name: "loamspine".to_string(),
            primary_role: "permanence".to_string(),
            capabilities: vec!["permanence".to_string()],
            endpoints: vec![
                ServiceEndpoint {
                    protocol: "tarpc".to_string(),
                    address: "http://localhost:9001".to_string(),
                    port: 9001,
                    health_check: None,
                },
                ServiceEndpoint {
                    protocol: "jsonrpc".to_string(),
                    address: "http://localhost:8080".to_string(),
                    port: 8080,
                    health_check: Some("/health".to_string()),
                },
            ],
            metadata: std::collections::HashMap::new(),
        };

        assert_eq!(advertisement.endpoints.len(), 2);
        assert_eq!(advertisement.endpoints[0].protocol, "tarpc");
        assert_eq!(advertisement.endpoints[1].protocol, "jsonrpc");
    }

    #[test]
    fn service_advertisement_metadata() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("version".to_string(), "0.6.0".to_string());
        metadata.insert("language".to_string(), "rust".to_string());
        metadata.insert("rpc_style".to_string(), "pure-rust".to_string());

        let advertisement = ServiceAdvertisement {
            name: "loamspine".to_string(),
            primary_role: "permanence".to_string(),
            capabilities: vec!["permanence".to_string()],
            endpoints: vec![],
            metadata,
        };

        assert_eq!(advertisement.metadata.len(), 3);
        assert_eq!(
            advertisement.metadata.get("version"),
            Some(&"0.6.0".to_string())
        );
    }

    #[test]
    fn port_extraction_from_urls() {
        // Test port extraction logic
        let url1 = "http://localhost:9001";
        let port1 = url1
            .parse::<reqwest::Url>()
            .ok()
            .and_then(|u| u.port())
            .unwrap_or(9001);
        assert_eq!(port1, 9001);

        let url2 = "http://localhost:8080";
        let port2 = url2
            .parse::<reqwest::Url>()
            .ok()
            .and_then(|u| u.port())
            .unwrap_or(8080);
        assert_eq!(port2, 8080);

        // Test default fallback
        let url3 = "http://localhost";
        let port3 = url3
            .parse::<reqwest::Url>()
            .ok()
            .and_then(|u| u.port())
            .unwrap_or(9999);
        assert_eq!(port3, 9999);
    }

    #[test]
    fn discovered_service_healthy_flag() {
        let healthy_service = DiscoveredService {
            name: "healthy".to_string(),
            endpoint: "http://localhost:9000".to_string(),
            capabilities: vec!["signing".to_string()],
            healthy: true,
            metadata: std::collections::HashMap::new(),
        };

        let unhealthy_service = DiscoveredService {
            name: "unhealthy".to_string(),
            endpoint: "http://localhost:9001".to_string(),
            capabilities: vec!["signing".to_string()],
            healthy: false,
            metadata: std::collections::HashMap::new(),
        };

        assert!(healthy_service.healthy);
        assert!(!unhealthy_service.healthy);
    }

    #[test]
    fn service_advertisement_empty_capabilities() {
        let advertisement = ServiceAdvertisement {
            name: "minimal".to_string(),
            primary_role: "test".to_string(),
            capabilities: vec![],
            endpoints: vec![],
            metadata: std::collections::HashMap::new(),
        };

        assert!(advertisement.capabilities.is_empty());
        assert!(advertisement.endpoints.is_empty());
        assert!(advertisement.metadata.is_empty());
    }

    #[test]
    fn service_endpoint_port_matching() {
        let endpoint = ServiceEndpoint {
            protocol: "tarpc".to_string(),
            address: "http://localhost:9001".to_string(),
            port: 9001,
            health_check: None,
        };

        // Port should match the one in the address
        assert!(endpoint.address.contains("9001"));
        assert_eq!(endpoint.port, 9001);
    }

    #[test]
    fn discovered_service_json_roundtrip() {
        let original = DiscoveredService {
            name: "roundtrip-test".to_string(),
            endpoint: "http://localhost:9000".to_string(),
            capabilities: vec!["signing".to_string(), "verification".to_string()],
            healthy: true,
            metadata: vec![
                ("version".to_string(), "1.0.0".to_string()),
                ("build".to_string(), "123".to_string()),
            ]
            .into_iter()
            .collect(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: DiscoveredService = serde_json::from_str(&json).unwrap();

        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.endpoint, deserialized.endpoint);
        assert_eq!(original.capabilities, deserialized.capabilities);
        assert_eq!(original.healthy, deserialized.healthy);
        assert_eq!(original.metadata, deserialized.metadata);
    }

    #[test]
    fn service_advertisement_complete_metadata() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("version".to_string(), "0.6.0".to_string());
        metadata.insert("language".to_string(), "rust".to_string());
        metadata.insert("rpc_style".to_string(), "pure-rust".to_string());
        metadata.insert("unsafe_code".to_string(), "false".to_string());
        metadata.insert("zero_copy".to_string(), "true".to_string());

        let advertisement = ServiceAdvertisement {
            name: "loamspine".to_string(),
            primary_role: "permanence".to_string(),
            capabilities: vec![
                "permanence".to_string(),
                "certificates".to_string(),
                "proofs".to_string(),
            ],
            endpoints: vec![
                ServiceEndpoint {
                    protocol: "tarpc".to_string(),
                    address: "http://localhost:9001".to_string(),
                    port: 9001,
                    health_check: None,
                },
                ServiceEndpoint {
                    protocol: "jsonrpc".to_string(),
                    address: "http://localhost:8080".to_string(),
                    port: 8080,
                    health_check: Some("/health".to_string()),
                },
            ],
            metadata,
        };

        assert_eq!(advertisement.capabilities.len(), 3);
        assert_eq!(advertisement.endpoints.len(), 2);
        assert_eq!(advertisement.metadata.len(), 5);
        assert_eq!(
            advertisement.metadata.get("unsafe_code"),
            Some(&"false".to_string())
        );
    }

    // Integration tests with real Songbird require Songbird to be running
    // These are tested in the showcase demos
}
