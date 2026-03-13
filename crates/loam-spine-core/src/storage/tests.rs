// SPDX-License-Identifier: AGPL-3.0-only

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
#[allow(clippy::module_inception)]
mod tests {
    use std::sync::Arc;

    use crate::entry::{Entry, EntryType, SpineConfig};
    use crate::spine::Spine;
    use crate::storage::{
        EntryStorage, InMemoryEntryStorage, InMemorySpineStorage, SledEntryStorage,
        SledSpineStorage, SledStorage, SpineStorage,
    };
    use crate::types::{Did, SpineId};
    use serial_test::serial;

    fn create_test_spine() -> Spine {
        let owner = Did::new("did:key:z6MkOwner");
        Spine::new(owner, Some("Test".into()), SpineConfig::default())
            .unwrap_or_else(|_| unreachable!())
    }

    #[tokio::test]
    async fn spine_storage_crud() {
        let storage = InMemorySpineStorage::new();

        let spine = create_test_spine();
        let id = spine.id;

        // Save
        storage
            .save_spine(&spine)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(storage.spine_count().await, 1);

        // Get
        let retrieved = storage
            .get_spine(id)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap_or_else(|| unreachable!()).id, id);

        // List
        let ids = storage
            .list_spines()
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&id));

        // Delete
        storage
            .delete_spine(id)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(storage.spine_count().await, 0);

        // Get after delete
        let retrieved = storage
            .get_spine(id)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn entry_storage_crud() {
        let storage = InMemoryEntryStorage::new();

        let owner = Did::new("did:key:z6MkOwner");
        let spine_id = SpineId::now_v7();
        let entry = Entry::genesis(owner, spine_id, SpineConfig::default());

        // Save
        let hash = storage
            .save_entry(&entry)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(storage.entry_count().await, 1);

        // Get
        let retrieved = storage
            .get_entry(hash)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(retrieved.is_some());

        // Exists
        let exists = storage
            .entry_exists(hash)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(exists);

        // Not exists
        let not_exists = storage
            .entry_exists([0u8; 32])
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(!not_exists);
    }

    #[tokio::test]
    async fn entry_spine_index() {
        let storage = InMemoryEntryStorage::new();

        let owner = Did::new("did:key:z6MkOwner");
        let spine_id = SpineId::now_v7();

        // Add 5 entries
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

        // Get all
        let entries = storage
            .get_entries_for_spine(spine_id, 0, 10)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(entries.len(), 5);

        // Get range
        let entries = storage
            .get_entries_for_spine(spine_id, 1, 2)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(entries.len(), 2);

        // Get from non-existent spine
        let entries = storage
            .get_entries_for_spine(SpineId::now_v7(), 0, 10)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(entries.is_empty());
    }

    // ========================================================================
    // Sled Storage Tests
    // ========================================================================

    #[tokio::test]
    async fn sled_spine_storage_crud() {
        let storage = SledSpineStorage::temporary().unwrap_or_else(|_| unreachable!());

        let spine = create_test_spine();
        let id = spine.id;

        // Save
        storage
            .save_spine(&spine)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(storage.spine_count(), 1);

        // Get
        let retrieved = storage
            .get_spine(id)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap_or_else(|| unreachable!()).id, id);

        // List
        let ids = storage
            .list_spines()
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&id));

        // Delete
        storage
            .delete_spine(id)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(storage.spine_count(), 0);

        // Get after delete
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

        // Save
        let hash = storage
            .save_entry(&entry)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(storage.entry_count(), 1);

        // Get
        let retrieved = storage
            .get_entry(hash)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(retrieved.is_some());

        // Exists
        let exists = storage
            .entry_exists(hash)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(exists);

        // Not exists
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

        // Add 5 entries
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

        // Get all
        let entries = storage
            .get_entries_for_spine(spine_id, 0, 10)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(entries.len(), 5);

        // Get range
        let entries = storage
            .get_entries_for_spine(spine_id, 1, 2)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(entries.len(), 2);

        // Get from non-existent spine
        let entries = storage
            .get_entries_for_spine(SpineId::now_v7(), 0, 10)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn sled_combined_storage() {
        let storage = SledStorage::temporary().unwrap_or_else(|_| unreachable!());

        // Save a spine
        let spine = create_test_spine();
        storage
            .spines
            .save_spine(&spine)
            .await
            .unwrap_or_else(|_| unreachable!());

        // Save an entry
        let owner = Did::new("did:key:z6MkOwner");
        let entry = Entry::genesis(owner, spine.id, SpineConfig::default());
        let hash = storage
            .entries
            .save_entry(&entry)
            .await
            .unwrap_or_else(|_| unreachable!());

        // Flush
        storage.flush().unwrap_or_else(|_| unreachable!());

        // Verify both are stored
        assert_eq!(storage.spines.spine_count(), 1);
        assert_eq!(storage.entries.entry_count(), 1);

        // Retrieve both
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
    async fn concurrent_spine_save() {
        let storage = Arc::new(InMemorySpineStorage::new());

        // Spawn multiple tasks saving spines concurrently
        let mut handles = vec![];
        for i in 0..20 {
            let stor = Arc::clone(&storage);
            let handle = tokio::spawn(async move {
                let owner = Did::new(format!("did:key:z6MkOwner{i}"));
                let spine = Spine::new(owner, Some(format!("Spine {i}")), SpineConfig::default())
                    .unwrap_or_else(|_| unreachable!());
                stor.save_spine(&spine).await
            });
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            assert!(handle.await.unwrap_or_else(|_| unreachable!()).is_ok());
        }

        assert_eq!(storage.spine_count().await, 20);
    }

    #[tokio::test]
    async fn concurrent_entry_save() {
        let storage = Arc::new(InMemoryEntryStorage::new());
        let spine_id = SpineId::now_v7();

        // Spawn multiple tasks saving entries concurrently
        let mut handles = vec![];
        for i in 0..20 {
            let stor = Arc::clone(&storage);
            let handle = tokio::spawn(async move {
                let owner = Did::new("did:key:z6MkOwner");
                let entry = Entry::new(i, None, owner, EntryType::SpineSealed { reason: None })
                    .with_spine_id(spine_id);
                stor.save_entry(&entry).await
            });
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            assert!(handle.await.unwrap_or_else(|_| unreachable!()).is_ok());
        }

        assert_eq!(storage.entry_count().await, 20);
    }

    #[tokio::test]
    async fn large_entry_dataset() {
        let storage = InMemoryEntryStorage::new();
        let spine_id = SpineId::now_v7();
        let owner = Did::new("did:key:z6MkOwner");

        // Save 1000 entries
        for i in 0..1000 {
            let entry = Entry::new(
                i,
                None,
                owner.clone(),
                EntryType::SpineSealed { reason: None },
            )
            .with_spine_id(spine_id);
            storage
                .save_entry(&entry)
                .await
                .unwrap_or_else(|_| unreachable!());
        }

        assert_eq!(storage.entry_count().await, 1000);

        // Query large range
        let entries = storage
            .get_entries_for_spine(spine_id, 0, 1000)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(entries.len(), 1000);
    }

    #[tokio::test]
    async fn entry_range_edge_cases() {
        let storage = InMemoryEntryStorage::new();
        let spine_id = SpineId::now_v7();
        let owner = Did::new("did:key:z6MkOwner");

        // Save 10 entries
        for i in 0..10 {
            let entry = Entry::new(
                i,
                None,
                owner.clone(),
                EntryType::SpineSealed { reason: None },
            )
            .with_spine_id(spine_id);
            storage
                .save_entry(&entry)
                .await
                .unwrap_or_else(|_| unreachable!());
        }

        // Query with offset > count (should return empty)
        let entries = storage
            .get_entries_for_spine(spine_id, 20, 10)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(entries.is_empty());

        // Query with limit = 0 (should return empty)
        let entries = storage
            .get_entries_for_spine(spine_id, 0, 0)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(entries.is_empty());

        // Query with very large limit (should return all)
        let entries = storage
            .get_entries_for_spine(spine_id, 0, 10000)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(entries.len(), 10);

        // Query partial range
        let entries = storage
            .get_entries_for_spine(spine_id, 5, 3)
            .await
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].index, 5);
        assert_eq!(entries[2].index, 7);
    }

    #[tokio::test]
    #[serial]
    async fn sled_storage_persistence() {
        let temp_dir =
            std::env::temp_dir().join(format!("loamspine-test-{}", uuid::Uuid::now_v7()));

        // Create storage, save data, then explicitly drop to release sled lock
        {
            let storage = SledStorage::open(&temp_dir)
                .unwrap_or_else(|e| unreachable!("sled open failed: {e}"));
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

        // Yield to allow sled's background threads to finalize lock release
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Reopen storage and verify data persists
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

        // Concurrent spine saves
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

    #[tokio::test]
    async fn storage_delete_nonexistent() {
        let storage = InMemorySpineStorage::new();

        // Delete nonexistent spine should succeed (idempotent)
        let result = storage.delete_spine(SpineId::now_v7()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn storage_update_spine() {
        let storage = InMemorySpineStorage::new();

        let mut spine = create_test_spine();
        let id = spine.id;

        // Save initial
        storage
            .save_spine(&spine)
            .await
            .unwrap_or_else(|_| unreachable!());

        // Update spine
        spine.height += 10;
        storage
            .save_spine(&spine)
            .await
            .unwrap_or_else(|_| unreachable!());

        // Verify update
        let retrieved = storage
            .get_spine(id)
            .await
            .unwrap_or_else(|_| unreachable!())
            .unwrap_or_else(|| unreachable!());
        assert_eq!(retrieved.height, spine.height);
    }
}
