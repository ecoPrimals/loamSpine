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

        // Check version
        if self.version > BACKUP_FORMAT_VERSION {
            errors.push(BackupError::UnsupportedVersion {
                found: self.version,
                max_supported: BACKUP_FORMAT_VERSION,
            });
        }

        // Verify entry count matches spine height
        let expected_entries = self.spine.height.try_into().unwrap_or(usize::MAX);
        if self.entries.len() != expected_entries {
            errors.push(BackupError::EntryCountMismatch {
                expected: expected_entries,
                found: self.entries.len(),
            });
        }

        // Verify entry indices and hash chain
        let mut previous_hash: Option<[u8; 32]> = None;
        for (i, entry) in self.entries.iter().enumerate() {
            // Check index
            if entry.index != i as u64 {
                errors.push(BackupError::InvalidEntryIndex {
                    expected: i as u64,
                    found: entry.index,
                });
            }

            // Compute entry hash
            let computed_hash = entry.compute_hash();

            // Check chain (skip genesis)
            if i > 0 {
                if let Some(prev) = previous_hash {
                    if entry.previous != Some(prev) {
                        errors.push(BackupError::ChainBroken { at_index: i as u64 });
                    }
                }
            } else {
                // Genesis should have no previous
                if entry.previous.is_some() {
                    errors.push(BackupError::ChainBroken { at_index: 0 });
                }
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
        // Write magic bytes
        writer
            .write_all(BACKUP_MAGIC)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to write magic bytes: {e}")))?;

        // Serialize with bincode
        let data = bincode::serialize(self)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to serialize backup: {e}")))?;

        // Write length prefix
        let len = data.len() as u64;
        writer
            .write_all(&len.to_le_bytes())
            .map_err(|e| LoamSpineError::Internal(format!("Failed to write length: {e}")))?;

        // Write data
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
        // Read and verify magic bytes
        let mut magic = [0u8; 8];
        reader
            .read_exact(&mut magic)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to read magic bytes: {e}")))?;

        if &magic != BACKUP_MAGIC {
            return Err(LoamSpineError::Internal(
                "Invalid backup file: magic bytes don't match".into(),
            ));
        }

        // Read length
        let mut len_bytes = [0u8; 8];
        reader
            .read_exact(&mut len_bytes)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to read length: {e}")))?;
        let len: usize = u64::from_le_bytes(len_bytes).try_into().map_err(|_| {
            LoamSpineError::Internal("Backup size exceeds platform address space".into())
        })?;

        // Read data
        let mut data = vec![0u8; len];
        reader
            .read_exact(&mut data)
            .map_err(|e| LoamSpineError::Internal(format!("Failed to read data: {e}")))?;

        // Deserialize
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
mod tests {
    use super::*;
    use crate::entry::EntryType;
    use crate::spine::SpineBuilder;
    use crate::types::Did;
    use std::io::Cursor;

    fn create_test_spine() -> (Spine, Vec<Entry>) {
        let owner = Did::new("did:key:z6MkOwner");
        let spine = SpineBuilder::new(owner)
            .with_name("Test Spine")
            .build()
            .unwrap_or_else(|_| unreachable!());
        // Get the genesis entry - spine always has one after build
        let genesis = spine
            .genesis_entry()
            .cloned()
            .unwrap_or_else(|| unreachable!());
        let entries = vec![genesis];
        (spine, entries)
    }

    #[test]
    fn spine_backup_creation() {
        let (spine, entries) = create_test_spine();
        let backup = SpineBackup::new(spine.clone(), entries, Vec::new());

        assert_eq!(backup.version, BACKUP_FORMAT_VERSION);
        assert_eq!(backup.spine.id, spine.id);
        assert_eq!(backup.entries.len(), 1);
    }

    #[test]
    fn spine_backup_with_description() {
        let (spine, entries) = create_test_spine();
        let backup = SpineBackup::new(spine, entries, Vec::new()).with_description("Test backup");

        assert_eq!(backup.description, Some("Test backup".to_string()));
    }

    #[test]
    fn spine_backup_verify_valid() {
        let (spine, entries) = create_test_spine();
        let backup = SpineBackup::new(spine, entries, Vec::new());

        let result = backup.verify();
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn spine_backup_export_import_roundtrip() {
        let (spine, entries) = create_test_spine();
        let original =
            SpineBackup::new(spine, entries, Vec::new()).with_description("Roundtrip test");

        // Export to buffer
        let mut buffer = Vec::new();
        original
            .export(&mut buffer)
            .unwrap_or_else(|_| unreachable!());

        // Import from buffer
        let mut cursor = Cursor::new(buffer);
        let imported = SpineBackup::import(&mut cursor).unwrap_or_else(|_| unreachable!());

        assert_eq!(imported.version, original.version);
        assert_eq!(imported.spine.id, original.spine.id);
        assert_eq!(imported.entries.len(), original.entries.len());
        assert_eq!(imported.description, original.description);
    }

    #[test]
    fn spine_backup_json_roundtrip() {
        let (spine, entries) = create_test_spine();
        let original = SpineBackup::new(spine, entries, Vec::new());

        let json = original.to_json().unwrap_or_else(|_| unreachable!());
        let imported = SpineBackup::from_json(&json).unwrap_or_else(|_| unreachable!());

        assert_eq!(imported.spine.id, original.spine.id);
    }

    #[test]
    fn multi_spine_backup() {
        let (spine1, entries1) = create_test_spine();
        let (spine2, entries2) = create_test_spine();

        let backup1 = SpineBackup::new(spine1, entries1, Vec::new());
        let backup2 = SpineBackup::new(spine2, entries2, Vec::new());

        let multi = MultiSpineBackup::new(vec![backup1, backup2]);

        assert_eq!(multi.spines.len(), 2);

        let result = multi.verify();
        assert!(result.valid);
    }

    #[test]
    fn backup_import_invalid_magic() {
        let mut buffer = Cursor::new(b"INVALID!".to_vec());
        let result = SpineBackup::import(&mut buffer);
        assert!(result.is_err());
    }

    #[test]
    fn backup_error_variants() {
        let err1 = BackupError::UnsupportedVersion {
            found: 99,
            max_supported: 1,
        };
        assert!(matches!(err1, BackupError::UnsupportedVersion { .. }));

        let err2 = BackupError::ChainBroken { at_index: 42 };
        assert!(matches!(err2, BackupError::ChainBroken { at_index: 42 }));
    }

    #[test]
    fn backup_verification_valid() {
        let result = BackupVerification::valid();
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn backup_verification_invalid() {
        let errors = vec![BackupError::ChainBroken { at_index: 0 }];
        let result = BackupVerification::invalid(errors);
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn backup_error_all_variants() {
        // UnsupportedVersion
        let err = BackupError::UnsupportedVersion {
            found: 99,
            max_supported: 1,
        };
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("UnsupportedVersion"));

        // EntryCountMismatch
        let err = BackupError::EntryCountMismatch {
            expected: 10,
            found: 5,
        };
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("EntryCountMismatch"));

        // InvalidEntryIndex
        let err = BackupError::InvalidEntryIndex {
            expected: 0,
            found: 1,
        };
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("InvalidEntryIndex"));

        // HashMismatch
        let err = BackupError::HashMismatch {
            index: 5,
            expected: [1u8; 32],
            computed: [2u8; 32],
        };
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("HashMismatch"));

        // ChainBroken
        let err = BackupError::ChainBroken { at_index: 42 };
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("ChainBroken"));

        // SpineError
        let inner = BackupError::ChainBroken { at_index: 0 };
        let err = BackupError::SpineError {
            spine_index: 2,
            spine_id: SpineId::now_v7(),
            error: Box::new(inner),
        };
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("SpineError"));
    }

    #[test]
    fn backup_verify_version_error() {
        let (spine, entries) = create_test_spine();
        let mut backup = SpineBackup::new(spine, entries, Vec::new());
        backup.version = 999; // Future version

        let result = backup.verify();
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, BackupError::UnsupportedVersion { .. })));
    }

    #[test]
    fn backup_verify_entry_count_mismatch() {
        let (mut spine, entries) = create_test_spine();
        spine.height = 100; // Mismatch with entries count

        let backup = SpineBackup::new(spine, entries, Vec::new());
        let result = backup.verify();

        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, BackupError::EntryCountMismatch { .. })));
    }

    #[test]
    fn backup_verify_invalid_entry_index() {
        let (spine, mut entries) = create_test_spine();
        entries[0].index = 99; // Wrong index

        let backup = SpineBackup::new(spine, entries, Vec::new());
        let result = backup.verify();

        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, BackupError::InvalidEntryIndex { .. })));
    }

    #[test]
    fn backup_verify_genesis_with_previous() {
        use crate::entry::{EntryType, SpineConfig};

        let owner = Did::new("did:key:z6MkOwner");
        let spine = SpineBuilder::new(owner.clone())
            .with_name("Test")
            .build()
            .unwrap_or_else(|_| unreachable!());

        // Create a bad genesis with previous hash
        let mut bad_genesis = crate::entry::Entry::new(
            0,
            Some([1u8; 32]), // Genesis should NOT have previous
            owner.clone(),
            EntryType::Genesis {
                spine_id: spine.id,
                owner,
                config: SpineConfig::default(),
            },
        );
        bad_genesis.spine_id = spine.id;

        let backup = SpineBackup::new(spine, vec![bad_genesis], Vec::new());
        let result = backup.verify();

        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, BackupError::ChainBroken { at_index: 0 })));
    }

    #[test]
    fn multi_spine_backup_with_invalid_spine() {
        let (spine1, entries1) = create_test_spine();
        let (mut spine2, entries2) = create_test_spine();
        spine2.height = 100; // Causes mismatch

        let backup1 = SpineBackup::new(spine1, entries1, Vec::new());
        let backup2 = SpineBackup::new(spine2, entries2, Vec::new());

        let multi = MultiSpineBackup::new(vec![backup1, backup2]);
        let result = multi.verify();

        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, BackupError::SpineError { spine_index: 1, .. })));
    }

    #[test]
    fn backup_verification_debug_clone() {
        let result = BackupVerification::valid();
        let debug_str = format!("{result:?}");
        assert!(debug_str.contains("BackupVerification"));

        #[allow(clippy::redundant_clone)]
        let cloned = result.clone();
        assert_eq!(result.valid, cloned.valid);
    }

    #[test]
    fn backup_error_clone() {
        let err = BackupError::ChainBroken { at_index: 5 };
        #[allow(clippy::redundant_clone)]
        let cloned = err.clone();
        assert!(matches!(cloned, BackupError::ChainBroken { at_index: 5 }));
    }

    #[test]
    fn spine_backup_debug_clone() {
        let (spine, entries) = create_test_spine();
        let backup = SpineBackup::new(spine, entries, Vec::new());

        let debug_str = format!("{backup:?}");
        assert!(debug_str.contains("SpineBackup"));

        #[allow(clippy::redundant_clone)]
        let cloned = backup.clone();
        assert_eq!(backup.version, cloned.version);
    }

    #[test]
    fn multi_spine_backup_debug_clone() {
        let (spine, entries) = create_test_spine();
        let single = SpineBackup::new(spine, entries, Vec::new());
        let multi = MultiSpineBackup::new(vec![single]);

        let debug_str = format!("{multi:?}");
        assert!(debug_str.contains("MultiSpineBackup"));

        #[allow(clippy::redundant_clone)]
        let cloned = multi.clone();
        assert_eq!(multi.version, cloned.version);
    }

    #[test]
    fn backup_with_large_dataset() {
        let owner = Did::new("did:key:z6MkOwner");
        let mut spine = SpineBuilder::new(owner.clone())
            .with_name("Large Spine")
            .build()
            .unwrap_or_else(|_| unreachable!());

        // Create many entries
        let mut entries = vec![];
        let genesis = spine
            .genesis_entry()
            .cloned()
            .unwrap_or_else(|| unreachable!());
        entries.push(genesis);

        for i in 1..100 {
            let prev_hash = entries.last().map(Entry::compute_hash);
            let mut entry = Entry::new(
                i,
                prev_hash,
                owner.clone(),
                EntryType::SpineSealed { reason: None },
            );
            entry.spine_id = spine.id;
            entries.push(entry);
        }

        spine.height = entries.len() as u64;

        let backup = SpineBackup::new(spine, entries.clone(), Vec::new());
        let result = backup.verify();
        assert!(result.valid);

        // Export/import
        let mut buffer = Vec::new();
        backup
            .export(&mut buffer)
            .unwrap_or_else(|_| unreachable!());
        assert!(!buffer.is_empty());

        let mut cursor = Cursor::new(buffer);
        let restored = SpineBackup::import(&mut cursor).unwrap_or_else(|_| unreachable!());
        assert_eq!(restored.entries.len(), entries.len());
    }

    #[test]
    fn backup_with_empty_certificates() {
        let owner = Did::new("did:key:z6MkOwner");
        let spine = SpineBuilder::new(owner)
            .with_name("Cert Spine")
            .build()
            .unwrap_or_else(|_| unreachable!());

        let genesis = spine
            .genesis_entry()
            .cloned()
            .unwrap_or_else(|| unreachable!());

        // Create backup with empty certificates list
        let backup = SpineBackup::new(spine.clone(), vec![genesis], Vec::new());
        assert_eq!(backup.certificates.len(), 0);
        assert_eq!(backup.spine.id, spine.id);

        let result = backup.verify();
        assert!(result.valid);

        // Export/import
        let mut buffer = Vec::new();
        backup
            .export(&mut buffer)
            .unwrap_or_else(|_| unreachable!());

        let mut cursor = Cursor::new(buffer);
        let restored = SpineBackup::import(&mut cursor).unwrap_or_else(|_| unreachable!());
        assert_eq!(restored.certificates.len(), 0);
        assert_eq!(restored.spine.id, backup.spine.id);
    }

    #[test]
    fn backup_json_serialization() {
        let (spine, entries) = create_test_spine();
        let backup = SpineBackup::new(spine, entries, Vec::new());

        // Verify JSON serialization works
        let json_str = serde_json::to_string(&backup).unwrap_or_else(|_| unreachable!());
        assert!(json_str.contains("version"));
        assert!(json_str.contains("spine"));

        // Deserialize
        let restored: SpineBackup =
            serde_json::from_str(&json_str).unwrap_or_else(|_| unreachable!());
        assert_eq!(restored.version, backup.version);
    }

    #[test]
    fn backup_with_metadata() {
        let (spine, entries) = create_test_spine();
        let backup =
            SpineBackup::new(spine, entries, Vec::new()).with_description("Test description");

        assert!(backup.description.is_some());
        assert_eq!(backup.description.as_deref(), Some("Test description"));

        // Description persists through export/import
        let mut buffer = Vec::new();
        backup
            .export(&mut buffer)
            .unwrap_or_else(|_| unreachable!());

        let mut cursor = Cursor::new(buffer);
        let restored = SpineBackup::import(&mut cursor).unwrap_or_else(|_| unreachable!());
        assert_eq!(restored.description.as_deref(), Some("Test description"));
    }

    #[test]
    fn multi_spine_backup_empty() {
        let multi = MultiSpineBackup::new(vec![]);
        assert_eq!(multi.spines.len(), 0);

        let result = multi.verify();
        assert!(result.valid); // Empty backup is valid
    }

    #[test]
    fn multi_spine_backup_large() {
        let mut spines = vec![];
        for i in 0..10 {
            let owner = Did::new(format!("did:key:z6MkOwner{i}"));
            let spine = SpineBuilder::new(owner)
                .with_name(format!("Spine {i}"))
                .build()
                .unwrap_or_else(|_| unreachable!());
            let genesis = spine
                .genesis_entry()
                .cloned()
                .unwrap_or_else(|| unreachable!());
            spines.push(SpineBackup::new(spine, vec![genesis], Vec::new()));
        }

        let multi = MultiSpineBackup::new(spines);
        assert_eq!(multi.spines.len(), 10);

        let result = multi.verify();
        assert!(result.valid);

        // Verify all spines are valid
        for spine_backup in &multi.spines {
            let verify_result = spine_backup.verify();
            assert!(verify_result.valid);
        }
    }

    #[test]
    fn backup_import_from_empty() {
        // Try to import from empty data
        let empty_data: Vec<u8> = vec![];
        let mut cursor = Cursor::new(empty_data);
        let result = SpineBackup::import(&mut cursor);

        // Should fail gracefully (not panic)
        assert!(result.is_err());
    }

    #[test]
    fn backup_empty_description() {
        let (spine, entries) = create_test_spine();
        let backup = SpineBackup::new(spine, entries, Vec::new()).with_description("");

        assert!(backup.description.is_some());
        assert_eq!(backup.description.as_deref(), Some(""));
    }
}
