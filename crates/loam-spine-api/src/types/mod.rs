// SPDX-License-Identifier: AGPL-3.0-or-later

//! RPC message types for `LoamSpine`.
//!
//! These types use native Rust serde serialization - no protobuf required.
//! Domain-specific types are grouped in submodules; everything is re-exported
//! at this level for backward compatibility.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export core types for convenience
pub use loam_spine_core::certificate::{
    Certificate, CertificateMetadata, CertificateState, CertificateType, LoanInfo, LoanTerms,
    MintInfo, Rarity,
};
pub use loam_spine_core::entry::SpineConfig;
pub use loam_spine_core::entry::{Entry, EntryType};
pub use loam_spine_core::primal::{HealthReport, HealthStatus};
pub use loam_spine_core::proof::{CertificateProof, InclusionProof};
pub use loam_spine_core::spine::{Spine, SpineState};
pub use loam_spine_core::types::{
    ByteBuffer, CertificateId, ContentHash, Did, EntryHash, PeerId, Signature, SliceId, SpineId,
    Timestamp,
};

mod anchor;
mod bond_ledger;
mod certificate;
mod permanent_storage;

pub use anchor::*;
pub use bond_ledger::*;
pub use certificate::*;
pub use permanent_storage::*;

/// Serde helpers for `Option<ByteBuffer>` fields in RPC types.
///
/// Used via `#[serde(with = "serde_opt_bytes", default)]` on struct fields.
/// Serializes `Some(bytes)` as a raw byte array and `None` as JSON null.
/// Deserializes by reading an optional `Vec<u8>` and converting to `ByteBuffer`.
mod serde_opt_bytes {
    use super::ByteBuffer;

    /// Serialize an `Option<ByteBuffer>` as raw bytes (Some) or null (None).
    #[expect(
        clippy::ref_option,
        reason = "serde serialize_with requires &Option<T> signature"
    )]
    pub fn serialize<S>(val: &Option<ByteBuffer>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match val {
            Some(b) => serializer.serialize_bytes(b),
            None => serializer.serialize_none(),
        }
    }

    /// Deserialize an `Option<ByteBuffer>` from an optional byte array.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ByteBuffer>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let opt: Option<Vec<u8>> = serde::Deserialize::deserialize(deserializer)?;
        Ok(opt.map(ByteBuffer::from))
    }
}

// ============================================================================
// Spine Operations
// ============================================================================

/// Request to create a new spine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSpineRequest {
    /// Name for the spine
    pub name: String,
    /// Owner DID
    pub owner: Did,
    /// Optional configuration
    pub config: Option<SpineConfig>,
}

/// Response from creating a spine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSpineResponse {
    /// The created spine ID
    pub spine_id: SpineId,
    /// Genesis entry hash
    pub genesis_hash: EntryHash,
}

/// Request to get a spine by ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSpineRequest {
    /// Spine ID to retrieve
    pub spine_id: SpineId,
}

/// Response containing spine data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSpineResponse {
    /// Whether the spine was found
    pub found: bool,
    /// The spine if found
    pub spine: Option<Spine>,
}

/// Request to seal a spine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealSpineRequest {
    /// Spine ID to seal
    pub spine_id: SpineId,
    /// Sealer DID
    pub sealer: Did,
}

/// Response from sealing a spine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealSpineResponse {
    /// Whether the seal was successful
    pub success: bool,
    /// Seal entry hash
    pub seal_hash: Option<EntryHash>,
}

// ============================================================================
// Entry Operations
// ============================================================================

/// Request to append an entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntryRequest {
    /// Target spine ID
    pub spine_id: SpineId,
    /// Entry type
    pub entry_type: EntryType,
    /// Committer DID
    pub committer: Did,
    /// Optional payload (zero-copy via `bytes::Bytes`)
    #[serde(with = "serde_opt_bytes", default)]
    pub payload: Option<ByteBuffer>,
}

/// Response from appending an entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntryResponse {
    /// Entry hash
    pub entry_hash: EntryHash,
    /// Entry index
    pub index: u64,
}

/// Request to get an entry by hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEntryRequest {
    /// Spine ID
    pub spine_id: SpineId,
    /// Entry hash
    pub entry_hash: EntryHash,
}

/// Response containing entry data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEntryResponse {
    /// Whether the entry was found
    pub found: bool,
    /// The entry if found
    pub entry: Option<Entry>,
}

/// Request to get the tip entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTipRequest {
    /// Spine ID
    pub spine_id: SpineId,
}

