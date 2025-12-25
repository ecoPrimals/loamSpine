//! Trait implementations for primal integration.
//!
//! This module provides implementations of integration traits that allow
//! LoamSpine to work with other primals:
//!
//! - **CommitAcceptor**: Accepts commits from ephemeral storage sessions
//! - **SliceManager**: Manages slice checkout and resolution
//! - **SpineQuery**: Provides query access to spines and entries
//! - **BraidAcceptor**: Accepts Braids from semantic attribution primals
//!
//! ## Capability-Based Discovery
//!
//! These traits enable runtime discovery - other primals can discover
//! LoamSpine's capabilities without compile-time coupling.

use crate::entry::{Entry, EntryType};
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::spine::Spine;
use crate::storage::{EntryStorage, SpineStorage};
use crate::traits::{
    BraidAcceptor, BraidSummary, CommitAcceptor, DehydrationSummary, LoamCommitRef, SliceManager,
    SliceOrigin, SliceResolution, SpineQuery,
};
use crate::types::{BraidId, ContentHash, Did, EntryHash, SessionId, SliceId, SpineId};

use super::LoamSpineService;

/// Default maximum entries to scan when searching.
/// This prevents unbounded iterations while still allowing reasonable search depth.
const DEFAULT_SEARCH_LIMIT: u64 = 10_000;

// ============================================================================
// CommitAcceptor Implementation (Ephemeral Storage → LoamSpine)
// ============================================================================

impl CommitAcceptor for LoamSpineService {
    async fn commit_session(
        &self,
        spine_id: SpineId,
        committer: Did,
        summary: DehydrationSummary,
    ) -> LoamSpineResult<LoamCommitRef> {
        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entry = spine.create_entry(EntryType::SessionCommit {
            session_id: summary.session_id,
            merkle_root: summary.merkle_root,
            vertex_count: summary.vertex_count,
            committer: committer.clone(),
        });

        let entry_hash = spine.append(entry.clone())?;
        let index = spine.height - 1;

        self.entry_storage.save_entry(&entry).await?;
        self.spine_storage.save_spine(&spine).await?;

        Ok(LoamCommitRef {
            spine_id,
            entry_hash,
            index,
        })
    }

    async fn verify_commit(&self, commit_ref: &LoamCommitRef) -> LoamSpineResult<bool> {
        let exists = self
            .entry_storage
            .entry_exists(commit_ref.entry_hash)
            .await?;
        Ok(exists)
    }

    async fn get_commit(&self, commit_ref: &LoamCommitRef) -> LoamSpineResult<Option<Entry>> {
        self.entry_storage.get_entry(commit_ref.entry_hash).await
    }
}

// ============================================================================
// SliceManager Implementation
// ============================================================================

impl SliceManager for LoamSpineService {
    async fn checkout_slice(
        &self,
        spine_id: SpineId,
        entry_hash: EntryHash,
        holder: Did,
        session_id: SessionId,
    ) -> LoamSpineResult<SliceOrigin> {
        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entry = self
            .entry_storage
            .get_entry(entry_hash)
            .await?
            .ok_or(LoamSpineError::EntryNotFound(entry_hash))?;

        let slice_id = SliceId::now_v7();

        let checkout_entry = spine.create_entry(EntryType::SliceCheckout {
            slice_id,
            source_entry: entry_hash,
            session_id,
            holder: holder.clone(),
        });

        let _checkout_hash = spine.append(checkout_entry.clone())?;

        self.entry_storage.save_entry(&checkout_entry).await?;
        self.spine_storage.save_spine(&spine).await?;

        {
            let mut slices = self.active_slices.write().await;
            slices.insert(slice_id, (spine_id, entry_hash, holder.clone()));
        }

        Ok(SliceOrigin {
            spine_id,
            entry_hash,
            entry_index: entry.index,
            certificate_id: None,
            owner: spine.owner.clone(),
        })
    }

    async fn resolve_slice(
        &self,
        slice_id: SliceId,
        resolution: SliceResolution,
    ) -> LoamSpineResult<EntryHash> {
        let (spine_id, source_entry, _holder) = {
            let slices = self.active_slices.read().await;
            slices
                .get(&slice_id)
                .cloned()
                .ok_or_else(|| LoamSpineError::Internal(format!("slice not found: {slice_id}")))?
        };

        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entry = match &resolution {
            SliceResolution::Merged { summary } => spine.create_entry(EntryType::SliceReturn {
                slice_id,
                checkout_entry: source_entry,
                success: true,
                summary: Some(*summary),
            }),
            SliceResolution::Abandoned { .. } | SliceResolution::Expired => {
                spine.create_entry(EntryType::SliceReturn {
                    slice_id,
                    checkout_entry: source_entry,
                    success: false,
                    summary: None,
                })
            }
        };

        let entry_hash = spine.append(entry.clone())?;

        self.entry_storage.save_entry(&entry).await?;
        self.spine_storage.save_spine(&spine).await?;

        {
            let mut slices = self.active_slices.write().await;
            slices.remove(&slice_id);
        }

        Ok(entry_hash)
    }
}

