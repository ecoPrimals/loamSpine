// SPDX-License-Identifier: AGPL-3.0-only

//! Tests for backup and restore functionality.

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
    let original = SpineBackup::new(spine, entries, Vec::new()).with_description("Roundtrip test");

    let mut buffer = Vec::new();
    original
        .export(&mut buffer)
        .unwrap_or_else(|_| unreachable!());

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
    let err = BackupError::UnsupportedVersion {
        found: 99,
        max_supported: 1,
    };
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("UnsupportedVersion"));

    let err = BackupError::EntryCountMismatch {
        expected: 10,
        found: 5,
    };
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("EntryCountMismatch"));

    let err = BackupError::InvalidEntryIndex {
        expected: 0,
        found: 1,
    };
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("InvalidEntryIndex"));

    let err = BackupError::HashMismatch {
        index: 5,
        expected: [1u8; 32],
        computed: [2u8; 32],
    };
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("HashMismatch"));

    let err = BackupError::ChainBroken { at_index: 42 };
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("ChainBroken"));

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
    backup.version = 999;

    let result = backup.verify();
    assert!(!result.valid);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, BackupError::UnsupportedVersion { .. }))
    );
}

#[test]
fn backup_verify_entry_count_mismatch() {
    let (mut spine, entries) = create_test_spine();
    spine.height = 100;

    let backup = SpineBackup::new(spine, entries, Vec::new());
    let result = backup.verify();

    assert!(!result.valid);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, BackupError::EntryCountMismatch { .. }))
    );
}

#[test]
fn backup_verify_invalid_entry_index() {
    let (spine, mut entries) = create_test_spine();
    entries[0].index = 99;

    let backup = SpineBackup::new(spine, entries, Vec::new());
    let result = backup.verify();

    assert!(!result.valid);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, BackupError::InvalidEntryIndex { .. }))
    );
}

#[test]
fn backup_verify_genesis_with_previous() {
    use crate::entry::SpineConfig;

    let owner = Did::new("did:key:z6MkOwner");
    let spine = SpineBuilder::new(owner.clone())
        .with_name("Test")
        .build()
        .unwrap_or_else(|_| unreachable!());

    let mut bad_genesis = crate::entry::Entry::new(
        0,
        Some([1u8; 32]),
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
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, BackupError::ChainBroken { at_index: 0 }))
    );
}

#[test]
fn multi_spine_backup_with_invalid_spine() {
    let (spine1, entries1) = create_test_spine();
    let (mut spine2, entries2) = create_test_spine();
    spine2.height = 100;

    let backup1 = SpineBackup::new(spine1, entries1, Vec::new());
    let backup2 = SpineBackup::new(spine2, entries2, Vec::new());

    let multi = MultiSpineBackup::new(vec![backup1, backup2]);
    let result = multi.verify();

    assert!(!result.valid);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, BackupError::SpineError { spine_index: 1, .. }))
    );
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

    let mut entries = vec![];
    let genesis = spine
        .genesis_entry()
        .cloned()
        .unwrap_or_else(|| unreachable!());
    entries.push(genesis);

    for i in 1..100 {
        let prev_hash = entries
            .last()
            .map(|e| e.compute_hash().expect("compute_hash"));
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

    let backup = SpineBackup::new(spine.clone(), vec![genesis], Vec::new());
    assert_eq!(backup.certificates.len(), 0);
    assert_eq!(backup.spine.id, spine.id);

    let result = backup.verify();
    assert!(result.valid);

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

    let json_str = serde_json::to_string(&backup).unwrap_or_else(|_| unreachable!());
    assert!(json_str.contains("version"));
    assert!(json_str.contains("spine"));

    let restored: SpineBackup = serde_json::from_str(&json_str).unwrap_or_else(|_| unreachable!());
    assert_eq!(restored.version, backup.version);
}

