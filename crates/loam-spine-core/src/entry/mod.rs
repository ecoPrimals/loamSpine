// SPDX-License-Identifier: AGPL-3.0-or-later

//! Entry types for LoamSpine.
//!
//! An Entry is a single, immutable record in a LoamSpine. Entries are
//! cryptographically linked to form a chain, with each entry referencing
//! the hash of the previous entry.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::temporal::{Moment, MomentId};
use crate::types::{
    BraidId, CertificateId, ContentHash, Did, EntryHash, PayloadRef, SessionId, Signature, SliceId,
    SpineId, Timestamp, hash_bytes,
};
use crate::waypoint::WaypointConfig;

/// Serde helpers for `ByteBuffer` fields in derived enums/structs.
pub(crate) mod serde_byte_buffer {
    use crate::types::ByteBuffer;

    pub fn serialize<S>(val: &ByteBuffer, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(val)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ByteBuffer, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = serde::Deserialize::deserialize(deserializer)?;
        Ok(ByteBuffer::from(bytes))
    }
}

/// A single entry in a LoamSpine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    /// Sequential index within this spine (0 for genesis).
    pub index: u64,

    /// Hash of the previous entry (None for genesis).
    #[serde(
        default,
        deserialize_with = "crate::types::serde_opt_content_hash::deserialize"
    )]
    pub previous: Option<EntryHash>,

    /// Spine this entry belongs to.
    pub spine_id: SpineId,

    /// Timestamp of commitment.
    pub timestamp: Timestamp,

    /// The agent committing this entry (DID from signing primal).
    pub committer: Did,

    /// Entry type.
    pub entry_type: EntryType,

    /// Optional payload reference (content-addressed).
    pub payload: Option<PayloadRef>,

    /// Inline metadata.
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,

    /// Cryptographic signature from committer.
    pub signature: Signature,

    /// Cached hash (computed on demand).
    #[serde(skip)]
    cached_hash: Option<EntryHash>,
}

impl Entry {
    /// Create a new entry.
    #[must_use]
    pub fn new(
        index: u64,
        previous: Option<EntryHash>,
        committer: Did,
        entry_type: EntryType,
    ) -> Self {
        Self {
            index,
            previous,
            spine_id: SpineId::nil(),
            timestamp: Timestamp::now(),
            committer,
            entry_type,
            payload: None,
            metadata: BTreeMap::new(),
            signature: Signature::empty(),
            cached_hash: None,
        }
    }

    /// Set the spine ID.
    #[must_use]
    pub const fn with_spine_id(mut self, spine_id: SpineId) -> Self {
        self.spine_id = spine_id;
        self.cached_hash = None;
        self
    }

    /// Create a genesis entry.
    #[must_use]
    pub fn genesis(owner: Did, spine_id: SpineId, config: SpineConfig) -> Self {
        Self {
            index: 0,
            previous: None,
            spine_id,
            timestamp: Timestamp::now(),
            committer: owner.clone(),
            entry_type: EntryType::Genesis {
                spine_id,
                owner,
                config,
            },
            payload: None,
            metadata: BTreeMap::new(),
            signature: Signature::empty(),
            cached_hash: None,
        }
    }

    /// Check if this is a genesis entry.
    #[must_use]
    pub const fn is_genesis(&self) -> bool {
        self.index == 0 && self.previous.is_none()
    }

    /// Set the payload reference.
    #[must_use]
    pub fn with_payload(mut self, payload: PayloadRef) -> Self {
        self.payload = Some(payload);
        self.cached_hash = None;
        self
    }

    /// Add metadata.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self.cached_hash = None;
        self
    }

    /// Set the signature.
    #[must_use]
    pub fn with_signature(mut self, signature: Signature) -> Self {
        self.signature = signature;
        self.cached_hash = None;
        self
    }

    /// Compute the entry hash (Blake3 of canonical form).
    ///
    /// # Errors
    ///
    /// Returns an error if canonical serialization fails.
    pub fn compute_hash(&self) -> crate::error::LoamSpineResult<EntryHash> {
        let canonical = self.to_canonical_bytes()?;
        Ok(hash_bytes(&canonical))
    }

    /// Get the entry hash (cached).
    ///
    /// # Errors
    ///
    /// Returns an error if canonical serialization fails.
    pub fn hash(&mut self) -> crate::error::LoamSpineResult<EntryHash> {
        if let Some(hash) = self.cached_hash {
            Ok(hash)
        } else {
            let hash = self.compute_hash()?;
            self.cached_hash = Some(hash);
            Ok(hash)
        }
    }

    /// Serialize to canonical bytes for hashing.
    ///
    /// Uses `rmp-serde` (MessagePack) for compact, deterministic serialization. Metadata is
    /// stored in a `BTreeMap`, so keys are always sorted — no extra
    /// canonicalisation step is needed.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails (should never occur for valid entries).
    pub fn to_canonical_bytes(&self) -> crate::error::LoamSpineResult<Vec<u8>> {
        rmp_serde::to_vec(self).map_err(|e| {
            crate::error::LoamSpineError::Serialization(format!(
                "canonical serialization failed: {e}"
            ))
        })
    }

    /// Get the entry type domain.
    #[must_use]
    pub const fn domain(&self) -> &'static str {
        self.entry_type.domain()
    }
}

