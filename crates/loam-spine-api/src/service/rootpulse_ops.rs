// SPDX-License-Identifier: AGPL-3.0-or-later

//! rootPulse graph step handlers for loamSpine.
//!
//! Implements the `ledger_commit` step in the `rootpulse_commit` graph.
//! biomeOS calls this via the `ledger.append` wire alias after:
//!   1. rhizoCrypt dehydrates cascade state
//!   2. bearDog signs the provenance blob
//!   3. nestGate stores in CAS
//!
//! loamSpine records the commit in an append-only spine, producing a
//! `ledger_ref` that sweetGrass weaves into an attribution braid.

use super::LoamSpineRpcService;
use crate::error::{ApiError, ApiResult};
use crate::types::*;
use loam_spine_core::entry::EntryType;
use loam_spine_core::traits::{CommitAcceptor, DehydrationSummary, SpineQuery};
use loam_spine_core::types::{Did, SessionId};
use tracing::debug;

/// Default DID for rootPulse graph commits when no committer is provided.
const ROOTPULSE_COMMITTER: &str = "did:primal:rootpulse";

/// Default provenance spine name.
const PROVENANCE_SPINE_NAME: &str = "rootpulse-provenance";

impl LoamSpineRpcService {
    /// rootPulse `ledger_commit` step — graph-step-compatible entry point.
    ///
    /// Accepts the `rootpulse_commit` graph's `{cas_ref, signed_provenance}`
    /// inputs and records a permanence commit. Auto-creates the provenance
    /// spine if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns error if the commit cannot be recorded.
    pub async fn rootpulse_ledger_commit(
        &self,
        request: RootpulseLedgerCommitRequest,
    ) -> ApiResult<RootpulseLedgerCommitResponse> {
        let committer = Did::new(request.committer.as_deref().unwrap_or(ROOTPULSE_COMMITTER));

        let spine_id = if let Some(id) = &request.spine_id {
            id.parse::<uuid::Uuid>()
                .map_err(|e| ApiError::InvalidRequest(format!("invalid spine_id: {e}")))?
        } else {
            let core = self.core_mut().await;
            core.ensure_spine(committer.clone(), Some(PROVENANCE_SPINE_NAME.into()))
                .await
                .map_err(ApiError::from)?
        };

        let session_id = match &request.session_id {
            Some(s) => s
                .parse::<uuid::Uuid>()
                .map_err(|e| ApiError::InvalidRequest(format!("invalid session_id: {e}")))?,
            None => SessionId::now_v7(),
        };

        let merkle_root = parse_cas_ref_to_hash(&request.cas_ref)?;

        let mut summary = DehydrationSummary::new(session_id, "rootpulse_commit", merkle_root)
            .with_vertex_count(1);

        summary = summary.with_metadata("cas_ref", &request.cas_ref);
        summary = summary.with_metadata("graph", "rootpulse_commit");
        summary = summary.with_metadata("step", "ledger_commit");

        if let Some(ref content_hash) = request.content_hash {
            summary = summary.with_metadata("content_hash", content_hash);
        }

        if let serde_json::Value::Object(ref map) = request.signed_provenance
            && let Some(serde_json::Value::String(sig)) = map.get("signature")
        {
            summary = summary.with_metadata("provenance_signature", sig);
        }

        let commit_ref = {
            let core = self.core_mut().await;
            core.commit_session(spine_id, committer, summary)
                .await
                .map_err(ApiError::from)?
        };

        let entry_hash_hex = bytes_to_hex(&commit_ref.entry_hash);
        let ledger_ref = format!("{spine_id}:{entry_hash_hex}:{}", commit_ref.index);

        debug!(
            spine_id = %spine_id,
            index = commit_ref.index,
            ledger_ref = %ledger_ref,
            "rootpulse.ledger_commit complete"
        );

        Ok(RootpulseLedgerCommitResponse {
            ledger_ref,
            spine_id,
            entry_hash: commit_ref.entry_hash,
            index: commit_ref.index,
            committed_at: commit_ref.committed_at,
        })
    }

