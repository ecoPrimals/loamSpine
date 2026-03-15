// SPDX-License-Identifier: AGPL-3.0-only

//! # `LoamSpine`
//!
//! Permanence Layer - Selective Memory & Certificates
//!
//! `LoamSpine` is the permanent ledger of the ecoPrimals ecosystem. It provides
//! immutable, sovereign storage for committed state—the "fossil record" where
//! ephemeral DAG operations compress into permanent history.
//!
//! ## Philosophy
//!
//! `LoamSpine` embodies **selective permanence**: only what is deliberately committed
//! becomes permanent. This is the complement to ephemeral storage primals' philosophy of forgetting.
//!
//! ## Design Principles
//!
//! - **Self-knowledge only**: LoamSpine knows only its own capabilities
//! - **Runtime discovery**: Other primals are discovered at runtime, not compile time
//! - **Capability-based**: Request capabilities, not specific primals
//! - **Zero unsafe**: All operations are safe Rust
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use loam_spine_core::{LoamSpine, LoamSpineConfig};
//! use loam_spine_core::primal::PrimalLifecycle;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = LoamSpineConfig::default();
//! let mut spine = LoamSpine::new(config);
//! spine.start().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Core Concepts
//!
//! - **Spine**: A sovereign, append-only ledger owned by a DID
//! - **Entry**: A single immutable record in a spine
//! - **Certificate**: A memory-bound object with ownership and history
//! - **Waypoint**: A local spine for borrowed state (slice anchoring)
//! - **Capability**: A runtime-discovered service (signing, verification, etc.)

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![forbid(unsafe_code)]
// Allow some pedantic lints that are too noisy
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::doc_markdown)] // Allow product names without backticks in docs

// Core modules
pub mod backup;
pub mod capabilities;
pub mod certificate;
pub mod config;
pub mod constants;
pub mod discovery_client;
pub mod entry;
pub mod error;
pub mod infant_discovery;
pub mod manager;
pub mod primal;
pub mod proof;
pub mod spine;
pub mod storage;
pub mod temporal;
pub mod transport;
pub mod trio_types;
pub mod types;
pub mod waypoint;

// New architecture modules
pub mod discovery;
pub mod neural_api;
pub mod resilience;
pub mod service;
pub mod sync;
pub mod traits;

// NOTE: The `integration` module was removed in v0.3.0.
// Use `traits` and `service` modules instead.

use std::time::Instant;

use primal::{
    ComponentHealth, HealthReport, HealthStatus, PrimalError, PrimalHealth, PrimalLifecycle,
    PrimalState,
};

/// `LoamSpine` configuration.
pub use config::LoamSpineConfig;

/// `LoamSpine` errors.
pub use error::{LoamSpineError, LoamSpineResult};

/// Core types.
pub use types::{
    hash_bytes,
    BraidId,
    CertificateId,
    ContentHash,
    Did,
    EntryHash,
    PayloadRef,
    PeerId,
    SessionId,
    Signature,
    SliceId,
    SpineId,
    Timestamp,
    // Size constants
    GB,
    KB,
    MB,
};

/// Entry types.
pub use entry::{Entry, EntryType, SpineConfig as EntrySpineConfig};

/// Spine types.
pub use spine::{ChainError, ChainVerification, Spine, SpineBuilder, SpineState};

/// Certificate types.
pub use certificate::{
    AcquisitionType,
    Certificate,
    CertificateHistory,
    CertificateLocation,
    CertificateMetadata,
    CertificateState,
    CertificateType,
    EscrowCondition,
    EscrowId,
    LoanInfo,
    LoanRecord,
    LoanTerms,
    MediaType,
    MintInfo,
    OwnershipRecord,
    Rarity,
    RevocationReason,
    TransferConditions,
    UsageSummary,
    // Time constants for loan durations
    SECONDS_PER_DAY,
    SECONDS_PER_HOUR,
    SECONDS_PER_MINUTE,
    SECONDS_PER_WEEK,
    SECONDS_PER_YEAR,
};

/// Proof types.
pub use proof::{
    CertificateOwnershipProof, CertificateProof, HistorySummary, InclusionProof, ProvenanceProof,
    VerificationError, VerificationResult,
};

/// Manager types.
pub use manager::CertificateManager;

/// Storage types.
pub use storage::{
    EntryStorage, InMemoryEntryStorage, InMemorySpineStorage, InMemoryStorage, SpineStorage,
};

/// Integration traits (new architecture).
pub use traits::{
    BraidAcceptor, BraidSummary, CommitAcceptor, DehydrationSummary, LoamCommitRef, ResultEntry,
    SignatureVerification, Signer, SliceManager, SliceOrigin, SliceResolution, SpineQuery,
    Verifier,
};

