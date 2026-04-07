// SPDX-License-Identifier: AGPL-3.0-or-later

//! Permanent storage compatibility types for the rhizoCrypt wire format.
//!
//! rhizoCrypt's `LoamSpineHttpClient` calls `permanent-storage.*` methods
//! with its own request shapes. These types accept that wire format and
//! translate to loamSpine's native types.

use serde::{Deserialize, Serialize};

/// Dehydration summary subset sent by rhizoCrypt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermanentStorageDehydrationSummary {
    /// Session type (game, transaction, etc.)
    pub session_type: String,
    /// Vertex count
    pub vertex_count: u64,
    /// Leaf/result count
    pub leaf_count: u64,
    /// Session start time (nanoseconds since epoch)
    pub started_at: u64,
    /// Session end time (nanoseconds since epoch)
    pub ended_at: u64,
    /// Outcome as debug string (e.g. "Success", "Failure { reason: ... }")
    pub outcome: String,
}

/// Request from rhizoCrypt's `permanent-storage.commitSession`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermanentStorageCommitRequest {
    /// Session UUID as string
    pub session_id: String,
    /// Merkle root as hex-encoded 32 bytes
    pub merkle_root: String,
    /// Dehydration summary subset
    pub summary: PermanentStorageDehydrationSummary,
    /// Committer DID (first agent in session)
    pub committer_did: Option<String>,
}

/// Response for rhizoCrypt's `permanent-storage.commitSession`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermanentStorageCommitResponse {
    /// Whether the commit was accepted
    pub accepted: bool,
    /// Commit ID (spine entry hash as hex)
    pub commit_id: Option<String>,
    /// Spine entry hash as hex
    pub spine_entry_hash: Option<String>,
    /// Entry index in spine
    pub entry_index: Option<u64>,
    /// Spine ID (so rhizoCrypt can reference it later)
    pub spine_id: Option<String>,
    /// Error message if rejected
    pub error: Option<String>,
}

/// Request for rhizoCrypt's `permanent-storage.verifyCommit`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermanentStorageVerifyRequest {
    /// Spine ID
    pub spine_id: String,
    /// Entry hash (hex-encoded)
    pub entry_hash: String,
    /// Entry index
    pub index: u64,
}

/// Request for rhizoCrypt's `permanent-storage.getCommit`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermanentStorageGetCommitRequest {
    /// Spine ID
    pub spine_id: String,
    /// Entry hash (hex-encoded)
    pub entry_hash: String,
    /// Entry index
    pub index: u64,
}

impl From<&loam_spine_core::trio_types::WireDehydrationSummary>
    for PermanentStorageDehydrationSummary
{
    fn from(w: &loam_spine_core::trio_types::WireDehydrationSummary) -> Self {
        Self {
            session_type: w.session_type.clone(),
            vertex_count: w.vertex_count,
            leaf_count: w.branch_count,
            started_at: w.session_start,
            ended_at: w.dehydrated_at,
            outcome: w.outcome.clone(),
        }
    }
}

impl From<&loam_spine_core::trio_types::WireDehydrationSummary> for PermanentStorageCommitRequest {
    fn from(w: &loam_spine_core::trio_types::WireDehydrationSummary) -> Self {
        Self {
            session_id: w.session_id.clone(),
            merkle_root: w.merkle_root.clone(),
            summary: PermanentStorageDehydrationSummary::from(w),
            committer_did: w.agents.first().cloned(),
        }
    }
}
