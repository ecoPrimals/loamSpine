// SPDX-License-Identifier: AGPL-3.0-or-later

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
//! Wire types are defined locally — each primal owns its own representation of the
//! JSON-RPC boundary. The JSON shape is the contract, not a shared Rust crate.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::LoamSpineError;
use crate::types::{Did, EntryHash, Signature, SpineId, Timestamp};

// ─── Wire types (JSON boundary) ─────────────────────────────────────────────
// These mirror the JSON shapes produced by rhizoCrypt and consumed by biomeOS.
// Each primal owns its own copy; the wire format (JSON) is the shared contract.

/// Dehydration summary received from rhizoCrypt over JSON-RPC.
///
/// All optional fields use `#[serde(default)]` for backward-compatible
/// deserialization — rhizoCrypt may evolve its payload over time.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireDehydrationSummary {
    /// The primal that performed the dehydration.
    pub source_primal: String,
    /// The session that was dehydrated.
    pub session_id: String,
    /// Merkle root of the collapsed DAG (hex or prefixed hash string).
    pub merkle_root: String,
    /// Total number of vertices in the original DAG.
    pub vertex_count: u64,
    /// Number of branches explored (0 if unknown).
    #[serde(default)]
    pub branch_count: u64,
    /// Total payload bytes (0 if unknown).
    #[serde(default)]
    pub payload_bytes: u64,
    /// DIDs of agents who participated.
    #[serde(default)]
    pub agents: Vec<String>,
    /// When the session was created (nanoseconds since epoch, 0 if unknown).
    #[serde(default)]
    pub session_start: u64,
    /// When dehydration occurred (nanoseconds since epoch, 0 if unknown).
    #[serde(default)]
    pub dehydrated_at: u64,
    /// Session type identifier (e.g., "experiment", "rootpulse").
    #[serde(default)]
    pub session_type: String,
    /// Session outcome as a string (e.g., "Success", "Failed").
    #[serde(default)]
    pub outcome: String,
    /// Agent participation summaries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agent_summaries: Vec<WireAgentRef>,
    /// Session witnesses (signatures, hash observations, checkpoints, markers).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub witnesses: Vec<WireWitnessRef>,
    /// Operations performed during the session.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<WireSessionOperationRef>,
    /// Frontier hashes (leaf nodes of the DAG at dehydration time).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frontier: Vec<String>,
    /// Niche context (e.g., "rootpulse", "chemistry").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub niche: Option<String>,
    /// Compression ratio if the DAG was compressed before dehydration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compression_ratio: Option<f64>,
}

/// Per-agent participation summary in a dehydration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireAgentRef {
    /// Agent DID.
    pub agent: String,
    /// When the agent joined (nanoseconds since epoch).
    #[serde(default)]
    pub joined_at: u64,
    /// When the agent left (None if still active).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_at: Option<u64>,
    /// Number of events produced by this agent.
    #[serde(default)]
    pub event_count: u64,
    /// Agent role in the session.
    #[serde(default)]
    pub role: String,
}

