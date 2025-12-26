//! Health check endpoints for `LoamSpine`.
//!
//! Provides standard health check endpoints compatible with container orchestrators
//! (Kubernetes, Nomad, Docker Swarm) and service meshes (Consul, etcd, etc.):
//! - `/health` - Detailed health status
//! - `/health/live` - Liveness probe (is process alive?)
//! - `/health/ready` - Readiness probe (ready for traffic?)

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

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

    /// DEPRECATED: Use `discovery` instead.
    ///
    /// This field is maintained for backward compatibility and will be removed in v1.0.0.
    #[deprecated(since = "0.7.0", note = "Use discovery field instead")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub songbird: Option<bool>,
}

/// Liveness probe response.
///
/// Standard liveness endpoint compatible with container orchestrators
/// and service meshes. Returns whether the process is alive.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LivenessProbe {
    /// Is the process alive?
    pub alive: bool,
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

/// Health checker for `LoamSpine` service.
pub struct HealthChecker {
    /// Service start time.
    start_time: SystemTime,
}

impl HealthChecker {
    /// Create a new health checker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
        }
    }

    /// Get detailed health status.
    ///
    /// # Errors
    ///
    /// Returns error if health check fails.
    #[allow(clippy::unused_async)] // Async for future extensibility
    pub async fn check_health(&self) -> Result<HealthStatus, String> {
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
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: uptime,
            dependencies: DependencyHealth {
                storage: storage_healthy,
                discovery: discovery_healthy,
                #[allow(deprecated)]
                songbird: discovery_healthy, // Backward compatibility
            },
            capabilities: vec![
                "persistent-ledger".to_string(),
                "waypoint-anchoring".to_string(),
                "certificate-manager".to_string(),
                "proof-generation".to_string(),
            ],
        })
    }

    /// Check liveness (is process alive?).
    #[must_use]
    pub fn check_liveness(&self) -> LivenessProbe {
        // If we can execute this code, we're alive
        LivenessProbe { alive: true }
    }

    /// Check readiness (ready for traffic?).
    ///
    /// # Errors
    ///
    /// Returns error if readiness check fails.
    #[allow(clippy::unused_async)] // Async for future extensibility
    pub async fn check_readiness(&self) -> Result<ReadinessProbe, String> {
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
    fn check_storage(&self) -> bool {
        // TODO: Implement actual storage health check
        // For now, assume storage is healthy if we can reach this code
        let _ = self; // Use self to satisfy clippy
        true
    }

    /// Check discovery service health (universal adapter).
    fn check_discovery(&self) -> Option<bool> {
        // TODO: Implement actual discovery service health check
        // Return None if discovery is not configured
        // Return Some(true/false) based on connectivity
        let _ = self; // Use self to satisfy clippy
        None
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
        assert!(liveness.alive);
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
        assert!(health
            .capabilities
            .contains(&"persistent-ledger".to_string()));
    }

    #[tokio::test]
    async fn health_status_tracks_uptime() {
        let checker = HealthChecker::new();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let health = checker.check_health().await.unwrap();
        // Uptime should be at least 100ms (0 seconds)
        assert!(health.uptime_seconds < 60); // Sanity check: less than a minute
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
}
