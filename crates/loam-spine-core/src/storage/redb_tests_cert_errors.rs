// SPDX-License-Identifier: AGPL-3.0-or-later

//! Redb storage tests: certificate operations, flush behavior, corrupted data
//! handling, persistence across reopens, and large-scale entry storage.
//!
//! Split from `redb_tests.rs` by domain: spine/entry CRUD stays there,
//! certificate domain + error paths + durability live here.

#![expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]

use crate::certificate::{Certificate, CertificateType, MintInfo};
use crate::entry::{Entry, EntryType, SpineConfig};
use crate::spine::Spine;
use crate::storage::{
    CertificateStorage, EntryStorage, RedbCertificateStorage, RedbEntryStorage, RedbSpineStorage,
    RedbStorage, SpineStorage,
};
use crate::types::{CertificateId, Did, SpineId, Timestamp};

fn create_test_spine() -> Spine {
    let owner = Did::new("did:key:z6MkRedbOwner");
    Spine::new(owner, Some("RedbTest".into()), SpineConfig::default())
        .unwrap_or_else(|_| unreachable!())
}

fn create_test_certificate(owner: &Did, spine_id: SpineId) -> Certificate {
    let cert_id = CertificateId::now_v7();
    let mint_info = MintInfo {
        minter: owner.clone(),
        spine: spine_id,
        entry: [0u8; 32],
        timestamp: Timestamp::now(),
        authority: None,
    };

    Certificate::new(
        cert_id,
        CertificateType::DigitalGame {
            platform: "steam".into(),
            game_id: "hl3".into(),
            edition: None,
        },
        owner,
        &mint_info,
    )
}

// ========================================================================
// Certificate CRUD
// ========================================================================

#[tokio::test]
async fn redb_certificate_storage_crud() {
    let storage = RedbCertificateStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkCertOwner");
    let spine_id = SpineId::now_v7();
    let cert = create_test_certificate(&owner, spine_id);
    let cert_id = cert.id;

    assert_eq!(storage.certificate_count(), 0);

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
    assert!(storage.get_certificate(cert_id).await.unwrap().is_none());
}

#[tokio::test]
async fn redb_certificate_storage_get_nonexistent() {
    let storage = RedbCertificateStorage::temporary().unwrap();
    let result = storage
        .get_certificate(CertificateId::now_v7())
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn redb_certificate_storage_list_empty() {
    let storage = RedbCertificateStorage::temporary().unwrap();
    let ids = storage.list_certificates().await.unwrap();
    assert!(ids.is_empty());
}

#[tokio::test]
async fn redb_certificate_storage_delete_nonexistent() {
    let storage = RedbCertificateStorage::temporary().unwrap();
    storage
        .delete_certificate(CertificateId::now_v7())
        .await
        .unwrap();
}

#[tokio::test]
async fn redb_certificate_storage_multiple() {
    let storage = RedbCertificateStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkCertOwner");

    for _ in 0..5 {
        let spine_id = SpineId::now_v7();
        let cert = create_test_certificate(&owner, spine_id);
        storage.save_certificate(&cert, spine_id).await.unwrap();
    }

    assert_eq!(storage.certificate_count(), 5);
    let ids = storage.list_certificates().await.unwrap();
    assert_eq!(ids.len(), 5);
}

#[tokio::test]
async fn redb_certificate_count_on_fresh_db() {
    let storage = RedbCertificateStorage::temporary().unwrap();
    assert_eq!(storage.certificate_count(), 0);
}

// ========================================================================
// Flush behavior across storage types
// ========================================================================

#[tokio::test]
async fn redb_entry_storage_flush() {
    let storage = RedbEntryStorage::temporary().unwrap();
    assert!(storage.flush().is_ok());
}

#[tokio::test]
async fn redb_entry_storage_flush_after_save() {
    let storage = RedbEntryStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let entry = Entry::genesis(owner, SpineId::now_v7(), SpineConfig::default());
    storage.save_entry(&entry).await.unwrap();
    assert!(storage.flush().is_ok());
}

#[tokio::test]
async fn redb_certificate_storage_flush() {
    let storage = RedbCertificateStorage::temporary().unwrap();
    assert!(storage.flush().is_ok());
}

#[tokio::test]
async fn redb_certificate_storage_flush_after_save() {
    let storage = RedbCertificateStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let cert = create_test_certificate(&owner, spine_id);
    storage.save_certificate(&cert, spine_id).await.unwrap();
    assert!(storage.flush().is_ok());
}

#[tokio::test]
async fn redb_combined_flush_all_components() {
    let storage = RedbStorage::temporary().unwrap();
    let spine = create_test_spine();
    storage.spines.save_spine(&spine).await.unwrap();

    let owner = Did::new("did:key:z6MkOwner");
    let entry = Entry::genesis(owner.clone(), spine.id, SpineConfig::default());
    storage.entries.save_entry(&entry).await.unwrap();

    let cert = create_test_certificate(&owner, spine.id);
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
// Corrupted data and error handling
// ========================================================================

#[tokio::test]
async fn redb_get_spine_corrupted_data_returns_error() {
    use redb::{Database, TableDefinition};

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("spines.redb");
    let bad_id = SpineId::now_v7();

    let table_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new("spines");
    {
        let db = Database::create(&path).unwrap();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(table_def).unwrap();
            table
                .insert(
                    bad_id.as_bytes().as_slice(),
                    b"invalid-bincode-data" as &[u8],
                )
                .unwrap();
        }
        write_txn.commit().unwrap();
    }

    let storage = RedbSpineStorage::open(&path).unwrap();
    let result = storage.get_spine(bad_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("deserialize"));
}

#[tokio::test]
async fn redb_get_entry_corrupted_data_returns_error() {
    use redb::{Database, TableDefinition};

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("entries.redb");

    let entries_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new("entries");
    let index_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new("entry_index");
    {
        let db = Database::create(&path).unwrap();
        let write_txn = db.begin_write().unwrap();
        {
            let mut entries = write_txn.open_table(entries_def).unwrap();
            let mut index = write_txn.open_table(index_def).unwrap();
            let bad_hash = [0u8; 32];
            entries
                .insert(bad_hash.as_slice(), b"corrupt" as &[u8])
                .unwrap();
            index
                .insert([0u8; 24].as_slice(), bad_hash.as_slice())
                .unwrap();
        }
        write_txn.commit().unwrap();
    }

    let storage = RedbEntryStorage::open(&path).unwrap();
    let result = storage.get_entry([0u8; 32]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn redb_get_certificate_corrupted_data_returns_error() {
    use redb::{Database, TableDefinition};

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("certs.redb");
    let bad_id = CertificateId::now_v7();

    let certs_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new("certificates");
    {
        let db = Database::create(&path).unwrap();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(certs_def).unwrap();
            table
                .insert(bad_id.as_bytes().as_slice(), b"garbage" as &[u8])
                .unwrap();
        }
        write_txn.commit().unwrap();
    }

    let storage = RedbCertificateStorage::open(&path).unwrap();
    let result = storage.get_certificate(bad_id).await;
    assert!(result.is_err());
}

// ========================================================================
// Persistence and durability
// ========================================================================

#[tokio::test]
async fn redb_list_spines_skips_malformed_keys() {
    let storage = RedbSpineStorage::temporary().unwrap();
    let spine = create_test_spine();
    storage.save_spine(&spine).await.unwrap();

    let ids = storage.list_spines().await.unwrap();
    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&spine.id));
}

