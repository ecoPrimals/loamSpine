// SPDX-License-Identifier: AGPL-3.0-or-later

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap for failure clarity"
)]

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
    let temp_dir = tempfile::tempdir().unwrap();

    {
        let storage = SledStorage::open(temp_dir.path())
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

    {
        let storage = SledStorage::open(temp_dir.path())
            .unwrap_or_else(|e| unreachable!("sled reopen failed: {e}"));
        assert_eq!(storage.spines.spine_count(), 1);
    }
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

// ========================================================================
// Corrupted data, edge cases, certificate storage (similar to redb)
// ========================================================================

#[tokio::test]
#[serial]
async fn sled_get_spine_corrupted_data_returns_error() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("spines");

    let db = sled::open(&path).unwrap();
    let tree = db.open_tree("spines").unwrap();
    let bad_id = SpineId::now_v7();
    tree.insert(bad_id.as_bytes(), b"invalid-bincode").unwrap();
    db.flush().unwrap();
    drop(tree);
    drop(db);

    let storage = SledSpineStorage::open(&path).unwrap();
    let result = storage.get_spine(bad_id).await;
    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn sled_get_entry_corrupted_data_returns_error() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("entries");

    let db = sled::open(&path).unwrap();
    let entries = db.open_tree("entries").unwrap();
    let index = db.open_tree("entry_index").unwrap();
    let bad_hash = [0u8; 32];
    entries.insert(&bad_hash[..], b"corrupt").unwrap();
    index.insert(&[0u8; 24][..], &bad_hash[..]).unwrap();
    db.flush().unwrap();
    drop(entries);
    drop(index);
    drop(db);

    let storage = SledEntryStorage::open(&path).unwrap();
    let result = storage.get_entry(bad_hash).await;
    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn sled_get_certificate_corrupted_data_returns_error() {
    use crate::storage::{CertificateStorage, SledCertificateStorage};
    use crate::types::CertificateId;

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("certs");

    let db = sled::open(&path).unwrap();
    let tree = db.open_tree("certificates").unwrap();
    let bad_id = CertificateId::now_v7();
    tree.insert(bad_id.as_bytes(), b"garbage").unwrap();
    db.flush().unwrap();
    drop(tree);
    drop(db);

    let storage = SledCertificateStorage::open(&path).unwrap();
    let result = storage.get_certificate(bad_id).await;
    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn sled_storage_open_with_base_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let storage = SledStorage::open(temp_dir.path()).unwrap();
    assert_eq!(storage.spines.spine_count(), 0);
    assert_eq!(storage.entries.entry_count(), 0);
    assert_eq!(storage.certificates.certificate_count(), 0);
}

#[tokio::test]
async fn sled_certificate_storage_crud() {
    use crate::certificate::{Certificate, CertificateType, MintInfo};
    use crate::storage::{CertificateStorage, SledCertificateStorage};
    use crate::types::{CertificateId, Timestamp};

    let storage = SledCertificateStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkCertOwner");
    let spine_id = SpineId::now_v7();

    let cert_id = CertificateId::now_v7();
    let mint_info = MintInfo {
        minter: owner.clone(),
        spine: spine_id,
        entry: [0u8; 32],
        timestamp: Timestamp::now(),
        authority: None,
    };
    let cert = Certificate::new(
        cert_id,
        CertificateType::DigitalGame {
            platform: "steam".into(),
            game_id: "test".into(),
            edition: None,
        },
        &owner,
        &mint_info,
    );

    storage.save_certificate(&cert, spine_id).await.unwrap();
    assert_eq!(storage.certificate_count(), 1);

    let retrieved = storage.get_certificate(cert_id).await.unwrap();
    assert!(retrieved.is_some());
    let (retrieved_cert, retrieved_spine) = retrieved.unwrap();
    assert_eq!(retrieved_cert.id, cert_id);
    assert_eq!(retrieved_spine, spine_id);

    let ids = storage.list_certificates().await.unwrap();
    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&cert_id));

    storage.delete_certificate(cert_id).await.unwrap();
    assert_eq!(storage.certificate_count(), 0);
}

#[tokio::test]
async fn sled_entry_storage_get_entries_limit_zero() {
    let storage = SledEntryStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let entry = Entry::genesis(owner, spine_id, SpineConfig::default());
    storage.save_entry(&entry).await.unwrap();

    let entries = storage.get_entries_for_spine(spine_id, 0, 0).await.unwrap();
    assert!(entries.is_empty());
}

