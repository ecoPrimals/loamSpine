// SPDX-License-Identifier: AGPL-3.0-or-later

//! Health check endpoints for `LoamSpine`.
//!
//! Provides standard health check endpoints compatible with container orchestrators
//! (Kubernetes, Nomad, Docker Swarm) and service meshes (Consul, etcd, etc.):
//! - `/health` - Detailed health status
//! - `/health/live` - Liveness probe (is process alive?)
//! - `/health/ready` - Readiness probe (ready for traffic?)

use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime};

/// Cached version string — initialized once from compile-time `CARGO_PKG_VERSION`.
static VERSION_CACHE: OnceLock<String> = OnceLock::new();

/// Cached capability strings — initialized once from the canonical ADVERTISED set.
static CAPABILITIES_CACHE: OnceLock<Vec<String>> = OnceLock::new();

fn cached_version() -> &'static str {
    VERSION_CACHE.get_or_init(|| env!("CARGO_PKG_VERSION").to_string())
}

fn cached_capabilities() -> &'static [String] {
    CAPABILITIES_CACHE.get_or_init(|| {
        loam_spine_core::capabilities::identifiers::loamspine::ADVERTISED
            .iter()
            .map(|&s| s.to_string())
            .collect()
    })
}

/// Structured error type for health check failures.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HealthError {
    /// Storage backend is unavailable.
    #[error("storage backend unavailable")]
    StorageUnavailable,

    /// Discovery service is unavailable.
    #[error("discovery service unavailable")]
    DiscoveryUnavailable,
}

/// Health status response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall service status.
    pub status: ServiceStatus,

    /// Service version.
    pub version: String,

    /// Uptime in seconds.
    pub uptime_seconds: u64,

    /// Dependency health.
    pub dependencies: DependencyHealth,

    /// Advertised capabilities.
    pub capabilities: Vec<String>,
}

/// Service status.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    /// Service is fully operational.
    Healthy,

    /// Service is running but some capabilities unavailable.
    Degraded,

    /// Service has critical errors.
    Error,
}

/// Dependency health status.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DependencyHealth {
    /// Storage backend health.
    pub storage: bool,

    /// Discovery service health (universal adapter).
    ///
    /// `None` indicates discovery is not configured.
    /// `Some(true)` indicates discovery service is healthy.
    /// `Some(false)` indicates discovery service is unavailable.
    pub discovery: Option<bool>,
}

/// Liveness probe response.
///
/// Standard liveness endpoint compatible with container orchestrators,
/// service meshes, and the Semantic Method Naming Standard v2.1
/// (`health.liveness` returns `{"status": "alive"}`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LivenessProbe {
    /// Process status — always `"alive"` when reachable.
    pub status: String,
}

/// Readiness probe response.
///
/// Standard readiness endpoint compatible with container orchestrators
/// and service meshes. Returns whether the service is ready for traffic.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadinessProbe {
    /// Is the service ready for traffic?
    pub ready: bool,

    /// Reason if not ready.
    pub reason: Option<String>,
}

/// Health check function for storage backend.
///
/// This is a trait object that can check storage health without coupling
/// to specific storage implementations.
pub type StorageHealthCheck = Arc<dyn Fn() -> bool + Send + Sync>;

/// Health check function for discovery service.
///
/// This is a trait object that can check discovery service health without
/// coupling to specific discovery implementations.
pub type DiscoveryHealthCheck = Arc<dyn Fn() -> Option<bool> + Send + Sync>;

/// Health checker for `LoamSpine` service.
///
/// Uses dependency injection for health checks to maintain capability-based
/// architecture without hardcoding specific implementations.
pub struct HealthChecker {
    /// Service start time.
    start_time: SystemTime,

    /// Storage health check function (optional).
    storage_check: Option<StorageHealthCheck>,

    /// Discovery service health check function (optional).
    discovery_check: Option<DiscoveryHealthCheck>,
}

