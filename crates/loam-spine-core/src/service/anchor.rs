// SPDX-License-Identifier: AGPL-3.0-or-later

//! Public chain anchor service methods.
//!
//! Records receipts from external append-only ledgers (blockchains, data
//! commons, federated spines) without performing chain interaction directly.
//! The actual submission is handled by a capability-discovered `"chain-anchor"`
//! primal — loamSpine only stores the result.

use crate::entry::{AnchorTarget, Entry, EntryType};
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::proof::{compute_merkle_root, generate_aggregate_proof};
use crate::storage::{EntryStorage, SpineStorage};
use crate::types::{ContentHash, EntryHash, SpineId, Timestamp};

use super::LoamSpineService;

/// Result of recording a public chain anchor.
#[derive(Clone, Debug)]
pub struct AnchorReceipt {
    /// Hash of the new `PublicChainAnchor` entry on the spine.
    pub entry_hash: EntryHash,
    /// The spine state hash that was anchored.
    pub state_hash: ContentHash,
}

/// Result of recording a batch aggregate anchor across multiple spines.
#[derive(Clone, Debug)]
pub struct AnchorBatchReceipt {
    /// The aggregate Merkle root of all state hashes.
    pub aggregate_root: ContentHash,
    /// Per-spine anchor results.
    pub entries: Vec<AnchorBatchEntry>,
}

/// Per-spine result within a batch anchor.
#[derive(Clone, Debug)]
pub struct AnchorBatchEntry {
    /// Spine that was anchored.
    pub spine_id: SpineId,
    /// Hash of the new `PublicChainAnchor` entry on this spine.
    pub entry_hash: EntryHash,
    /// This spine's state hash (leaf in aggregate tree).
    pub state_hash: ContentHash,
}

/// Verification result for a recorded public chain anchor.
#[derive(Clone, Debug)]
pub struct AnchorVerification {
    /// Whether the recorded state hash matches the spine's actual state at
    /// the anchor point.
    pub verified: bool,
    /// The anchor target system.
    pub anchor_target: AnchorTarget,
    /// The recorded state hash.
    pub state_hash: ContentHash,
    /// Transaction reference on the external system.
    pub tx_ref: String,
    /// Block height or sequence number.
    pub block_height: u64,
    /// When the anchor was confirmed externally.
    pub anchor_timestamp: Timestamp,
    /// If part of an aggregate batch, whether the inclusion proof verified.
    pub aggregate_verified: Option<bool>,
}

impl LoamSpineService {
    /// Record a public chain anchor on a spine.
    ///
    /// Computes the current tip entry hash as the `state_hash` and appends a
    /// `PublicChainAnchor` entry to the spine. The caller (or a
    /// capability-discovered chain-anchor primal) is responsible for the
    /// actual chain submission — loamSpine only records the receipt.
    ///
    /// # Errors
    ///
    /// Returns an error if the spine is not found, is sealed, has no tip
    /// entry, or if serialization fails.
    pub async fn anchor_to_public_chain(
        &self,
        spine_id: SpineId,
        anchor_target: AnchorTarget,
        tx_ref: String,
        block_height: u64,
        anchor_timestamp: Timestamp,
    ) -> LoamSpineResult<AnchorReceipt> {
        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let state_hash = spine
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("spine has no tip entry".into()))?
            .compute_hash()?;

        let entry = spine.create_entry(EntryType::PublicChainAnchor {
            anchor_target,
            state_hash,
            tx_ref,
            block_height,
            anchor_timestamp,
            aggregate_root: None,
            inclusion_proof: None,
        });

