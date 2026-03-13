// SPDX-License-Identifier: AGPL-3.0-only

//! LoamSpine configuration.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for LoamSpine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoamSpineConfig {
    /// Service name.
    pub name: String,

    /// Storage path for spine data.
    pub storage_path: PathBuf,

    /// Maximum entries before auto-rollup.
    pub auto_rollup_threshold: Option<u64>,

    /// Enable replication.
    pub replication_enabled: bool,

    /// Log level.
    pub log_level: String,

    /// Discovery configuration.
    #[serde(default)]
    pub discovery: DiscoveryConfig,
}

/// Discovery configuration for finding other primals.
///
/// **Infant Discovery Philosophy**: LoamSpine starts knowing only itself and discovers
/// everything else at runtime through the universal adapter (service registry).
///
/// The service registry can be any RFC 2782 compliant system:
/// - Songbird (reference implementation for ecoPrimals)
/// - Consul
/// - etcd
/// - Custom implementations following the protocol
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enable discovery service (universal adapter).
    ///
    /// When enabled, LoamSpine will attempt to discover and register with a
    /// discovery service using the configured endpoint or auto-discovery methods.
    pub discovery_enabled: bool,

    /// Discovery service endpoint (universal adapter).
    ///
    /// Compatible with any RFC 2782 compliant service discovery system:
    /// - Songbird (reference implementation)
    /// - Consul
    /// - etcd
    /// - Custom implementations
    ///
    /// If `None`, will attempt auto-discovery via:
    /// 1. `DISCOVERY_ENDPOINT` environment variable
    /// 2. DNS SRV records (_discovery._tcp.local)
    /// 3. mDNS (local network)
    /// 4. Development fallback (localhost:8082, logged as warning)
    pub discovery_endpoint: Option<String>,

    /// tarpc endpoint for binary RPC (primal-to-primal communication).
    ///
    /// Set to `0.0.0.0:0` to let the OS assign an available port.
    /// Can be overridden via `TARPC_ENDPOINT` environment variable.
    pub tarpc_endpoint: String,

    /// JSON-RPC 2.0 endpoint for external clients.
    ///
    /// Set to `0.0.0.0:0` to let the OS assign an available port.
    /// Can be overridden via `JSONRPC_ENDPOINT` environment variable.
    pub jsonrpc_endpoint: String,

    /// Auto-advertise capabilities on startup.
    ///
    /// When enabled, LoamSpine will automatically register its capabilities
    /// with the discovery service after successful connection.
    pub auto_advertise: bool,

    /// Heartbeat interval for maintaining registration (seconds).
    pub heartbeat_interval_seconds: u64,

    /// Heartbeat retry configuration.
    #[serde(default)]
    pub heartbeat_retry: HeartbeatRetryConfig,

    /// Discovery methods (in priority order).
    ///
    /// LoamSpine will attempt each method in order until successful.
    pub methods: Vec<DiscoveryMethod>,
}

/// Heartbeat retry configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeartbeatRetryConfig {
    /// Exponential backoff delays in seconds [10, 30, 60, 120].
    pub backoff_seconds: Vec<u64>,

    /// Maximum consecutive failures before marking as degraded.
    pub max_failures_before_degraded: u32,

    /// Maximum consecutive failures before giving up.
    pub max_failures_total: u32,
}

impl Default for HeartbeatRetryConfig {
    fn default() -> Self {
        Self {
            backoff_seconds: vec![10, 30, 60, 120],
            max_failures_before_degraded: 3,
            max_failures_total: 10,
        }
    }
}

/// Discovery methods for finding capabilities.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiscoveryMethod {
    /// Environment variables (e.g., LOAMSPINE_SIGNER_PATH).
    Environment,
    /// Service registry (universal adapter).
    ///
    /// Compatible with any RFC 2782 compliant service discovery system:
    /// - Songbird (reference implementation)
    /// - Consul
    /// - etcd
    /// - Custom implementations
    #[serde(alias = "songbird")] // Backward compatibility
    ServiceRegistry,
    /// Multicast DNS.
    Mdns,
    /// Local binaries (../bins/).
    LocalBinaries,
    /// Config file.
    ConfigFile,
    /// Fallback defaults.
    Fallback,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            discovery_enabled: true,
            discovery_endpoint: std::env::var("DISCOVERY_ENDPOINT").ok(),

            // Our own endpoints — prefer OS-assigned ports in production
            tarpc_endpoint: std::env::var("TARPC_ENDPOINT").unwrap_or_else(|_| {
                format!(
                    "http://{}:{}",
                    crate::constants::BIND_ALL_IPV4,
                    crate::constants::DEFAULT_TARPC_PORT
                )
            }),
            jsonrpc_endpoint: std::env::var("JSONRPC_ENDPOINT").unwrap_or_else(|_| {
                format!(
                    "http://{}:{}",
                    crate::constants::BIND_ALL_IPV4,
                    crate::constants::DEFAULT_JSONRPC_PORT
                )
            }),

            auto_advertise: true,
            heartbeat_interval_seconds: 60,
            heartbeat_retry: HeartbeatRetryConfig::default(),
            methods: vec![
                DiscoveryMethod::Environment,
                DiscoveryMethod::ServiceRegistry, // Generic service registry (Songbird, Consul, etcd, etc.)
                DiscoveryMethod::LocalBinaries,
                DiscoveryMethod::Fallback,
            ],
        }
    }
}

