// SPDX-License-Identifier: AGPL-3.0-or-later

//! Primal lifecycle and health traits.
//!
//! These traits define the standard interface for all ecoPrimals services.
//! Originally from sourDough, inlined here for self-containment.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Primal lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PrimalState {
    /// Initial state after construction.
    #[default]
    Created,
    /// Service is starting up.
    Starting,
    /// Service is running and healthy.
    Running,
    /// Service is shutting down.
    Stopping,
    /// Service has stopped.
    Stopped,
    /// Service encountered an error.
    Failed,
}

impl PrimalState {
    /// Check if the primal is in a running state.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if the primal can accept requests.
    #[must_use]
    pub const fn is_available(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if the primal is in a terminal state.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Stopped | Self::Failed)
    }
}

impl fmt::Display for PrimalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Starting => write!(f, "starting"),
            Self::Running => write!(f, "running"),
            Self::Stopping => write!(f, "stopping"),
            Self::Stopped => write!(f, "stopped"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Primal lifecycle trait.
///
/// All ecoPrimals implement this trait for unified lifecycle management.
pub trait PrimalLifecycle {
    /// Get the current state.
    fn state(&self) -> PrimalState;

    /// Start the primal service.
    ///
    /// # Errors
    ///
    /// Returns an error if the service fails to start.
    fn start(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;

    /// Stop the primal service.
    ///
    /// # Errors
    ///
    /// Returns an error if the service fails to stop cleanly.
    fn stop(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;

    /// Reload configuration without full restart.
    ///
    /// # Errors
    ///
    /// Returns an error if reload fails.
    fn reload(&mut self) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send {
        async { Ok(()) }
    }
}

/// Health status of a primal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Service is healthy.
    Healthy,
    /// Service is degraded but functional.
    Degraded {
        /// Reason for degradation.
        reason: String,
    },
    /// Service is unhealthy.
    Unhealthy {
        /// Reason for unhealthy state.
        reason: String,
    },
}

impl HealthStatus {
    /// Check if the status is healthy.
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Check if the status is degraded.
    #[must_use]
    pub const fn is_degraded(&self) -> bool {
        matches!(self, Self::Degraded { .. })
    }

    /// Check if the status is unhealthy.
    #[must_use]
    pub const fn is_unhealthy(&self) -> bool {
        matches!(self, Self::Unhealthy { .. })
    }
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded { reason } => write!(f, "degraded: {reason}"),
            Self::Unhealthy { reason } => write!(f, "unhealthy: {reason}"),
        }
    }
}

/// Health report for a primal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Service name.
    pub name: String,
    /// Service version.
    pub version: String,
    /// Current health status.
    pub status: HealthStatus,
    /// Uptime in seconds.
    pub uptime_secs: Option<u64>,
    /// Component health checks.
    pub components: Vec<ComponentHealth>,
}

impl HealthReport {
    /// Create a new health report.
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            status: HealthStatus::Healthy,
            uptime_secs: None,
            components: Vec::new(),
        }
    }

    /// Set the health status.
    #[must_use]
    pub fn with_status(mut self, status: HealthStatus) -> Self {
        self.status = status;
        self
    }

    /// Set the uptime.
    #[must_use]
    pub const fn with_uptime(mut self, secs: u64) -> Self {
        self.uptime_secs = Some(secs);
        self
    }

    /// Add a component health check.
    #[must_use]
    pub fn with_component(mut self, component: ComponentHealth) -> Self {
        self.components.push(component);
        self
    }
}

/// Health of a subcomponent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name.
    pub name: String,
    /// Component status.
    pub status: HealthStatus,
    /// Optional message.
    pub message: Option<String>,
}

impl ComponentHealth {
    /// Create a healthy component.
    #[must_use]
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            message: None,
        }
    }

    /// Create a degraded component.
    #[must_use]
    pub fn degraded(name: impl Into<String>, reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self {
            name: name.into(),
            status: HealthStatus::Degraded {
                reason: reason.clone(),
            },
            message: Some(reason),
        }
    }

    /// Create an unhealthy component.
    #[must_use]
    pub fn unhealthy(name: impl Into<String>, reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy {
                reason: reason.clone(),
            },
            message: Some(reason),
        }
    }
}

/// Primal health trait.
///
/// All ecoPrimals implement this trait for health monitoring.
pub trait PrimalHealth {
    /// Get the current health status.
    fn health_status(&self) -> HealthStatus;