        let entry_hash = spine.append(entry)?;
        let appended = spine
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;
        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&spine).await?;

        Ok(AnchorReceipt {
            entry_hash,
            state_hash,
        })
    }

    /// Verify a spine's state against a recorded public chain anchor.
    ///
    /// If `anchor_entry_hash` is `None`, the most recent `PublicChainAnchor`
    /// entry on the spine is used. Verification checks that the recorded
    /// `state_hash` matches the hash of the entry immediately preceding
    /// the anchor entry (i.e., the tip at the time the anchor was made).
    ///
    /// # Errors
    ///
    /// Returns an error if the spine or anchor entry is not found.
    pub async fn verify_anchor(
        &self,
        spine_id: SpineId,
        anchor_entry_hash: Option<EntryHash>,
    ) -> LoamSpineResult<AnchorVerification> {
        let spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let anchor_entry = match anchor_entry_hash {
            Some(hash) => self
                .entry_storage
                .get_entry(hash)
                .await?
                .ok_or(LoamSpineError::EntryNotFound(hash))?,
            None => Self::find_latest_anchor(&spine)?,
        };

        let (anchor_target, state_hash, tx_ref, block_height, anchor_timestamp, agg_root, inc_proof) =
            match &anchor_entry.entry_type {
                EntryType::PublicChainAnchor {
                    anchor_target,
                    state_hash,
                    tx_ref,
                    block_height,
                    anchor_timestamp,
                    aggregate_root,
                    inclusion_proof,
                } => (
                    anchor_target.clone(),
                    *state_hash,
                    tx_ref.clone(),
                    *block_height,
                    *anchor_timestamp,
                    *aggregate_root,
                    inclusion_proof.clone(),
                ),
                _ => {
                    return Err(LoamSpineError::InvalidEntryType(
                        "entry is not a PublicChainAnchor".into(),
                    ));
                }
            };

        let preceding_index = anchor_entry.index.checked_sub(1);
        let verified = if let Some(idx) = preceding_index {
            if let Some(preceding) = spine.get_entry(idx) {
                preceding.compute_hash()? == state_hash
            } else {
                false
            }
        } else {
            false
        };

        let aggregate_verified = match (&agg_root, &inc_proof) {
            (Some(root), Some(proof)) => Some(proof.verify(root)),
            _ => None,
        };

        Ok(AnchorVerification {
            verified,
            anchor_target,
            state_hash,
            tx_ref,
            block_height,
            anchor_timestamp,
            aggregate_verified,
        })
    }

    /// Record an aggregate batch anchor across multiple spines.
    ///
    /// Collects each spine's tip state hash, builds an aggregation Merkle tree,
    /// and appends a `PublicChainAnchor` entry (with `aggregate_root` and
    /// `inclusion_proof`) to each spine.
    ///
    /// # Errors
    ///
    /// Returns an error if any spine is not found, is sealed, has no tip, or
    /// if there are fewer than 2 spines (use `anchor_to_public_chain` for
    /// single-spine anchoring).
    pub async fn anchor_batch(
        &self,
        spine_ids: &[SpineId],
        anchor_target: AnchorTarget,
        tx_ref: String,
        block_height: u64,
        anchor_timestamp: Timestamp,
    ) -> LoamSpineResult<AnchorBatchReceipt> {
        if spine_ids.len() < 2 {
            return Err(LoamSpineError::Internal(
                "anchor_batch requires at least 2 spines".into(),
            ));
        }

        let mut state_hashes = Vec::with_capacity(spine_ids.len());
        for &sid in spine_ids {
            let spine = self
                .spine_storage
                .get_spine(sid)
                .await?
                .ok_or(LoamSpineError::SpineNotFound(sid))?;
            let hash = spine
                .tip_entry()
                .ok_or_else(|| LoamSpineError::Internal("spine has no tip entry".into()))?
                .compute_hash()?;
            state_hashes.push(hash);
        }

        let aggregate_root = compute_merkle_root(&state_hashes);

        let mut entries = Vec::with_capacity(spine_ids.len());
        for (i, &sid) in spine_ids.iter().enumerate() {
            let proof = generate_aggregate_proof(&state_hashes, i)
                .ok_or_else(|| LoamSpineError::Internal("proof generation failed".into()))?;

            let mut spine = self
                .spine_storage
                .get_spine(sid)
                .await?
                .ok_or(LoamSpineError::SpineNotFound(sid))?;

            let entry = spine.create_entry(EntryType::PublicChainAnchor {
                anchor_target: anchor_target.clone(),
                state_hash: state_hashes[i],
                tx_ref: tx_ref.clone(),
                block_height,
                anchor_timestamp,
                aggregate_root: Some(aggregate_root),
                inclusion_proof: Some(proof),
            });

            let entry_hash = spine.append(entry)?;
            let appended = spine
                .tip_entry()
                .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;
            self.entry_storage.save_entry(appended).await?;
            self.spine_storage.save_spine(&spine).await?;

            entries.push(AnchorBatchEntry {
                spine_id: sid,
                entry_hash,
                state_hash: state_hashes[i],
            });
        }

        Ok(AnchorBatchReceipt {
            aggregate_root,
            entries,
        })
    }

    /// Walk the spine backwards to find the most recent `PublicChainAnchor`.
    fn find_latest_anchor(spine: &crate::spine::Spine) -> LoamSpineResult<Entry> {
        for idx in (0..spine.height).rev() {
            if let Some(entry) = spine.get_entry(idx)
                && matches!(entry.entry_type, EntryType::PublicChainAnchor { .. })
            {
                return Ok(entry.clone());
            }
        }
        Err(LoamSpineError::Internal(
            "no PublicChainAnchor entry found on spine".into(),
        ))
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "tests use expect for concise error paths"
)]
#[path = "anchor_tests.rs"]
mod tests;
