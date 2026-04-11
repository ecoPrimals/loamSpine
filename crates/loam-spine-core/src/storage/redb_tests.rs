// SPDX-License-Identifier: AGPL-3.0-or-later

#![expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]

use std::sync::Arc;

use crate::entry::{Entry, EntryType, SpineConfig};
use crate::spine::Spine;
use crate::storage::{EntryStorage, RedbEntryStorage, RedbSpineStorage, RedbStorage, SpineStorage};
use crate::types::{Did, SpineId};

fn create_test_spine() -> Spine {
    let owner = Did::new("did:key:z6MkOwner");
    Spine::new(owner, Some("Test".into()), SpineConfig::default())
        .unwrap_or_else(|_| unreachable!())
}

#[tokio::test]
async fn redb_spine_storage_crud() {
    let storage = RedbSpineStorage::temporary().unwrap_or_else(|_| unreachable!());

    let spine = create_test_spine();
    let id = spine.id;

    storage
        .save_spine(&spine)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(storage.spine_count(), 1);

    let retrieved = storage
        .get_spine(id)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap_or_else(|| unreachable!()).id, id);

    let ids = storage
        .list_spines()
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&id));

    storage
        .delete_spine(id)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(storage.spine_count(), 0);

    let retrieved = storage
        .get_spine(id)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn redb_entry_storage_crud() {
    let storage = RedbEntryStorage::temporary().unwrap_or_else(|_| unreachable!());

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let entry = Entry::genesis(owner, spine_id, SpineConfig::default());

    let hash = storage
        .save_entry(&entry)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(storage.entry_count(), 1);

    let retrieved = storage
        .get_entry(hash)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(retrieved.is_some());

    let exists = storage
        .entry_exists(hash)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(exists);

    let not_exists = storage
        .entry_exists([0u8; 32])
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(!not_exists);
}

#[tokio::test]
async fn redb_entry_spine_index() {
    let storage = RedbEntryStorage::temporary().unwrap_or_else(|_| unreachable!());

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();

    let mut prev_hash = None;
    for i in 0..5 {
        let entry = if i == 0 {
            Entry::genesis(owner.clone(), spine_id, SpineConfig::default())
        } else {
            Entry::new(
                i,
                prev_hash,
                owner.clone(),
                EntryType::SpineSealed { reason: None },
            )
            .with_spine_id(spine_id)
        };
        prev_hash = Some(
            storage
                .save_entry(&entry)
                .await
                .unwrap_or_else(|_| unreachable!()),
        );
    }

    let entries = storage
        .get_entries_for_spine(spine_id, 0, 10)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(entries.len(), 5);

    let entries = storage
        .get_entries_for_spine(spine_id, 1, 2)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(entries.len(), 2);

    let entries = storage
        .get_entries_for_spine(SpineId::now_v7(), 0, 10)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(entries.is_empty());
}

#[tokio::test]
async fn redb_combined_storage() {
    let storage = RedbStorage::temporary().unwrap_or_else(|_| unreachable!());

    let spine = create_test_spine();
    storage
        .spines
        .save_spine(&spine)
        .await
        .unwrap_or_else(|_| unreachable!());

    let owner = Did::new("did:key:z6MkOwner");
    let entry = Entry::genesis(owner, spine.id, SpineConfig::default());
    let hash = storage
        .entries
        .save_entry(&entry)
        .await
        .unwrap_or_else(|_| unreachable!());

    storage.flush().unwrap_or_else(|_| unreachable!());

    assert_eq!(storage.spines.spine_count(), 1);
    assert_eq!(storage.entries.entry_count(), 1);

    let retrieved_spine = storage
        .spines
        .get_spine(spine.id)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(retrieved_spine.is_some());

    let retrieved_entry = storage
        .entries
        .get_entry(hash)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(retrieved_entry.is_some());
}

