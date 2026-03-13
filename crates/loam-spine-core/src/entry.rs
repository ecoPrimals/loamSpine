// SPDX-License-Identifier: AGPL-3.0-only

//! Entry types for LoamSpine.
//!
//! An Entry is a single, immutable record in a LoamSpine. Entries are
//! cryptographically linked to form a chain, with each entry referencing
//! the hash of the previous entry.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::temporal::{Moment, MomentId};
use crate::types::{
    hash_bytes, BraidId, CertificateId, ContentHash, Did, EntryHash, PayloadRef, SessionId,
    Signature, SliceId, SpineId, Timestamp,
};

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
    /// Uses `bincode` for compact, deterministic serialization. Metadata is
    /// stored in a `BTreeMap`, so keys are always sorted — no extra
    /// canonicalisation step is needed.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails (should never occur for valid entries).
    pub fn to_canonical_bytes(&self) -> crate::error::LoamSpineResult<Vec<u8>> {
        bincode::serialize(self).map_err(|e| {
            crate::error::LoamSpineError::Serialization(format!(
                "canonical serialization failed: {e}"
            ))
        })
    }

    /// Get the entry type domain.
    #[must_use]
    pub fn domain(&self) -> &'static str {
        self.entry_type.domain()
    }
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
        checkout_entry: EntryHash,
        /// Whether resolution was successful.
        success: bool,
        /// Summary hash (if merged).
        summary: Option<ContentHash>,
    },

    // === Data Anchoring ===
    /// Anchor a content hash.
    DataAnchor {
        /// Content hash.
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
        braid_hash: ContentHash,
        /// Subject hash.
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
        loan_entry: EntryHash,
    },

    // === Slice Operations ===
    /// Slice anchored at this spine (waypoint).
    SliceAnchor {
        /// Slice identifier.
        slice_id: SliceId,
        /// Origin spine.
        origin_spine: SpineId,
        /// Origin entry.
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
        moment_id: MomentId,
        /// Moment data (boxed to reduce enum size).
        moment: Box<Moment>,
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
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn entry_creation() {
        let did = Did::new("did:key:z6MkTest");
        let entry = Entry::new(
            0,
            None,
            did.clone(),
            EntryType::SpineSealed { reason: None },
        );

        assert_eq!(entry.index, 0);
        assert!(entry.previous.is_none());
        assert_eq!(entry.committer, did);
    }

    #[test]
    fn genesis_entry() {
        let owner = Did::new("did:key:z6MkOwner");
        let spine_id = SpineId::now_v7();
        let entry = Entry::genesis(owner, spine_id, SpineConfig::default());

        assert!(entry.is_genesis());
        assert_eq!(entry.domain(), "spine");
    }

    #[test]
    fn entry_hash_deterministic() {
        let did = Did::new("did:key:z6MkTest");
        let entry = Entry::new(0, None, did, EntryType::SpineSealed { reason: None });

        let hash1 = entry.compute_hash().expect("compute_hash");
        let hash2 = entry.compute_hash().expect("compute_hash");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn entry_builder() {
        let did = Did::new("did:key:z6MkTest");
        let entry = Entry::new(
            1,
            Some([0u8; 32]),
            did,
            EntryType::SpineSealed { reason: None },
        )
        .with_metadata("key", "value")
        .with_signature(Signature::from_vec(vec![1, 2, 3]));

        assert_eq!(entry.metadata.get("key"), Some(&"value".to_string()));
        assert!(!entry.signature.is_empty());
    }

    #[test]
    fn entry_type_domain() {
        let committer = Did::new("did:key:z6MkCommitter");
        assert_eq!(
            EntryType::SessionCommit {
                session_id: SessionId::now_v7(),
                merkle_root: [0u8; 32],
                vertex_count: 10,
                committer,
            }
            .domain(),
            "session"
        );

        assert_eq!(
            EntryType::CertificateMint {
                cert_id: CertificateId::now_v7(),
                cert_type: "game".into(),
                initial_owner: Did::new("did:key:test"),
            }
            .domain(),
            "certificate"
        );
    }

    #[test]
    fn waypoint_allowed_types() {
        let genesis = EntryType::Genesis {
            spine_id: SpineId::now_v7(),
            owner: Did::new("test"),
            config: SpineConfig::default(),
        };
        assert!(genesis.allowed_in_waypoint());

        let committer = Did::new("did:key:z6MkCommitter");
        let session = EntryType::SessionCommit {
            session_id: SessionId::now_v7(),
            merkle_root: [0u8; 32],
            vertex_count: 10,
            committer,
        };
        assert!(!session.allowed_in_waypoint());
    }

    #[test]
    fn entry_type_domain_all_variants() {
        let owner = Did::new("did:key:z6MkOwner");
        let committer = Did::new("did:key:z6MkCommitter");

        assert_eq!(
            EntryType::MetadataUpdate {
                field: "name".into(),
                value: "foo".into(),
            }
            .domain(),
            "spine"
        );

        assert_eq!(
            EntryType::SliceCheckout {
                slice_id: SliceId::now_v7(),
                source_entry: [0u8; 32],
                session_id: SessionId::now_v7(),
                holder: committer.clone(),
            }
            .domain(),
            "session"
        );

        assert_eq!(
            EntryType::SliceReturn {
                slice_id: SliceId::now_v7(),
                checkout_entry: [0u8; 32],
                success: true,
                summary: None,
            }
            .domain(),
            "session"
        );

        assert_eq!(
            EntryType::DataAnchor {
                data_hash: [1u8; 32],
                mime_type: Some("text/plain".into()),
                size: 100,
            }
            .domain(),
            "data"
        );

        assert_eq!(
            EntryType::BraidCommit {
                braid_id: BraidId::now_v7(),
                braid_hash: [1u8; 32],
                subject_hash: [2u8; 32],
            }
            .domain(),
            "data"
        );

        assert_eq!(
            EntryType::CertificateTransfer {
                cert_id: CertificateId::now_v7(),
                from: owner,
                to: committer,
            }
            .domain(),
            "certificate"
        );
    }

    #[test]
    fn entry_type_domain_certificate_slice_custom() {
        let owner = Did::new("did:key:z6MkOwner");
        let committer = Did::new("did:key:z6MkCommitter");

        assert_eq!(
            EntryType::CertificateLoan {
                cert_id: CertificateId::now_v7(),
                lender: owner,
                borrower: committer,
                duration_secs: Some(3600),
                auto_return: true,
            }
            .domain(),
            "certificate"
        );

        assert_eq!(
            EntryType::CertificateReturn {
                cert_id: CertificateId::now_v7(),
                loan_entry: [0u8; 32],
            }
            .domain(),
            "certificate"
        );

        assert_eq!(
            EntryType::SliceAnchor {
                slice_id: SliceId::now_v7(),
                origin_spine: SpineId::now_v7(),
                origin_entry: [0u8; 32],
            }
            .domain(),
            "slice"
        );

        assert_eq!(
            EntryType::SliceOperation {
                slice_id: SliceId::now_v7(),
                operation: "merge".into(),
            }
            .domain(),
            "slice"
        );

        assert_eq!(
            EntryType::SliceDeparture {
                slice_id: SliceId::now_v7(),
                reason: "complete".into(),
            }
            .domain(),
            "slice"
        );

        assert_eq!(
            EntryType::Custom {
                type_uri: "example.com/custom".into(),
                payload: crate::types::ByteBuffer::from_static(b"payload"),
            }
            .domain(),
            "custom"
        );
    }

    #[test]
    fn entry_type_domain_temporal_moment() {
        use crate::temporal::{Moment, MomentContext};

        let moment = Moment {
            id: [0u8; 32],
            timestamp: std::time::UNIX_EPOCH,
            agent: "did:key:z6MkAgent".into(),
            state_hash: [1u8; 32],
            signature: Signature::empty(),
            context: MomentContext::Generic {
                category: "test".into(),
                metadata: std::collections::HashMap::new(),
                content_hash: None,
            },
            parents: vec![],
            anchor: None,
            ephemeral_provenance: None,
        };
        let ty = EntryType::TemporalMoment {
            moment_id: [0u8; 32],
            moment: Box::new(moment),
        };
        assert_eq!(ty.domain(), "temporal");
    }

    #[test]
    fn waypoint_allowed_slice_variants() {
        assert!(EntryType::SliceAnchor {
            slice_id: SliceId::now_v7(),
            origin_spine: SpineId::now_v7(),
            origin_entry: [0u8; 32],
        }
        .allowed_in_waypoint());

        assert!(EntryType::SliceOperation {
            slice_id: SliceId::now_v7(),
            operation: "op".into(),
        }
        .allowed_in_waypoint());

        assert!(EntryType::SliceDeparture {
            slice_id: SliceId::now_v7(),
            reason: "done".into(),
        }
        .allowed_in_waypoint());

        assert!(!EntryType::DataAnchor {
            data_hash: [0u8; 32],
            mime_type: None,
            size: 0,
        }
        .allowed_in_waypoint());
    }

    #[test]
    fn entry_with_payload() {
        let did = Did::new("did:key:z6MkTest");
        let payload = PayloadRef::new([1u8; 32], 1024).with_mime_type("application/octet-stream");

        let entry = Entry::new(
            1,
            Some([0u8; 32]),
            did,
            EntryType::DataAnchor {
                data_hash: [2u8; 32],
                mime_type: None,
                size: 1024,
            },
        )
        .with_spine_id(SpineId::now_v7())
        .with_payload(payload.clone());

        assert_eq!(entry.payload.as_ref(), Some(&payload));
    }

    #[test]
    fn entry_hash_cached() {
        let did = Did::new("did:key:z6MkTest");
        let mut entry = Entry::new(0, None, did, EntryType::SpineSealed { reason: None })
            .with_spine_id(SpineId::now_v7());

        let hash1 = entry.hash().expect("hash");
        let hash2 = entry.hash().expect("hash");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn entry_is_genesis_non_genesis() {
        let did = Did::new("did:key:z6MkTest");
        let entry = Entry::new(
            1,
            Some([0u8; 32]),
            did,
            EntryType::DataAnchor {
                data_hash: [0u8; 32],
                mime_type: None,
                size: 0,
            },
        );
        assert!(!entry.is_genesis());
    }

    #[test]
    fn entry_serde_roundtrip() {
        let did = Did::new("did:key:z6MkTest");
        let entry = Entry::new(
            1,
            Some([1u8; 32]),
            did,
            EntryType::DataAnchor {
                data_hash: [2u8; 32],
                mime_type: Some("text/plain".into()),
                size: 100,
            },
        )
        .with_spine_id(SpineId::now_v7())
        .with_metadata("k1", "v1")
        .with_metadata("k2", "v2");

        let bytes = serde_json::to_vec(&entry).expect("serialize");
        let restored: Entry = serde_json::from_slice(&bytes).expect("deserialize");

        assert_eq!(entry.index, restored.index);
        assert_eq!(entry.spine_id, restored.spine_id);
        assert_eq!(entry.metadata, restored.metadata);
    }

    #[test]
    fn entry_type_serde_roundtrip_data_anchor() {
        let ty = EntryType::DataAnchor {
            data_hash: [1u8; 32],
            mime_type: Some("image/png".into()),
            size: 2048,
        };
        let bytes = serde_json::to_vec(&ty).expect("serialize");
        let restored: EntryType = serde_json::from_slice(&bytes).expect("deserialize");
        assert!(matches!(restored, EntryType::DataAnchor { size: 2048, .. }));
    }

    #[test]
    fn entry_type_serde_roundtrip_slice_anchor() {
        let ty = EntryType::SliceAnchor {
            slice_id: SliceId::now_v7(),
            origin_spine: SpineId::now_v7(),
            origin_entry: [2u8; 32],
        };
        let bytes = serde_json::to_vec(&ty).expect("serialize");
        let restored: EntryType = serde_json::from_slice(&bytes).expect("deserialize");
        assert!(matches!(
            restored,
            EntryType::SliceAnchor { origin_entry, .. } if origin_entry == [2u8; 32]
        ));
    }

    #[test]
    fn entry_metadata_default_on_deserialize() {
        let json = r#"{"index":0,"previous":null,"spine_id":"00000000-0000-0000-0000-000000000000","timestamp":0,"committer":"did:key:test","entry_type":{"SpineSealed":{"reason":null}},"payload":null,"signature":[]}"#;
        let entry: Entry = serde_json::from_str(json).expect("deserialize");
        assert!(entry.metadata.is_empty());
    }

    #[test]
    fn entry_type_serde_roundtrip_custom() {
        let ty = EntryType::Custom {
            type_uri: "urn:test:custom".into(),
            payload: crate::types::ByteBuffer::from_static(b"custom data"),
        };
        let bytes = serde_json::to_vec(&ty).expect("serialize");
        let restored: EntryType = serde_json::from_slice(&bytes).expect("deserialize");
        assert!(
            matches!(&restored, EntryType::Custom { type_uri, payload } if type_uri == "urn:test:custom" && payload.as_ref() == b"custom data"),
            "expected Custom variant"
        );
    }

    #[test]
    fn spine_config_serde_roundtrip() {
        let config = SpineConfig {
            spine_type: SpineType::Waypoint {
                max_anchor_depth: Some(5),
            },
            auto_rollup_threshold: Some(10_000),
            replication_enabled: true,
        };
        let bytes = serde_json::to_vec(&config).expect("serialize");
        let restored: SpineConfig = serde_json::from_slice(&bytes).expect("deserialize");
        assert_eq!(config.auto_rollup_threshold, restored.auto_rollup_threshold);
        assert_eq!(config.replication_enabled, restored.replication_enabled);
    }

    #[test]
    fn spine_type_serde_roundtrip() {
        let types = [
            SpineType::Personal,
            SpineType::Professional,
            SpineType::Public,
            SpineType::Community {
                community_id: "cid".into(),
            },
            SpineType::Waypoint {
                max_anchor_depth: Some(3),
            },
            SpineType::Waypoint {
                max_anchor_depth: None,
            },
        ];

        for ty in &types {
            let bytes = serde_json::to_vec(ty).expect("serialize");
            let restored: SpineType = serde_json::from_slice(&bytes).expect("deserialize");
            assert_eq!(ty, &restored);
        }
    }

    #[test]
    fn entry_to_canonical_bytes_deterministic_with_metadata() {
        let did = Did::new("did:key:z6MkTest");
        let spine_id = SpineId::now_v7();
        let ts = Timestamp::now();

        let mut entry1 = Entry::new(
            0,
            None,
            did.clone(),
            EntryType::SpineSealed { reason: None },
        )
        .with_spine_id(spine_id)
        .with_metadata("b", "2")
        .with_metadata("a", "1");
        entry1.timestamp = ts;

        let mut entry2 = Entry::new(0, None, did, EntryType::SpineSealed { reason: None })
            .with_spine_id(spine_id)
            .with_metadata("a", "1")
            .with_metadata("b", "2");
        entry2.timestamp = ts;

        let bytes1 = entry1.to_canonical_bytes().expect("canonical");
        let bytes2 = entry2.to_canonical_bytes().expect("canonical");
        assert_eq!(bytes1, bytes2, "canonical bytes should be deterministic");
    }

    #[test]
    fn entry_with_spine_id_clears_cache() {
        let did = Did::new("did:key:z6MkTest");
        let spine_id1 = SpineId::now_v7();
        let spine_id2 = SpineId::now_v7();

        let mut entry = Entry::new(0, None, did, EntryType::SpineSealed { reason: None })
            .with_spine_id(spine_id1);
        let _ = entry.hash().expect("hash");

        let entry2 = entry.with_spine_id(spine_id2);
        assert_eq!(entry2.spine_id, spine_id2);
    }
}
