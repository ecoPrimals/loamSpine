// SPDX-License-Identifier: AGPL-3.0-only

//! Entry append, get, and tip operations.

use super::LoamSpineRpcService;
use crate::error::{ApiError, ApiResult};
use crate::types::*;
use loam_spine_core::traits::SpineQuery;

impl LoamSpineRpcService {
    /// Append an entry.
    ///
    /// # Errors
    ///
    /// Returns error if append fails.
    pub async fn append_entry(
        &self,
        request: AppendEntryRequest,
    ) -> ApiResult<AppendEntryResponse> {
        let core = self.core_mut().await;
        let entry_hash = core
            .append_entry(request.spine_id, request.entry_type)
            .await
            .map_err(ApiError::from)?;

        // Get the spine to get the new height
        let spine = core
            .get_spine(request.spine_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::SpineNotFound(format!("{:?}", request.spine_id)))?;

        Ok(AppendEntryResponse {
            entry_hash,
            index: spine.height - 1,
        })
    }

    /// Get an entry by hash.
    ///
    /// # Errors
    ///
    /// Returns error if lookup fails.
    pub async fn get_entry(&self, request: GetEntryRequest) -> ApiResult<GetEntryResponse> {
        let core = self.core().await;
        // Note: Core get_entry takes only hash, not spine_id
        match core.get_entry(request.entry_hash).await {
            Ok(Some(entry)) => Ok(GetEntryResponse {
                found: true,
                entry: Some(entry),
            }),
            Ok(None) | Err(_) => Ok(GetEntryResponse {
                found: false,
                entry: None,
            }),
        }
    }

    /// Get the tip entry.
    ///
    /// # Errors
    ///
    /// Returns error if spine not found.
    pub async fn get_tip(&self, request: GetTipRequest) -> ApiResult<GetTipResponse> {
        let core = self.core().await;
        let mut entry = core
            .get_tip(request.spine_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::SpineNotFound(format!("{:?}", request.spine_id)))?;

        let spine = core
            .get_spine(request.spine_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::SpineNotFound(format!("{:?}", request.spine_id)))?;

        let tip_hash = entry.hash();

        Ok(GetTipResponse {
            tip_hash,
            entry,
            height: spine.height,
        })
    }
}