/// Response containing the tip entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTipResponse {
    /// Tip entry hash
    pub tip_hash: EntryHash,
    /// Tip entry
    pub entry: Entry,
    /// Current height
    pub height: u64,
}

// ============================================================================
// Slice/Waypoint Operations
// ============================================================================

/// Request to anchor a slice on a waypoint spine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorSliceRequest {
    /// Waypoint spine ID
    pub waypoint_spine_id: SpineId,
    /// Slice ID
    pub slice_id: SliceId,
    /// Origin spine ID
    pub origin_spine_id: SpineId,
    /// Committer DID
    pub committer: Did,
}

/// Response from anchoring a slice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorSliceResponse {
    /// Anchor entry hash
    pub anchor_hash: EntryHash,
}

/// Request to checkout a slice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutSliceRequest {
    /// Waypoint spine ID
    pub waypoint_spine_id: SpineId,
    /// Slice ID
    pub slice_id: SliceId,
    /// Requester DID
    pub requester: Did,
}

/// Response from checking out a slice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutSliceResponse {
    /// Whether checkout succeeded
    pub success: bool,
    /// Checkout entry hash
    pub checkout_hash: Option<EntryHash>,
}

// ============================================================================
// Proof Operations
// ============================================================================

/// Request to generate an inclusion proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateInclusionProofRequest {
    /// Spine ID
    pub spine_id: SpineId,
    /// Entry hash to prove
    pub entry_hash: EntryHash,
}

/// Response containing an inclusion proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateInclusionProofResponse {
    /// The inclusion proof
    pub proof: InclusionProof,
}

/// Request to verify an inclusion proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyInclusionProofRequest {
    /// The proof to verify
    pub proof: InclusionProof,
}

/// Response from verifying a proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyInclusionProofResponse {
    /// Whether the proof is valid
    pub valid: bool,
    /// Verification message
    pub message: String,
}

// ============================================================================
// Health Operations
// ============================================================================

/// Request for health check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckRequest {
    /// Whether to include detailed component health.
    ///
    /// Defaults to `false` when omitted, so consumers can call
    /// `health.check` with `{}` or no params.
    #[serde(default)]
    pub include_details: bool,
}

/// Response containing health status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// Overall health status
    pub status: HealthStatus,
    /// Detailed health report if requested
    pub report: Option<HealthReport>,
}

// ============================================================================
// Ephemeral Storage Integration
// ============================================================================

/// Session commit request from an ephemeral storage primal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSessionRequest {
    /// Target spine ID
    pub spine_id: SpineId,
    /// Session ID
    pub session_id: Uuid,
    /// Session hash (DAG root)
    pub session_hash: ContentHash,
    /// Vertex count in session
    pub vertex_count: u64,
    /// Committer DID
    pub committer: Did,
}

/// Response from committing a session — self-contained provenance receipt.
///
/// Contains both the ledger anchor (spine + hash + index + time) and the
/// session binding (`session_id` + `merkle_root` + `vertex_count` + committer)
/// so downstream consumers can trace computation provenance without
/// follow-up entry fetches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSessionResponse {
    // -- Ledger anchor --
    /// Spine where the commit was recorded.
    pub spine_id: SpineId,
    /// Commit entry hash.
    pub commit_hash: EntryHash,
    /// Entry index in the spine.
    pub index: u64,
    /// Timestamp of the committed entry.
    pub committed_at: Timestamp,

    // -- Session binding (echoed from request) --
    /// Session that was committed.
    pub session_id: Uuid,
    /// Merkle root of the session DAG.
    pub merkle_root: ContentHash,
    /// Number of vertices in the session.
    pub vertex_count: u64,
    /// DID of the committer.
    pub committer: Did,

    // -- Tower signature (when signing is enabled) --
    /// Ed25519 signature over the entry's canonical bytes (base64),
    /// present only when `BEARDOG_SOCKET` is configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tower_signature: Option<String>,
}

// ============================================================================
// Semantic Attribution Integration
// ============================================================================

/// Braid commit request from a semantic attribution primal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitBraidRequest {
    /// Target spine ID
    pub spine_id: SpineId,
    /// Braid ID
    pub braid_id: Uuid,
    /// Braid hash
    pub braid_hash: ContentHash,
    /// Subjects referenced
    pub subjects: Vec<Did>,
    /// Committer DID
    pub committer: Did,
}

/// Response from committing a braid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitBraidResponse {
    /// Commit entry hash
    pub commit_hash: EntryHash,
    /// Entry index
    pub index: u64,
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "tests use expect for concise error paths"
)]
#[path = "tests.rs"]
mod tests;