#[tokio::test]
async fn redb_storage_persistence() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|e| unreachable!("tempdir failed: {e}"));

    let spine_id = {
        let storage = RedbStorage::open(temp_dir.path())
            .unwrap_or_else(|e| unreachable!("redb open failed: {e}"));
        let spine = create_test_spine();
        let id = spine.id;
        storage
            .spines
            .save_spine(&spine)
            .await
            .unwrap_or_else(|e| unreachable!("save spine failed: {e}"));
        storage
            .flush()
            .unwrap_or_else(|e| unreachable!("flush failed: {e}"));
        drop(storage);
        id
    };

    {
        let storage = RedbStorage::open(temp_dir.path())
            .unwrap_or_else(|e| unreachable!("redb reopen failed: {e}"));
        assert_eq!(storage.spines.spine_count(), 1);
        let retrieved = storage
            .spines
            .get_spine(spine_id)
            .await
            .unwrap_or_else(|e| unreachable!("get spine failed: {e}"));
        assert!(
            retrieved.is_some(),
            "persisted spine should be retrievable after reopen"
        );
    }
}

#[tokio::test]
async fn redb_concurrent_operations() {
    let storage = Arc::new(
        RedbStorage::temporary().unwrap_or_else(|e| unreachable!("temporary redb failed: {e}")),
    );

    let mut handles = vec![];
    for i in 0..10 {
        let stor = Arc::clone(&storage);
        let handle = tokio::spawn(async move {
            let owner = Did::new(format!("did:key:z6MkOwner{i}"));
            let spine = Spine::new(owner, Some(format!("Spine {i}")), SpineConfig::default())
                .unwrap_or_else(|_| unreachable!());
            stor.spines.save_spine(&spine).await
        });
        handles.push(handle);
    }

    for handle in handles {
        assert!(handle.await.unwrap_or_else(|_| unreachable!()).is_ok());
    }

    assert_eq!(storage.spines.spine_count(), 10);
}

// Error-path and edge-case coverage

fn create_redb_test_spine() -> Spine {
    let owner = Did::new("did:key:z6MkRedbOwner");
    Spine::new(owner, Some("RedbTest".into()), SpineConfig::default())
        .unwrap_or_else(|_| unreachable!())
}

#[tokio::test]
async fn redb_spine_count_on_fresh_db() {
    let storage = RedbSpineStorage::temporary().unwrap();
    assert_eq!(storage.spine_count(), 0);
}

#[tokio::test]
async fn redb_entry_count_on_fresh_db() {
    let storage = RedbEntryStorage::temporary().unwrap();
    assert_eq!(storage.entry_count(), 0);
}

