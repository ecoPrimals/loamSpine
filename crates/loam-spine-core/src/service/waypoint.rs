// SPDX-License-Identifier: AGPL-3.0-or-later

//! Waypoint operations and proof generation.
//!
//! This module provides:
//! - **Slice anchoring**: Anchor borrowed state on waypoint spines
//! - **Proof generation**: Create inclusion proofs for entries
//!
//! ## Waypoint Spines
//!
//! Waypoints are local spines that anchor borrowed slices, providing
//! cryptographic provenance for data borrowed from other spines.

use crate::entry::EntryType;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::proof::InclusionProof;
use crate::storage::{EntryStorage, SpineStorage};
use crate::types::{EntryHash, SliceId, SpineId};
use crate::waypoint::{AttestationContext, DepartureReason, WaypointConfig};

use super::LoamSpineService;

impl LoamSpineService {
    // ========================================================================
    // Attestation Enforcement
    // ========================================================================

    /// Check attestation requirement for a waypoint operation.
    ///
    /// Loads `WaypointConfig` from the spine's config. If attestation is
    /// required for this operation and no provider is available, returns error.
    /// If required and provider available, requests attestation from the capability registry.
    ///
    /// # Errors
    ///
    /// Returns error if attestation required but provider unavailable, or if
    /// attestation is denied.
    async fn check_attestation_requirement(
        &self,
        waypoint_spine_id: SpineId,
        slice_id: SliceId,
        operation: &str,
    ) -> LoamSpineResult<()> {
        let waypoint = self
            .spine_storage
            .get_spine(waypoint_spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(waypoint_spine_id))?;

        let waypoint_config: WaypointConfig =
            waypoint.config.waypoint_config.clone().unwrap_or_default();

        if !waypoint_config
            .operation_attestation
            .requires_for_operation(operation)
        {
            return Ok(());
        }

        let context = AttestationContext {
            operation: operation.to_string(),
            waypoint_spine_id,
            slice_id,
            caller: None,
        };

        self.capabilities.request_attestation(context).await?;
        Ok(())
    }

    // ========================================================================
    // Waypoint Operations
    // ========================================================================

    /// Anchor a slice on a waypoint spine.
    ///
    /// Records the slice's origin on the local waypoint spine, establishing
    /// a cryptographic link to the borrowed data's provenance.
    ///
    /// # Errors
    ///
    /// Returns error if waypoint spine not found or sealed.
    pub async fn anchor_slice(
        &self,
        waypoint_spine_id: SpineId,
        slice_id: SliceId,
        origin_spine_id: SpineId,
        origin_entry: EntryHash,
    ) -> LoamSpineResult<EntryHash> {
        self.check_attestation_requirement(waypoint_spine_id, slice_id, "anchor")
            .await?;

        let mut waypoint = self
            .spine_storage
            .get_spine(waypoint_spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(waypoint_spine_id))?;

        let entry = waypoint.create_entry(EntryType::SliceAnchor {
            slice_id,
            origin_spine: origin_spine_id,
            origin_entry,
        });

        let anchor_hash = waypoint.append(entry)?;
        let appended = waypoint
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;
        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&waypoint).await?;

        Ok(anchor_hash)
    }

    /// Record an operation on an anchored slice at a waypoint spine.
    ///
    /// The operation is validated against the entry's `allowed_in_waypoint`
    /// check and appended as a `SliceOperation` entry.
    ///
    /// # Errors
    ///
    /// Returns error if waypoint spine not found or sealed.
    pub async fn record_operation(
        &self,
        waypoint_spine_id: SpineId,
        slice_id: SliceId,
        operation: String,
    ) -> LoamSpineResult<EntryHash> {
        self.check_attestation_requirement(waypoint_spine_id, slice_id, &operation)
            .await?;

        let mut waypoint = self
            .spine_storage
            .get_spine(waypoint_spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(waypoint_spine_id))?;

        let entry = waypoint.create_entry(EntryType::SliceOperation {
            slice_id,
            operation,
        });

        if !entry.entry_type.allowed_in_waypoint() {
            return Err(LoamSpineError::InvalidEntryType(
                "entry type not allowed in waypoint".into(),
            ));
        }

        let op_hash = waypoint.append(entry)?;
        let appended = waypoint
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;
        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&waypoint).await?;

        Ok(op_hash)
    }

    /// Depart a slice from a waypoint spine.
    ///
    /// Records a `SliceDeparture` entry on the waypoint with the given reason.
    ///
    /// # Errors
    ///
    /// Returns error if waypoint spine not found or sealed.
    pub async fn depart_slice(
        &self,
        waypoint_spine_id: SpineId,
        slice_id: SliceId,
        reason: DepartureReason,
    ) -> LoamSpineResult<EntryHash> {
        self.check_attestation_requirement(waypoint_spine_id, slice_id, "depart")
            .await?;

        let mut waypoint = self
            .spine_storage
            .get_spine(waypoint_spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(waypoint_spine_id))?;

        let entry = waypoint.create_entry(EntryType::SliceDeparture {
            slice_id,
            reason: reason.to_string(),
        });

        let departure_hash = waypoint.append(entry)?;
        let appended = waypoint
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;
        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&waypoint).await?;

        Ok(departure_hash)
    }

    // ========================================================================
    // Proof Generation
    // ========================================================================

    /// Generate an inclusion proof for an entry.
    ///
    /// The proof demonstrates that an entry exists in a spine and provides
    /// the path from the entry to the current tip.
    ///
    /// # Errors
    ///
    /// Returns error if spine or entry not found.
    pub async fn generate_inclusion_proof(
        &self,
        spine_id: SpineId,
        entry_hash: EntryHash,
    ) -> LoamSpineResult<InclusionProof> {
        let spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entry = self
            .entry_storage
            .get_entry(entry_hash)
            .await?
            .ok_or(LoamSpineError::EntryNotFound(entry_hash))?;

        let tip_hash = spine.tip;

        let mut path = Vec::new();
        let entry_index = entry.index;

        // Build proof path from entry to tip (zero-copy: compute_hash is &self)
        for idx in (entry_index + 1)..=spine.height {
            if let Some(e) = spine.get_entry(idx) {
                path.push(e.compute_hash()?);
            }
        }

        let proof = InclusionProof::new(entry, spine_id, tip_hash)?.with_path(path);

        Ok(proof)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests use unwrap for conciseness"
)]
#[path = "waypoint_svc_tests.rs"]
mod tests;
