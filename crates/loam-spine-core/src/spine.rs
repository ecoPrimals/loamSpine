// SPDX-License-Identifier: AGPL-3.0-or-later

//! Spine types for LoamSpine.
//!
//! A Spine is a linear chain of entries with common ownership.
//! Each spine is identified by a unique ID and owned by a DID.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::entry::{Entry, EntryType, SpineConfig, SpineType};
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::types::{Did, EntryHash, SpineId, Timestamp};

/// A LoamSpine (linear chain of entries).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Spine {
    /// Unique spine identifier.
    pub id: SpineId,

    /// Human-readable spine name.
    pub name: Option<String>,

    /// Spine owner (can transfer ownership).
    pub owner: Did,

    /// Spine type.
    pub spine_type: SpineType,

    /// Spine configuration.
    pub config: SpineConfig,

    /// Genesis entry hash.
    pub genesis: EntryHash,

    /// Current tip (latest entry hash).
    pub tip: EntryHash,

    /// Current height (number of entries).
    pub height: u64,

    /// Creation timestamp.
    pub created_at: Timestamp,

    /// Last update timestamp.
    pub updated_at: Timestamp,

    /// Spine state.
    pub state: SpineState,

    /// Custom metadata.
    pub metadata: HashMap<String, String>,

    /// In-memory entry cache (for small spines).
    entries: Vec<Entry>,
}

impl Spine {
    /// Create a new spine with a genesis entry.
    ///
    /// # Errors
    ///
    /// Returns an error if the genesis entry is malformed.
    pub fn new(owner: Did, name: Option<String>, config: SpineConfig) -> LoamSpineResult<Self> {
        let id = SpineId::now_v7();
        let now = Timestamp::now();

        // Create genesis entry
        let mut genesis_entry = Entry::genesis(owner.clone(), id, config.clone());
        let genesis_hash = genesis_entry.hash()?;

        Ok(Self {
            id,
            name,
            owner,
            spine_type: config.spine_type.clone(),
            config,
            genesis: genesis_hash,
            tip: genesis_hash,
            height: 1,
            created_at: now,
            updated_at: now,
            state: SpineState::Active,
            metadata: HashMap::new(),
            entries: vec![genesis_entry],
        })
    }

    /// Create a spine builder.
    #[must_use]
    pub fn builder(owner: Did) -> SpineBuilder {
        SpineBuilder::new(owner)
    }

    /// Get the genesis entry.
    #[must_use]
    pub fn genesis_entry(&self) -> Option<&Entry> {
        self.entries.first()
    }

    /// Get the tip entry.
    #[must_use]
    pub fn tip_entry(&self) -> Option<&Entry> {
        self.entries.last()
    }

    /// Get an entry by index.
    #[must_use]
    pub fn get_entry(&self, index: u64) -> Option<&Entry> {
        usize::try_from(index)
            .ok()
            .and_then(|idx| self.entries.get(idx))
    }