/// CLI-based signer integration for external signing services.
/// These replace mocks in production and use real signing service binaries.
pub use traits::{CliSigner, CliVerifier};

/// Service implementation.
pub use service::{ExpirySweeper, ExpirySweeperConfig, ExpirySweeperHandle, LoamSpineService};

/// Waypoint types.
pub use waypoint::{
    AttestationRequirement, AttestationResult, RelendingChain, RelendingLink, WaypointConfig,
    WaypointSummary,
};

/// Capability discovery.
pub use discovery::{BoxedSigner, BoxedVerifier, CapabilityRegistry, CapabilityStatus};

/// Resilience patterns for PrimalAdapter (retry, circuit-breaker).
pub use resilience::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState, ResilientAdapter, RetryPolicy,
    RetryPolicyConfig,
};

/// Test utilities (only available with `testing` feature or in tests).
#[cfg(any(test, feature = "testing"))]
pub use traits::signing::testing::{MockSigner, MockVerifier};

/// Mock transport for testing discovery without a live backend.
#[cfg(any(test, feature = "testing"))]
pub use transport::mock::MockTransport;

/// The `LoamSpine` primal - Permanence Layer.
///
/// `LoamSpine` provides sovereign, append-only ledgers for permanent storage
/// of committed state. Each spine is owned by a DID and contains a chain
/// of cryptographically linked entries.
pub struct LoamSpine {
    config: LoamSpineConfig,
    state: PrimalState,
    started_at: Option<Instant>,
    capabilities: CapabilityRegistry,
}

impl LoamSpine {
    /// Create a new `LoamSpine` instance.
    #[must_use]
    pub fn new(config: LoamSpineConfig) -> Self {
        Self {
            config,
            state: PrimalState::Created,
            started_at: None,
            capabilities: CapabilityRegistry::new(),
        }
    }

    /// Create a new `LoamSpine` with a shared capability registry.
    #[must_use]
    pub const fn with_capabilities(
        config: LoamSpineConfig,
        capabilities: CapabilityRegistry,
    ) -> Self {
        Self {
            config,
            state: PrimalState::Created,
            started_at: None,
            capabilities,
        }
    }

    /// Get the configuration.
    #[must_use]
    pub const fn config(&self) -> &LoamSpineConfig {
        &self.config
    }

    /// Get the capability registry.
    #[must_use]
    pub const fn capabilities(&self) -> &CapabilityRegistry {
        &self.capabilities
    }

    /// Get the uptime in seconds.
    #[must_use]
    pub fn uptime_secs(&self) -> Option<u64> {
        self.started_at.map(|t| t.elapsed().as_secs())
    }
}

impl PrimalLifecycle for LoamSpine {
    fn state(&self) -> PrimalState {
        self.state
    }