#[test]
fn backup_with_metadata() {
    let (spine, entries) = create_test_spine();
    let backup = SpineBackup::new(spine, entries, Vec::new()).with_description("Test description");

    assert!(backup.description.is_some());
    assert_eq!(backup.description.as_deref(), Some("Test description"));

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
    assert!(result.valid);
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

    for spine_backup in &multi.spines {
        let verify_result = spine_backup.verify();
        assert!(verify_result.valid);
    }
}

#[test]
fn backup_import_from_empty() {
    let empty_data: Vec<u8> = vec![];
    let mut cursor = Cursor::new(empty_data);
    let result = SpineBackup::import(&mut cursor);

    assert!(result.is_err());
}

#[test]
fn backup_empty_description() {
    let (spine, entries) = create_test_spine();
    let backup = SpineBackup::new(spine, entries, Vec::new()).with_description("");

    assert!(backup.description.is_some());
    assert_eq!(backup.description.as_deref(), Some(""));
}

#[test]
fn backup_on_empty_spine_entry_count_mismatch() {
    let (mut spine, _entries) = create_test_spine();
    spine.height = 1;

    let backup = SpineBackup::new(spine, Vec::new(), Vec::new());
    let result = backup.verify();

    assert!(!result.valid);
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e, BackupError::EntryCountMismatch { .. }))
    );
}

#[test]
fn backup_restore_from_truncated_data() {
    let (spine, entries) = create_test_spine();
    let backup = SpineBackup::new(spine, entries, Vec::new());

    let mut buffer = Vec::new();
    backup.export(&mut buffer).expect("export should succeed");

    let mut truncated = Vec::new();
    truncated.extend_from_slice(BACKUP_MAGIC);
    let len: u64 = 1000;
    truncated.extend_from_slice(&len.to_le_bytes());
    truncated.extend_from_slice(&[0u8; 10]);

    let mut cursor = Cursor::new(truncated);
    let result = SpineBackup::import(&mut cursor);
    assert!(result.is_err());
}

#[test]
fn backup_restore_from_corrupt_bincode() {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(BACKUP_MAGIC);
    let len: u64 = 20;
    buffer.extend_from_slice(&len.to_le_bytes());
    buffer.extend_from_slice(b"corrupt-bincode-data!!");

    let mut cursor = Cursor::new(buffer);
    let result = SpineBackup::import(&mut cursor);
    assert!(result.is_err());
}

#[test]
fn backup_from_json_invalid() {
    let result = SpineBackup::from_json("not valid json {");
    assert!(result.is_err());
}

#[test]
fn backup_roundtrip_create_add_entries_export_import_verify() {
    let owner = Did::new("did:key:z6MkRoundtrip");
    let spine = SpineBuilder::new(owner.clone())
        .with_name("Roundtrip Spine")
        .build()
        .expect("spine build should succeed");

    let mut entries = vec![];
    let genesis = spine
        .genesis_entry()
        .cloned()
        .expect("genesis should exist");
    entries.push(genesis);

    for i in 1..5 {
        let prev_hash = entries.last().and_then(|e| e.compute_hash().ok());
        let mut entry = Entry::new(
            i,
            prev_hash,
            owner.clone(),
            EntryType::DataAnchor {
                #[allow(clippy::cast_possible_truncation)]
                data_hash: [{ i as u8 }; 32],
                mime_type: Some("application/octet-stream".into()),
                size: 64,
            },
        );
        entry.spine_id = spine.id;
        entries.push(entry);
    }

    let mut spine_with_height = spine;
    spine_with_height.height = entries.len() as u64;

    let backup = SpineBackup::new(spine_with_height, entries.clone(), Vec::new());
    let verify_before = backup.verify();
    assert!(verify_before.valid, "backup should verify before export");

    let mut buffer = Vec::new();
    backup.export(&mut buffer).expect("export should succeed");
    assert!(!buffer.is_empty());

    let mut cursor = Cursor::new(buffer);
    let restored = SpineBackup::import(&mut cursor).expect("import should succeed");

    let verify_after = restored.verify();
    assert!(verify_after.valid, "restored backup should verify");
    assert_eq!(restored.entries.len(), entries.len());
    assert_eq!(restored.spine.id, backup.spine.id);
}
