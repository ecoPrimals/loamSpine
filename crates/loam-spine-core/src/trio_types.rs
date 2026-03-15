// SPDX-License-Identifier: AGPL-3.0-only

//! Type bridge for the provenance trio coordination.
//!
//! The provenance trio consists of:
//! - **rhizoCrypt** (ephemeral DAG): uses `String` for UUIDs and hex hashes
//! - **LoamSpine** (permanent history): uses `uuid::Uuid` for IDs and `[u8; 32]` for hashes
//! - **sweetGrass** (attribution): uses `BraidId` as URN strings like `"urn:braid:..."`
//!
//! This module provides conversion types and `TryFrom` implementations to bridge
//! between these representations for trio-coordinated commits.
//!
//! The canonical wire types live in [`provenance_trio_types`] and are re-exported
//! here for convenience. All IPC boundaries should use the canonical types.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::LoamSpineError;
use crate::types::{Did, EntryHash, Signature, SpineId, Timestamp};

pub use provenance_trio_types::{
    self as wire, AgentRef as WireAgentRef, AttestationRef as WireAttestationRef,
    DehydrationSummary as WireDehydrationSummary, PipelineRequest, PipelineResult,
    SessionOperationRef as WireSessionOperationRef,
};

/// Ephemeral session ID from rhizoCrypt (opaque string, typically UUID v7 hex).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EphemeralSessionId(pub String);

/// Braid identifier from sweetGrass (URN format: `urn:braid:{uuid}`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BraidRef(pub String);

/// Content hash from rhizoCrypt (hex-encoded blake3 digest).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EphemeralContentHash(pub String);

impl EphemeralSessionId {
    /// Create from a string.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    /// Get the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl BraidRef {
    /// URN prefix for braid references.
    pub const URN_PREFIX: &'static str = "urn:braid:";
    /// Create from a string.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    /// Get the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl EphemeralContentHash {
    /// Create from a hex string.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    /// Get the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EphemeralSessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for BraidRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for EphemeralContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<EphemeralSessionId> for uuid::Uuid {
    type Error = LoamSpineError;

    fn try_from(id: EphemeralSessionId) -> Result<Self, Self::Error> {
        Self::parse_str(&id.0)
            .map_err(|e| LoamSpineError::InvalidData(format!("invalid session id hex: {e}")))
    }
}

impl TryFrom<uuid::Uuid> for EphemeralSessionId {
    type Error = LoamSpineError;

    fn try_from(uuid: uuid::Uuid) -> Result<Self, Self::Error> {
        Ok(Self(uuid.as_simple().to_string()))
    }
}

impl TryFrom<EphemeralContentHash> for EntryHash {
    type Error = LoamSpineError;

    fn try_from(hash: EphemeralContentHash) -> Result<Self, Self::Error> {
        let hex_str = hash.0.strip_prefix("0x").unwrap_or(&hash.0);
        if hex_str.len() != 64 {
            return Err(LoamSpineError::InvalidData(format!(
                "expected 64 hex chars for content hash, got {}",
                hex_str.len()
            )));
        }
        let mut out = [0u8; 32];
        for (i, byte) in out.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&hex_str[i * 2..i * 2 + 2], 16)
                .map_err(|e| LoamSpineError::InvalidData(format!("hex parse at byte {i}: {e}")))?;
        }
        Ok(out)
    }
}

impl TryFrom<EntryHash> for EphemeralContentHash {
    type Error = LoamSpineError;

    fn try_from(hash: EntryHash) -> Result<Self, Self::Error> {
        use std::fmt::Write;
        let s = hash.iter().fold(String::with_capacity(64), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        });
        Ok(Self(s))
    }
}

impl TryFrom<BraidRef> for uuid::Uuid {
    type Error = LoamSpineError;

    fn try_from(br: BraidRef) -> Result<Self, Self::Error> {
        let s = br.0.strip_prefix(BraidRef::URN_PREFIX).ok_or_else(|| {
            LoamSpineError::InvalidData(format!(
                "braid ref must start with '{}', got: {}",
                BraidRef::URN_PREFIX,
                br.0
            ))
        })?;
        Self::parse_str(s)
            .map_err(|e| LoamSpineError::InvalidData(format!("invalid braid uuid: {e}")))
    }
}

impl TryFrom<uuid::Uuid> for BraidRef {
    type Error = LoamSpineError;

    fn try_from(uuid: uuid::Uuid) -> Result<Self, Self::Error> {
        Ok(Self(format!("{}{}", Self::URN_PREFIX, uuid)))
    }
}

/// A trio-coordinated commit request.
/// Bridges rhizoCrypt's dehydrated session into LoamSpine's permanent record.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[must_use]
pub struct TrioCommitRequest {
    /// Ephemeral session being committed (from rhizoCrypt).
    pub session_id: EphemeralSessionId,
    /// Content hash of the dehydrated DAG (from rhizoCrypt).
    pub content_hash: EphemeralContentHash,
    /// DID of the committer.
    pub committer: Did,
    /// Optional braid reference for attribution (from sweetGrass).
    pub braid_ref: Option<BraidRef>,
    /// Optional signature over the content hash.
    pub signature: Option<Signature>,
}

/// Receipt returned after a trio-coordinated commit.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[must_use]
pub struct TrioCommitReceipt {
    /// LoamSpine's spine ID where the commit was recorded.
    pub spine_id: SpineId,
    /// Entry hash of the committed record in LoamSpine.
    pub entry_hash: EntryHash,
    /// Index of the entry in the spine.
    pub entry_index: u64,
    /// Timestamp of the commit.
    pub committed_at: Timestamp,
}