    /// Get all entries.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Check if the spine is active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.state.is_active()
    }

    /// Check if the spine is sealed.
    #[must_use]
    pub const fn is_sealed(&self) -> bool {
        self.state.is_sealed()
    }

    /// Append an entry to the spine.
    ///
    /// Takes ownership of the entry (zero-copy into the spine's entry list).
    /// After append, use [`tip_entry()`](Self::tip_entry) to get a reference
    /// to the stored entry for persistence without cloning.
    ///
    /// # Errors
    ///
    /// Returns an error if the spine is sealed or the entry is invalid.
    pub fn append(&mut self, mut entry: Entry) -> LoamSpineResult<EntryHash> {
        if !self.is_active() {
            return Err(LoamSpineError::SpineSealed(self.id));
        }

        if entry.index != self.height {
            return Err(LoamSpineError::ChainValidation {
                index: entry.index,
                reason: format!("expected index {}, got {}", self.height, entry.index),
            });
        }

        if entry.previous != Some(self.tip) {
            return Err(LoamSpineError::ChainValidation {
                index: entry.index,
                reason: "previous hash mismatch".into(),
            });
        }

        let hash = entry.hash()?;
        self.entries.push(entry);
        self.tip = hash;
        self.height += 1;
        self.updated_at = Timestamp::now();

        Ok(hash)
    }

    /// Create an entry for this spine.
    #[must_use]
    pub fn create_entry(&self, entry_type: EntryType) -> Entry {
        Entry::new(self.height, Some(self.tip), self.owner.clone(), entry_type)
            .with_spine_id(self.id)
    }

    /// Seal the spine (make read-only).
    ///
    /// # Errors
    ///
    /// Returns an error if the spine is already sealed.
    pub fn seal(&mut self, reason: Option<String>) -> LoamSpineResult<EntryHash> {
        if self.is_sealed() {
            return Err(LoamSpineError::SpineSealed(self.id));
        }

        let seal_entry = self.create_entry(EntryType::SpineSealed { reason });
        let hash = self.append(seal_entry)?;

        self.state = SpineState::Sealed {
            sealed_at: Timestamp::now(),
            final_entry: hash,
        };

        Ok(hash)
    }

    /// Verify the chain integrity.
    #[must_use]
    pub fn verify(&self) -> ChainVerification {
        let mut errors = Vec::new();
        let mut prev_hash: Option<EntryHash> = None;

        for (i, entry) in self.entries.iter().enumerate() {
            // Check index
            if entry.index != i as u64 {
                errors.push(ChainError::IndexGap {
                    expected: i as u64,
                    actual: entry.index,
                });
            }

            // Check previous hash
            if entry.previous != prev_hash {
                errors.push(ChainError::HashMismatch {
                    index: entry.index,
                    expected: prev_hash.unwrap_or([0u8; 32]),
                    actual: entry.previous.unwrap_or([0u8; 32]),
                });
            }

            prev_hash = match entry.compute_hash() {
                Ok(h) => Some(h),
                Err(e) => {
                    errors.push(ChainError::HashComputationFailed {
                        index: entry.index,
                        message: e.to_string(),
                    });
                    None
                }
            };
        }

        ChainVerification {
            spine_id: self.id,
            entries_verified: self.height,
            valid: errors.is_empty(),
            errors,
        }
    }
}

/// Builder for creating spines.
pub struct SpineBuilder {
    owner: Did,
    name: Option<String>,
    spine_type: SpineType,
    config: SpineConfig,
    metadata: HashMap<String, String>,
}

impl SpineBuilder {
    /// Create a new spine builder.
    #[must_use]
    pub fn new(owner: Did) -> Self {
        Self {
            owner,
            name: None,
            spine_type: SpineType::Personal,
            config: SpineConfig::default(),
            metadata: HashMap::new(),
        }
    }

    /// Set the spine name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the spine type.
    #[must_use]
    pub fn with_type(mut self, spine_type: SpineType) -> Self {
        self.spine_type = spine_type.clone();
        self.config.spine_type = spine_type;
        self
    }

    /// Create a personal spine.
    #[must_use]
    pub fn personal(mut self) -> Self {
        self.spine_type = SpineType::Personal;
        self.config.spine_type = SpineType::Personal;
        self
    }

    /// Create a waypoint spine.
    #[must_use]
    pub fn waypoint(mut self, max_depth: Option<u32>) -> Self {
        let spine_type = SpineType::Waypoint {
            max_anchor_depth: max_depth,
        };
        self.spine_type = spine_type.clone();
        self.config.spine_type = spine_type;
        self
    }

    /// Set auto-rollup threshold.
    #[must_use]
    pub const fn with_auto_rollup(mut self, threshold: u64) -> Self {
        self.config.auto_rollup_threshold = Some(threshold);
        self
    }

    /// Enable replication.
    #[must_use]
    pub const fn with_replication(mut self, enabled: bool) -> Self {
        self.config.replication_enabled = enabled;
        self
    }

    /// Add metadata.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the spine.
    ///
    /// # Errors
    ///
    /// Returns an error if spine creation fails.
    pub fn build(self) -> LoamSpineResult<Spine> {
        let mut spine = Spine::new(self.owner, self.name, self.config)?;
        spine.metadata = self.metadata;
        Ok(spine)
    }
}

