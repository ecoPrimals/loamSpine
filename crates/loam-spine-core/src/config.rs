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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enable Songbird discovery.
    pub songbird_enabled: bool,

    /// Songbird discovery endpoint.
    pub songbird_endpoint: Option<String>,

    /// tarpc endpoint for binary RPC (primal-to-primal communication).
    pub tarpc_endpoint: String,

    /// JSON-RPC 2.0 endpoint for external clients.
    pub jsonrpc_endpoint: String,

    /// Auto-advertise to Songbird on startup.
    pub auto_advertise: bool,

    /// Heartbeat interval for re-advertising (seconds).
    pub heartbeat_interval_seconds: u64,

    /// Heartbeat retry configuration.
    #[serde(default)]
    pub heartbeat_retry: HeartbeatRetryConfig,

    /// Discovery methods (in priority order).
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
    /// Songbird universal adapter.
    Songbird,
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
            songbird_enabled: true,
            songbird_endpoint: Some("http://localhost:8082".to_string()),
            tarpc_endpoint: "http://localhost:9001".to_string(),
            jsonrpc_endpoint: "http://localhost:8080".to_string(),
            auto_advertise: true,
            heartbeat_interval_seconds: 60,
            heartbeat_retry: HeartbeatRetryConfig::default(),
            methods: vec![
                DiscoveryMethod::Environment,
                DiscoveryMethod::Songbird,
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

    /// Enable Songbird discovery.
    #[must_use]
    pub fn with_songbird(mut self, endpoint: impl Into<String>) -> Self {
        self.discovery.songbird_enabled = true;
        self.discovery.songbird_endpoint = Some(endpoint.into());
        self
    }
}

#[cfg(test)]
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
}
