// SPDX-License-Identifier: AGPL-3.0-or-later

//! Trait implementations for primal integration.
//!
//! This module provides implementations of integration traits that allow
//! LoamSpine to work with other primals:
//!
//! - **CommitAcceptor**: Accepts commits from ephemeral storage sessions
//! - **SliceManager**: Manages slice checkout and resolution
//! - **SpineQuery**: Provides query access to spines and entries
//! - **BraidAcceptor**: Accepts Braids from semantic attribution primals
//! - **ProvenanceSource**: Provides provenance data to attribution primals
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
    ActiveSlice, AttributionRecord, BraidAcceptor, BraidSummary, CommitAcceptor,
    DehydrationSummary, LoamCommitRef, ProvenanceLink, ProvenanceSource, SliceManager, SliceOrigin,
    SliceResolution, SliceStatus, SpineQuery,
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

        let entry_hash = spine.append(entry)?;
        let index = spine.height - 1;
        let appended = spine
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;
        let committed_at = appended.timestamp;
        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&spine).await?;

        Ok(LoamCommitRef {
            spine_id,
            entry_hash,
            index,
            committed_at,
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

        let _checkout_hash = spine.append(checkout_entry)?;
        let appended = spine
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;
        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&spine).await?;

        let origin = SliceOrigin {
            spine_id,
            entry_hash,
            entry_index: entry.index,
            certificate_id: None,
            owner: spine.owner.clone(),
        };

        {
            let mut slices = self.active_slices.write().await;
            slices.insert(
                slice_id,
                super::ActiveSliceInfo {
                    spine_id,
                    entry_hash,
                    holder: holder.clone(),
                    entry_index: entry.index,
                    owner: spine.owner.clone(),
                    session_id,
                    checked_out_at: crate::types::Timestamp::now(),
                },
            );
        }

        Ok(origin)
    }

    async fn resolve_slice(
        &self,
        slice_id: SliceId,
        resolution: SliceResolution,
    ) -> LoamSpineResult<EntryHash> {
        let info = {
            let slices = self.active_slices.read().await;
            slices
                .get(&slice_id)
                .cloned()
                .ok_or_else(|| LoamSpineError::Internal(format!("slice not found: {slice_id}")))?
        };
        let spine_id = info.spine_id;
        let source_entry = info.entry_hash;

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

        let entry_hash = spine.append(entry)?;
        let appended = spine
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;
        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&spine).await?;

        {
            let mut slices = self.active_slices.write().await;
            slices.remove(&slice_id);
        }

        Ok(entry_hash)
    }

    async fn mark_sliced(&self, slice_id: SliceId, holder: Did) -> LoamSpineResult<()> {
        {
            let mut slices = self.active_slices.write().await;
            if let Some(info) = slices.get_mut(&slice_id) {
                info.holder = holder;
            }
        }
        Ok(())
    }

    async fn clear_slice_mark(&self, slice_id: SliceId) -> LoamSpineResult<()> {
        {
            let mut slices = self.active_slices.write().await;
            slices.remove(&slice_id);
        }
        Ok(())
    }

    async fn record_slice_checkout(
        &self,
        spine_id: SpineId,
        slice_id: SliceId,
        holder: Did,
        origin: &SliceOrigin,
    ) -> LoamSpineResult<EntryHash> {
        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entry = spine.create_entry(EntryType::SliceCheckout {
            slice_id,
            source_entry: origin.entry_hash,
            session_id: SessionId::now_v7(),
            holder,
        });

        let entry_hash = spine.append(entry)?;
        let appended = spine
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;
        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&spine).await?;

        Ok(entry_hash)
    }

    async fn record_slice_return(
        &self,
        spine_id: SpineId,
        slice_id: SliceId,
        resolution: &SliceResolution,
    ) -> LoamSpineResult<EntryHash> {
        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let (success, summary) = match resolution {
            SliceResolution::Merged { summary } => (true, Some(*summary)),
            SliceResolution::Abandoned { .. } | SliceResolution::Expired => (false, None),
        };

        let checkout_entry = spine
            .tip_entry()
            .map(Entry::compute_hash)
            .transpose()?
            .unwrap_or_default();

        let entry = spine.create_entry(EntryType::SliceReturn {
            slice_id,
            checkout_entry,
            success,
            summary,
        });

        let entry_hash = spine.append(entry)?;
        let appended = spine
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;
        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&spine).await?;

        Ok(entry_hash)
    }

    async fn get_slice_status(&self, slice_id: SliceId) -> LoamSpineResult<SliceStatus> {
        let holder = {
            let slices = self.active_slices.read().await;
            slices.get(&slice_id).map(|info| info.holder.clone())
        };
        Ok(holder.map_or(SliceStatus::Unknown, |h| SliceStatus::Active { holder: h }))
    }

    async fn list_active_slices(&self, spine_id: SpineId) -> LoamSpineResult<Vec<ActiveSlice>> {
        let result: Vec<ActiveSlice> = self
            .active_slices
            .read()
            .await
            .iter()
            .filter(|(_, info)| info.spine_id == spine_id)
            .map(|(slice_id, info)| ActiveSlice {
                slice_id: *slice_id,
                origin: SliceOrigin {
                    spine_id: info.spine_id,
                    entry_hash: info.entry_hash,
                    entry_index: info.entry_index,
                    certificate_id: None,
                    owner: info.owner.clone(),
                },
                holder: info.holder.clone(),
                checked_out_at: info.checked_out_at,
                session_id: info.session_id,
            })
            .collect();
        Ok(result)
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

        let entry_hash = spine.append(entry)?;
        let appended = spine
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;
        self.entry_storage.save_entry(appended).await?;
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
                if let EntryType::BraidCommit { braid_id: bid, .. } = &entry.entry_type
                    && *bid == braid_id
                {
                    return Ok(true);
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
                    && *sh == subject_hash
                {
                    results.push(entry.compute_hash()?);
                }
            }
        }
        Ok(results)
    }
}

