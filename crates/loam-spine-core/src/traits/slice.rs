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
#[derive(Clone, Debug)]
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

/// Slice manager trait - for slice checkout/return operations.
///
/// This trait enables borrowing state across spine boundaries with
/// proper provenance tracking.
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
}