/// Witness reference in a dehydration — an agent's record that something
/// occurred.
///
/// The trio is agnostic to what a witness contains. A witness may be a
/// cryptographic signature, a hash observation, a game-state checkpoint,
/// a conversation marker, or a bare timestamp. The `kind` field
/// discriminates; the `evidence` field carries the payload (opaque string,
/// empty when the witness needs no payload).
///
/// When the witness is cryptographic (`kind: "signature"`), verification
/// is delegated to `BearDog` (`crypto.verify`) or an external verifier.
/// `loamSpine` never decodes or validates evidence — it stores and commits.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireWitnessRef {
    /// Agent or system that produced this witness.
    pub agent: String,
    /// What this witness represents.
    /// `"signature"` = cryptographic signature,
    /// `"hash"` = hash observation, `"checkpoint"` = state snapshot,
    /// `"marker"` = boundary/event marker, `"timestamp"` = bare time witness.
    #[serde(default = "default_witness_kind")]
    pub kind: String,
    /// Evidence payload (opaque). For signatures this is the encoded
    /// signature bytes; for non-crypto witnesses this may be empty or
    /// carry a hash, checkpoint token, or other payload.
    #[serde(default)]
    pub evidence: String,
    /// When the witness was created (nanoseconds since epoch).
    #[serde(default)]
    pub witnessed_at: u64,
    /// How the evidence payload is encoded. Only meaningful when `evidence`
    /// is non-empty. Values: `"hex"`, `"base64"`, `"base64url"`, `"multibase"`,
    /// `"utf8"` (plain text), `"none"` (no encoding / empty payload).
    #[serde(default = "default_witness_encoding")]
    pub encoding: String,
    /// Cryptographic algorithm (when `kind` = `"signature"`).
    /// `None` for non-crypto witnesses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
    /// Provenance tier.
    /// `"local"` = same gate, `"gateway"` = remote gate,
    /// `"anchor"` = public chain, `"external"` = third-party,
    /// `"open"` = unsigned / no cryptographic backing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
    /// Freeform context for the witness.
    /// `"game:tick:42"`, `"conversation:thread:abc"`, `"experiment:run:7"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

fn default_witness_kind() -> String {
    "signature".to_string()
}

fn default_witness_encoding() -> String {
    "hex".to_string()
}

/// A high-level operation recorded during a dehydrated session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireSessionOperationRef {
    /// Operation type (e.g., "create", "modify", "derive", "merge").
    pub op_type: String,
    /// Content hash of the affected artifact.
    pub content_hash: String,
    /// Agent who performed the operation.
    pub agent: String,
    /// When the operation occurred (nanoseconds since epoch).
    #[serde(default)]
    pub timestamp: u64,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Request to execute the provenance pipeline (biomeOS graph input).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineRequest {
    /// rhizoCrypt session to dehydrate.
    pub session_id: String,
    /// DID of the agent performing the commit.
    pub agent_did: String,
    /// biomeOS family identifier.
    #[serde(default)]
    pub family_id: String,
    /// Optional experiment identifier (for Spring experiments).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experiment_id: Option<String>,
    /// Optional niche context (e.g., "rootpulse", "ludospring").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub niche: Option<String>,
    /// Per-agent contribution data for attribution braids.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agent_summaries: Vec<WireAgentContribution>,
}

/// Per-agent contribution data for attribution braids.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireAgentContribution {
    /// Agent DID.
    pub agent_did: String,
    /// Description of the agent's contribution.
    #[serde(default)]
    pub description: String,
    /// Relative weight of this agent's contribution (0.0 to 1.0).
    #[serde(default = "default_contribution_weight")]
    pub weight: f64,
}

const fn default_contribution_weight() -> f64 {
    1.0
}

/// Result of a completed provenance pipeline execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineResult {
    /// The dehydration merkle root from rhizoCrypt.
    pub dehydration_merkle_root: String,
    /// LoamSpine commit reference (entry hash).
    pub commit_ref: String,
    /// sweetGrass braid identifier (if attribution was created).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub braid_ref: Option<String>,
    /// BearDog signature over the dehydration summary (if signing was available).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// NestGate content address (if content was stored).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_ref: Option<String>,
}

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
    #![expect(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "test assertions use unwrap/expect for failure clarity"
    )]
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

    #[test]
    fn ephemeral_session_id_as_str_accessor() {
        let id = EphemeralSessionId::new("session-42");
        assert_eq!(id.as_str(), "session-42");
    }

    #[test]
    fn ephemeral_content_hash_as_str_accessor() {
        let h = EphemeralContentHash::new("deadbeef");
        assert_eq!(h.as_str(), "deadbeef");
    }

    #[test]
    fn wire_agent_contribution_default_weight() {
        let json = r#"{"agent_did":"did:key:z6MkTest","description":"test agent"}"#;
        let agent: WireAgentContribution = serde_json::from_str(json).unwrap();
        assert!((agent.weight - 1.0).abs() < f64::EPSILON);
    }
}