impl HealthChecker {
    /// Create a new health checker with no health check functions.
    ///
    /// This is suitable for basic health checks that only verify the process is alive.
    #[must_use]
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
            storage_check: None,
            discovery_check: None,
        }
    }

    /// Create a health checker with storage health check.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use loam_spine_api::health::HealthChecker;
    /// use std::sync::Arc;
    ///
    /// let storage_health = Arc::new(|| {
    ///     // Check if storage is accessible
    ///     true  // Replace with actual storage ping
    /// });
    ///
    /// let checker = HealthChecker::with_storage_check(storage_health);
    /// ```
    #[must_use]
    pub fn with_storage_check(storage_check: StorageHealthCheck) -> Self {
        Self {
            start_time: SystemTime::now(),
            storage_check: Some(storage_check),
            discovery_check: None,
        }
    }

    /// Create a health checker with both storage and discovery health checks.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use loam_spine_api::health::HealthChecker;
    /// use std::sync::Arc;
    ///
    /// let storage_health = Arc::new(|| true);
    /// let discovery_health = Arc::new(|| Some(true));
    ///
    /// let checker = HealthChecker::with_checks(storage_health, discovery_health);
    /// ```
    #[must_use]
    pub fn with_checks(
        storage_check: StorageHealthCheck,
        discovery_check: DiscoveryHealthCheck,
    ) -> Self {
        Self {
            start_time: SystemTime::now(),
            storage_check: Some(storage_check),
            discovery_check: Some(discovery_check),
        }
    }

    /// Get detailed health status.
    ///
    /// # Errors
    ///
    /// Returns error if health check fails.
    #[expect(
        clippy::unused_async,
        reason = "will become truly async when health probes query network"
    )]
    pub async fn check_health(&self) -> Result<HealthStatus, HealthError> {
        // Check storage health
        let storage_healthy = self.check_storage();

        // Check discovery service health (optional dependency)
        let discovery_healthy = self.check_discovery();

        // Determine overall status
        let status = match (storage_healthy, discovery_healthy) {
            (true, Some(true) | None) => ServiceStatus::Healthy,
            (true, Some(false)) => ServiceStatus::Degraded, // Can continue without discovery
            (false, _) => ServiceStatus::Error,             // Storage is critical
        };

        // Calculate uptime
        let uptime = self
            .start_time
            .elapsed()
            .unwrap_or(Duration::ZERO)
            .as_secs();

        Ok(HealthStatus {
            status,
            version: cached_version().to_string(),
            uptime_seconds: uptime,
            dependencies: DependencyHealth {
                storage: storage_healthy,
                discovery: discovery_healthy,
            },
            capabilities: cached_capabilities().to_vec(),
        })
    }

    /// Check liveness (is process alive?).
    #[must_use]
    pub fn check_liveness(&self) -> LivenessProbe {
        LivenessProbe {
            status: "alive".into(),
        }
    }

    /// Check readiness (ready for traffic?).
    ///
    /// # Errors
    ///
    /// Returns error if readiness check fails.
    #[expect(
        clippy::unused_async,
        reason = "will become truly async when readiness probes query storage"
    )]
    pub async fn check_readiness(&self) -> Result<ReadinessProbe, HealthError> {
        // Check critical dependencies
        let storage_healthy = self.check_storage();

        if storage_healthy {
            Ok(ReadinessProbe {
                ready: true,
                reason: None,
            })
        } else {
            Ok(ReadinessProbe {
                ready: false,
                reason: Some("Storage backend unavailable".to_string()),
            })
        }
    }

    /// Check storage backend health.
    ///
    /// Executes the configured storage health check function, or returns `true`
    /// if no check function is configured (optimistic default).
    fn check_storage(&self) -> bool {
        self.storage_check.as_deref().is_none_or(|check| check())
    }

    /// Check discovery service health (universal adapter).
    ///
    /// Executes the configured discovery health check function, or returns `None`
    /// if no check function is configured (discovery not enabled).
    ///
    /// Returns:
    /// - `None`: Discovery service not configured
    /// - `Some(true)`: Discovery service is healthy
    /// - `Some(false)`: Discovery service is unavailable
    fn check_discovery(&self) -> Option<bool> {
        self.discovery_check.as_ref().and_then(|check| check())
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Example: Create health check functions from storage and discovery dependencies.
///
/// This shows how to integrate with actual storage backends and discovery clients
/// while maintaining capability-based architecture.
///
/// # Example with Sled storage
///
/// ```no_run
/// use loam_spine_api::health::HealthChecker;
/// use std::sync::Arc;
///
/// // Example: Create storage health check
/// // In production, you would pass the actual storage backend
/// let storage_health = Arc::new(|| {
///     // Try to verify storage is accessible
///     // For Sled: check if database can be opened/pinged
///     // For in-memory: always return true
///     true
/// });
///
/// // Example: Create discovery service health check
/// let discovery_health = Arc::new(|| {
///     // Try to ping discovery service
///     // Return None if not configured
///     // Return Some(true/false) based on connectivity
///     Some(true)
/// });
///
/// let checker = HealthChecker::with_checks(storage_health, discovery_health);
/// ```
#[cfg(doc)]
pub fn example_health_checks() {}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_checker_creation() {
        let checker = HealthChecker::new();
        let health = checker.check_health().await;
        assert!(health.is_ok());
    }

    #[tokio::test]
    async fn liveness_probe_always_alive() {
        let checker = HealthChecker::new();
        let liveness = checker.check_liveness();
        assert_eq!(liveness.status, "alive");
    }

    #[tokio::test]
    async fn readiness_probe_ready_when_storage_healthy() {
        let checker = HealthChecker::new();
        let readiness = checker.check_readiness().await;
        assert!(readiness.is_ok());
        assert!(readiness.unwrap().ready);
    }

    #[tokio::test]
    async fn health_status_includes_version() {
        let checker = HealthChecker::new();
        let health = checker.check_health().await.unwrap();
        assert!(!health.version.is_empty());
    }

    #[tokio::test]
    async fn health_status_includes_capabilities() {
        let checker = HealthChecker::new();
        let health = checker.check_health().await.unwrap();
        assert!(!health.capabilities.is_empty());
        assert!(health.capabilities.contains(
            &loam_spine_core::capabilities::identifiers::loamspine::PERMANENT_LEDGER.to_string()
        ));
    }

    #[tokio::test(start_paused = true)]
    async fn health_status_tracks_uptime() {
        let checker = HealthChecker::new();

        // First check - should be very recent
        let health1 = checker.check_health().await.unwrap();
        assert!(
            health1.uptime_seconds < 60,
            "Initial uptime should be less than a minute"
        );

        tokio::time::advance(Duration::from_millis(100)).await;

        // Second check - uptime should have increased
        let health2 = checker.check_health().await.unwrap();
        assert!(
            health2.uptime_seconds >= health1.uptime_seconds,
            "Uptime should increase or stay the same"
        );
        assert!(
            health2.uptime_seconds < 60,
            "Uptime should still be less than a minute"
        );
    }

    #[test]
    fn service_status_serialization() {
        let status = ServiceStatus::Healthy;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"healthy\"");

        let status = ServiceStatus::Degraded;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"degraded\"");

        let status = ServiceStatus::Error;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"error\"");
    }

    #[tokio::test]
    async fn health_checker_with_storage_check() {
        let storage = Arc::new(|| true);
        let checker = HealthChecker::with_storage_check(storage);
        let health = checker.check_health().await.unwrap();
        assert_eq!(health.status, ServiceStatus::Healthy);
        assert!(health.dependencies.storage);
    }

    #[tokio::test]
    async fn health_checker_with_checks_all_healthy() {
        let storage = Arc::new(|| true);
        let discovery = Arc::new(|| Some(true));
        let checker = HealthChecker::with_checks(storage, discovery);
        let health = checker.check_health().await.unwrap();
        assert_eq!(health.status, ServiceStatus::Healthy);
        assert!(health.dependencies.storage);
        assert_eq!(health.dependencies.discovery, Some(true));
    }

    #[tokio::test]
    async fn health_checker_degraded_when_discovery_unavailable() {
        let storage = Arc::new(|| true);
        let discovery = Arc::new(|| Some(false));
        let checker = HealthChecker::with_checks(storage, discovery);
        let health = checker.check_health().await.unwrap();
        assert_eq!(health.status, ServiceStatus::Degraded);
        assert_eq!(health.dependencies.discovery, Some(false));
    }

    #[tokio::test]
    async fn health_checker_error_when_storage_unavailable() {
        let storage = Arc::new(|| false);
        let checker = HealthChecker::with_storage_check(storage);
        let health = checker.check_health().await.unwrap();
        assert_eq!(health.status, ServiceStatus::Error);
        assert!(!health.dependencies.storage);
    }

    #[tokio::test]
    async fn readiness_not_ready_when_storage_down() {
        let storage = Arc::new(|| false);
        let checker = HealthChecker::with_storage_check(storage);
        let readiness = checker.check_readiness().await.unwrap();
        assert!(!readiness.ready);
        assert!(readiness.reason.is_some());
    }

    #[test]
    fn health_checker_default_same_as_new() {
        let checker = HealthChecker::default();
        let liveness = checker.check_liveness();
        assert_eq!(liveness.status, "alive");
    }
}