    /// Perform a comprehensive health check.
    ///
    /// # Errors
    ///
    /// Returns an error if the health check itself fails.
    fn health_check(
        &self,
    ) -> impl std::future::Future<Output = Result<HealthReport, PrimalError>> + Send;
}

/// Primal errors.
#[derive(Debug, Error)]
pub enum PrimalError {
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Initialization error.
    #[error("initialization error: {0}")]
    Init(String),

    /// Runtime error.
    #[error("runtime error: {0}")]
    Runtime(String),

    /// Shutdown error.
    #[error("shutdown error: {0}")]
    Shutdown(String),

    /// Health check error.
    #[error("health check error: {0}")]
    HealthCheck(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::SECONDS_PER_HOUR;

    #[test]
    fn primal_state_display() {
        assert_eq!(PrimalState::Created.to_string(), "created");
        assert_eq!(PrimalState::Starting.to_string(), "starting");
        assert_eq!(PrimalState::Running.to_string(), "running");
        assert_eq!(PrimalState::Stopping.to_string(), "stopping");
        assert_eq!(PrimalState::Stopped.to_string(), "stopped");
        assert_eq!(PrimalState::Failed.to_string(), "failed");
    }

    #[test]
    fn primal_state_checks() {
        assert!(PrimalState::Running.is_running());
        assert!(!PrimalState::Created.is_running());
        assert!(PrimalState::Stopped.is_terminal());
        assert!(PrimalState::Failed.is_terminal());
        assert!(!PrimalState::Running.is_terminal());

        // Test is_available
        assert!(PrimalState::Running.is_available());
        assert!(!PrimalState::Created.is_available());
        assert!(!PrimalState::Stopped.is_available());
    }

    #[test]
    fn health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(
            HealthStatus::Degraded {
                reason: "test".into()
            }
            .to_string(),
            "degraded: test"
        );
        assert_eq!(
            HealthStatus::Unhealthy {
                reason: "failed".into()
            }
            .to_string(),
            "unhealthy: failed"
        );
    }

    #[test]
    fn health_status_checks() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(!HealthStatus::Healthy.is_degraded());
        assert!(!HealthStatus::Healthy.is_unhealthy());

        let degraded = HealthStatus::Degraded {
            reason: "test".into(),
        };
        assert!(!degraded.is_healthy());
        assert!(degraded.is_degraded());
        assert!(!degraded.is_unhealthy());

        let unhealthy = HealthStatus::Unhealthy {
            reason: "test".into(),
        };
        assert!(!unhealthy.is_healthy());
        assert!(!unhealthy.is_degraded());
        assert!(unhealthy.is_unhealthy());
    }

    #[test]
    fn health_report_builder() {
        let report = HealthReport::new("test", "0.1.0")
            .with_status(HealthStatus::Healthy)
            .with_uptime(SECONDS_PER_HOUR)
            .with_component(ComponentHealth::healthy("storage"));

        assert_eq!(report.name, "test");
        assert_eq!(report.version, "0.1.0");
        assert!(report.status.is_healthy());
        assert_eq!(report.uptime_secs, Some(SECONDS_PER_HOUR));
        assert_eq!(report.components.len(), 1);
    }

    #[test]
    fn component_health_variants() {
        let healthy = ComponentHealth::healthy("storage");
        assert!(healthy.status.is_healthy());
        assert!(healthy.message.is_none());

        let degraded = ComponentHealth::degraded("cache", "high latency");
        assert!(degraded.status.is_degraded());
        assert_eq!(degraded.message, Some("high latency".to_string()));

        let unhealthy = ComponentHealth::unhealthy("database", "connection failed");
        assert!(unhealthy.status.is_unhealthy());
        assert_eq!(unhealthy.message, Some("connection failed".to_string()));
    }

    #[test]
    fn primal_error_display() {
        let config_err = PrimalError::Config("bad config".into());
        assert!(config_err.to_string().contains("configuration error"));

        let init_err = PrimalError::Init("failed to start".into());
        assert!(init_err.to_string().contains("initialization error"));

        let runtime_err = PrimalError::Runtime("crashed".into());
        assert!(runtime_err.to_string().contains("runtime error"));

        let shutdown_err = PrimalError::Shutdown("timeout".into());
        assert!(shutdown_err.to_string().contains("shutdown error"));

        let health_err = PrimalError::HealthCheck("check failed".into());
        assert!(health_err.to_string().contains("health check error"));

        let internal_err = PrimalError::Internal("bug".into());
        assert!(internal_err.to_string().contains("internal error"));
    }
}
