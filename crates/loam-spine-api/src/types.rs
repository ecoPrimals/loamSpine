// SPDX-License-Identifier: AGPL-3.0-or-later

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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn create_spine_request_roundtrip() {
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
    fn create_spine_response_roundtrip() {
        let resp = CreateSpineResponse {
            spine_id: uuid::Uuid::nil(),
            genesis_hash: [0u8; 32],
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("spine_id"));
        let parsed: CreateSpineResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.spine_id, uuid::Uuid::nil());
    }

    #[test]
    fn get_spine_request_roundtrip() {
        let req = GetSpineRequest {
            spine_id: uuid::Uuid::now_v7(),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: GetSpineRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.spine_id, req.spine_id);
    }

    #[test]
    fn seal_spine_request_roundtrip() {
        let req = SealSpineRequest {
            spine_id: uuid::Uuid::now_v7(),
            sealer: Did::new("did:key:sealer"),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: SealSpineRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.spine_id, req.spine_id);
    }

    #[test]
    fn append_entry_request_roundtrip() {
        let req = AppendEntryRequest {
            spine_id: uuid::Uuid::now_v7(),
            entry_type: EntryType::SpineSealed { reason: None },
            committer: Did::new("did:key:z6Mk1"),
            payload: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: AppendEntryRequest = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed.entry_type, EntryType::SpineSealed { .. }));
    }

    #[test]
    fn mint_certificate_request_roundtrip() {
        let req = MintCertificateRequest {
            spine_id: uuid::Uuid::now_v7(),
            cert_type: CertificateType::DigitalCollectible {
                collection_id: "test-collection".into(),
                item_number: Some(1),
                total_supply: Some(100),
                rarity: Some(Rarity::Rare),
            },
            owner: Did::new("did:key:owner"),
            metadata: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: MintCertificateRequest = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(
            parsed.cert_type,
            CertificateType::DigitalCollectible { .. }
        ));
    }

    #[test]
    fn health_check_response_roundtrip() {
        let resp = HealthCheckResponse {
            status: HealthStatus::Healthy,
            report: None,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let parsed: HealthCheckResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed.status, HealthStatus::Healthy));
    }

    #[test]
    fn commit_session_request_roundtrip() {
        let req = CommitSessionRequest {
            spine_id: uuid::Uuid::now_v7(),
            session_id: uuid::Uuid::now_v7(),
            session_hash: [1u8; 32],
            vertex_count: 42,
            committer: Did::new("did:key:committer"),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: CommitSessionRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.vertex_count, 42);
    }

    #[test]
    fn permanent_storage_commit_request_roundtrip() {
        let req = PermanentStorageCommitRequest {
            session_id: uuid::Uuid::now_v7().to_string(),
            merkle_root: "a".repeat(64),
            committer_did: Some("did:key:z6MkTest".into()),
            summary: PermanentStorageDehydrationSummary {
                session_type: "game".into(),
                vertex_count: 10,
                leaf_count: 5,
                started_at: 1_000_000,
                ended_at: 2_000_000,
                outcome: "committed".to_string(),
            },
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: PermanentStorageCommitRequest =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.summary.vertex_count, 10);
    }

    #[test]
    fn permanent_storage_commit_response_roundtrip() {
        let resp = PermanentStorageCommitResponse {
            accepted: true,
            commit_id: Some("abc123".into()),
            spine_entry_hash: Some("def456".into()),
            entry_index: Some(5),
            spine_id: Some(uuid::Uuid::nil().to_string()),
            error: None,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let parsed: PermanentStorageCommitResponse =
            serde_json::from_str(&json).expect("deserialize");
        assert!(parsed.accepted);
        assert_eq!(parsed.entry_index, Some(5));
    }

    #[test]
    fn serde_opt_bytes_none_roundtrip() {
        let req = AppendEntryRequest {
            spine_id: uuid::Uuid::now_v7(),
            entry_type: EntryType::SpineSealed { reason: None },
            committer: Did::new("did:key:z6Mk1"),
            payload: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: AppendEntryRequest = serde_json::from_str(&json).expect("deserialize");
        assert!(parsed.payload.is_none());
    }

    #[test]
    fn serde_opt_bytes_some_roundtrip() {
        let payload_data = vec![1u8, 2, 3, 4, 5];
        let req = AppendEntryRequest {
            spine_id: uuid::Uuid::now_v7(),
            entry_type: EntryType::SpineSealed { reason: None },
            committer: Did::new("did:key:z6Mk1"),
            payload: Some(ByteBuffer::from(payload_data)),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: AppendEntryRequest = serde_json::from_str(&json).expect("deserialize");
        assert!(parsed.payload.is_some());
    }

    #[test]
    fn anchor_slice_request_roundtrip() {
        let req = AnchorSliceRequest {
            waypoint_spine_id: uuid::Uuid::now_v7(),
            origin_spine_id: uuid::Uuid::now_v7(),
            slice_id: uuid::Uuid::now_v7(),
            committer: Did::new("did:key:z6MkWaypoint"),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: AnchorSliceRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.waypoint_spine_id, req.waypoint_spine_id);
    }

    #[test]
    fn checkout_slice_request_roundtrip() {
        let req = CheckoutSliceRequest {
            waypoint_spine_id: uuid::Uuid::now_v7(),
            slice_id: uuid::Uuid::now_v7(),
            requester: Did::new("did:key:z6MkRequester"),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: CheckoutSliceRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.slice_id, req.slice_id);
    }

    #[test]
    fn commit_braid_request_roundtrip() {
        let req = CommitBraidRequest {
            spine_id: uuid::Uuid::now_v7(),
            braid_id: uuid::Uuid::now_v7(),
            braid_hash: [2u8; 32],
            subjects: vec![Did::new("did:key:agent1"), Did::new("did:key:agent2")],
            committer: Did::new("did:key:z6MkCommitter"),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: CommitBraidRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.subjects.len(), 2);
    }

    #[test]
    fn permanent_storage_verify_request_roundtrip() {
        let req = PermanentStorageVerifyRequest {
            spine_id: uuid::Uuid::nil().to_string(),
            entry_hash: "b".repeat(64),
            index: 7,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: PermanentStorageVerifyRequest =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.index, 7);
    }
}