/// Target system for public chain anchoring.
///
/// Multi-target by design: different anchors serve different purposes.
/// Bitcoin/Ethereum are public immutable ledgers — gas cost buys public
/// verifiability, nothing more. RFC 3161 TSA provides legal-grade timestamps.
/// Federated spines and data commons serve cross-trust and persistence roles.
/// Chain-agnostic — any append-only ledger works.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnchorTarget {
    /// Bitcoin OP_RETURN — strongest public immutability proof.
    Bitcoin,
    /// Ethereum event log or L2 — faster confirmation, lower cost via rollups.
    Ethereum,
    /// RFC 3161 TSA — legal-grade timestamp (ISO 18014-2), zero cost, sub-second.
    Rfc3161Tsa {
        /// TSA endpoint URL.
        tsa_url: String,
    },
    /// Federated LoamSpine instance — cross-trust-domain verification.
    FederatedSpine {
        /// Peer identifier of the federated spine.
        peer_id: String,
    },
    /// Data commons (IPFS, Arweave) — content-addressed persistence.
    DataCommons {
        /// Commons identifier (e.g. `"ipfs:<cid>"`).
        commons_id: String,
    },
    /// Any other append-only system.
    Other {
        /// Chain or system name.
        name: String,
    },
}

/// Types of entries that can be committed to LoamSpine.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EntryType {
    // === Spine Lifecycle ===
    /// Genesis entry (first in spine).
    Genesis {
        /// Spine identifier.
        spine_id: SpineId,
        /// Spine owner.
        owner: Did,
        /// Spine configuration.
        config: SpineConfig,
    },

    /// Spine metadata update.
    MetadataUpdate {
        /// Field being updated.
        field: String,
        /// New value.
        value: String,
    },

    /// Spine sealed (no more entries).
    SpineSealed {
        /// Reason for sealing.
        reason: Option<String>,
    },

    // === Ephemeral Storage Integration ===
    /// Dehydrated session from an ephemeral storage primal.
    SessionCommit {
        /// Session identifier.
        session_id: SessionId,
        /// Merkle root of session data.
        #[serde(deserialize_with = "crate::types::serde_content_hash::deserialize")]
        merkle_root: ContentHash,
        /// Number of vertices in the session.
        vertex_count: u64,
        /// Committer DID.
        committer: Did,
    },

    /// Slice checked out from this spine.
    SliceCheckout {
        /// Slice identifier.
        slice_id: SliceId,
        /// Source entry being sliced.
        #[serde(deserialize_with = "crate::types::serde_content_hash::deserialize")]
        source_entry: EntryHash,
        /// Session ID receiving the slice.
        session_id: SessionId,
        /// Holder DID.
        holder: Did,
    },

    /// Slice returned to this spine.
    SliceReturn {
        /// Slice identifier.
        slice_id: SliceId,
        /// Original checkout entry.
        #[serde(deserialize_with = "crate::types::serde_content_hash::deserialize")]
        checkout_entry: EntryHash,
        /// Whether resolution was successful.
        success: bool,
        /// Summary hash (if merged).
        #[serde(
            default,
            deserialize_with = "crate::types::serde_opt_content_hash::deserialize"
        )]
        summary: Option<ContentHash>,
    },

    // === Data Anchoring ===
    /// Anchor a content hash.
    DataAnchor {
        /// Content hash.
        #[serde(deserialize_with = "crate::types::serde_content_hash::deserialize")]
        data_hash: ContentHash,
        /// MIME type.
        mime_type: Option<String>,
        /// Size in bytes.
        size: u64,
    },

    /// Semantic attribution Braid commitment.
    BraidCommit {
        /// Braid identifier.
        braid_id: BraidId,
        /// Braid hash.
        #[serde(deserialize_with = "crate::types::serde_content_hash::deserialize")]
        braid_hash: ContentHash,
        /// Subject hash.
        #[serde(deserialize_with = "crate::types::serde_content_hash::deserialize")]
        subject_hash: ContentHash,
    },

    // === Certificate Operations ===
    /// Mint a new certificate.
    CertificateMint {
        /// Certificate identifier.
        cert_id: CertificateId,
        /// Certificate type.
        cert_type: String,
        /// Initial owner.
        initial_owner: Did,
    },

    /// Transfer certificate ownership.
    CertificateTransfer {
        /// Certificate identifier.
        cert_id: CertificateId,
        /// Previous owner.
        from: Did,
        /// New owner.
        to: Did,
    },

    /// Loan certificate (temporary transfer).
    CertificateLoan {
        /// Certificate identifier.
        cert_id: CertificateId,
        /// Lender.
        lender: Did,
        /// Borrower.
        borrower: Did,
        /// Loan duration in seconds.
        duration_secs: Option<u64>,
        /// Auto-return on expiry.
        auto_return: bool,
    },

    /// Return loaned certificate.
    CertificateReturn {
        /// Certificate identifier.
        cert_id: CertificateId,
        /// Original loan entry.
        #[serde(deserialize_with = "crate::types::serde_content_hash::deserialize")]
        loan_entry: EntryHash,
        /// Usage summary from the loan period.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        usage_summary: Option<crate::certificate::UsageSummary>,
    },

    // === Slice Operations ===
    /// Slice anchored at this spine (waypoint).
    SliceAnchor {
        /// Slice identifier.
        slice_id: SliceId,
        /// Origin spine.
        origin_spine: SpineId,
        /// Origin entry.
        #[serde(deserialize_with = "crate::types::serde_content_hash::deserialize")]
        origin_entry: EntryHash,
    },

    /// Slice operation at waypoint.
    SliceOperation {
        /// Slice identifier.
        slice_id: SliceId,
        /// Operation type.
        operation: String,
    },

    /// Slice departing waypoint.
    SliceDeparture {
        /// Slice identifier.
        slice_id: SliceId,
        /// Reason for departure.
        reason: String,
    },

    // === Temporal Moments ===
    /// Temporal moment (universal time tracking).
    TemporalMoment {
        /// Unique moment identifier.
        #[serde(deserialize_with = "crate::types::serde_content_hash::deserialize")]
        moment_id: MomentId,
        /// Moment data (boxed to reduce enum size).
        moment: Box<Moment>,
    },

    // === External Anchoring ===
    /// Public chain anchor proving spine state existed at a point in time.
    ///
    /// Records the result of anchoring a spine's state hash to an external
    /// append-only ledger (blockchain, data commons, federated spine).
    /// The actual chain submission is performed by a capability-discovered
    /// `"chain-anchor"` primal — loamSpine only records the receipt.
    PublicChainAnchor {
        /// Which chain or anchor system was used.
        anchor_target: AnchorTarget,
        /// The spine state hash that was anchored (Blake3 of tip entry).
        #[serde(deserialize_with = "crate::types::serde_content_hash::deserialize")]
        state_hash: ContentHash,
        /// Transaction hash or proof reference on the external system.
        tx_ref: String,
        /// Block height or sequence number (0 if not applicable).
        block_height: u64,
        /// Timestamp when the anchor was confirmed on the external system.
        anchor_timestamp: Timestamp,
        /// If this anchor was part of an aggregate batch, the batch root.
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "crate::types::serde_opt_content_hash::deserialize"
        )]
        aggregate_root: Option<ContentHash>,
        /// Merkle proof path from this spine's `state_hash` to `aggregate_root`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        inclusion_proof: Option<crate::proof::AggregateInclusionProof>,
    },

    // === Bond Ledger ===
    /// Ionic bond ledger record for cross-primal contract persistence.
    ///
    /// The crypto capability primal signs ionic bond contracts via
    /// `crypto.sign_contract` and delegates persistence to loamSpine
    /// via `bonding.ledger.store`. Each record captures the bond state
    /// at a point in time; the spine's append-only model guarantees an
    /// immutable audit trail.
    BondLedgerRecord {
        /// Unique bond identifier (from the signing primal's `IonicBond.bond_id`).
        bond_id: String,
        /// Opaque bond data (serialized `IonicBond`, contract terms, etc.).
        /// loamSpine stores this verbatim — schema validation is the
        /// caller's responsibility.
        data: serde_json::Value,
    },

    // === Cross-Gate Trust ===
    /// Ed25519 key exchange between two gates, establishing a shared trust
    /// relationship. The `public_key_hash` is the Blake3 hash of the exchanged
    /// public key — the raw key material stays with the crypto capability primal.
    KeyExchange {
        /// DID of the local gate initiating or accepting the exchange.
        local_gate: Did,
        /// DID of the remote gate.
        remote_gate: Did,
        /// Blake3 hash of the exchanged Ed25519 public key.
        #[serde(deserialize_with = "crate::types::serde_content_hash::deserialize")]
        public_key_hash: ContentHash,
        /// Direction: `"initiated"` or `"accepted"`.
        direction: String,
        /// Family ID scope (gates within the same family).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        family_id: Option<String>,
    },

    /// Registration of a trust issuer in the `TrustedIssuerRegistry`. Records
    /// that a specific gate or DID has been recognized as a valid issuer of
    /// trust tokens within a family or cross-family scope.
    TrustIssuerRegistration {
        /// DID of the issuer being registered.
        issuer_did: Did,
        /// Gate that registered this issuer.
        registering_gate: Did,
        /// Scope of trust: `"family"`, `"cross-gate"`, or `"global"`.
        trust_scope: String,
        /// Capabilities the issuer is trusted to attest (e.g. `["signing", "verification"]`).
        capabilities: Vec<String>,
        /// Expiry timestamp (None = no expiry).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expires_at: Option<Timestamp>,
    },

    /// Cross-gate token verification event. Records that a token issued by
    /// one gate was successfully verified by another gate, establishing
    /// a permanent audit trail of cross-gate trust exercises.
    TokenVerificationCrossGate {
        /// DID of the gate that issued the token.
        issuer_gate: Did,
        /// DID of the gate that verified the token.
        verifier_gate: Did,
        /// Blake3 hash of the verified token payload.
        #[serde(deserialize_with = "crate::types::serde_content_hash::deserialize")]
        token_hash: ContentHash,
        /// Whether verification succeeded.
        verified: bool,
        /// Verification failure reason (None if verified = true).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        failure_reason: Option<String>,
    },

    // === Custom ===
    /// Custom entry type with zero-copy payload.
    Custom {
        /// Type URI.
        type_uri: String,
        /// Payload bytes (zero-copy via `bytes::Bytes`).
        #[serde(
            serialize_with = "crate::entry::serde_byte_buffer::serialize",
            deserialize_with = "crate::entry::serde_byte_buffer::deserialize"
        )]
        payload: crate::types::ByteBuffer,
    },
}

