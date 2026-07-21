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
mod btsp_ops;
mod certificate_ops;
mod entry_ops;
mod integration_ops;
mod proof_ops;
mod spine_ops;
mod trust_ops;

use crate::error::{ApiError, ApiResult};
use crate::types::*;
use loam_spine_core::service::LoamSpineService as CoreService;
use loam_spine_core::traits::crypto_provider::JsonRpcCryptoSigner;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// RPC service implementation backed by the core `LoamSpineService`.
///
/// When `tower_signer` is configured (via `TOWER_SIGNER_SOCKET`), all entry
/// appends are signed via the tower signer's `crypto.sign_ed25519` and the
/// signature is stored in entry metadata (`tower_signature`, `tower_signature_alg`).
///
/// BTSP Phase 3: `btsp_sessions` stores handshake keys keyed by session ID.
/// When a `btsp.negotiate` request arrives, the handler looks up the key,
/// derives `SessionKeys` via HKDF, and returns `cipher: "chacha20-poly1305"`.
#[derive(Clone)]
pub struct LoamSpineRpcService {
    core: Arc<RwLock<CoreService>>,
    tower_signer: Option<Arc<JsonRpcCryptoSigner>>,
    btsp_sessions: Arc<RwLock<HashMap<String, [u8; 32]>>>,
    started_at: std::time::Instant,
}

impl LoamSpineRpcService {
    /// Create a new RPC service.
    #[must_use]
    pub fn new(core: CoreService) -> Self {
        Self {
            core: Arc::new(RwLock::new(core)),
            tower_signer: None,
            btsp_sessions: Arc::new(RwLock::new(HashMap::new())),
            started_at: std::time::Instant::now(),
        }
    }

    /// Create with default core service.
    #[must_use]
    pub fn default_service() -> Self {
        Self::new(CoreService::new())
    }

    /// Set the Tower crypto signer for entry signing delegation.
    ///
    /// When set, all `entry.append` and `session.commit` operations will
    /// sign the entry via the tower signer's `crypto.sign_ed25519` before persisting.
    #[must_use]
    pub fn with_tower_signer(mut self, signer: Arc<JsonRpcCryptoSigner>) -> Self {
        self.tower_signer = Some(signer);
        self
    }

    /// Register a BTSP session's handshake key for Phase 3 negotiation.
    ///
    /// Called after a successful BTSP handshake when the verify response
    /// includes a Tower-provided `session_key`.
    pub async fn register_btsp_session(
        &self,
        session_id: impl Into<String>,
        handshake_key: [u8; 32],
    ) {
        self.btsp_sessions
            .write()
            .await
            .insert(session_id.into(), handshake_key);
    }

    /// Look up a BTSP session's handshake key by session ID.
    pub(crate) async fn get_btsp_handshake_key(&self, session_id: &str) -> Option<[u8; 32]> {
        self.btsp_sessions.read().await.get(session_id).copied()
    }

    /// Number of active BTSP sessions.
    pub async fn btsp_sessions_count(&self) -> usize {
        self.btsp_sessions.read().await.len()
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
    /// Queries storage with a 5-second timeout. Returns `Unhealthy` if the
    /// storage lock times out (indicates deadlock or extreme contention).
    ///
    /// # Errors
    ///
    /// Returns error if health check fails.
    pub async fn health_check(
        &self,
        request: HealthCheckRequest,
    ) -> ApiResult<HealthCheckResponse> {
        let storage_probe = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let core = self.core().await;
            (core.spine_count().await, core.entry_count().await)
        })
        .await;

        let (status, components) = match storage_probe {
            Ok((spine_count, entry_count)) => (
                HealthStatus::Healthy,
                vec![loam_spine_core::primal::ComponentHealth::healthy(format!(
                    "storage: {spine_count} spines, {entry_count} entries"
                ))],
            ),
            Err(_) => (
                HealthStatus::Unhealthy {
                    reason: "storage check timed out".into(),
                },
                vec![loam_spine_core::primal::ComponentHealth::unhealthy(
                    "storage",
                    "lock timed out — possible deadlock or extreme contention",
                )],
            ),
        };

        let report = if request.include_details {
            Some(HealthReport {
                name: loam_spine_core::primal_names::SELF_ID.to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                status: status.clone(),
                uptime_secs: Some(self.started_at.elapsed().as_secs()),
                components,
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
    pub async fn permanence_healthy(&self) -> serde_json::Value {
        let core = self.core().await;
        let spine_count = core.spine_count().await;
        let entry_count = core.entry_count().await;
        let uptime_s = self.started_at.elapsed().as_secs();
        drop(core);
        serde_json::json!({
            "healthy": true,
            "spine_count": spine_count,
            "entry_count": entry_count,
            "uptime_s": uptime_s,
        })
    }

    /// Readiness probe (standard container orchestrator endpoint).
    ///
    /// Returns whether the service is ready for traffic.
    /// A 5-second timeout on the storage probe prevents hung locks from
    /// reporting false readiness to orchestrators.
    ///
    /// # Errors
    ///
    /// Returns error if readiness check fails.
    pub async fn readiness(&self) -> ApiResult<crate::health::ReadinessProbe> {
        let probe = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let core = self.core().await;
            core.spine_count().await
        })
        .await;

        match probe {
            Ok(spine_count) => Ok(crate::health::ReadinessProbe {
                ready: true,
                reason: Some(format!("storage accessible, {spine_count} spines")),
            }),
            Err(_) => Ok(crate::health::ReadinessProbe {
                ready: false,
                reason: Some("storage check timed out".into()),
            }),
        }
    }

    /// Sign an entry via Tower delegation (`crypto.sign_ed25519`).
    ///
    /// Signs the entry's canonical bytes (with empty metadata at this point)
    /// and stores the base64-encoded signature in entry metadata. The chain
    /// hash computed by `Spine::append` will commit to these metadata fields.
    ///
    /// Verification: strip `tower_signature` + `tower_signature_alg` from
    /// metadata, recompute `to_canonical_bytes()`, verify against the stored
    /// signature.
    pub(crate) async fn tower_sign_entry(
        entry: loam_spine_core::entry::Entry,
        signer: &JsonRpcCryptoSigner,
    ) -> ApiResult<loam_spine_core::entry::Entry> {
        use loam_spine_core::traits::Signer;

        let preimage = entry.to_canonical_bytes().map_err(ApiError::from)?;
        let signature = signer.sign(&preimage).await.map_err(ApiError::from)?;
        let sig_b64 = signature.to_base64();

        Ok(entry
            .with_metadata("tower_signature", sig_b64)
            .with_metadata("tower_signature_alg", "ed25519"))
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "tests use expect for concise error paths"
)]
#[path = "service_tests.rs"]
mod tests;

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "tests use expect for concise error paths"
)]
#[path = "service_tests_integration.rs"]
mod tests_integration;

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "tests use expect for concise error paths"
)]
#[path = "service_tests_btsp.rs"]
mod tests_btsp;

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "tests use expect for concise error paths"
)]
#[path = "service_tests_tower_signing.rs"]
mod tests_tower_signing;
