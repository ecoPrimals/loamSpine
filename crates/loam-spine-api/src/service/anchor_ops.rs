// SPDX-License-Identifier: AGPL-3.0-or-later

//! Public chain anchor operations (`anchor.publish`, `anchor.verify`).

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
        })
    }
}
