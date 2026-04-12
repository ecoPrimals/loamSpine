// SPDX-License-Identifier: AGPL-3.0-or-later

//! LoamSpine configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Validated log level for tracing integration.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Error-level only.
    Error,
    /// Warnings and errors.
    Warn,
    /// Informational (default).
    #[default]
    Info,
    /// Debug diagnostics.
    Debug,
    /// Full trace output.
    Trace,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => f.write_str("error"),
            Self::Warn => f.write_str("warn"),
            Self::Info => f.write_str("info"),
            Self::Debug => f.write_str("debug"),
            Self::Trace => f.write_str("trace"),
        }
    }
}

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
    pub log_level: LogLevel,

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
/// - mDNS / DNS SRV (zero-config and production)
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
    /// - Any RFC 2782 SRV-capable registry
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

    /// tarpc endpoint for structured RPC (primal-to-primal communication).
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

    /// Environment variable overrides for infant discovery (e.g. tests injecting `DISCOVERY_ENDPOINT`).
    #[serde(default)]
    pub env_overrides: HashMap<String, String>,
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
    /// - Any RFC 2782 SRV-capable registry
    /// - Consul
    /// - etcd
    /// - Custom implementations
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
                DiscoveryMethod::ServiceRegistry, // Generic service registry (Consul, etcd, etc.)
                DiscoveryMethod::LocalBinaries,
                DiscoveryMethod::Fallback,
            ],

            env_overrides: HashMap::new(),
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
            log_level: LogLevel::default(),
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
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
#[path = "config_tests.rs"]
mod tests;