#[tokio::test]
async fn redb_get_nonexistent_spine() {
    let storage = RedbSpineStorage::temporary().unwrap();
    let result = storage.get_spine(SpineId::now_v7()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn redb_get_nonexistent_entry() {
    let storage = RedbEntryStorage::temporary().unwrap();
    let result = storage.get_entry([0u8; 32]).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn redb_list_spines_empty() {
    let storage = RedbSpineStorage::temporary().unwrap();
    let ids = storage.list_spines().await.unwrap();
    assert!(ids.is_empty());
}

#[tokio::test]
async fn redb_get_entries_for_spine_empty() {
    let storage = RedbEntryStorage::temporary().unwrap();
    let entries = storage
        .get_entries_for_spine(SpineId::now_v7(), 0, 100)
        .await
        .unwrap();
    assert!(entries.is_empty());
}

#[tokio::test]
async fn redb_save_and_retrieve_multiple_entries() {
    let spine_storage = RedbSpineStorage::temporary().unwrap();
    let entry_storage = RedbEntryStorage::temporary().unwrap();

    let spine = create_redb_test_spine();
    let owner = Did::new("did:key:z6MkRedbOwner");
    spine_storage.save_spine(&spine).await.unwrap();

    let mut prev_hash = None;
    for i in 0..5 {
        let entry = if i == 0 {
            Entry::genesis(owner.clone(), spine.id, SpineConfig::default())
        } else {
            Entry::new(
                i,
                prev_hash,
                owner.clone(),
                EntryType::SpineSealed { reason: None },
            )
            .with_spine_id(spine.id)
        };
        prev_hash = Some(entry_storage.save_entry(&entry).await.unwrap());
    }

    let entries = entry_storage
        .get_entries_for_spine(spine.id, 0, 10)
        .await
        .unwrap();
    assert_eq!(entries.len(), 5);
    assert_eq!(entry_storage.entry_count(), 5);
}

#[tokio::test]
async fn redb_flush_is_infallible() {
    let storage = RedbSpineStorage::temporary().unwrap();
    assert!(storage.flush().is_ok());
}

#[tokio::test]
async fn redb_entry_exists_false_for_missing() {
    let storage = RedbEntryStorage::temporary().unwrap();
    let exists = storage.entry_exists([42u8; 32]).await.unwrap();
    assert!(!exists);
}

#[tokio::test]
async fn redb_spine_delete() {
    let storage = RedbSpineStorage::temporary().unwrap();
    let spine = create_redb_test_spine();
    storage.save_spine(&spine).await.unwrap();
    assert_eq!(storage.spine_count(), 1);
    let _ = storage.delete_spine(spine.id).await;
    assert_eq!(storage.spine_count(), 0);
}

// ========================================================================
// Constructor variations
// ========================================================================

#[tokio::test]
async fn redb_open_with_new_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let storage = RedbSpineStorage::open(temp_dir.path().join("spines.redb")).unwrap();
    assert_eq!(storage.spine_count(), 0);
}

#[tokio::test]
async fn redb_open_with_nested_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let nested = temp_dir.path().join("a/b");
    let storage = RedbSpineStorage::open(nested.join("spines.redb")).unwrap();
    assert_eq!(storage.spine_count(), 0);
}

#[tokio::test]
async fn redb_storage_open_with_base_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let storage = RedbStorage::open(temp_dir.path()).unwrap();
    assert_eq!(storage.spines.spine_count(), 0);
    assert_eq!(storage.entries.entry_count(), 0);
    assert_eq!(storage.certificates.certificate_count(), 0);
}

#[tokio::test]
async fn redb_open_existing_dir_preserves_data() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("spines.redb");

    {
        let storage = RedbSpineStorage::open(&db_path).unwrap();
        let spine = create_redb_test_spine();
        storage.save_spine(&spine).await.unwrap();
        storage.flush().unwrap();
        drop(storage);
    }

    {
        let storage = RedbSpineStorage::open(&db_path).unwrap();
        assert_eq!(storage.spine_count(), 1);
    }
}

// ========================================================================
// Operations on empty storage
// ========================================================================

#[tokio::test]
async fn redb_entry_storage_get_entries_empty() {
    let storage = RedbEntryStorage::temporary().unwrap();
    let entries = storage
        .get_entries_for_spine(SpineId::now_v7(), 0, 100)
        .await
        .unwrap();
    assert!(entries.is_empty());
}

#[tokio::test]
async fn redb_entry_storage_get_entries_with_offset_on_empty() {
    let storage = RedbEntryStorage::temporary().unwrap();
    let entries = storage
        .get_entries_for_spine(SpineId::now_v7(), 5, 10)
        .await
        .unwrap();
    assert!(entries.is_empty());
}

#[tokio::test]
async fn redb_entry_storage_get_entries_limit_zero() {
    let storage = RedbEntryStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let entry = Entry::genesis(owner, spine_id, SpineConfig::default());
    storage.save_entry(&entry).await.unwrap();

    let entries = storage.get_entries_for_spine(spine_id, 0, 0).await.unwrap();
    assert!(entries.is_empty());
}

// ========================================================================
// Multiple sequential operations
// ========================================================================