impl TrioCommitReceipt {
    /// Convert to the canonical [`PipelineResult`] for biomeOS graph execution.
    #[must_use]
    pub fn to_pipeline_result(
        &self,
        merkle_root: &str,
        braid_ref: Option<String>,
    ) -> PipelineResult {
        use std::fmt::Write;
        let commit_ref = self
            .entry_hash
            .iter()
            .fold(String::with_capacity(64), |mut acc, b| {
                let _ = write!(acc, "{b:02x}");
                acc
            });
        PipelineResult {
            dehydration_merkle_root: merkle_root.to_string(),
            commit_ref,
            braid_ref,
            signature: None,
            content_ref: None,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn ephemeral_session_id_roundtrip() {
        let uuid = uuid::Uuid::now_v7();
        let ephemeral: EphemeralSessionId = uuid.try_into().unwrap();
        let back: uuid::Uuid = ephemeral.try_into().unwrap();
        assert_eq!(uuid, back);
    }

    #[test]
    fn ephemeral_session_id_from_invalid_hex_fails() {
        let id = EphemeralSessionId::new("not-a-valid-uuid");
        let result: Result<uuid::Uuid, _> = id.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn ephemeral_content_hash_roundtrip() {
        let hash: EntryHash = blake3::hash(b"test").into();
        let ephemeral: EphemeralContentHash = hash.try_into().unwrap();
        let back: EntryHash = ephemeral.try_into().unwrap();
        assert_eq!(hash, back);
    }

    #[test]
    fn ephemeral_content_hash_from_invalid_hex_fails() {
        let h = EphemeralContentHash::new("not-valid-hex");
        let result: Result<EntryHash, _> = h.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn ephemeral_content_hash_wrong_length_fails() {
        let h = EphemeralContentHash::new("abc"); // too short
        let result: Result<EntryHash, _> = h.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn braid_ref_roundtrip() {
        let uuid = uuid::Uuid::now_v7();
        let braid: BraidRef = uuid.try_into().unwrap();
        assert!(braid.as_str().starts_with(BraidRef::URN_PREFIX));
        let back: uuid::Uuid = braid.try_into().unwrap();
        assert_eq!(uuid, back);
    }

    #[test]
    fn braid_ref_wrong_prefix_fails() {
        let br = BraidRef::new("urn:other:12345678-1234-1234-1234-123456789012");
        let result: Result<uuid::Uuid, _> = br.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn ephemeral_session_id_display() {
        let id = EphemeralSessionId::new("abc123");
        assert_eq!(id.to_string(), "abc123");
    }

    #[test]
    fn braid_ref_display() {
        let br = BraidRef::new("urn:braid:12345678-1234-1234-1234-123456789012");
        assert!(br.to_string().contains("urn:braid:"));
    }

    #[test]
    fn ephemeral_content_hash_display() {
        let h = EphemeralContentHash::new("deadbeef");
        assert_eq!(h.to_string(), "deadbeef");
    }

    #[test]
    fn trio_commit_request_serialization() {
        let req = TrioCommitRequest {
            session_id: EphemeralSessionId::new(uuid::Uuid::now_v7().as_simple().to_string()),
            content_hash: EphemeralContentHash::new("a".repeat(64)),
            committer: Did::new("did:key:z6MkTest"),
            braid_ref: Some(BraidRef::new(format!("urn:braid:{}", uuid::Uuid::now_v7()))),
            signature: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let _: TrioCommitRequest = serde_json::from_str(&json).expect("deserialize");
    }

    #[test]
    fn trio_commit_receipt_serialization() {
        let hash: EntryHash = blake3::hash(b"receipt").into();
        let receipt = TrioCommitReceipt {
            spine_id: uuid::Uuid::now_v7(),
            entry_hash: hash,
            entry_index: 42,
            committed_at: Timestamp::now(),
        };
        let json = serde_json::to_string(&receipt).expect("serialize");
        let _: TrioCommitReceipt = serde_json::from_str(&json).expect("deserialize");
    }

    #[test]
    fn wire_dehydration_summary_deserializes() {
        let payload = serde_json::json!({
            "session_id": "sess-1",
            "source_primal": "rhizoCrypt",
            "merkle_root": "sha256:abc",
            "vertex_count": 10,
            "agents": ["did:key:z6MkAlice"],
            "session_type": "experiment",
            "outcome": "Success"
        });
        let s: WireDehydrationSummary = serde_json::from_value(payload).unwrap();
        assert_eq!(s.session_id, "sess-1");
        assert_eq!(s.vertex_count, 10);
        assert_eq!(s.branch_count, 0); // serde default
    }

    #[test]
    fn trio_receipt_to_pipeline_result() {
        let hash: EntryHash = blake3::hash(b"test").into();
        let receipt = TrioCommitReceipt {
            spine_id: uuid::Uuid::now_v7(),
            entry_hash: hash,
            entry_index: 1,
            committed_at: Timestamp::now(),
        };
        let result = receipt.to_pipeline_result("sha256:abc", Some("urn:braid:123".into()));
        assert_eq!(result.dehydration_merkle_root, "sha256:abc");
        assert!(!result.commit_ref.is_empty());
        assert_eq!(result.braid_ref.as_deref(), Some("urn:braid:123"));
    }
}
