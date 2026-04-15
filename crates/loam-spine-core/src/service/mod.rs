// SPDX-License-Identifier: AGPL-3.0-or-later

//! Core LoamSpine service implementation.
//!
//! This module provides the main `LoamSpineService` that implements all
//! integration traits. It is organized into domain-specific submodules:
//!
//! - **Core**: Service definition and spine management
//! - **Certificate**: Certificate lifecycle operations (mint, transfer, loan, return)
//! - **Integration**: Trait implementations (CommitAcceptor, SliceManager, SpineQuery, BraidAcceptor)
//! - **Waypoint**: Slice anchoring and proof generation
//! - **Infant Discovery**: Zero-knowledge startup with runtime service discovery
//!
//! ## Capability-Based Design
//!
//! The service uses the capability registry for runtime discovery of
//! signing/verification capabilities, rather than hardcoding dependencies.
//! Other primals are discovered at runtime, not compile time.

pub mod anchor;
mod bond_ledger;
mod certificate;
mod certificate_escrow;
mod certificate_loan;
pub mod expiry_sweeper;
pub mod infant_discovery;
mod integration;
mod lifecycle;
pub mod signals;
mod waypoint;

// Re-export lifecycle manager, service state, infant discovery, expiry sweeper, and anchor types
pub use anchor::{AnchorReceipt, AnchorVerification};
pub use expiry_sweeper::{ExpirySweeper, ExpirySweeperConfig, ExpirySweeperHandle};
pub use infant_discovery::InfantDiscovery;
pub use lifecycle::{LifecycleManager, ServiceState};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::certificate::TransferConditions;
use crate::discovery::CapabilityRegistry;
use crate::entry::{EntryType, SpineConfig, SpineType};
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::spine::Spine;
use crate::storage::{
    EntryStorage, InMemoryCertificateStorage, InMemoryEntryStorage, InMemorySpineStorage,
    SpineStorage,
};
use crate::types::{Did, EntryHash, SliceId, SpineId};
use crate::waypoint::WaypointConfig;

/// Stored metadata for an active slice, tracked in the in-memory registry.
#[derive(Clone, Debug)]
pub(crate) struct ActiveSliceInfo {
    pub spine_id: SpineId,
    pub entry_hash: EntryHash,
    pub holder: Did,
    pub entry_index: u64,
    pub owner: Did,
    pub session_id: crate::types::SessionId,
    pub checked_out_at: crate::types::Timestamp,
}

/// LoamSpine service that implements all integration traits.
///
/// This is the main entry point for interacting with LoamSpine functionality.
/// It provides:
/// - Spine management (create, query, seal)
/// - Entry operations (append, query)
/// - Certificate lifecycle (mint, transfer, loan, return)
/// - Slice operations (checkout, anchor, resolve)
/// - Proof generation
///
/// ## Capability-Based Design
///
/// The service uses the capability registry for runtime discovery of
/// signing/verification capabilities, rather than hardcoding dependencies.
///
/// ## Primal Self-Knowledge
///
/// `LoamSpine` only knows its own capabilities. Other primals (signing services,
/// discovery services, etc.) are discovered at runtime through the capability
/// registry, not hardcoded at compile time.
#[derive(Clone)]
pub struct LoamSpineService {
    pub(crate) spine_storage: InMemorySpineStorage,
    pub(crate) entry_storage: InMemoryEntryStorage,
    /// Active slices: slice_id -> (spine_id, entry_hash, holder)
    pub(crate) active_slices: Arc<RwLock<HashMap<SliceId, ActiveSliceInfo>>>,
    /// Certificate storage (trait-backed, currently in-memory).
    pub(crate) certificate_storage: InMemoryCertificateStorage,
    /// Active escrows: escrow_id -> transfer conditions.
    pub(crate) escrows: Arc<RwLock<HashMap<uuid::Uuid, TransferConditions>>>,
    /// Capability registry for runtime discovery.
    capabilities: CapabilityRegistry,
    /// Owner → spine_id index for O(1) `ensure_spine` lookups.
    owner_index: Arc<RwLock<HashMap<Did, SpineId>>>,
    /// Bond ledger: bond_id → latest bond data (in-memory index backed by spine entries).
    pub(crate) bond_ledger: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// Spine ID dedicated to the bond ledger (lazily created on first store).
    pub(crate) bond_ledger_spine: Arc<RwLock<Option<SpineId>>>,
}

impl Default for LoamSpineService {
    fn default() -> Self {
        Self::new()
    }
}