    async fn start(&mut self) -> Result<(), PrimalError> {
        if self.state.is_running() {
            return Ok(());
        }

        self.state = PrimalState::Starting;
        tracing::info!(name = %self.config.name, "LoamSpine starting...");

        // Ensure storage directory exists
        if let Err(e) = std::fs::create_dir_all(&self.config.storage_path) {
            self.state = PrimalState::Failed;
            return Err(PrimalError::Init(format!(
                "failed to create storage directory: {e}"
            )));
        }

        self.started_at = Some(Instant::now());
        self.state = PrimalState::Running;
        tracing::info!(name = %self.config.name, "LoamSpine running");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PrimalError> {
        if self.state.is_terminal() {
            return Ok(());
        }

        self.state = PrimalState::Stopping;
        tracing::info!(name = %self.config.name, "LoamSpine stopping...");

        // Cleanup: flush pending storage ops, cancel discovery timers.
        // Currently a no-op; sled flushes on drop, in-memory is transient.

        self.state = PrimalState::Stopped;
        self.started_at = None;
        tracing::info!(name = %self.config.name, "LoamSpine stopped");
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
        let mut report = HealthReport::new(&self.config.name, env!("CARGO_PKG_VERSION"))
            .with_status(self.health_status());

        if let Some(uptime) = self.uptime_secs() {
            report = report.with_uptime(uptime);
        }

        // Check storage accessibility
        let storage_health = if self.config.storage_path.exists() {
            ComponentHealth::healthy("storage")
        } else {
            ComponentHealth::unhealthy("storage", "path does not exist")
        };
        report = report.with_component(storage_health);

        // Report capability statuses
        for (name, status) in self.capabilities.all_statuses().await {
            let health = match status {
                CapabilityStatus::Available => ComponentHealth::healthy(name),
                CapabilityStatus::Degraded { reason } => ComponentHealth::degraded(name, reason),
                CapabilityStatus::Unavailable => {
                    ComponentHealth::degraded(name, "capability not registered")
                }
            };
            report = report.with_component(health);
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn lifecycle_start_stop() {
        let config = LoamSpineConfig::default().with_storage_path("/tmp/loamspine-test");

        let mut spine = LoamSpine::new(config);
        assert_eq!(spine.state(), PrimalState::Created);

        spine.start().await.ok();
        assert_eq!(spine.state(), PrimalState::Running);
        assert!(spine.uptime_secs().is_some());

        spine.stop().await.ok();
        assert_eq!(spine.state(), PrimalState::Stopped);
    }

    #[tokio::test]
    async fn health_check_when_running() {
        let config = LoamSpineConfig::default().with_storage_path("/tmp/loamspine-test-health");

        let mut spine = LoamSpine::new(config);
        spine.start().await.ok();

        let report = spine.health_check().await;
        assert!(report.is_ok());

        let report = report.unwrap_or_else(|_| unreachable!());
        assert!(report.status.is_healthy());
        assert!(!report.components.is_empty());
    }

    #[test]
    fn health_status_when_not_running() {
        let config = LoamSpineConfig::default();
        let spine = LoamSpine::new(config);

        assert!(spine.health_status().is_unhealthy());
    }

    #[test]
    fn config_accessor() {
        let config = LoamSpineConfig::new("TestSpine");
        let spine = LoamSpine::new(config);

        assert_eq!(spine.config().name, "TestSpine");
    }

    #[tokio::test]
    async fn capabilities_accessor() {
        let config = LoamSpineConfig::default();
        let spine = LoamSpine::new(config);

        // Initially no capabilities registered
        assert_eq!(
            spine.capabilities().signer_status().await,
            CapabilityStatus::Unavailable
        );
    }

    #[tokio::test]
    async fn with_capabilities_constructor() {
        let config = LoamSpineConfig::default();
        let caps = CapabilityRegistry::new();

        // Register a signer
        let signer = std::sync::Arc::new(MockSigner::new(Did::new("did:test")));
        caps.register_signer(signer).await;

        let spine = LoamSpine::with_capabilities(config, caps);
        assert_eq!(
            spine.capabilities().signer_status().await,
            CapabilityStatus::Available
        );
    }

    #[tokio::test]
    async fn start_already_running() {
        let config = LoamSpineConfig::default().with_storage_path("/tmp/loamspine-test-running");

        let mut spine = LoamSpine::new(config);
        spine.start().await.ok();
        assert_eq!(spine.state(), PrimalState::Running);

        // Starting again should be idempotent
        let result = spine.start().await;
        assert!(result.is_ok());
        assert_eq!(spine.state(), PrimalState::Running);
    }

    #[tokio::test]
    async fn stop_already_stopped() {
        let config = LoamSpineConfig::default().with_storage_path("/tmp/loamspine-test-stopped");

        let mut spine = LoamSpine::new(config);
        spine.start().await.ok();
        spine.stop().await.ok();
        assert_eq!(spine.state(), PrimalState::Stopped);

        // Stopping again should be idempotent
        let result = spine.stop().await;
        assert!(result.is_ok());
        assert_eq!(spine.state(), PrimalState::Stopped);
    }

    #[tokio::test]
    async fn uptime_when_not_started() {
        let config = LoamSpineConfig::default();
        let spine = LoamSpine::new(config);

        // Uptime should be None when not started
        assert!(spine.uptime_secs().is_none());
    }

    #[tokio::test]
    async fn health_check_storage_exists() {
        let config =
            LoamSpineConfig::default().with_storage_path("/tmp/loamspine-test-health-exists");

        let mut spine = LoamSpine::new(config);
        spine.start().await.ok();

        let report = spine
            .health_check()
            .await
            .unwrap_or_else(|_| unreachable!());

        // Check storage component exists
        let storage = report.components.iter().find(|c| c.name == "storage");
        assert!(storage.is_some());
    }

    #[tokio::test]
    async fn health_check_with_capabilities() {
        let config =
            LoamSpineConfig::default().with_storage_path("/tmp/loamspine-test-health-caps");
        let caps = CapabilityRegistry::new();

        // Register a signer
        let signer = std::sync::Arc::new(MockSigner::new(Did::new("did:test")));
        caps.register_signer(signer).await;

        let mut spine = LoamSpine::with_capabilities(config, caps);
        spine.start().await.ok();

        let report = spine
            .health_check()
            .await
            .unwrap_or_else(|_| unreachable!());

        // Should include capability status
        let signer_status = report.components.iter().find(|c| c.name == "Signer");
        assert!(signer_status.is_some());
    }
}