/// Spine state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SpineState {
    /// Actively accepting entries.
    #[default]
    Active,

    /// Temporarily frozen.
    Frozen {
        /// Reason for freezing.
        reason: String,
        /// Frozen until timestamp.
        until: Option<Timestamp>,
    },

    /// Permanently sealed (read-only).
    Sealed {
        /// When the spine was sealed.
        sealed_at: Timestamp,
        /// Final entry hash.
        final_entry: EntryHash,
    },

    /// Archived (cold storage).
    Archived {
        /// When the spine was archived.
        archived_at: Timestamp,
        /// Archive location.
        archive_location: String,
    },
}

impl SpineState {
    /// Check if spine is active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Check if spine is sealed.
    #[must_use]
    pub const fn is_sealed(&self) -> bool {
        matches!(self, Self::Sealed { .. })
    }

    /// Check if spine is terminal (sealed or archived).
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Sealed { .. } | Self::Archived { .. })
    }
}

/// Chain verification result.
#[derive(Clone, Debug)]
pub struct ChainVerification {
    /// Spine ID.
    pub spine_id: SpineId,
    /// Number of entries verified.
    pub entries_verified: u64,
    /// Whether the chain is valid.
    pub valid: bool,
    /// Any errors found.
    pub errors: Vec<ChainError>,
}

/// Chain verification error.
#[derive(Clone, Debug)]
pub enum ChainError {
    /// Hash mismatch between entries.
    HashMismatch {
        /// Entry index.
        index: u64,
        /// Expected hash.
        expected: EntryHash,
        /// Actual hash.
        actual: EntryHash,
    },

    /// Gap in index sequence.
    IndexGap {
        /// Expected index.
        expected: u64,
        /// Actual index.
        actual: u64,
    },

    /// Invalid signature.
    InvalidSignature {
        /// Entry index.
        index: u64,
    },