// ============================================================================
// SpineQuery Implementation
// ============================================================================

impl SpineQuery for LoamSpineService {
    async fn get_entry(&self, hash: EntryHash) -> LoamSpineResult<Option<Entry>> {
        self.entry_storage.get_entry(hash).await
    }

    async fn get_entries(
        &self,
        spine_id: SpineId,
        start: u64,
        limit: u64,
    ) -> LoamSpineResult<Vec<Entry>> {
        self.entry_storage
            .get_entries_for_spine(spine_id, start, limit)
            .await
    }

    async fn get_tip(&self, spine_id: SpineId) -> LoamSpineResult<Option<Entry>> {
        let spine = self.spine_storage.get_spine(spine_id).await?;
        Ok(spine.and_then(|s| s.tip_entry().cloned()))
    }

    async fn get_spine(&self, id: SpineId) -> LoamSpineResult<Option<Spine>> {
        self.spine_storage.get_spine(id).await
    }
}

// ============================================================================
// BraidAcceptor Implementation (Semantic Attribution → LoamSpine)
// ============================================================================

impl BraidAcceptor for LoamSpineService {
    async fn commit_braid(
        &self,
        spine_id: SpineId,
        _committer: Did,
        braid: BraidSummary,
    ) -> LoamSpineResult<EntryHash> {
        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entry = spine.create_entry(EntryType::BraidCommit {
            braid_id: braid.braid_id,
            braid_hash: braid.braid_hash,
            subject_hash: braid.subject_hash,
        });

        let entry_hash = spine.append(entry.clone())?;

        self.entry_storage.save_entry(&entry).await?;
        self.spine_storage.save_spine(&spine).await?;

        Ok(entry_hash)
    }

