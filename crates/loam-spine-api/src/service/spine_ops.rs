// SPDX-License-Identifier: AGPL-3.0-only

//! Spine create, get, and seal operations.

use super::LoamSpineRpcService;
use crate::error::{ApiError, ApiResult};
use crate::types::*;
use loam_spine_core::traits::SpineQuery;

impl LoamSpineRpcService {
    /// Create a new spine.
    ///
    /// # Errors
    ///
    /// Returns error if spine creation fails.
    pub async fn create_spine(
        &self,
        request: CreateSpineRequest,
    ) -> ApiResult<CreateSpineResponse> {
        let core = self.core_mut().await;
        let spine_id = core
            .ensure_spine(request.owner.clone(), Some(request.name))
            .await
            .map_err(ApiError::from)?;

        // Get the spine to get genesis hash
        let spine = core
            .get_spine(spine_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::SpineNotFound(format!("{spine_id:?}")))?;

        Ok(CreateSpineResponse {
            spine_id,
            genesis_hash: spine.genesis,
        })
    }

    /// Get a spine by ID.
    ///
    /// # Errors
    ///
    /// Returns error if spine lookup fails.
    pub async fn get_spine(&self, request: GetSpineRequest) -> ApiResult<GetSpineResponse> {
        let core = self.core().await;
        match core.get_spine(request.spine_id).await {
            Ok(Some(spine)) => Ok(GetSpineResponse {
                found: true,
                spine: Some(spine),
            }),
            Ok(None) | Err(_) => Ok(GetSpineResponse {
                found: false,
                spine: None,
            }),
        }
    }

    /// Seal a spine.
    ///
    /// # Errors
    ///
    /// Returns error if sealing fails.
    pub async fn seal_spine(&self, request: SealSpineRequest) -> ApiResult<SealSpineResponse> {
        let core = self.core_mut().await;
        match core.seal_spine(request.spine_id, None).await {
            Ok(hash) => Ok(SealSpineResponse {
                success: true,
                seal_hash: Some(hash),
            }),
            Err(e) => Err(ApiError::from(e)),
        }
    }
}
