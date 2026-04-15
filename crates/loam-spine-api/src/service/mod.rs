// SPDX-License-Identifier: AGPL-3.0-or-later

//! RPC service implementation for `LoamSpine`.
//!
//! Implements the `LoamSpineRpc` trait defined in `rpc.rs`.

// Some trait dispatch methods are async for uniform interface but don't await internally.
#![expect(
    clippy::unused_async,
    reason = "async trait methods required by interface even when impl is sync"
)]
#![allow(
    clippy::wildcard_imports,
    reason = "tarpc service macro requires wildcard imports from crate::types::*; allow not expect: unfulfilled in test target"
)]

mod anchor_ops;
mod bond_ops;
mod certificate_ops;
mod entry_ops;
mod integration_ops;
mod proof_ops;
mod spine_ops;

use crate::error::ApiResult;
use crate::types::*;
use loam_spine_core::service::LoamSpineService as CoreService;
use std::sync::Arc;
use tokio::sync::RwLock;

/// RPC service implementation backed by the core `LoamSpineService`.
#[derive(Clone)]
pub struct LoamSpineRpcService {
    core: Arc<RwLock<CoreService>>,
}

impl LoamSpineRpcService {
    /// Create a new RPC service.
    #[must_use]
    pub fn new(core: CoreService) -> Self {
        Self {
            core: Arc::new(RwLock::new(core)),
        }
    }

    /// Create with default core service.
    #[must_use]
    pub fn default_service() -> Self {
        Self::new(CoreService::new())
    }

    /// Get read access to the core service.
    pub async fn core(&self) -> tokio::sync::RwLockReadGuard<'_, CoreService> {
        self.core.read().await
    }

    /// Get write access to the core service.
    pub async fn core_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, CoreService> {
        self.core.write().await
    }

    /// Health check.
    ///
    /// # Errors
    ///
    /// Returns error if health check fails.
    pub async fn health_check(
        &self,
        request: HealthCheckRequest,
    ) -> ApiResult<HealthCheckResponse> {
        let spine_count = {
            let core = self.core().await;
            core.spine_count().await
        };

        let status = HealthStatus::Healthy;
        let report = if request.include_details {
            Some(HealthReport {
                name: loam_spine_core::primal_names::SELF_ID.to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                status: status.clone(),
                uptime_secs: Some(0),
                components: vec![loam_spine_core::primal::ComponentHealth::healthy(format!(
                    "storage: {spine_count} spines"
                ))],
            })
        } else {
            None
        };

        Ok(HealthCheckResponse { status, report })
    }

    /// Liveness probe (standard container orchestrator endpoint).
    ///
    /// Returns whether the process is alive.
    pub async fn liveness(&self) -> crate::health::LivenessProbe {
        crate::health::LivenessProbe {
            status: "alive".into(),
        }
    }

    /// Check whether the permanence layer (spine + entry storage) is healthy.
    pub async fn permanence_healthy(&self) -> bool {
        let core = self.core().await;
        core.spine_count().await;
        core.entry_count().await;
        drop(core);
        true
    }

    /// Readiness probe (standard container orchestrator endpoint).
    ///
    /// Returns whether the service is ready for traffic.
    ///
    /// # Errors
    ///
    /// Returns error if readiness check fails.
    pub async fn readiness(&self) -> ApiResult<crate::health::ReadinessProbe> {
        // Check if we can access core service
        let _core = self.core().await;

        // If we got here, we're ready
        Ok(crate::health::ReadinessProbe {
            ready: true,
            reason: None,
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "tests use expect for concise error paths"
)]
#[path = "service_tests.rs"]
mod tests;