// ============================================================================
// ProvenanceSource Implementation (Attribution Primals → LoamSpine)
// ============================================================================

impl ProvenanceSource for LoamSpineService {
    async fn get_entries_for_data(&self, content_hash: ContentHash) -> LoamSpineResult<Vec<Entry>> {
        let mut results = Vec::new();
        let spine_ids = self.spine_storage.list_spines().await?;
        for spine_id in spine_ids {
            let entries = self
                .entry_storage
                .get_entries_for_spine(spine_id, 0, DEFAULT_SEARCH_LIMIT)
                .await?;
            for entry in entries {
                if let EntryType::DataAnchor { data_hash, .. } = &entry.entry_type
                    && *data_hash == content_hash
                {
                    results.push(entry);
                }
            }
        }
        Ok(results)
    }

    async fn get_certificate_history(
        &self,
        certificate_id: crate::types::CertificateId,
    ) -> LoamSpineResult<Vec<Entry>> {
        let mut results = Vec::new();
        let spine_ids = self.spine_storage.list_spines().await?;
        for spine_id in spine_ids {
            let entries = self
                .entry_storage
                .get_entries_for_spine(spine_id, 0, DEFAULT_SEARCH_LIMIT)
                .await?;
            for entry in entries {
                let matches = match &entry.entry_type {
                    EntryType::CertificateMint { cert_id, .. }
                    | EntryType::CertificateTransfer { cert_id, .. }
                    | EntryType::CertificateLoan { cert_id, .. }
                    | EntryType::CertificateReturn { cert_id, .. } => *cert_id == certificate_id,
                    _ => false,
                };
                if matches {
                    results.push(entry);
                }
            }
        }
        Ok(results)
    }

    async fn get_attribution(
        &self,
        content_hash: ContentHash,
    ) -> LoamSpineResult<Option<AttributionRecord>> {
        let spine_ids = self.spine_storage.list_spines().await?;
        for spine_id in &spine_ids {
            let entries = self
                .entry_storage
                .get_entries_for_spine(*spine_id, 0, DEFAULT_SEARCH_LIMIT)
                .await?;
            for entry in &entries {
                if let EntryType::DataAnchor { data_hash, .. } = &entry.entry_type
                    && *data_hash == content_hash
                {
                    let spine = self.spine_storage.get_spine(*spine_id).await?;
                    let creator = spine
                        .as_ref()
                        .map_or_else(|| Did::new("did:key:unknown"), |s| s.owner.clone());

                    return Ok(Some(AttributionRecord {
                        content_hash,
                        creator,
                        contributors: Vec::new(),
                        certificate_id: None,
                        recorded_at: entry.timestamp,
                    }));
                }
            }
        }
        Ok(None)
    }

    async fn get_provenance_chain(
        &self,
        content_hash: ContentHash,
    ) -> LoamSpineResult<Vec<ProvenanceLink>> {
        let mut chain = Vec::new();
        let spine_ids = self.spine_storage.list_spines().await?;
        for spine_id in spine_ids {
            let spine = self.spine_storage.get_spine(spine_id).await?;
            let owner = spine
                .as_ref()
                .map_or_else(|| Did::new("did:key:unknown"), |s| s.owner.clone());

            let entries = self
                .entry_storage
                .get_entries_for_spine(spine_id, 0, DEFAULT_SEARCH_LIMIT)
                .await?;
            for entry in entries {
                let relationship = match &entry.entry_type {
                    EntryType::DataAnchor { data_hash, .. } if *data_hash == content_hash => {
                        Some("anchored-by")
                    }
                    EntryType::BraidCommit { subject_hash, .. }
                        if *subject_hash == content_hash =>
                    {
                        Some("attributed-to")
                    }
                    _ => None,
                };
                if let Some(rel) = relationship {
                    chain.push(ProvenanceLink {
                        entry_hash: entry.compute_hash()?,
                        spine_id,
                        index: entry.index,
                        agent: owner.clone(),
                        timestamp: entry.timestamp,
                        relationship: rel.to_string(),
                    });
                }
            }
        }
        Ok(chain)
    }
}

#[cfg(test)]
#[path = "integration_tests.rs"]
mod tests;
