// SPDX-License-Identifier: AGPL-3.0-or-later

//! Slice management traits for waypoint operations.
//!
//! These traits define the interface for checking out and resolving slices
//! (borrowed state) across spine boundaries.

use crate::error::LoamSpineResult;
use crate::types::{Did, EntryHash, SessionId, SliceId, SpineId};

/// Origin information for a slice checkout.
#[derive(Clone, Debug)]
pub struct SliceOrigin {
    /// Source spine ID.
    pub spine_id: SpineId,
    /// Source entry hash.
    pub entry_hash: EntryHash,
    /// Entry index in the spine.
    pub entry_index: u64,
    /// Associated certificate (if any).
    pub certificate_id: Option<crate::types::CertificateId>,
    /// Spine owner.
    pub owner: Did,
}

/// Slice resolution outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SliceResolution {
    /// Slice was merged back into origin spine.
    Merged {
        /// Summary of changes.
        summary: crate::types::ContentHash,
    },
    /// Slice was abandoned.
    Abandoned {
        /// Reason for abandonment.
        reason: String,
    },
    /// Slice expired.
    Expired,
}

/// Slice manager trait — for slice checkout/return operations.
///
/// This trait enables borrowing state across spine boundaries with
/// proper provenance tracking. Slice operations maintain full provenance
/// chains so that the origin of any borrowed state can always be verified.
pub trait SliceManager: Send + Sync {
    /// Checkout a slice from a spine.
    fn checkout_slice(
        &self,
        spine_id: SpineId,
        entry_hash: EntryHash,
        holder: Did,
        session_id: SessionId,
    ) -> impl std::future::Future<Output = LoamSpineResult<SliceOrigin>> + Send;

    /// Resolve a slice back to the origin spine.
    fn resolve_slice(
        &self,
        slice_id: SliceId,
        resolution: SliceResolution,
    ) -> impl std::future::Future<Output = LoamSpineResult<EntryHash>> + Send;

    /// Mark a slice as active (being used by a borrowing primal).
    fn mark_sliced(
        &self,
        slice_id: SliceId,
        holder: Did,
    ) -> impl std::future::Future<Output = LoamSpineResult<()>> + Send;

    /// Clear the slice mark (slice is no longer active).
    fn clear_slice_mark(
        &self,
        slice_id: SliceId,
    ) -> impl std::future::Future<Output = LoamSpineResult<()>> + Send;

    /// Record a slice checkout event in the spine log.
    fn record_slice_checkout(
        &self,
        spine_id: SpineId,
        slice_id: SliceId,
        holder: Did,
        origin: &SliceOrigin,
    ) -> impl std::future::Future<Output = LoamSpineResult<EntryHash>> + Send;

    /// Record a slice return event in the spine log.
    fn record_slice_return(
        &self,
        spine_id: SpineId,
        slice_id: SliceId,
        resolution: &SliceResolution,
    ) -> impl std::future::Future<Output = LoamSpineResult<EntryHash>> + Send;

    /// Get the current status of a slice.
    fn get_slice_status(
        &self,
        slice_id: SliceId,
    ) -> impl std::future::Future<Output = LoamSpineResult<SliceStatus>> + Send;

    /// List all active slices for a spine.
    fn list_active_slices(
        &self,
        spine_id: SpineId,
    ) -> impl std::future::Future<Output = LoamSpineResult<Vec<ActiveSlice>>> + Send;
}

/// Status of a slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SliceStatus {
    /// Slice is checked out and active.
    Active {
        /// Who holds the slice.
        holder: Did,
    },
    /// Slice has been returned and resolved.
    Resolved {
        /// How the slice was resolved.
        resolution: SliceResolution,
    },
    /// Slice does not exist or has been pruned.
    Unknown,
}

/// An active slice currently checked out.
#[derive(Clone, Debug)]
pub struct ActiveSlice {
    /// Slice ID.
    pub slice_id: SliceId,
    /// Origin information.
    pub origin: SliceOrigin,
    /// Who holds the slice.
    pub holder: Did,
    /// When the slice was checked out.
    pub checked_out_at: crate::types::Timestamp,
    /// Session associated with the checkout.
    pub session_id: SessionId,
}

impl SliceResolution {
    /// Check if the slice was successfully merged.
    #[must_use]
    pub const fn is_merged(&self) -> bool {
        matches!(self, Self::Merged { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_origin_creation() {
        let origin = SliceOrigin {
            spine_id: SpineId::now_v7(),
            entry_hash: [1u8; 32],
            entry_index: 42,
            certificate_id: None,
            owner: Did::new("did:key:test"),
        };
        assert_eq!(origin.entry_index, 42);
    }

    #[test]
    fn slice_resolution_variants() {
        let merged = SliceResolution::Merged { summary: [1u8; 32] };
        assert!(merged.is_merged());

        let abandoned = SliceResolution::Abandoned {
            reason: "test".to_string(),
        };
        assert!(!abandoned.is_merged());

        let expired = SliceResolution::Expired;
        assert!(!expired.is_merged());
    }

    #[test]
    fn slice_status_variants() {
        let active = SliceStatus::Active {
            holder: Did::new("did:key:test"),
        };
        assert!(matches!(active, SliceStatus::Active { .. }));

        let resolved = SliceStatus::Resolved {
            resolution: SliceResolution::Expired,
        };
        assert!(matches!(resolved, SliceStatus::Resolved { .. }));

        assert_eq!(SliceStatus::Unknown, SliceStatus::Unknown);
    }

    #[test]
    fn active_slice_creation() {
        let slice = ActiveSlice {
            slice_id: SliceId::now_v7(),
            origin: SliceOrigin {
                spine_id: SpineId::now_v7(),
                entry_hash: [0u8; 32],
                entry_index: 0,
                certificate_id: None,
                owner: Did::new("did:key:owner"),
            },
            holder: Did::new("did:key:holder"),
            checked_out_at: crate::types::Timestamp::now(),
            session_id: SessionId::now_v7(),
        };
        let _ = format!("{slice:?}");
    }

    #[test]
    #[expect(
        clippy::redundant_clone,
        reason = "clone verifies owned-value semantics"
    )]
    fn slice_types_clone() {
        let origin = SliceOrigin {
            spine_id: SpineId::now_v7(),
            entry_hash: [0u8; 32],
            entry_index: 0,
            certificate_id: None,
            owner: Did::new("did:key:test"),
        };
        let cloned = origin.clone();
        assert_eq!(origin.entry_index, cloned.entry_index);

        let merged = SliceResolution::Merged { summary: [1u8; 32] };
        let cloned = merged.clone();
        assert!(cloned.is_merged());

        let status = SliceStatus::Active {
            holder: Did::new("did:key:test"),
        };
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }
}
