//! # LoamSpine
//!
//! Permanence Layer - Selective Memory & Certificates
//!
//! ## Overview
//!
//! LoamSpine is part of the ecoPrimals ecosystem. It provides selective
//! permanence semantics on top of RhizoCrypt's DAG engine.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use loam_spine_core::LoamSpine;
//!
//! let primal = LoamSpine::new(config);
//! primal.start().await?;
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod config;
pub mod error;

use sourdough_core::{
    PrimalLifecycle, PrimalHealth, PrimalState,
    HealthStatus, health::HealthReport, PrimalError,
};

/// LoamSpine configuration.
pub use config::LoamSpineConfig;

/// LoamSpine errors.
pub use error::LoamSpineError;

/// The LoamSpine primal - Permanence Layer.
pub struct LoamSpine {
    #[allow(dead_code)]
    config: LoamSpineConfig,
    state: PrimalState,
}

impl LoamSpine {
    /// Create a new LoamSpine instance.
    #[must_use]
    pub fn new(config: LoamSpineConfig) -> Self {
        Self {
            config,
            state: PrimalState::Created,
        }
    }
}

impl PrimalLifecycle for LoamSpine {
    fn state(&self) -> PrimalState {
        self.state
    }

    async fn start(&mut self) -> Result<(), PrimalError> {
        self.state = PrimalState::Starting;
        tracing::info!("LoamSpine starting...");
        
        // TODO: Initialize resources
        
        self.state = PrimalState::Running;
        tracing::info!("LoamSpine running");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PrimalError> {
        self.state = PrimalState::Stopping;
        tracing::info!("LoamSpine stopping...");
        
        // TODO: Clean up resources
        
        self.state = PrimalState::Stopped;
        tracing::info!("LoamSpine stopped");
        Ok(())
    }
}

impl PrimalHealth for LoamSpine {
    fn health_status(&self) -> HealthStatus {
        if self.state.is_running() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy {
                reason: format!("state: {}", self.state),
            }
        }
    }

    async fn health_check(&self) -> Result<HealthReport, PrimalError> {
        Ok(HealthReport::new("LoamSpine", env!("CARGO_PKG_VERSION"))
            .with_status(self.health_status()))
    }
}