#[tokio::test]
async fn redb_spine_sequential_save_delete_save() {
    let storage = RedbSpineStorage::temporary().unwrap();
    let spine = create_redb_test_spine();
    let id = spine.id;

    storage.save_spine(&spine).await.unwrap();
    assert_eq!(storage.spine_count(), 1);

    storage.delete_spine(id).await.unwrap();
    assert_eq!(storage.spine_count(), 0);

    storage.save_spine(&spine).await.unwrap();
    assert_eq!(storage.spine_count(), 1);

    let retrieved = storage.get_spine(id).await.unwrap();
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn redb_entry_sequential_saves() {
    let storage = RedbEntryStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();

    let mut hashes = Vec::new();
    for i in 0..10 {
        let entry = if i == 0 {
            Entry::genesis(owner.clone(), spine_id, SpineConfig::default())
        } else {
            Entry::new(
                i,
                hashes.last().copied(),
                owner.clone(),
                EntryType::SpineSealed { reason: None },
            )
            .with_spine_id(spine_id)
        };
        let hash = storage.save_entry(&entry).await.unwrap();
        hashes.push(hash);
    }

    assert_eq!(storage.entry_count(), 10);
    let entries = storage
        .get_entries_for_spine(spine_id, 0, 20)
        .await
        .unwrap();
    assert_eq!(entries.len(), 10);
}

// ========================================================================
// Count methods after operations
// ========================================================================

#[tokio::test]
async fn redb_spine_count_after_multiple_operations() {
    let storage = RedbSpineStorage::temporary().unwrap();

    for i in 0..5 {
        let owner = Did::new(format!("did:key:z6MkOwner{i}"));
        let spine = Spine::new(owner, Some(format!("Spine {i}")), SpineConfig::default()).unwrap();
        storage.save_spine(&spine).await.unwrap();
        assert_eq!(storage.spine_count(), i + 1);
    }

    assert_eq!(storage.spine_count(), 5);
}

#[tokio::test]
async fn redb_entry_count_after_multiple_operations() {
    let storage = RedbEntryStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();

    let mut prev_hash = None;
    for i in 0..7 {
        let entry = if i == 0 {
            Entry::genesis(owner.clone(), spine_id, SpineConfig::default())
        } else {
            Entry::new(
                i,
                prev_hash,
                owner.clone(),
                EntryType::SpineSealed { reason: None },
            )
            .with_spine_id(spine_id)
        };
        prev_hash = Some(storage.save_entry(&entry).await.unwrap());
        assert_eq!(storage.entry_count(), usize::try_from(i + 1).unwrap_or(0));
    }

    assert_eq!(storage.entry_count(), 7);
}

#[tokio::test]
async fn redb_get_entries_for_spine_boundary_start_index() {
    let storage = RedbEntryStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();

    let mut prev_hash = None;
    for i in 0..5 {
        let entry = if i == 0 {
            Entry::genesis(owner.clone(), spine_id, SpineConfig::default())
        } else {
            Entry::new(
                i,
                prev_hash,
                owner.clone(),
                EntryType::SpineSealed { reason: None },
            )
            .with_spine_id(spine_id)
        };
        prev_hash = Some(storage.save_entry(&entry).await.unwrap());
    }

    let entries = storage.get_entries_for_spine(spine_id, 2, 2).await.unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].index, 2);
    assert_eq!(entries[1].index, 3);
}

#[tokio::test]
async fn redb_get_entries_for_spine_limit_exceeds_available() {
    let storage = RedbEntryStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let entry = Entry::genesis(owner, spine_id, SpineConfig::default());
    storage.save_entry(&entry).await.unwrap();

    let entries = storage
        .get_entries_for_spine(spine_id, 0, 1000)
        .await
        .unwrap();
    assert_eq!(entries.len(), 1);
}

// Certificate operations, flush, corrupted data/error handling, and large-scale
// tests live in `redb_tests_cert_errors.rs` (domain-focused split).
