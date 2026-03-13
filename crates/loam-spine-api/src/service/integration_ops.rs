// SPDX-License-Identifier: AGPL-3.0-only

//! Session commit, braid commit, and slice operations.
//!
//! Integration points for ephemeral storage, semantic attribution,
//! and waypoint slice management. Includes `permanent-storage.*`
//! compatibility layer for rhizoCrypt's wire format.

use super::LoamSpineRpcService;
use crate::error::{ApiError, ApiResult};
use crate::types::*;
use loam_spine_core::traits::{
    BraidAcceptor, BraidSummary, CommitAcceptor, DehydrationSummary, SliceManager,
};
use tracing::debug;

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
        use loam_spine_core::traits::SpineQuery;

        let core = self.core_mut().await;

        let origin_entry = match core.get_tip(request.origin_spine_id).await {
            Ok(Some(tip)) => tip.compute_hash().map_err(ApiError::from)?,
            Ok(None) => {
                return Err(ApiError::InvalidRequest(format!(
                    "origin spine {} has no entries",
                    request.origin_spine_id
                )));
            }
            Err(e) => return Err(ApiError::from(e)),
        };

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
        use loam_spine_core::entry::EntryType;
        use loam_spine_core::traits::SpineQuery;

        let core = self.core_mut().await;

        let session_id = loam_spine_core::types::SessionId::now_v7();

        let entries = core
            .get_entries(request.waypoint_spine_id, 0, 10_000)
            .await
            .map_err(ApiError::from)?;

        let anchor_entry = entries.iter().rev().find(|e| {
            matches!(
                &e.entry_type,
                EntryType::SliceAnchor { slice_id, .. } if *slice_id == request.slice_id
            )
        });

        let entry_hash = match anchor_entry {
            Some(entry) => entry.compute_hash().map_err(ApiError::from)?,
            None => {
                return Err(ApiError::InvalidRequest(format!(
                    "no anchored slice {} found on waypoint {}",
                    request.slice_id, request.waypoint_spine_id
                )));
            }
        };

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
                checkout_hash: Some(entry_hash),
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

        Ok(CommitBraidResponse {
            commit_hash: hash,
            index: 0,
        })
    }

    // ====================================================================
    // permanent-storage.* compatibility (rhizoCrypt wire format)
    // ====================================================================

    /// Translate rhizoCrypt's `permanent-storage.commitSession` to loamSpine's
    /// native `session.commit`. Auto-creates a permanence spine for the
    /// committer if one doesn't already exist.
    ///
    /// # Errors
    ///
    /// Returns error if translation or commit fails.
    pub async fn permanent_storage_commit_session(
        &self,
        request: PermanentStorageCommitRequest,
    ) -> ApiResult<PermanentStorageCommitResponse> {
        debug!(
            "permanent-storage.commitSession: session_id={}, outcome={}",
            request.session_id, request.summary.outcome
        );

        let session_id = request
            .session_id
            .parse::<uuid::Uuid>()
            .map_err(|e| ApiError::InvalidRequest(format!("invalid session_id UUID: {e}")))?;

        let merkle_root = hex_to_content_hash(&request.merkle_root)
            .map_err(|e| ApiError::InvalidRequest(format!("invalid merkle_root hex: {e}")))?;

        let committer = Did::new(
            request
                .committer_did
                .as_deref()
                .unwrap_or("did:key:anonymous"),
        );

        let spine_id = self.ensure_permanence_spine(&committer).await?;

        let native_request = CommitSessionRequest {
            spine_id,
            session_id,
            session_hash: merkle_root,
            vertex_count: request.summary.vertex_count,
            committer,
        };

        match self.commit_session(native_request).await {
            Ok(resp) => {
                let hash_hex = bytes_to_hex(&resp.commit_hash);
                Ok(PermanentStorageCommitResponse {
                    accepted: true,
                    commit_id: Some(hash_hex.clone()),
                    spine_entry_hash: Some(hash_hex),
                    entry_index: Some(resp.index),
                    spine_id: Some(spine_id.to_string()),
                    error: None,
                })
            }
            Err(e) => Ok(PermanentStorageCommitResponse {
                accepted: false,
                commit_id: None,
                spine_entry_hash: None,
                entry_index: None,
                spine_id: None,
                error: Some(e.to_string()),
            }),
        }
    }

    /// Verify a commit exists using rhizoCrypt's wire format.
    ///
    /// # Errors
    ///
    /// Returns error if verification fails.
    pub async fn permanent_storage_verify_commit(
        &self,
        request: PermanentStorageVerifyRequest,
    ) -> ApiResult<bool> {
        let spine_id = request
            .spine_id
            .parse::<uuid::Uuid>()
            .map_err(|e| ApiError::InvalidRequest(format!("invalid spine_id UUID: {e}")))?;

        let entry_hash = hex_to_content_hash(&request.entry_hash)
            .map_err(|e| ApiError::InvalidRequest(format!("invalid entry_hash hex: {e}")))?;

        let get_resp = self
            .get_entry(GetEntryRequest {
                spine_id,
                entry_hash,
            })
            .await?;

        Ok(get_resp.found)
    }

    /// Get a commit using rhizoCrypt's wire format.
    ///
    /// # Errors
    ///
    /// Returns error if retrieval fails.
    pub async fn permanent_storage_get_commit(
        &self,
        request: PermanentStorageGetCommitRequest,
    ) -> ApiResult<serde_json::Value> {
        let spine_id = request
            .spine_id
            .parse::<uuid::Uuid>()
            .map_err(|e| ApiError::InvalidRequest(format!("invalid spine_id UUID: {e}")))?;

        let entry_hash = hex_to_content_hash(&request.entry_hash)
            .map_err(|e| ApiError::InvalidRequest(format!("invalid entry_hash hex: {e}")))?;

        let get_resp = self
            .get_entry(GetEntryRequest {
                spine_id,
                entry_hash,
            })
            .await?;

        if let Some(entry) = get_resp.entry {
            serde_json::to_value(&entry)
                .map_err(|e| ApiError::Internal(format!("serialization failed: {e}")))
        } else {
            Ok(serde_json::Value::Null)
        }
    }

    /// Ensure a permanence spine exists for the given committer DID.
    /// Uses the core service's idempotent `ensure_spine` method.
    async fn ensure_permanence_spine(&self, committer: &Did) -> ApiResult<SpineId> {
        let core = self.core_mut().await;
        core.ensure_spine(committer.clone(), None)
            .await
            .map_err(ApiError::from)
    }
}

/// Parse a hex-encoded string into a 32-byte `ContentHash`.
fn hex_to_content_hash(hex_str: &str) -> Result<ContentHash, String> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    if hex_str.len() != 64 {
        return Err(format!("expected 64 hex chars, got {}", hex_str.len()));
    }
    let mut hash = [0u8; 32];
    for (i, byte) in hash.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex_str[i * 2..i * 2 + 2], 16)
            .map_err(|e| format!("hex parse at byte {i}: {e}"))?;
    }
    Ok(hash)
}

/// Encode a 32-byte hash as lowercase hex.
fn bytes_to_hex(bytes: &[u8; 32]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::with_capacity(64), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}