    /// Hash computation failed for an entry.
    HashComputationFailed {
        /// Entry index.
        index: u64,
        /// Error message.
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spine_creation() {
        let owner = Did::new("did:key:z6MkOwner");
        let spine = Spine::new(owner.clone(), Some("Test".into()), SpineConfig::default());

        assert!(spine.is_ok());
        let spine = spine.unwrap_or_else(|_| unreachable!());

        assert_eq!(spine.owner, owner);
        assert_eq!(spine.height, 1);
        assert!(spine.is_active());
        assert!(spine.genesis_entry().is_some());
    }

    #[test]
    fn spine_builder() {
        let owner = Did::new("did:key:z6MkOwner");
        let spine = Spine::builder(owner)
            .with_name("My Spine")
            .personal()
            .with_auto_rollup(10_000)
            .with_metadata("created_by", "test")
            .build();

        assert!(spine.is_ok());
        let spine = spine.unwrap_or_else(|_| unreachable!());

        assert_eq!(spine.name, Some("My Spine".to_string()));
        assert_eq!(spine.spine_type, SpineType::Personal);
        assert_eq!(spine.metadata.get("created_by"), Some(&"test".to_string()));
    }

    #[test]
    fn spine_append() {
        let owner = Did::new("did:key:z6MkOwner");
        let mut spine =
            Spine::new(owner, None, SpineConfig::default()).unwrap_or_else(|_| unreachable!());

        let entry = spine.create_entry(EntryType::DataAnchor {
            data_hash: [1u8; 32],
            mime_type: Some("text/plain".into()),
            size: 100,
        });

        let result = spine.append(entry);
        assert!(result.is_ok());
        assert_eq!(spine.height, 2);
    }

    #[test]
    fn spine_seal() {
        let owner = Did::new("did:key:z6MkOwner");
        let mut spine =
            Spine::new(owner, None, SpineConfig::default()).unwrap_or_else(|_| unreachable!());

        let result = spine.seal(Some("Test complete".into()));
        assert!(result.is_ok());
        assert!(spine.is_sealed());

        // Cannot append after sealing
        let entry = spine.create_entry(EntryType::SpineSealed { reason: None });
        let result = spine.append(entry);
        assert!(result.is_err());
    }

    #[test]
    fn spine_verify() {
        let owner = Did::new("did:key:z6MkOwner");
        let mut spine =
            Spine::new(owner, None, SpineConfig::default()).unwrap_or_else(|_| unreachable!());

        // Add some entries
        for i in 0u8..5 {
            let entry = spine.create_entry(EntryType::DataAnchor {
                data_hash: [i; 32],
                mime_type: None,
                size: u64::from(i),
            });
            spine.append(entry).ok();
        }

        let verification = spine.verify();
        assert!(verification.valid);
        assert!(verification.errors.is_empty());
        assert_eq!(verification.entries_verified, 6); // genesis + 5
    }

    #[test]
    fn waypoint_spine() {
        let owner = Did::new("did:key:z6MkOwner");
        let spine = Spine::builder(owner)
            .waypoint(Some(3))
            .with_name("Borrowed Games")
            .build();

        assert!(spine.is_ok());
        let spine = spine.unwrap_or_else(|_| unreachable!());

        assert!(matches!(
            spine.spine_type,
            SpineType::Waypoint {
                max_anchor_depth: Some(3)
            }
        ));
    }

    #[test]
    fn spine_state_checks() {
        assert!(SpineState::Active.is_active());
        assert!(!SpineState::Active.is_sealed());
        assert!(!SpineState::Active.is_terminal());

        let sealed = SpineState::Sealed {
            sealed_at: Timestamp::now(),
            final_entry: [0u8; 32],
        };
        assert!(!sealed.is_active());
        assert!(sealed.is_sealed());
        assert!(sealed.is_terminal());
    }

    #[test]
    fn spine_get_entry_and_entries() {
        let owner = Did::new("did:key:z6MkOwner");
        let mut spine =
            Spine::new(owner, None, SpineConfig::default()).unwrap_or_else(|_| unreachable!());

        let entry = spine.create_entry(EntryType::DataAnchor {
            data_hash: [1u8; 32],
            mime_type: None,
            size: 42,
        });
        spine.append(entry).ok();

        assert!(spine.get_entry(0).is_some());
        assert!(spine.get_entry(1).is_some());
        assert!(spine.get_entry(2).is_none());
        assert!(spine.get_entry(u64::MAX).is_none());

        assert_eq!(spine.entries().len(), 2);
        assert_eq!(spine.tip_entry().map(|e| e.index), Some(1));
    }

    #[test]
    fn spine_append_index_mismatch() {
        let owner = Did::new("did:key:z6MkOwner");
        let mut spine = Spine::new(owner.clone(), None, SpineConfig::default())
            .unwrap_or_else(|_| unreachable!());

        let _wrong_index_entry = spine.create_entry(EntryType::DataAnchor {
            data_hash: [1u8; 32],
            mime_type: None,
            size: 1,
        });
        let mut bad_entry = Entry::new(
            999,
            Some(spine.tip),
            owner,
            EntryType::DataAnchor {
                data_hash: [1u8; 32],
                mime_type: None,
                size: 1,
            },
        )
        .with_spine_id(spine.id);
        bad_entry.index = 999;

        let result = spine.append(bad_entry);
        assert!(result.is_err());
        if let Err(LoamSpineError::ChainValidation { index, reason }) = result {
            assert_eq!(index, 999);
            assert!(reason.contains("expected index"));
        }
    }

    #[test]
    fn spine_append_previous_hash_mismatch() {
        let owner = Did::new("did:key:z6MkOwner");
        let mut spine = Spine::new(owner.clone(), None, SpineConfig::default())
            .unwrap_or_else(|_| unreachable!());

        let bad_entry = Entry::new(
            1,
            Some([0xFFu8; 32]),
            owner,
            EntryType::DataAnchor {
                data_hash: [1u8; 32],
                mime_type: None,
                size: 1,
            },
        )
        .with_spine_id(spine.id);

        let result = spine.append(bad_entry);
        assert!(result.is_err());
        if let Err(LoamSpineError::ChainValidation { reason, .. }) = result {
            assert!(reason.contains("previous hash mismatch"));
        }
    }

    #[test]
    fn spine_state_frozen_and_archived() {
        let frozen = SpineState::Frozen {
            reason: "maintenance".into(),
            until: Some(Timestamp::from_nanos(1_000_000_000)),
        };
        assert!(!frozen.is_active());
        assert!(!frozen.is_sealed());
        assert!(!frozen.is_terminal());

        let archived = SpineState::Archived {
            archived_at: Timestamp::now(),
            archive_location: "cold://bucket/path".into(),
        };
        assert!(!archived.is_active());
        assert!(!archived.is_sealed());
        assert!(archived.is_terminal());
    }

    #[test]
    fn spine_verify_invalid_chain() {
        let owner = Did::new("did:key:z6MkOwner");
        let mut spine =
            Spine::new(owner, None, SpineConfig::default()).unwrap_or_else(|_| unreachable!());

        let entry = spine.create_entry(EntryType::DataAnchor {
            data_hash: [1u8; 32],
            mime_type: None,
            size: 1,
        });
        spine.append(entry).ok();

        spine.entries[1].index = 99;

        let verification = spine.verify();
        assert!(!verification.valid);
        assert!(!verification.errors.is_empty());
    }

    #[test]
    fn spine_builder_professional_and_community() {
        let owner = Did::new("did:key:z6MkOwner");

        let spine = Spine::builder(owner.clone())
            .with_type(SpineType::Professional)
            .build();
        assert!(spine.is_ok());
        assert_eq!(
            spine.unwrap_or_else(|_| unreachable!()).spine_type,
            SpineType::Professional
        );

        let spine = Spine::builder(owner)
            .with_type(SpineType::Community {
                community_id: "comm-123".into(),
            })
            .build();
        assert!(spine.is_ok());
        assert!(matches!(
            spine.unwrap_or_else(|_| unreachable!()).spine_type,
            SpineType::Community {
                community_id: ref id
            } if id == "comm-123"
        ));
    }

    #[test]
    fn spine_builder_waypoint_none() {
        let owner = Did::new("did:key:z6MkOwner");
        let spine = Spine::builder(owner).waypoint(None).build();
        assert!(spine.is_ok());
        assert!(matches!(
            spine.unwrap_or_else(|_| unreachable!()).spine_type,
            SpineType::Waypoint {
                max_anchor_depth: None
            }
        ));
    }

    #[test]
    fn spine_builder_with_replication() {
        let owner = Did::new("did:key:z6MkOwner");
        let spine = Spine::builder(owner).with_replication(true).build();
        assert!(spine.is_ok());
        assert!(
            spine
                .unwrap_or_else(|_| unreachable!())
                .config
                .replication_enabled
        );
    }

    #[test]
    fn spine_serde_roundtrip() {
        let owner = Did::new("did:key:z6MkOwner");
        let spine = Spine::new(owner, Some("Roundtrip".into()), SpineConfig::default())
            .unwrap_or_else(|_| unreachable!());

        let bytes = serde_json::to_vec(&spine).unwrap_or_else(|_| unreachable!());
        let restored: Spine = serde_json::from_slice(&bytes).unwrap_or_else(|_| unreachable!());

        assert_eq!(spine.id, restored.id);
        assert_eq!(spine.name, restored.name);
        assert_eq!(spine.height, restored.height);
    }

    #[test]
    fn chain_error_debug() {
        let err = ChainError::IndexGap {
            expected: 0,
            actual: 1,
        };
        let s = format!("{err:?}");
        assert!(s.contains("IndexGap"));
    }

    #[test]
    fn chain_error_hash_mismatch_and_invalid_signature() {
        let err = ChainError::HashMismatch {
            index: 1,
            expected: [0u8; 32],
            actual: [1u8; 32],
        };
        let s = format!("{err:?}");
        assert!(s.contains("HashMismatch"));

        let err2 = ChainError::InvalidSignature { index: 2 };
        let s2 = format!("{err2:?}");
        assert!(s2.contains("InvalidSignature"));
    }

    #[test]
    fn spine_seal_with_no_reason() {
        let owner = Did::new("did:key:z6MkOwner");
        let mut spine =
            Spine::new(owner, None, SpineConfig::default()).unwrap_or_else(|_| unreachable!());
        let result = spine.seal(None);
        assert!(result.is_ok());
        assert!(spine.is_sealed());
    }

    #[test]
    fn chain_verification_with_errors() {
        let v = ChainVerification {
            spine_id: SpineId::now_v7(),
            entries_verified: 3,
            valid: false,
            errors: vec![ChainError::IndexGap {
                expected: 1,
                actual: 2,
            }],
        };
        assert!(!v.valid);
        assert_eq!(v.errors.len(), 1);
    }
}
