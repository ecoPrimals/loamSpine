// SPDX-License-Identifier: AGPL-3.0-or-later

//! Entry types for LoamSpine.
//!
//! An Entry is a single, immutable record in a LoamSpine. Entries are
//! cryptographically linked to form a chain, with each entry referencing
//! the hash of the previous entry.
//!
//! Type definitions (`EntryType`, `AnchorTarget`, `SpineConfig`, `SpineType`)
//! live in the `types` submodule and are re-exported here.

mod types;

pub use types::{AnchorTarget, EntryType, SpineConfig, SpineType};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg(test)]
use crate::types::{BraidId, CertificateId, ContentHash, SessionId, SliceId};
use crate::types::{Did, EntryHash, PayloadRef, Signature, SpineId, Timestamp, hash_bytes};

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

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "tests use expect for concise error paths"
)]
#[expect(clippy::unwrap_used, reason = "proptests use unwrap for assertions")]
#[path = "entry_tests.rs"]
mod tests;

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "tests use expect for concise error paths"
)]
#[path = "entry_tests_trust.rs"]
mod tests_trust;