    async fn verify_braid(&self, braid_id: BraidId) -> LoamSpineResult<bool> {
        let spine_ids = self.spine_storage.list_spines().await?;
        for spine_id in spine_ids {
            let entries = self
                .entry_storage
                .get_entries_for_spine(spine_id, 0, DEFAULT_SEARCH_LIMIT)
                .await?;
            for entry in entries {
                if let EntryType::BraidCommit { braid_id: bid, .. } = &entry.entry_type {
                    if *bid == braid_id {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    async fn get_braids_for_subject(
        &self,
        subject_hash: ContentHash,
    ) -> LoamSpineResult<Vec<EntryHash>> {
        let mut results = Vec::new();
        let spine_ids = self.spine_storage.list_spines().await?;
        for spine_id in spine_ids {
            let entries = self
                .entry_storage
                .get_entries_for_spine(spine_id, 0, DEFAULT_SEARCH_LIMIT)
                .await?;
            for entry in entries {
                if let EntryType::BraidCommit {
                    subject_hash: sh, ..
                } = &entry.entry_type
                {
                    if *sh == subject_hash {
                        results.push(entry.compute_hash());
                    }
                }
            }
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Timestamp;

    #[tokio::test]
    async fn test_slice_checkout_and_resolve() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        // Get genesis entry hash
        let spine = service
            .get_spine(spine_id)
            .await
            .unwrap_or_else(|_| unreachable!())
            .unwrap_or_else(|| unreachable!());

        let genesis = spine.genesis_entry().unwrap_or_else(|| unreachable!());
        let entry_hash = genesis.compute_hash();

        // Checkout slice
        let session_id = SessionId::now_v7();
        let origin = service
            .checkout_slice(spine_id, entry_hash, owner.clone(), session_id)
            .await
            .unwrap_or_else(|_| unreachable!());

        assert_eq!(origin.spine_id, spine_id);
    }

    #[tokio::test]
    async fn test_slice_resolve_merged() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Resolve Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let spine = service
            .get_spine(spine_id)
            .await
            .unwrap_or_else(|_| unreachable!())
            .unwrap_or_else(|| unreachable!());

        let genesis = spine.genesis_entry().unwrap_or_else(|| unreachable!());
        let entry_hash = genesis.compute_hash();

        let session_id = SessionId::now_v7();
        let origin = service
            .checkout_slice(spine_id, entry_hash, owner.clone(), session_id)
            .await
            .unwrap_or_else(|_| unreachable!());

        // Get slice_id from active slices
        let slice_id = {
            let slices = service.active_slices.read().await;
            slices
                .keys()
                .next()
                .copied()
                .unwrap_or_else(|| unreachable!())
        };

        // Resolve with merge
        let resolution = SliceResolution::Merged {
            summary: [0xABu8; 32],
        };
        let result_hash = service
            .resolve_slice(slice_id, resolution)
            .await
            .unwrap_or_else(|_| unreachable!());

        assert_ne!(result_hash, [0u8; 32]);
        assert_eq!(origin.spine_id, spine_id);
    }

    #[tokio::test]
    async fn test_slice_resolve_abandoned() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Abandon Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let spine = service
            .get_spine(spine_id)
            .await
            .unwrap_or_else(|_| unreachable!())
            .unwrap_or_else(|| unreachable!());

        let genesis = spine.genesis_entry().unwrap_or_else(|| unreachable!());
        let entry_hash = genesis.compute_hash();

        let session_id = SessionId::now_v7();
        let _origin = service
            .checkout_slice(spine_id, entry_hash, owner.clone(), session_id)
            .await
            .unwrap_or_else(|_| unreachable!());

        let slice_id = {
            let slices = service.active_slices.read().await;
            slices
                .keys()
                .next()
                .copied()
                .unwrap_or_else(|| unreachable!())
        };

        // Resolve with abandon
        let resolution = SliceResolution::Abandoned {
            reason: "test".to_string(),
        };
        let result_hash = service
            .resolve_slice(slice_id, resolution)
            .await
            .unwrap_or_else(|_| unreachable!());

        assert_ne!(result_hash, [0u8; 32]);
    }

    #[tokio::test]
    async fn test_get_entries() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let entries = service
            .get_entries(spine_id, 0, 10)
            .await
            .unwrap_or_else(|_| unreachable!());

        // Should have at least genesis entry
        assert!(!entries.is_empty());
    }

    #[tokio::test]
    async fn test_commit_session_and_verify() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Session Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let session_id = SessionId::now_v7();
        let summary = DehydrationSummary {
            session_id,
            session_type: "test-session".to_string(),
            merkle_root: [0xABu8; 32],
            vertex_count: 42,
            started_at: Timestamp::now(),
            ended_at: Timestamp::now(),
            result_entries: Vec::new(),
            metadata: std::collections::HashMap::new(),
        };

        let commit_ref = service
            .commit_session(spine_id, owner.clone(), summary)
            .await
            .unwrap_or_else(|_| unreachable!());

        assert_eq!(commit_ref.spine_id, spine_id);
        assert!(commit_ref.index > 0);

        // Verify commit
        let verified = service
            .verify_commit(&commit_ref)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(verified);

        // Get commit
        let entry = service
            .get_commit(&commit_ref)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(entry.is_some());
    }

    #[tokio::test]
    async fn test_braid_commit_and_verify() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Braid Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let braid_id = BraidId::now_v7();
        let subject_hash = [0xCDu8; 32];
        let braid = BraidSummary {
            braid_id,
            braid_type: "attribution".to_string(),
            subject_hash,
            braid_hash: [0xEFu8; 32],
            agents: vec![owner.clone()],
            created_at: Timestamp::now(),
            signature: None,
        };

        let entry_hash = service
            .commit_braid(spine_id, owner.clone(), braid)
            .await
            .unwrap_or_else(|_| unreachable!());

        assert_ne!(entry_hash, [0u8; 32]);

        // Verify braid exists
        let exists = service
            .verify_braid(braid_id)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(exists);

        // Verify non-existent braid doesn't exist
        let not_exists = service
            .verify_braid(BraidId::now_v7())
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(!not_exists);

        // Get braids for subject
        let braids = service
            .get_braids_for_subject(subject_hash)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(braids.len(), 1);
        assert_eq!(braids[0], entry_hash);

        // Get braids for non-existent subject
        let no_braids = service
            .get_braids_for_subject([0x00u8; 32])
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(no_braids.is_empty());
    }

    #[tokio::test]
    async fn test_get_tip() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Tip Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let tip = service
            .get_tip(spine_id)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(tip.is_some());

        // Non-existent spine returns None
        let no_tip = service
            .get_tip(SpineId::now_v7())
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(no_tip.is_none());
    }

    #[tokio::test]
    async fn test_get_entry() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Entry Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let spine = service
            .get_spine(spine_id)
            .await
            .unwrap_or_else(|_| unreachable!())
            .unwrap_or_else(|| unreachable!());

        let genesis = spine.genesis_entry().unwrap_or_else(|| unreachable!());
        let entry_hash = genesis.compute_hash();

        let entry = service
            .get_entry(entry_hash)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(entry.is_some());

        // Non-existent entry returns None
        let no_entry = service
            .get_entry([0u8; 32])
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(no_entry.is_none());
    }
}
