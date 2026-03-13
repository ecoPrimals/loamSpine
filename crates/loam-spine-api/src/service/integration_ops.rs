// SPDX-License-Identifier: AGPL-3.0-only

//! Session commit, braid commit, and slice operations.
//!
//! Integration points for ephemeral storage, semantic attribution,
//! and waypoint slice management.

use super::LoamSpineRpcService;
use crate::error::{ApiError, ApiResult};
use crate::types::*;
use loam_spine_core::traits::{
    BraidAcceptor, BraidSummary, CommitAcceptor, DehydrationSummary, SliceManager,
};

impl LoamSpineRpcService {
    /// Anchor a slice.
    ///
    /// # Errors
    ///
    /// Returns error if anchoring fails.
    pub async fn anchor_slice(
        &self,
        request: AnchorSliceRequest,
    ) -> ApiResult<AnchorSliceResponse> {
        let core = self.core_mut().await;

        // We need an origin entry hash - use a placeholder for now
        // In a real implementation, this would be looked up from the origin spine
        let origin_entry = [0u8; 32]; // Placeholder

        let anchor_hash = core
            .anchor_slice(
                request.waypoint_spine_id,
                request.slice_id,
                request.origin_spine_id,
                origin_entry,
            )
            .await
            .map_err(ApiError::from)?;

        Ok(AnchorSliceResponse { anchor_hash })
    }

    /// Checkout a slice.
    ///
    /// # Errors
    ///
    /// Returns error if checkout fails.
    pub async fn checkout_slice(
        &self,
        request: CheckoutSliceRequest,
    ) -> ApiResult<CheckoutSliceResponse> {
        let core = self.core_mut().await;

        // Generate a session ID for this checkout
        let session_id = loam_spine_core::types::SessionId::now_v7();

        // We need the entry hash - for now use a placeholder
        // In a real implementation, we'd look up the slice's entry hash
        let entry_hash = [0u8; 32];

        match core
            .checkout_slice(
                request.waypoint_spine_id,
                entry_hash,
                request.requester,
                session_id,
            )
            .await
        {
            Ok(_origin) => Ok(CheckoutSliceResponse {
                success: true,
                checkout_hash: Some(entry_hash), // Return the checkout entry hash
            }),
            Err(e) => Err(ApiError::from(e)),
        }
    }

    /// Commit a session from an ephemeral storage primal.
    ///
    /// # Errors
    ///
    /// Returns error if commit fails.
    pub async fn commit_session(
        &self,
        request: CommitSessionRequest,
    ) -> ApiResult<CommitSessionResponse> {
        let core = self.core_mut().await;

        // Build dehydration summary from request
        let summary = DehydrationSummary::new(request.session_id, "session", request.session_hash)
            .with_vertex_count(request.vertex_count);

        let commit_ref = core
            .commit_session(request.spine_id, request.committer, summary)
            .await
            .map_err(ApiError::from)?;

        Ok(CommitSessionResponse {
            commit_hash: commit_ref.entry_hash,
            index: commit_ref.index,
        })
    }

    /// Commit a braid from a semantic attribution primal.
    ///
    /// # Errors
    ///
    /// Returns error if commit fails.
    pub async fn commit_braid(
        &self,
        request: CommitBraidRequest,
    ) -> ApiResult<CommitBraidResponse> {
        let core = self.core_mut().await;

        // Build braid summary from request
        // BraidSummary::new takes (braid_id, braid_type, subject_hash, braid_hash)
        let mut braid = BraidSummary::new(
            request.braid_id,
            "attribution",
            request.braid_hash, // Using braid_hash as subject_hash
            request.braid_hash,
        );

        // Add agents from subjects
        for agent in request.subjects {
            braid = braid.with_agent(agent);
        }

        let hash = core
            .commit_braid(request.spine_id, request.committer, braid)
            .await
            .map_err(ApiError::from)?;

        // Note: commit_braid returns EntryHash, not index
        // We return 0 for index since we don't have it
        Ok(CommitBraidResponse {
            commit_hash: hash,
            index: 0,
        })
    }
}
