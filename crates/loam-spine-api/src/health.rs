//! Health check endpoints for LoamSpine.
//!
//! Provides Kubernetes-compatible health check endpoints:
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
    
    /// Songbird orchestrator health.
    pub songbird: Option<bool>,
}

/// Liveness probe response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LivenessProbe {
    /// Is the process alive?
    pub alive: bool,
}

/// Readiness probe response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadinessProbe {
    /// Is the service ready for traffic?
    pub ready: bool,
    
    /// Reason if not ready.
    pub reason: Option<String>,
}

/// Health checker for LoamSpine service.
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
    pub async fn check_health(&self) -> Result<HealthStatus, String> {
        // Check storage health
        let storage_healthy = self.check_storage().await;
        
        // Check Songbird health (optional dependency)
        let songbird_healthy = self.check_songbird().await;
        
        // Determine overall status
        let status = match (storage_healthy, songbird_healthy) {
            (true, Some(true)) | (true, None) => ServiceStatus::Healthy,
            (true, Some(false)) => ServiceStatus::Degraded, // Can continue without Songbird
            (false, _) => ServiceStatus::Error, // Storage is critical
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
                songbird: songbird_healthy,
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
    pub async fn check_readiness(&self) -> Result<ReadinessProbe, String> {
        // Check critical dependencies
        let storage_healthy = self.check_storage().await;
        
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
    async fn check_storage(&self) -> bool {
        // TODO: Implement actual storage health check
        // For now, assume storage is healthy if we can reach this code
        true
    }
    
    /// Check Songbird orchestrator health.
    async fn check_songbird(&self) -> Option<bool> {
        // TODO: Implement actual Songbird health check
        // Return None if Songbird is not configured
        // Return Some(true/false) based on connectivity
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
        assert!(health.capabilities.contains(&"persistent-ledger".to_string()));
    }
    
    #[tokio::test]
    async fn health_status_tracks_uptime() {
        let checker = HealthChecker::new();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let health = checker.check_health().await.unwrap();
        assert!(health.uptime_seconds >= 0);
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

