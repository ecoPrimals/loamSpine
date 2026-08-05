// SPDX-License-Identifier: AGPL-3.0-or-later

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

        let spine = core
            .get_spine(spine_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::SpineNotFound(format!("{spine_id:?}")))?;
        let genesis_hash = spine.genesis;
        drop(core);

        Ok(CreateSpineResponse {
            spine_id,
            genesis_hash,
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
            Ok(None) => Ok(GetSpineResponse {
                found: false,
                spine: None,
            }),
            Err(e) => Err(ApiError::from(e)),
        }
    }

    /// List all spine IDs.
    ///
    /// # Errors
    ///
    /// Returns error if storage query fails.
    pub async fn list_spines(&self, _request: ListSpinesRequest) -> ApiResult<ListSpinesResponse> {
        let core = self.core().await;
        let spine_ids = core.list_spine_ids().await.map_err(ApiError::from)?;
        let count = spine_ids.len();
        Ok(ListSpinesResponse { spine_ids, count })
    }

    /// Get comprehensive spine status for observability.
    ///
    /// Scans entries for `SessionCommit` variants to report associated
    /// sessions alongside structural metrics (entry count, tip, state).
    ///
    /// # Errors
    ///
    /// Returns error if spine not found or storage query fails.
    pub async fn spine_status(
        &self,
        request: SpineStatusRequest,
    ) -> ApiResult<SpineStatusResponse> {
        let core = self.core().await;
        let spine = core
            .get_spine(request.spine_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::SpineNotFound(format!("{:?}", request.spine_id)))?;

        let mut sessions: Vec<SessionSummary> = spine
            .entries()
            .iter()
            .filter_map(|entry| {
                if let loam_spine_core::entry::EntryType::SessionCommit {
                    session_id,
                    merkle_root,
                    vertex_count,
                    ref committer,
                } = entry.entry_type
                {
                    Some(SessionSummary {
                        session_id,
                        merkle_root,
                        vertex_count,
                        committer: committer.clone(),
                        committed_at: entry.timestamp,
                        entry_index: entry.index,
                    })
                } else {
                    None
                }
            })
            .collect();

        sessions.reverse();
        let session_count = sessions.len();

        Ok(SpineStatusResponse {
            spine_id: spine.id,
            name: spine.name.clone(),
            owner: spine.owner.clone(),
            state: spine.state.clone(),
            entry_count: spine.height,
            tip_hash: spine.tip,
            genesis_hash: spine.genesis,
            created_at: spine.created_at,
            updated_at: spine.updated_at,
            sessions,
            session_count,
        })
    }

    /// Seal a spine.
    ///
    /// # Errors
    ///
    /// Returns error if sealing fails.
    pub async fn seal_spine(&self, request: SealSpineRequest) -> ApiResult<SealSpineResponse> {
        let core = self.core_mut().await;
        match core.seal_spine(request.spine_id, request.reason).await {
            Ok(hash) => Ok(SealSpineResponse {
                success: true,
                seal_hash: Some(hash),
            }),
            Err(e) => Err(ApiError::from(e)),
        }
    }
}