impl LoamSpineService {
    /// Create a new LoamSpine service.
    #[must_use]
    pub fn new() -> Self {
        Self {
            spine_storage: InMemorySpineStorage::new(),
            entry_storage: InMemoryEntryStorage::new(),
            active_slices: Arc::new(RwLock::new(HashMap::new())),
            certificate_storage: InMemoryCertificateStorage::new(),
            escrows: Arc::new(RwLock::new(HashMap::new())),
            capabilities: CapabilityRegistry::new(),
            owner_index: Arc::new(RwLock::new(HashMap::new())),
            bond_ledger: Arc::new(RwLock::new(HashMap::new())),
            bond_ledger_spine: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new LoamSpine service with a custom capability registry.
    ///
    /// This allows injecting capabilities discovered at runtime from other primals.
    #[must_use]
    pub fn with_capabilities(capabilities: CapabilityRegistry) -> Self {
        Self {
            spine_storage: InMemorySpineStorage::new(),
            entry_storage: InMemoryEntryStorage::new(),
            active_slices: Arc::new(RwLock::new(HashMap::new())),
            certificate_storage: InMemoryCertificateStorage::new(),
            escrows: Arc::new(RwLock::new(HashMap::new())),
            capabilities,
            owner_index: Arc::new(RwLock::new(HashMap::new())),
            bond_ledger: Arc::new(RwLock::new(HashMap::new())),
            bond_ledger_spine: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the capability registry.
    #[must_use]
    pub const fn capabilities(&self) -> &CapabilityRegistry {
        &self.capabilities
    }

    // ========================================================================
    // Spine Management
    // ========================================================================

    /// Create or get a spine for a DID.
    ///
    /// This is idempotent - calling with the same owner returns the existing spine.
    ///
    /// # Errors
    ///
    /// Returns an error if spine creation fails.
    pub async fn ensure_spine(&self, owner: Did, name: Option<String>) -> LoamSpineResult<SpineId> {
        // O(1) index lookup instead of scanning all spines
        if let Some(&existing) = self.owner_index.read().await.get(&owner) {
            return Ok(existing);
        }

        let spine = Spine::new(owner.clone(), name, SpineConfig::default())?;
        let id = spine.id;

        if let Some(genesis) = spine.genesis_entry() {
            self.entry_storage.save_entry(genesis).await?;
        }

        self.spine_storage.save_spine(&spine).await?;
        self.owner_index.write().await.insert(owner, id);

        Ok(id)
    }

    /// Create a waypoint spine with the given waypoint configuration.
    ///
    /// Use this when attestation or other waypoint policies need to be applied.
    /// Each call creates a new spine (no deduplication by owner).
    ///
    /// # Errors
    ///
    /// Returns an error if spine creation fails.
    pub async fn ensure_waypoint_spine(
        &self,
        owner: Did,
        name: Option<String>,
        waypoint_config: WaypointConfig,
    ) -> LoamSpineResult<SpineId> {
        let config = SpineConfig {
            spine_type: SpineType::Waypoint {
                max_anchor_depth: waypoint_config.max_anchor_depth,
            },
            auto_rollup_threshold: None,
            replication_enabled: false,
            waypoint_config: Some(waypoint_config),
        };

        let spine = Spine::new(owner.clone(), name, config)?;
        let id = spine.id;

        if let Some(genesis) = spine.genesis_entry() {
            self.entry_storage.save_entry(genesis).await?;
        }

        self.spine_storage.save_spine(&spine).await?;

        Ok(id)
    }

    /// Get the number of stored spines.
    pub async fn spine_count(&self) -> usize {
        self.spine_storage.spine_count().await
    }

    /// Get the number of stored entries.
    pub async fn entry_count(&self) -> usize {
        self.entry_storage.entry_count().await
    }

    /// Seal a spine, making it read-only.
    ///
    /// Once sealed, no new entries can be appended.
    ///
    /// # Errors
    ///
    /// Returns error if spine not found or already sealed.
    pub async fn seal_spine(
        &self,
        spine_id: SpineId,
        reason: Option<String>,
    ) -> LoamSpineResult<EntryHash> {
        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let seal_hash = spine.seal(reason)?;
        self.spine_storage.save_spine(&spine).await?;

        if let Some(entry) = spine.tip_entry() {
            self.entry_storage.save_entry(entry).await?;
        }

        Ok(seal_hash)
    }

    /// Append a generic entry to a spine.
    ///
    /// # Errors
    ///
    /// Returns error if spine not found or sealed.
    pub async fn append_entry(
        &self,
        spine_id: SpineId,
        entry_type: EntryType,
    ) -> LoamSpineResult<EntryHash> {
        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entry = spine.create_entry(entry_type);
        let entry_hash = spine.append(entry)?;
        let appended = spine
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;
        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&spine).await?;

        Ok(entry_hash)
    }
}

#[cfg(test)]
#[path = "service_mod_tests.rs"]
mod tests;
