// SPDX-License-Identifier: AGPL-3.0-or-later

//! Entry append, get, and tip operations.

use super::LoamSpineRpcService;
use crate::error::{ApiError, ApiResult};
use crate::types::*;
use loam_spine_core::traits::SpineQuery;

impl LoamSpineRpcService {
    /// Append an entry, signing via Tower if `TOWER_SIGNER_SOCKET` is configured.
    ///
    /// When a tower signer is present, the entry's canonical bytes are signed
    /// via the tower's `crypto.sign_ed25519` and the base64 signature is stored
    /// in entry metadata (`tower_signature`, `tower_signature_alg`) before the
    /// entry is appended to the spine chain. The chain hash commits to the
    /// signed entry.
    ///
    /// # Errors
    ///
    /// Returns error if append fails or Tower signing fails.
    pub async fn append_entry(
        &self,
        request: AppendEntryRequest,
    ) -> ApiResult<AppendEntryResponse> {
        let core = self.core_mut().await;

        let mut entry = core
            .prepare_entry(request.spine_id, request.entry_type)
            .await
            .map_err(ApiError::from)?;

        if let Some(ref signer) = self.tower_signer {
            entry = Self::tower_sign_entry(entry, signer).await?;
        }

        let entry_hash = core
            .append_prepared_entry(request.spine_id, entry)
            .await
            .map_err(ApiError::from)?;

        let spine = core
            .get_spine(request.spine_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::SpineNotFound(format!("{:?}", request.spine_id)))?;
        let index = spine.height - 1;
        drop(core);

        Ok(AppendEntryResponse { entry_hash, index })
    }

    /// Append multiple entries in a single batch.
    ///
    /// Delegates creation and chaining to the core batch method, which
    /// amortizes spine lookup and persistence. Tower signing is not yet
    /// supported for batch (entries are unsigned in batch mode).
    ///
    /// # Errors
    ///
    /// Returns error if spine not found, sealed, or any entry fails.
    pub async fn append_entry_batch(
        &self,
        request: AppendEntryBatchRequest,
    ) -> ApiResult<AppendEntryBatchResponse> {
        let core = self.core_mut().await;

        let hashes = core
            .append_entry_batch(request.spine_id, request.entries)
            .await
            .map_err(ApiError::from)?;

        let spine = core
            .get_spine(request.spine_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::SpineNotFound(format!("{:?}", request.spine_id)))?;
        let final_height = spine.height;
        drop(core);

        let count = hashes.len();
        let base_index = final_height - count as u64;
        let results = hashes
            .into_iter()
            .enumerate()
            .map(|(i, entry_hash)| BatchEntryResult {
                entry_hash,
                index: base_index + i as u64,
            })
            .collect();

        Ok(AppendEntryBatchResponse { results, count })
    }

    /// Get an entry by hash.
    ///
    /// # Errors
    ///
    /// Returns error if lookup fails.
    pub async fn get_entry(&self, request: GetEntryRequest) -> ApiResult<GetEntryResponse> {
        // Note: Core get_entry takes only hash, not spine_id
        let core = self.core().await;
        let result = core.get_entry(request.entry_hash).await;
        drop(core);
        match result {
            Ok(Some(entry)) => Ok(GetEntryResponse {
                found: true,
                entry: Some(entry),
            }),
            Ok(None) => Ok(GetEntryResponse {
                found: false,
                entry: None,
            }),
            Err(e) => Err(ApiError::from(e)),
        }
    }

    /// List entries in a spine (paginated).
    ///
    /// # Errors
    ///
    /// Returns error if spine not found or storage query fails.
    pub async fn list_entries(
        &self,
        request: ListEntriesRequest,
    ) -> ApiResult<ListEntriesResponse> {
        let core = self.core().await;
        let entries = core
            .get_entries(request.spine_id, request.start, request.limit + 1)
            .await
            .map_err(ApiError::from)?;
        drop(core);

        let limit = usize::try_from(request.limit).unwrap_or(usize::MAX);
        let has_more = entries.len() > limit;
        let entries: Vec<_> = entries.into_iter().take(limit).collect();
        let count = entries.len();
        Ok(ListEntriesResponse {
            entries,
            count,
            has_more,
        })
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
        let height = spine.height;
        drop(core);

        let tip_hash = entry.hash().map_err(ApiError::from)?;

        Ok(GetTipResponse {
            tip_hash,
            entry,
            height,
        })
    }
}
