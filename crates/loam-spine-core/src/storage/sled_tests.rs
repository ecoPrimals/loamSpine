// SPDX-License-Identifier: AGPL-3.0-only

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use crate::entry::{Entry, EntryType, SpineConfig};
use crate::spine::Spine;
use crate::storage::{EntryStorage, SledEntryStorage, SledSpineStorage, SledStorage, SpineStorage};
use crate::types::{Did, SpineId};
use serial_test::serial;

fn create_test_spine() -> Spine {
    let owner = Did::new("did:key:z6MkOwner");
    Spine::new(owner, Some("Test".into()), SpineConfig::default())
        .unwrap_or_else(|_| unreachable!())
}

#[tokio::test]
async fn sled_spine_storage_crud() {
    let storage = SledSpineStorage::temporary().unwrap_or_else(|_| unreachable!());

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
async fn sled_entry_storage_crud() {
    let storage = SledEntryStorage::temporary().unwrap_or_else(|_| unreachable!());

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
async fn sled_entry_spine_index() {
    let storage = SledEntryStorage::temporary().unwrap_or_else(|_| unreachable!());

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
async fn sled_combined_storage() {
    let storage = SledStorage::temporary().unwrap_or_else(|_| unreachable!());

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
#[serial]
async fn sled_storage_persistence() {
    let temp_dir = std::env::temp_dir().join(format!("loamspine-test-{}", uuid::Uuid::now_v7()));

    {
        let storage =
            SledStorage::open(&temp_dir).unwrap_or_else(|e| unreachable!("sled open failed: {e}"));
        let spine = create_test_spine();
        storage
            .spines
            .save_spine(&spine)
            .await
            .unwrap_or_else(|e| unreachable!("save spine failed: {e}"));
        storage
            .flush()
            .unwrap_or_else(|e| unreachable!("flush failed: {e}"));
    }

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    {
        let storage = SledStorage::open(&temp_dir)
            .unwrap_or_else(|e| unreachable!("sled reopen failed: {e}"));
        assert_eq!(storage.spines.spine_count(), 1);
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn sled_concurrent_operations() {
    let storage = Arc::new(
        SledStorage::temporary().unwrap_or_else(|e| unreachable!("temporary sled failed: {e}")),
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

fn create_sled_test_spine() -> Spine {
    let owner = Did::new("did:key:z6MkSledOwner");
    Spine::new(owner, Some("SledTest".into()), SpineConfig::default())
        .unwrap_or_else(|_| unreachable!())
}

#[tokio::test]
async fn sled_spine_count_on_fresh_db() {
    let storage = SledSpineStorage::temporary().unwrap();
    assert_eq!(storage.spine_count(), 0);
}

#[tokio::test]
async fn sled_entry_count_on_fresh_db() {
    let storage = SledEntryStorage::temporary().unwrap();
    assert_eq!(storage.entry_count(), 0);
}

#[tokio::test]
async fn sled_get_nonexistent_spine() {
    let storage = SledSpineStorage::temporary().unwrap();
    let result = storage.get_spine(SpineId::now_v7()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn sled_get_nonexistent_entry() {
    let storage = SledEntryStorage::temporary().unwrap();
    let result = storage.get_entry([0u8; 32]).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn sled_list_spines_empty() {
    let storage = SledSpineStorage::temporary().unwrap();
    let ids = storage.list_spines().await.unwrap();
    assert!(ids.is_empty());
}

#[tokio::test]
async fn sled_get_entries_for_spine_empty() {
    let storage = SledEntryStorage::temporary().unwrap();
    let entries = storage
        .get_entries_for_spine(SpineId::now_v7(), 0, 100)
        .await
        .unwrap();
    assert!(entries.is_empty());
}

#[tokio::test]
async fn sled_save_and_retrieve_multiple_entries() {
    let spine_storage = SledSpineStorage::temporary().unwrap();
    let entry_storage = SledEntryStorage::temporary().unwrap();

    let spine = create_sled_test_spine();
    let owner = Did::new("did:key:z6MkSledOwner");
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
async fn sled_flush_succeeds() {
    let storage = SledSpineStorage::temporary().unwrap();
    assert!(storage.flush().is_ok());
}

#[tokio::test]
async fn sled_entry_exists_false_for_missing() {
    let storage = SledEntryStorage::temporary().unwrap();
    let exists = storage.entry_exists([42u8; 32]).await.unwrap();
    assert!(!exists);
}

#[tokio::test]
async fn sled_spine_delete() {
    let storage = SledSpineStorage::temporary().unwrap();
    let spine = create_sled_test_spine();
    storage.save_spine(&spine).await.unwrap();
    assert_eq!(storage.spine_count(), 1);
    let _ = storage.delete_spine(spine.id).await;
    assert_eq!(storage.spine_count(), 0);
}
