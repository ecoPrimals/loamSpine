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
        let genesis_hash = genesis_entry.hash();

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
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    /// Check if the spine is sealed.
    #[must_use]
    pub fn is_sealed(&self) -> bool {
        self.state.is_sealed()
    }

    /// Append an entry to the spine.
    ///
    /// # Errors
    ///
    /// Returns an error if the spine is sealed or the entry is invalid.
    pub fn append(&mut self, mut entry: Entry) -> LoamSpineResult<EntryHash> {
        if !self.is_active() {
            return Err(LoamSpineError::SpineSealed(self.id));
        }

        // Validate entry
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

        // Compute hash and update spine
        let hash = entry.hash();
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

            prev_hash = Some(entry.compute_hash());
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpineState {
    /// Actively accepting entries.
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

impl Default for SpineState {
    fn default() -> Self {
        Self::Active
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
}