impl EntryType {
    /// Get the domain for this entry type.
    #[must_use]
    pub const fn domain(&self) -> &'static str {
        match self {
            Self::Genesis { .. } | Self::MetadataUpdate { .. } | Self::SpineSealed { .. } => {
                "spine"
            }
            // Generic capability domains - no primal names
            Self::SessionCommit { .. } | Self::SliceCheckout { .. } | Self::SliceReturn { .. } => {
                "session"
            }
            Self::DataAnchor { .. } | Self::BraidCommit { .. } => "data",
            Self::CertificateMint { .. }
            | Self::CertificateTransfer { .. }
            | Self::CertificateLoan { .. }
            | Self::CertificateReturn { .. } => "certificate",
            Self::SliceAnchor { .. }
            | Self::SliceOperation { .. }
            | Self::SliceDeparture { .. } => "slice",
            Self::TemporalMoment { .. } => "temporal",
            Self::PublicChainAnchor { .. } => "anchor",
            Self::BondLedgerRecord { .. } => "bonding",
            Self::KeyExchange { .. }
            | Self::TrustIssuerRegistration { .. }
            | Self::TokenVerificationCrossGate { .. } => "trust",
            Self::Custom { .. } => "custom",
        }
    }

    /// Check if this entry type is allowed in a waypoint spine.
    #[must_use]
    pub const fn allowed_in_waypoint(&self) -> bool {
        matches!(
            self,
            Self::Genesis { .. }
                | Self::SliceAnchor { .. }
                | Self::SliceOperation { .. }
                | Self::SliceDeparture { .. }
        )
    }
}

/// Spine configuration (embedded in genesis).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpineConfig {
    /// Spine type.
    pub spine_type: SpineType,

    /// Auto-rollup threshold.
    pub auto_rollup_threshold: Option<u64>,

    /// Replication enabled.
    pub replication_enabled: bool,

    /// Waypoint-specific config when this spine is used as a waypoint.
    ///
    /// When `Some`, attestation and other waypoint policies apply to
    /// anchor/record/depart operations. When `None`, defaults are used.
    #[serde(default)]
    pub waypoint_config: Option<WaypointConfig>,
}

/// Spine type.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpineType {
    /// Personal history.
    #[default]
    Personal,

    /// Professional/work spine.
    Professional,

    /// Community shared spine.
    Community {
        /// Community identifier.
        community_id: String,
    },

    /// Waypoint for borrowed state.
    Waypoint {
        /// Maximum anchor depth.
        max_anchor_depth: Option<u32>,
    },

    /// Public, globally verifiable.
    Public,
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "tests use expect for concise error paths"
)]
#[expect(clippy::unwrap_used, reason = "proptests use unwrap for assertions")]
#[path = "entry_tests.rs"]
mod tests;
