// SPDX-License-Identifier: AGPL-3.0-only

//! Backup and restore functionality for LoamSpine.
//!
//! Provides export/import capabilities for disaster recovery
//! and data portability.

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

use crate::certificate::Certificate;
use crate::entry::Entry;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::spine::Spine;
use crate::types::{SpineId, Timestamp};

/// Version of the backup format.
pub const BACKUP_FORMAT_VERSION: u32 = 1;

/// Magic bytes for backup file identification.
pub const BACKUP_MAGIC: &[u8; 8] = b"LOAMSPIN";

/// A complete backup of a spine and its entries.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpineBackup {
    /// Backup format version.
    pub version: u32,

    /// When this backup was created.
    pub created_at: Timestamp,

    /// The spine metadata.
    pub spine: Spine,

    /// All entries in the spine (ordered by index).
    pub entries: Vec<Entry>,

    /// Certificates associated with this spine.
    pub certificates: Vec<Certificate>,

    /// Optional description/notes.
    pub description: Option<String>,
}

impl SpineBackup {
    /// Create a new backup from a spine and its entries.
    #[must_use]
    pub fn new(spine: Spine, entries: Vec<Entry>, certificates: Vec<Certificate>) -> Self {
        Self {
            version: BACKUP_FORMAT_VERSION,
            created_at: Timestamp::now(),
            spine,
            entries,
            certificates,
            description: None,
        }
    }

    /// Add a description to the backup.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Verify the backup integrity.
    ///
    /// Checks that:
    /// - All entries have valid indices
    /// - Entry hashes are correct
    /// - The hash chain is valid
    #[must_use]
    pub fn verify(&self) -> BackupVerification {
        let mut errors = Vec::new();

        if self.version > BACKUP_FORMAT_VERSION {
            errors.push(BackupError::UnsupportedVersion {
                found: self.version,
                max_supported: BACKUP_FORMAT_VERSION,
            });
        }

        let expected_entries = self.spine.height.try_into().unwrap_or(usize::MAX);
        if self.entries.len() != expected_entries {
            errors.push(BackupError::EntryCountMismatch {
                expected: expected_entries,
                found: self.entries.len(),
            });
        }

        let mut previous_hash: Option<[u8; 32]> = None;
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.index != i as u64 {
                errors.push(BackupError::InvalidEntryIndex {
                    expected: i as u64,
                    found: entry.index,
                });
            }

            let computed_hash = match entry.compute_hash() {
                Ok(h) => h,
                Err(e) => {
                    errors.push(BackupError::HashComputationFailed {
                        index: i as u64,
                        message: e.to_string(),
                    });
                    continue;
                }
            };

            if i > 0 {
                if let Some(prev) = previous_hash
                    && entry.previous != Some(prev)
                {
                    errors.push(BackupError::ChainBroken { at_index: i as u64 });
                }
            } else if entry.previous.is_some() {
                errors.push(BackupError::ChainBroken { at_index: 0 });
            }

            previous_hash = Some(computed_hash);
        }

        if errors.is_empty() {
            BackupVerification::valid()
        } else {
            BackupVerification::invalid(errors)
        }
    }

    /// Export the backup to a writer in binary format.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or writing fails.
    pub fn export<W: Write>(&self, writer: &mut W) -> LoamSpineResult<()> {
        writer
            .write_all(BACKUP_MAGIC)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to write magic bytes: {e}")))?;

        let data = bincode::serialize(self)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to serialize backup: {e}")))?;

        let len = data.len() as u64;
        writer
            .write_all(&len.to_le_bytes())
            .map_err(|e| LoamSpineError::Internal(format!("Failed to write length: {e}")))?;

        writer
            .write_all(&data)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to write data: {e}")))?;

        Ok(())
    }

    /// Import a backup from a reader.
    ///
    /// # Errors
    ///
    /// Returns an error if the magic bytes don't match or deserialization fails.
    pub fn import<R: Read>(reader: &mut R) -> LoamSpineResult<Self> {
        let mut magic = [0u8; 8];
        reader
            .read_exact(&mut magic)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to read magic bytes: {e}")))?;

        if &magic != BACKUP_MAGIC {
            return Err(LoamSpineError::Internal(
                "Invalid backup file: magic bytes don't match".into(),
            ));
        }

        let mut len_bytes = [0u8; 8];
        reader
            .read_exact(&mut len_bytes)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to read length: {e}")))?;
        let len: usize = u64::from_le_bytes(len_bytes).try_into().map_err(|_| {
            LoamSpineError::Internal("Backup size exceeds platform address space".into())
        })?;

        let mut data = vec![0u8; len];
        reader
            .read_exact(&mut data)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to read data: {e}")))?;

        let backup: Self = bincode::deserialize(&data)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to deserialize backup: {e}")))?;

        Ok(backup)
    }

    /// Export to JSON for human-readable format.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> LoamSpineResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to serialize to JSON: {e}")))
    }

    /// Import from JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn from_json(json: &str) -> LoamSpineResult<Self> {
        serde_json::from_str(json)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to deserialize from JSON: {e}")))
    }
}