#[tokio::test]
async fn sled_large_entry_storage() {
    let storage = SledEntryStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();

    let mut prev_hash = None;
    for i in 0..50 {
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

    assert_eq!(storage.entry_count(), 50);
    let entries = storage
        .get_entries_for_spine(spine_id, 0, 100)
        .await
        .unwrap();
    assert_eq!(entries.len(), 50);
}

#[tokio::test]
async fn sled_combined_flush_all_components() {
    use crate::certificate::{Certificate, CertificateType, MintInfo};
    use crate::storage::CertificateStorage;
    use crate::types::{CertificateId, Timestamp};

    let storage = SledStorage::temporary().unwrap();
    let spine = create_sled_test_spine();
    storage.spines.save_spine(&spine).await.unwrap();

    let owner = Did::new("did:key:z6MkOwner");
    let entry = Entry::genesis(owner.clone(), spine.id, SpineConfig::default());
    storage.entries.save_entry(&entry).await.unwrap();

    let cert_id = CertificateId::now_v7();
    let mint_info = MintInfo {
        minter: owner.clone(),
        spine: spine.id,
        entry: [0u8; 32],
        timestamp: Timestamp::now(),
        authority: None,
    };
    let cert = Certificate::new(
        cert_id,
        CertificateType::DigitalGame {
            platform: "steam".into(),
            game_id: "test".into(),
            edition: None,
        },
        &owner,
        &mint_info,
    );
    storage
        .certificates
        .save_certificate(&cert, spine.id)
        .await
        .unwrap();

    assert!(storage.flush().is_ok());
    assert_eq!(storage.spines.spine_count(), 1);
    assert_eq!(storage.entries.entry_count(), 1);
    assert_eq!(storage.certificates.certificate_count(), 1);
}

// ========================================================================
// Additional coverage: certificate list/delete/get edge cases
// ========================================================================

#[tokio::test]
async fn sled_certificate_list_empty() {
    use crate::storage::{CertificateStorage, SledCertificateStorage};

    let storage = SledCertificateStorage::temporary().unwrap();
    let ids = storage.list_certificates().await.unwrap();
    assert!(ids.is_empty());
}

#[tokio::test]
async fn sled_certificate_get_nonexistent() {
    use crate::storage::{CertificateStorage, SledCertificateStorage};
    use crate::types::CertificateId;

    let storage = SledCertificateStorage::temporary().unwrap();
    let result = storage
        .get_certificate(CertificateId::now_v7())
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn sled_certificate_delete_nonexistent() {
    use crate::storage::{CertificateStorage, SledCertificateStorage};
    use crate::types::CertificateId;

    let storage = SledCertificateStorage::temporary().unwrap();
    storage
        .delete_certificate(CertificateId::now_v7())
        .await
        .unwrap();
}

#[tokio::test]
async fn sled_certificate_count_on_fresh_db() {
    use crate::storage::SledCertificateStorage;

    let storage = SledCertificateStorage::temporary().unwrap();
    assert_eq!(storage.certificate_count(), 0);
}

#[tokio::test]
async fn sled_entry_storage_get_entries_with_offset() {
    let storage = SledEntryStorage::temporary().unwrap();
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
async fn sled_list_spines_multiple() {
    let storage = SledSpineStorage::temporary().unwrap();

    for i in 0..5 {
        let owner = Did::new(format!("did:key:z6MkOwner{i}"));
        let spine = Spine::new(owner, Some(format!("Spine {i}")), SpineConfig::default()).unwrap();
        storage.save_spine(&spine).await.unwrap();
    }

    let ids = storage.list_spines().await.unwrap();
    assert_eq!(ids.len(), 5);
}

#[tokio::test]
async fn sled_entry_storage_flush_after_save() {
    let storage = SledEntryStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let entry = Entry::genesis(owner, SpineId::now_v7(), SpineConfig::default());
    storage.save_entry(&entry).await.unwrap();
    assert!(storage.flush().is_ok());
}

#[tokio::test]
async fn sled_get_entries_for_spine_offset_beyond_data() {
    let storage = SledEntryStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let entry = Entry::genesis(owner, spine_id, SpineConfig::default());
    storage.save_entry(&entry).await.unwrap();

    let entries = storage
        .get_entries_for_spine(spine_id, 10, 5)
        .await
        .unwrap();
    assert!(entries.is_empty());
}

#[tokio::test]
async fn sled_get_entries_for_spine_limit_exceeds_available() {
    let storage = SledEntryStorage::temporary().unwrap();
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

#[tokio::test]
#[serial]
async fn sled_list_spines_with_malformed_keys_skips_invalid() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("spines-malformed");

    {
        let db = sled::open(&path).unwrap();
        let tree = db.open_tree("spines").unwrap();

        let valid_id = SpineId::now_v7();
        let valid_spine = create_test_spine();
        let bytes = bincode::serialize(&valid_spine).unwrap();
        tree.insert(valid_id.as_bytes(), bytes).unwrap();

        tree.insert(b"short", b"val").unwrap();
        tree.insert(b"this-key-is-too-long-for-uuid", b"val")
            .unwrap();
        db.flush().unwrap();
    }

    let storage = SledSpineStorage::open(&path).unwrap();
    let ids = storage.list_spines().await.unwrap();
    assert_eq!(ids.len(), 1, "should skip malformed keys (len != 16)");
}

#[tokio::test]
#[serial]
async fn sled_list_certificates_with_malformed_keys_skips_invalid() {
    use crate::certificate::{Certificate, CertificateType, MintInfo};
    use crate::storage::{CertificateStorage, SledCertificateStorage};
    use crate::types::{CertificateId, Timestamp};

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("certs-malformed");

    let cert_id = CertificateId::now_v7();
    {
        let db = sled::open(&path).unwrap();
        let tree = db.open_tree("certificates").unwrap();

        let owner = Did::new("did:key:z6MkOwner");
        let spine_id = SpineId::now_v7();
        let mint_info = MintInfo {
            minter: owner.clone(),
            spine: spine_id,
            entry: [0u8; 32],
            timestamp: Timestamp::now(),
            authority: None,
        };
        let cert = Certificate::new(
            cert_id,
            CertificateType::DigitalGame {
                platform: "steam".into(),
                game_id: "test".into(),
                edition: None,
            },
            &owner,
            &mint_info,
        );
        let bytes = bincode::serialize(&(&cert, spine_id)).unwrap();
        tree.insert(cert_id.as_bytes().as_slice(), bytes).unwrap();

        tree.insert(b"short", b"val").unwrap();
        tree.insert(b"this-key-is-too-long-for-uuid", b"val")
            .unwrap();
        db.flush().unwrap();
    }

    let storage = SledCertificateStorage::open(&path).unwrap();
    let ids = storage.list_certificates().await.unwrap();
    assert_eq!(ids.len(), 1, "should skip malformed keys (len != 16)");
}

#[tokio::test]
#[serial]
async fn sled_entry_index_missing_entry_skipped() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("entries-orphan");

    let spine_id = SpineId::now_v7();
    {
        let db = sled::open(&path).unwrap();
        let entries = db.open_tree("entries").unwrap();
        let index = db.open_tree("entry_index").unwrap();

        let owner = Did::new("did:key:z6MkOwner");
        let entry = Entry::genesis(owner, spine_id, SpineConfig::default());
        let hash = entry.compute_hash().unwrap();
        let bytes = bincode::serialize(&entry).unwrap();
        entries.insert(&hash[..], bytes).unwrap();

        let mut key = [0u8; 24];
        key[..16].copy_from_slice(spine_id.as_bytes());
        key[16..].copy_from_slice(&0u64.to_be_bytes());
        index.insert(&key[..], &hash[..]).unwrap();

        // Orphan: index points to non-existent hash
        let orphan_hash = [0xFFu8; 32];
        let mut orphan_key = [0u8; 24];
        orphan_key[..16].copy_from_slice(spine_id.as_bytes());
        orphan_key[16..].copy_from_slice(&99u64.to_be_bytes());
        index.insert(&orphan_key[..], &orphan_hash[..]).unwrap();

        db.flush().unwrap();
    }

    let storage = SledEntryStorage::open(&path).unwrap();
    let entries = storage
        .get_entries_for_spine(spine_id, 0, 100)
        .await
        .unwrap();
    assert_eq!(entries.len(), 1, "orphan index entry should be skipped");
}

#[tokio::test]
#[serial]
async fn sled_get_entries_for_spine_corrupted_entry_in_index() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("entries-corrupt");

    let spine_id = SpineId::now_v7();
    {
        let db = sled::open(&path).unwrap();
        let entries = db.open_tree("entries").unwrap();
        let index = db.open_tree("entry_index").unwrap();

        let corrupt_hash = [0xABu8; 32];
        let mut index_key = [0u8; 24];
        index_key[..16].copy_from_slice(spine_id.as_bytes());
        index_key[16..].copy_from_slice(&0u64.to_be_bytes());
        index.insert(&index_key[..], &corrupt_hash[..]).unwrap();
        entries.insert(&corrupt_hash[..], b"corrupt").unwrap();

        db.flush().unwrap();
    }

    let storage = SledEntryStorage::open(&path).unwrap();
    let result = storage.get_entries_for_spine(spine_id, 0, 10).await;
    assert!(result.is_err(), "corrupted entry bytes should return error");
    assert!(
        result.unwrap_err().to_string().contains("deserialize"),
        "error should mention deserialization"
    );
}