#[tokio::test]
async fn redb_list_spines_with_raw_malformed_keys_skips_invalid() {
    use redb::{Database, TableDefinition};

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("spines.redb");
    let table_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new("spines");

    {
        let db = Database::create(&path).unwrap();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(table_def).unwrap();
            let spine = create_test_spine();
            let bytes = bincode::serialize(&spine).unwrap();
            table
                .insert(spine.id.as_bytes().as_slice(), bytes.as_slice())
                .unwrap();
            table.insert(b"short" as &[u8], b"x" as &[u8]).unwrap();
            table.insert([0u8; 24].as_slice(), b"y" as &[u8]).unwrap();
        }
        write_txn.commit().unwrap();
    }

    let storage = RedbSpineStorage::open(&path).unwrap();
    let ids = storage.list_spines().await.unwrap();
    assert_eq!(ids.len(), 1, "should skip malformed keys (len != 16)");
}

#[tokio::test]
async fn redb_entry_index_missing_entry_skipped() {
    use redb::{Database, TableDefinition};

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("entries_missing.redb");

    let entries_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new("entries");
    let index_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new("entry_index");
    let spine_id = SpineId::now_v7();

    {
        let db = Database::create(&path).unwrap();
        let write_txn = db.begin_write().unwrap();
        {
            let mut index = write_txn.open_table(index_def).unwrap();
            let _entries = write_txn.open_table(entries_def).unwrap();
            let mut key = [0u8; 24];
            key[..16].copy_from_slice(spine_id.as_bytes());
            key[16..].copy_from_slice(&0u64.to_be_bytes());
            index.insert(&key[..], &[0u8; 32][..]).unwrap();
        }
        write_txn.commit().unwrap();
    }

    let storage = RedbEntryStorage::open(&path).unwrap();
    let entries = storage
        .get_entries_for_spine(spine_id, 0, 10)
        .await
        .unwrap();
    assert!(
        entries.is_empty(),
        "index points to non-existent entry, should skip"
    );
}

// ========================================================================
// Large-scale operations
// ========================================================================

#[tokio::test]
async fn redb_large_entry_storage() {
    let storage = RedbEntryStorage::temporary().unwrap();
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
