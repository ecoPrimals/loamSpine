// SPDX-License-Identifier: AGPL-3.0-or-later

//! Public chain anchor operations (`anchor.publish`, `anchor.verify`,
//! `anchor.publish_batch`).

use super::LoamSpineRpcService;
use crate::error::{ApiError, ApiResult};
use crate::types::*;

impl LoamSpineRpcService {
    /// Record a public chain anchor on a spine.
    ///
    /// # Errors
    ///
    /// Returns error if the spine is not found, is sealed, or has no tip entry.
    pub async fn publish_anchor(
        &self,
        request: AnchorPublishRequest,
    ) -> ApiResult<AnchorPublishResponse> {
        let core = self.core_mut().await;
        let receipt = core
            .anchor_to_public_chain(
                request.spine_id,
                request.anchor_target,
                request.tx_ref,
                request.block_height,
                request.anchor_timestamp,
            )
            .await
            .map_err(ApiError::from)?;

        Ok(AnchorPublishResponse {
            entry_hash: receipt.entry_hash,
            state_hash: receipt.state_hash,
        })
    }

    /// Verify a spine's state against a recorded public chain anchor.
    ///
    /// When the anchor entry contains an `aggregate_root` and
    /// `inclusion_proof`, the response includes `aggregate_verified`
    /// indicating whether the Merkle inclusion proof is valid.
    ///
    /// # Errors
    ///
    /// Returns error if the spine or anchor entry is not found.
    pub async fn verify_anchor(
        &self,
        request: AnchorVerifyRequest,
    ) -> ApiResult<AnchorVerifyResponse> {
        let core = self.core().await;
        let verification = core
            .verify_anchor(request.spine_id, request.anchor_entry_hash)
            .await
            .map_err(ApiError::from)?;

        Ok(AnchorVerifyResponse {
            verified: verification.verified,
            anchor_target: verification.anchor_target,
            state_hash: verification.state_hash,
            tx_ref: verification.tx_ref,
            block_height: verification.block_height,
            anchor_timestamp: verification.anchor_timestamp,
            aggregate_verified: verification.aggregate_verified,
        })
    }

    /// Record an aggregate batch anchor across multiple spines.
    ///
    /// Computes an aggregation Merkle tree from all spines' tip state hashes,
    /// then appends a `PublicChainAnchor` entry (with aggregate root and
    /// inclusion proof) to each spine.
    ///
    /// # Errors
    ///
    /// Returns error if any spine is not found, is sealed, has no tip, or
    /// if fewer than 2 spines are provided.
    pub async fn publish_anchor_batch(
        &self,
        request: AnchorPublishBatchRequest,
    ) -> ApiResult<AnchorPublishBatchResponse> {
        let core = self.core_mut().await;
        let receipt = core
            .anchor_batch(
                &request.spine_ids,
                request.anchor_target,
                request.tx_ref,
                request.block_height,
                request.anchor_timestamp,
            )
            .await
            .map_err(ApiError::from)?;

        Ok(AnchorPublishBatchResponse {
            aggregate_root: receipt.aggregate_root,
            anchors: receipt
                .entries
                .into_iter()
                .map(|e| AnchorBatchEntryResponse {
                    spine_id: e.spine_id,
                    entry_hash: e.entry_hash,
                    state_hash: e.state_hash,
                })
                .collect(),
        })
    }
}
