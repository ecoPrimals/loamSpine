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

mod certificate;
pub mod expiry_sweeper;
pub mod infant_discovery;
mod integration;
mod lifecycle;
pub mod signals;
mod waypoint;

// Re-export lifecycle manager, service state, infant discovery, and expiry sweeper
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
        // Check if we already have a spine for this owner
        let spines = self.spine_storage.list_spines().await?;
        for spine_id in spines {
            if let Some(spine) = self.spine_storage.get_spine(spine_id).await?
                && spine.owner == owner
            {
                return Ok(spine_id);
            }
        }

        // Create new spine
        let spine = Spine::new(owner.clone(), name, SpineConfig::default())?;
        let id = spine.id;

        // Save genesis entry
        if let Some(genesis) = spine.genesis_entry() {
            self.entry_storage.save_entry(genesis).await?;
        }

        // Save spine
        self.spine_storage.save_spine(&spine).await?;

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
mod tests {
    use super::*;
    use crate::traits::CommitAcceptor;
    use crate::traits::{BraidAcceptor, BraidSummary, DehydrationSummary};
    use crate::types::SessionId;

    #[tokio::test]
    async fn service_basic() {
        let service = LoamSpineService::new();

        let owner = Did::new("did:key:z6MkOwner");
        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        assert_eq!(service.spine_count().await, 1);

        // Ensure idempotent
        let spine_id2 = service
            .ensure_spine(owner.clone(), None)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(spine_id, spine_id2);
        assert_eq!(service.spine_count().await, 1);
    }

    #[tokio::test]
    async fn test_service_creation() {
        let service = LoamSpineService::new();
        assert_eq!(service.spine_count().await, 0);
        assert_eq!(service.entry_count().await, 0);
        assert_eq!(service.certificate_count().await, 0);
    }

    #[tokio::test]
    async fn test_create_and_get_spine() {
        use crate::traits::SpineQuery;

        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test Spine".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        // Spine should exist
        let spine = service.get_spine(spine_id).await;
        assert!(spine.is_ok());

        // Entry count should include genesis
        assert!(service.entry_count().await >= 1);
    }

    #[tokio::test]
    async fn test_seal_spine() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        // Seal the spine
        let result = service.seal_spine(spine_id, Some("Done".into())).await;
        assert!(result.is_ok());

        // Trying to append should fail (sealed)
        let entry_type = crate::entry::EntryType::MetadataUpdate {
            field: "test".into(),
            value: "value".into(),
        };
        let result = service.append_entry(spine_id, entry_type).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_append_entry() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        // Append a metadata update entry
        let entry_type = crate::entry::EntryType::MetadataUpdate {
            field: "test".into(),
            value: "value".into(),
        };

        let result = service.append_entry(spine_id, entry_type).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn commit_session() {
        let service = LoamSpineService::new();

        let owner = Did::new("did:key:z6MkOwner");
        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let summary =
            DehydrationSummary::new(SessionId::now_v7(), "game", [0u8; 32]).with_vertex_count(100);

        let commit_ref = service
            .commit_session(spine_id, owner.clone(), summary)
            .await
            .unwrap_or_else(|_| unreachable!());

        assert_eq!(commit_ref.spine_id, spine_id);
        assert_eq!(commit_ref.index, 1);

        let exists = service
            .verify_commit(&commit_ref)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(exists);
    }

    #[tokio::test]
    async fn commit_braid() {
        let service = LoamSpineService::new();

        let owner = Did::new("did:key:z6MkOwner");
        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let braid = BraidSummary::new(
            crate::types::BraidId::now_v7(),
            "attribution",
            [1u8; 32],
            [2u8; 32],
        )
        .with_agent(owner.clone());

        let _entry_hash = service
            .commit_braid(spine_id, owner.clone(), braid.clone())
            .await
            .unwrap_or_else(|_| unreachable!());

        let exists = service
            .verify_braid(braid.braid_id)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(exists);
    }

    #[tokio::test]
    async fn test_with_capabilities() {
        let caps = CapabilityRegistry::new();
        let service = LoamSpineService::with_capabilities(caps);
        assert_eq!(service.spine_count().await, 0);
    }
}
