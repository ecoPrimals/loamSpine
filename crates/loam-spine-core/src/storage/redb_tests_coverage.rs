// SPDX-License-Identifier: AGPL-3.0-or-later

#![expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]

use crate::entry::{Entry, SpineConfig};
use crate::storage::{EntryStorage, RedbEntryStorage};
use crate::types::{CertificateId, Did, SpineId};
use serial_test::serial;

fn create_test_certificate(owner: &Did, spine_id: SpineId) -> crate::certificate::Certificate {
    use crate::certificate::{Certificate, CertificateType, MintInfo};
    use crate::types::Timestamp;

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

#[tokio::test]
#[serial]
async fn redb_list_certificates_with_malformed_key_skips_invalid() {
    use crate::storage::{CertificateStorage, RedbCertificateStorage};
    use redb::{Database, TableDefinition};

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("certs.redb");
    let certs_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new("certificates");

    {
        let db = Database::create(&path).unwrap();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(certs_def).unwrap();
            let valid_id = CertificateId::now_v7();
            let cert = create_test_certificate(&Did::new("did:key:z6MkOwner"), SpineId::now_v7());
            let bytes = bincode::serialize(&(cert, SpineId::now_v7())).unwrap();
            table
                .insert(valid_id.as_bytes().as_slice(), bytes.as_slice())
                .unwrap();
            table.insert(b"short_key" as &[u8], b"x" as &[u8]).unwrap();
            table.insert([0u8; 24].as_slice(), b"y" as &[u8]).unwrap();
        }
        write_txn.commit().unwrap();
    }

    let storage = RedbCertificateStorage::open(&path).unwrap();
    let ids = storage.list_certificates().await.unwrap();
    assert_eq!(
        ids.len(),
        1,
        "should skip malformed keys and return only valid 16-byte ids"
    );
}

#[tokio::test]
async fn redb_get_entries_for_spine_with_offset_beyond_data() {
    let storage = RedbEntryStorage::temporary().unwrap();
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
async fn redb_certificate_save_overwrite() {
    use crate::storage::{CertificateStorage, RedbCertificateStorage};

    let storage = RedbCertificateStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkCertOwner");
    let spine_id = SpineId::now_v7();
    let cert = create_test_certificate(&owner, spine_id);
    let cert_id = cert.id;

    storage.save_certificate(&cert, spine_id).await.unwrap();
    storage.save_certificate(&cert, spine_id).await.unwrap();
    assert_eq!(storage.certificate_count(), 1);

    let retrieved = storage.get_certificate(cert_id).await.unwrap();
    assert!(retrieved.is_some());
}

// ========================================================================
// get_entries_for_spine with corrupt data in entry table (via index)
// ========================================================================

#[tokio::test]
#[serial]
async fn redb_get_entries_for_spine_corrupted_entry_in_entries_table() {
    use redb::{Database, TableDefinition};

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("entries_corrupt_via_index.redb");

    let entries_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new("entries");
    let index_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new("entry_index");
    let spine_id = SpineId::now_v7();

    {
        let db = Database::create(&path).unwrap();
        let write_txn = db.begin_write().unwrap();
        {
            let mut entries = write_txn.open_table(entries_def).unwrap();
            let mut index = write_txn.open_table(index_def).unwrap();
            let corrupt_hash = [0xCDu8; 32];
            let mut key = [0u8; 24];
            key[..16].copy_from_slice(spine_id.as_bytes());
            key[16..].copy_from_slice(&0u64.to_be_bytes());
            index.insert(&key[..], &corrupt_hash[..]).unwrap();
            entries
                .insert(&corrupt_hash[..], b"not-bincode" as &[u8])
                .unwrap();
        }
        write_txn.commit().unwrap();
    }

    let storage = RedbEntryStorage::open(&path).unwrap();
    let result = storage.get_entries_for_spine(spine_id, 0, 10).await;
    assert!(result.is_err(), "corrupt entry data via index should error");
    assert!(result.unwrap_err().to_string().contains("deserialize"));
}

// ========================================================================
// get_entries_for_spine with short index key (key.len() < 16)
// ========================================================================

#[tokio::test]
#[serial]
async fn redb_get_entries_for_spine_short_index_key_terminates() {
    use redb::{Database, TableDefinition};

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("entries_short_key.redb");

    let entries_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new("entries");
    let index_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new("entry_index");
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();

    {
        let db = Database::create(&path).unwrap();
        let write_txn = db.begin_write().unwrap();
        {
            let mut entries_table = write_txn.open_table(entries_def).unwrap();
            let mut index = write_txn.open_table(index_def).unwrap();

            let mut key = [0u8; 24];
            key[..16].copy_from_slice(spine_id.as_bytes());
            key[16..].copy_from_slice(&0u64.to_be_bytes());

            let entry = Entry::genesis(owner, spine_id, SpineConfig::default());
            let hash = entry.compute_hash().unwrap();
            let bytes = bincode::serialize(&entry).unwrap();

            entries_table.insert(&hash[..], bytes.as_slice()).unwrap();
            index.insert(&key[..], &hash[..]).unwrap();
        }
        write_txn.commit().unwrap();
    }

    let storage = RedbEntryStorage::open(&path).unwrap();
    let entries = storage
        .get_entries_for_spine(spine_id, 0, 10)
        .await
        .unwrap();
    assert_eq!(entries.len(), 1);
}