impl Default for LoamSpineConfig {
    fn default() -> Self {
        Self {
            name: "LoamSpine".to_string(),
            storage_path: PathBuf::from("./data/loamspine"),
            auto_rollup_threshold: Some(10_000),
            replication_enabled: false,
            log_level: "info".to_string(),
            discovery: DiscoveryConfig::default(),
        }
    }
}

impl LoamSpineConfig {
    /// Create a new configuration with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    /// Set the storage path.
    #[must_use]
    pub fn with_storage_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.storage_path = path.into();
        self
    }

    /// Set the auto-rollup threshold.
    #[must_use]
    pub const fn with_auto_rollup(mut self, threshold: u64) -> Self {
        self.auto_rollup_threshold = Some(threshold);
        self
    }

    /// Enable replication.
    #[must_use]
    pub const fn with_replication(mut self, enabled: bool) -> Self {
        self.replication_enabled = enabled;
        self
    }

    /// Set discovery configuration.
    #[must_use]
    pub fn with_discovery(mut self, discovery: DiscoveryConfig) -> Self {
        self.discovery = discovery;
        self
    }

    /// Enable discovery service (universal adapter).
    ///
    /// # Examples
    ///
    /// ```
    /// use loam_spine_core::config::LoamSpineConfig;
    ///
    /// let config = LoamSpineConfig::new("MySpine")
    ///     .with_discovery_service("http://discovery.example.com:8082");
    /// ```
    #[must_use]
    pub fn with_discovery_service(mut self, endpoint: impl Into<String>) -> Self {
        self.discovery.discovery_enabled = true;
        self.discovery.discovery_endpoint = Some(endpoint.into());
        self
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = LoamSpineConfig::default();
        assert_eq!(config.name, "LoamSpine");
        assert!(!config.replication_enabled);
        assert_eq!(config.auto_rollup_threshold, Some(10_000));
    }

    #[test]
    fn config_builder() {
        let config = LoamSpineConfig::new("TestSpine")
            .with_storage_path("/tmp/test")
            .with_auto_rollup(5000)
            .with_replication(true);

        assert_eq!(config.name, "TestSpine");
        assert_eq!(config.storage_path, PathBuf::from("/tmp/test"));
        assert_eq!(config.auto_rollup_threshold, Some(5000));
        assert!(config.replication_enabled);
    }

    #[test]
    fn config_with_discovery() {
        let discovery = DiscoveryConfig {
            discovery_enabled: false,
            ..DiscoveryConfig::default()
        };
        let config = LoamSpineConfig::new("Test").with_discovery(discovery);
        assert!(!config.discovery.discovery_enabled);
    }

    #[test]
    fn config_with_discovery_service() {
        let config =
            LoamSpineConfig::new("Test").with_discovery_service("http://registry.local:8082");
        assert!(config.discovery.discovery_enabled);
        assert_eq!(
            config.discovery.discovery_endpoint.as_deref(),
            Some("http://registry.local:8082")
        );
    }

    #[test]
    fn heartbeat_retry_config_default() {
        let retry = HeartbeatRetryConfig::default();
        assert_eq!(retry.backoff_seconds, vec![10, 30, 60, 120]);
        assert_eq!(retry.max_failures_before_degraded, 3);
        assert_eq!(retry.max_failures_total, 10);
    }

    #[test]
    fn discovery_config_default_methods() {
        let config = DiscoveryConfig::default();
        assert!(config.discovery_enabled);
        assert!(config.auto_advertise);
        assert_eq!(config.heartbeat_interval_seconds, 60);
        assert!(config.methods.contains(&DiscoveryMethod::Environment));
        assert!(config.methods.contains(&DiscoveryMethod::ServiceRegistry));
    }

    #[test]
    fn discovery_method_serde_roundtrip() {
        let methods = vec![
            DiscoveryMethod::Environment,
            DiscoveryMethod::ServiceRegistry,
            DiscoveryMethod::Mdns,
            DiscoveryMethod::LocalBinaries,
            DiscoveryMethod::ConfigFile,
            DiscoveryMethod::Fallback,
        ];
        let json = serde_json::to_string(&methods).unwrap();
        let parsed: Vec<DiscoveryMethod> = serde_json::from_str(&json).unwrap();
        assert_eq!(methods, parsed);
    }

    #[test]
    fn discovery_method_songbird_alias() {
        let json = r#""songbird""#;
        let method: DiscoveryMethod = serde_json::from_str(json).unwrap();
        assert_eq!(method, DiscoveryMethod::ServiceRegistry);
    }

    #[test]
    fn loamspine_config_serde_roundtrip() {
        let config = LoamSpineConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: LoamSpineConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, config.name);
        assert_eq!(parsed.storage_path, config.storage_path);
    }
}
