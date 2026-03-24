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
                    expected: prev_hash,
                    actual: entry.previous,
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
#[non_exhaustive]
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
        /// Expected hash (None for genesis entry).
        expected: Option<EntryHash>,
        /// Actual hash (None if entry has no `previous` pointer).
        actual: Option<EntryHash>,
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
#[path = "spine_proptests.rs"]
mod proptests;
#[cfg(test)]
#[path = "spine_tests.rs"]
mod tests;
