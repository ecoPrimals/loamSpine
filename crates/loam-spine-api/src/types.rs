// SPDX-License-Identifier: AGPL-3.0-only

//! RPC message types for `LoamSpine`.
//!
//! These types use native Rust serde serialization - no protobuf required.

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

/// Serde helpers for `Option<ByteBuffer>` fields in RPC types.
mod serde_opt_bytes {
    use super::ByteBuffer;

    #[allow(clippy::ref_option)]
    pub fn serialize<S>(val: &Option<ByteBuffer>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match val {
            Some(b) => serializer.serialize_bytes(b),
            None => serializer.serialize_none(),
        }
    }

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
// Certificate Operations
// ============================================================================

/// Request to mint a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintCertificateRequest {
    /// Spine ID to mint on
    pub spine_id: SpineId,
    /// Certificate type
    pub cert_type: CertificateType,
    /// Owner DID
    pub owner: Did,
    /// Certificate metadata
    pub metadata: Option<CertificateMetadata>,
}

/// Response from minting a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintCertificateResponse {
    /// Certificate ID
    pub certificate_id: CertificateId,
    /// Mint entry hash
    pub mint_hash: EntryHash,
}

/// Request to transfer a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferCertificateRequest {
    /// Certificate ID
    pub certificate_id: CertificateId,
    /// Current owner DID
    pub from: Did,
    /// New owner DID
    pub to: Did,
}

/// Response from transferring a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferCertificateResponse {
    /// Whether transfer succeeded
    pub success: bool,
    /// Transfer entry hash
    pub transfer_hash: Option<EntryHash>,
}

/// Request to loan a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanCertificateRequest {
    /// Certificate ID
    pub certificate_id: CertificateId,
    /// Lender DID
    pub lender: Did,
    /// Borrower DID
    pub borrower: Did,
    /// Loan terms
    pub terms: LoanTerms,
}

/// Response from loaning a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanCertificateResponse {
    /// Whether loan succeeded
    pub success: bool,
    /// Loan entry hash
    pub loan_hash: Option<EntryHash>,
}

/// Request to return a loaned certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnCertificateRequest {
    /// Certificate ID
    pub certificate_id: CertificateId,
    /// Returner DID (borrower)
    pub returner: Did,
}

/// Response from returning a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnCertificateResponse {
    /// Whether return succeeded
    pub success: bool,
    /// Return entry hash
    pub return_hash: Option<EntryHash>,
}

/// Request to get a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCertificateRequest {
    /// Certificate ID
    pub certificate_id: CertificateId,
}

/// Response containing certificate data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCertificateResponse {
    /// Whether the certificate was found
    pub found: bool,
    /// The certificate if found
    pub certificate: Option<Certificate>,
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
    /// Whether to include detailed component health
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

/// Response from committing a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSessionResponse {
    /// Commit entry hash
    pub commit_hash: EntryHash,
    /// Entry index
    pub index: u64,
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

// ============================================================================
// Permanent Storage Compatibility (rhizoCrypt wire format)
// ============================================================================
// rhizoCrypt's LoamSpineHttpClient calls `permanent-storage.*` methods
// with its own request shapes. These types accept that wire format and
// translate to loamSpine's native types.

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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = CreateSpineRequest {
            name: "test-spine".to_string(),
            owner: Did::new("did:key:test"),
            config: None,
        };

        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: CreateSpineRequest = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.name, "test-spine");
    }

    #[test]
    fn test_response_serialization() {
        let resp = CreateSpineResponse {
            spine_id: uuid::Uuid::nil(),
            genesis_hash: [0u8; 32],
        };

        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("spine_id"));
        assert!(json.contains("genesis_hash"));
    }
}