/// Backup of multiple spines.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiSpineBackup {
    /// Backup format version.
    pub version: u32,

    /// When this backup was created.
    pub created_at: Timestamp,

    /// Individual spine backups.
    pub spines: Vec<SpineBackup>,

    /// Optional description.
    pub description: Option<String>,
}

impl MultiSpineBackup {
    /// Create a new multi-spine backup.
    #[must_use]
    pub fn new(spines: Vec<SpineBackup>) -> Self {
        Self {
            version: BACKUP_FORMAT_VERSION,
            created_at: Timestamp::now(),
            spines,
            description: None,
        }
    }

    /// Verify all spines in the backup.
    #[must_use]
    pub fn verify(&self) -> BackupVerification {
        let mut all_errors = Vec::new();

        for (i, spine) in self.spines.iter().enumerate() {
            let result = spine.verify();
            if !result.valid {
                for error in result.errors {
                    all_errors.push(BackupError::SpineError {
                        spine_index: i,
                        spine_id: spine.spine.id,
                        error: Box::new(error),
                    });
                }
            }
        }

        if all_errors.is_empty() {
            BackupVerification::valid()
        } else {
            BackupVerification::invalid(all_errors)
        }
    }
}

/// Result of backup verification.
#[derive(Clone, Debug)]
pub struct BackupVerification {
    /// Whether the backup is valid.
    pub valid: bool,

    /// Errors found during verification.
    pub errors: Vec<BackupError>,

    /// Verification timestamp.
    pub verified_at: Timestamp,
}

impl BackupVerification {
    /// Create a valid verification result.
    #[must_use]
    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            verified_at: Timestamp::now(),
        }
    }

    /// Create an invalid verification result.
    #[must_use]
    pub fn invalid(errors: Vec<BackupError>) -> Self {
        Self {
            valid: false,
            errors,
            verified_at: Timestamp::now(),
        }
    }
}

/// Errors that can occur during backup verification.
#[derive(Clone, Debug)]
pub enum BackupError {
    /// Unsupported backup format version.
    UnsupportedVersion {
        /// Version found in backup.
        found: u32,
        /// Maximum supported version.
        max_supported: u32,
    },

    /// Entry count doesn't match spine height.
    EntryCountMismatch {
        /// Expected count.
        expected: usize,
        /// Found count.
        found: usize,
    },

    /// Entry has invalid index.
    InvalidEntryIndex {
        /// Expected index.
        expected: u64,
        /// Found index.
        found: u64,
    },

    /// Entry hash doesn't match computed hash.
    HashMismatch {
        /// Entry index.
        index: u64,
        /// Expected hash.
        expected: [u8; 32],
        /// Computed hash.
        computed: [u8; 32],
    },

    /// Hash chain is broken.
    ChainBroken {
        /// Index where chain broke.
        at_index: u64,
    },

    /// Hash computation failed for an entry.
    HashComputationFailed {
        /// Entry index.
        index: u64,
        /// Error message.
        message: String,
    },

    /// Error in a specific spine (for multi-spine backups).
    SpineError {
        /// Index of the spine in the backup.
        spine_index: usize,
        /// Spine ID.
        spine_id: SpineId,
        /// The underlying error.
        error: Box<Self>,
    },
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
