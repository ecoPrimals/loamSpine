// SPDX-License-Identifier: AGPL-3.0-only

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
use crate::waypoint::DepartureReason;

use super::LoamSpineService;

impl LoamSpineService {
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

        let anchor_hash = waypoint.append(entry.clone())?;

        self.entry_storage.save_entry(&entry).await?;
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

        let op_hash = waypoint.append(entry.clone())?;

        self.entry_storage.save_entry(&entry).await?;
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
        let mut waypoint = self
            .spine_storage
            .get_spine(waypoint_spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(waypoint_spine_id))?;

        let entry = waypoint.create_entry(EntryType::SliceDeparture {
            slice_id,
            reason: reason.to_string(),
        });

        let departure_hash = waypoint.append(entry.clone())?;

        self.entry_storage.save_entry(&entry).await?;
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

        // Build proof path from entry to tip
        for idx in (entry_index + 1)..=spine.height {
            if let Some(e) = spine.get_entry(idx) {
                let mut e_clone = e.clone();
                path.push(e_clone.hash()?);
            }
        }

        let proof = InclusionProof::new(entry, spine_id, tip_hash)?.with_path(path);

        Ok(proof)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::traits::SpineQuery;
    use crate::types::Did;

    #[tokio::test]
    async fn test_anchor_slice() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let slice_id = SliceId::now_v7();
        let origin_spine_id = spine_id; // Use same spine for simplicity
        let origin_entry = [1u8; 32];

        let result = service
            .anchor_slice(spine_id, slice_id, origin_spine_id, origin_entry)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_inclusion_proof() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        // Get a valid entry hash from the spine (genesis)
        let spine_result = service.get_spine(spine_id).await;
        if let Ok(Some(spine)) = spine_result
            && let Some(genesis) = spine.genesis_entry()
        {
            let entry_hash = genesis.compute_hash().expect("compute_hash");

            let result = service.generate_inclusion_proof(spine_id, entry_hash).await;
            assert!(result.is_ok());

            if let Ok(proof) = result {
                assert!(proof.verify().expect("verify"));
            }
        }
    }

    #[tokio::test]
    async fn test_record_operation() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Waypoint".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let slice_id = SliceId::now_v7();

        // Anchor a slice first
        service
            .anchor_slice(spine_id, slice_id, spine_id, [1u8; 32])
            .await
            .unwrap_or_else(|_| unreachable!());

        // Record an operation
        let result = service
            .record_operation(spine_id, slice_id, "use".into())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_depart_slice() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Waypoint".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let slice_id = SliceId::now_v7();

        // Anchor first
        service
            .anchor_slice(spine_id, slice_id, spine_id, [1u8; 32])
            .await
            .unwrap_or_else(|_| unreachable!());

        // Record an operation
        service
            .record_operation(spine_id, slice_id, "view".into())
            .await
            .unwrap_or_else(|_| unreachable!());

        // Depart
        let result = service
            .depart_slice(spine_id, slice_id, DepartureReason::ManualReturn)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_depart_slice_expired() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Waypoint".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let slice_id = SliceId::now_v7();

        service
            .anchor_slice(spine_id, slice_id, spine_id, [1u8; 32])
            .await
            .unwrap_or_else(|_| unreachable!());

        let result = service
            .depart_slice(spine_id, slice_id, DepartureReason::Expired)
            .await;
        assert!(result.is_ok());
    }
}