    /// rootPulse `query_commit` — query recent provenance commits.
    ///
    /// Scans session-commit entries in the provenance spine, optionally
    /// filtering by metadata fields (`wave_id`, `target_triple`, `primal_name`).
    ///
    /// # Errors
    ///
    /// Returns error if the spine cannot be queried.
    pub async fn rootpulse_query_commit(
        &self,
        request: RootpulseQueryCommitRequest,
    ) -> ApiResult<RootpulseQueryCommitResponse> {
        let limit = request.limit.unwrap_or(20);
        let committer = Did::new(ROOTPULSE_COMMITTER);

        let spine_id = {
            let core = self.core_mut().await;
            match core
                .ensure_spine(committer, Some(PROVENANCE_SPINE_NAME.into()))
                .await
            {
                Ok(id) => id,
                Err(_) => {
                    return Ok(RootpulseQueryCommitResponse {
                        commits: Vec::new(),
                        total: 0,
                    });
                }
            }
        };

        let spine = {
            let core = self.core().await;
            core.get_spine(spine_id).await.map_err(ApiError::from)?
        };

        let height = match spine {
            Some(ref s) => s.height,
            None => {
                return Ok(RootpulseQueryCommitResponse {
                    commits: Vec::new(),
                    total: 0,
                });
            }
        };

        let entries = {
            let core = self.core().await;
            core.get_entries(spine_id, 0, height)
                .await
                .map_err(ApiError::from)?
        };

        let mut commits: Vec<RootpulseCommitEntry> = Vec::new();

        for (idx, entry) in entries.iter().enumerate().rev() {
            if !matches!(entry.entry_type, EntryType::SessionCommit { .. }) {
                continue;
            }

            if let Some(ref wave) = request.wave_id {
                let entry_wave = entry.metadata.get("wave_id").map_or("", String::as_str);
                if !entry_wave.contains(wave.as_str()) {
                    continue;
                }
            }

            if let Some(ref target) = request.target_triple {
                let entry_target = entry
                    .metadata
                    .get("target_triple")
                    .map_or("", String::as_str);
                if entry_target != target.as_str() {
                    continue;
                }
            }

            if let Some(ref primal) = request.primal_name {
                let entry_primal = entry.metadata.get("primal_name").map_or("", String::as_str);
                if entry_primal != primal.as_str() {
                    continue;
                }
            }

            let entry_hash = entry
                .compute_hash()
                .map_err(|e| ApiError::Internal(format!("entry hash: {e}")))?;
            let entry_hash_hex = bytes_to_hex(&entry_hash);
            let index = u64::try_from(idx).unwrap_or(u64::MAX);
            let ledger_ref = format!("{spine_id}:{entry_hash_hex}:{index}");

            let session_id_str =
                if let EntryType::SessionCommit { session_id, .. } = &entry.entry_type {
                    Some(session_id.to_string())
                } else {
                    None
                };

            let committer_str =
                if let EntryType::SessionCommit { committer, .. } = &entry.entry_type {
                    Some(committer.as_str().to_string())
                } else {
                    None
                };

            commits.push(RootpulseCommitEntry {
                ledger_ref,
                entry_hash: entry_hash_hex,
                spine_id,
                index,
                committed_at: entry.timestamp,
                session_id: session_id_str,
                committer: committer_str,
            });

            if u64::try_from(commits.len()).unwrap_or(0) >= limit {
                break;
            }
        }

        let total = u64::try_from(commits.len()).unwrap_or(0);

        Ok(RootpulseQueryCommitResponse { commits, total })
    }
}

/// Parse a CAS reference (hex string) into a 32-byte content hash.
fn parse_cas_ref_to_hash(cas_ref: &str) -> ApiResult<[u8; 32]> {
    let hex_str = cas_ref.strip_prefix("0x").unwrap_or(cas_ref);
    if hex_str.len() != 64 {
        return Err(ApiError::InvalidRequest(format!(
            "invalid cas_ref: expected 64 hex chars, got {}",
            hex_str.len()
        )));
    }
    let mut hash = [0u8; 32];
    for (i, byte) in hash.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex_str[i * 2..i * 2 + 2], 16).map_err(|e| {
            ApiError::InvalidRequest(format!("invalid cas_ref hex at byte {i}: {e}"))
        })?;
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
