// SPDX-License-Identifier: AGPL-3.0-or-later

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap for failure clarity"
)]

use crate::certificate::{Certificate, CertificateType, MintInfo};
use crate::entry::{Entry, EntryType, SpineConfig};
use crate::spine::Spine;
use crate::storage::{
    CertificateStorage, EntryStorage, SpineStorage, SqliteCertificateStorage, SqliteEntryStorage,
    SqliteSpineStorage, SqliteStorage,
};
use crate::types::{CertificateId, Did, SpineId, Timestamp};
use tempfile::TempDir;

fn create_test_spine() -> Spine {
    let owner = Did::new("did:key:z6MkOwner");
    Spine::new(owner, Some("Test".into()), SpineConfig::default())
        .unwrap_or_else(|_| unreachable!())
}

fn create_test_entry(owner: &Did, spine_id: SpineId) -> Entry {
    Entry::genesis(owner.clone(), spine_id, SpineConfig::default())
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

fn spine_storage_from_tempdir() -> (TempDir, SqliteSpineStorage) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("spines.db");
    let storage = SqliteSpineStorage::open(&db_path).unwrap();
    (temp_dir, storage)
}

fn entry_storage_from_tempdir() -> (TempDir, SqliteEntryStorage) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("entries.db");
    let storage = SqliteEntryStorage::open(&db_path).unwrap();
    (temp_dir, storage)
}

fn certificate_storage_from_tempdir() -> (TempDir, SqliteCertificateStorage) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("certs.db");
    let storage = SqliteCertificateStorage::open(&db_path).unwrap();
    (temp_dir, storage)
}

#[tokio::test]
async fn sqlite_spine_storage_crud() {
    let (_temp_dir, storage) = spine_storage_from_tempdir();
    let spine = create_test_spine();
    let id = spine.id;

    storage.save_spine(&spine).await.unwrap();
    assert_eq!(storage.spine_count(), 1);

    let retrieved = storage.get_spine(id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, id);

    let mut updated_spine = spine;
    updated_spine.height = 42;
    storage.save_spine(&updated_spine).await.unwrap();
    let retrieved = storage.get_spine(id).await.unwrap().unwrap();
    assert_eq!(retrieved.height, 42);

    storage.delete_spine(id).await.unwrap();
    assert_eq!(storage.spine_count(), 0);
    assert!(storage.get_spine(id).await.unwrap().is_none());
}

#[tokio::test]
async fn sqlite_spine_storage_list_empty() {
    let (_temp_dir, storage) = spine_storage_from_tempdir();
    let ids = storage.list_spines().await.unwrap();
    assert!(ids.is_empty());
}

#[tokio::test]
async fn sqlite_spine_storage_list_populated() {
    let (_temp_dir, storage) = spine_storage_from_tempdir();
    let spine1 = create_test_spine();
    let spine2 = Spine::new(
        Did::new("did:key:z6MkOther"),
        Some("Other".into()),
        SpineConfig::default(),
    )
    .unwrap();
    storage.save_spine(&spine1).await.unwrap();
    storage.save_spine(&spine2).await.unwrap();

    let ids = storage.list_spines().await.unwrap();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&spine1.id));
    assert!(ids.contains(&spine2.id));
}

#[tokio::test]
async fn sqlite_spine_storage_get_nonexistent() {
    let (_temp_dir, storage) = spine_storage_from_tempdir();
    let result = storage.get_spine(SpineId::now_v7()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn sqlite_spine_storage_delete_nonexistent_idempotent() {
    let (_temp_dir, storage) = spine_storage_from_tempdir();
    let result = storage.delete_spine(SpineId::now_v7()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn sqlite_spine_storage_flush() {
    let (_temp_dir, storage) = spine_storage_from_tempdir();
    storage.save_spine(&create_test_spine()).await.unwrap();
    let result = storage.flush();
    assert!(result.is_ok());
}

#[tokio::test]
async fn sqlite_entry_storage_save_and_retrieve() {
    let (_temp_dir, storage) = entry_storage_from_tempdir();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let entry = create_test_entry(&owner, spine_id);

    let hash = storage.save_entry(&entry).await.unwrap();
    assert_eq!(storage.entry_count(), 1);

    let retrieved = storage.get_entry(hash).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().spine_id, spine_id);
}

#[tokio::test]
async fn sqlite_entry_storage_entry_exists() {
    let (_temp_dir, storage) = entry_storage_from_tempdir();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let entry = create_test_entry(&owner, spine_id);

    let hash = storage.save_entry(&entry).await.unwrap();
    assert!(storage.entry_exists(hash).await.unwrap());

    assert!(!storage.entry_exists([0u8; 32]).await.unwrap());
}

#[tokio::test]
async fn sqlite_entry_storage_get_entries_for_spine() {
    let (_temp_dir, storage) = entry_storage_from_tempdir();
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

    let entries = storage
        .get_entries_for_spine(spine_id, 0, 10)
        .await
        .unwrap();
    assert_eq!(entries.len(), 5);

    let entries = storage.get_entries_for_spine(spine_id, 1, 2).await.unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].index, 1);
    assert_eq!(entries[1].index, 2);
}

#[tokio::test]
async fn sqlite_entry_storage_empty_spine_returns_empty() {
    let (_temp_dir, storage) = entry_storage_from_tempdir();
    let entries = storage
        .get_entries_for_spine(SpineId::now_v7(), 0, 10)
        .await
        .unwrap();
    assert!(entries.is_empty());
}

#[tokio::test]
async fn sqlite_entry_storage_flush() {
    let (_temp_dir, storage) = entry_storage_from_tempdir();
    let owner = Did::new("did:key:z6MkOwner");
    let entry = create_test_entry(&owner, SpineId::now_v7());
    storage.save_entry(&entry).await.unwrap();
    let result = storage.flush();
    assert!(result.is_ok());
}

#[tokio::test]
async fn sqlite_certificate_storage_crud() {
    let (_temp_dir, storage) = certificate_storage_from_tempdir();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let cert = create_test_certificate(&owner, spine_id);
    let cert_id = cert.id;

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
async fn sqlite_certificate_storage_list() {
    let (_temp_dir, storage) = certificate_storage_from_tempdir();
    let owner = Did::new("did:key:z6MkOwner");

    for _ in 0..3 {
        let spine_id = SpineId::now_v7();
        let cert = create_test_certificate(&owner, spine_id);
        storage.save_certificate(&cert, spine_id).await.unwrap();
    }

    let ids = storage.list_certificates().await.unwrap();
    assert_eq!(ids.len(), 3);
}

#[tokio::test]
async fn sqlite_certificate_storage_get_nonexistent() {
    let (_temp_dir, storage) = certificate_storage_from_tempdir();
    let result = storage
        .get_certificate(CertificateId::now_v7())
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn sqlite_certificate_storage_delete_nonexistent_idempotent() {
    let (_temp_dir, storage) = certificate_storage_from_tempdir();
    let result = storage.delete_certificate(CertificateId::now_v7()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn sqlite_certificate_storage_flush() {
    let (_temp_dir, storage) = certificate_storage_from_tempdir();
    let owner = Did::new("did:key:z6MkOwner");
    let cert = create_test_certificate(&owner, SpineId::now_v7());
    storage
        .save_certificate(&cert, cert.mint_info.spine)
        .await
        .unwrap();
    let result = storage.flush();
    assert!(result.is_ok());
}

// --- SqliteStorage combined tests ---

#[tokio::test]
async fn sqlite_storage_temporary_creates_valid_storage() {
    let storage = SqliteStorage::temporary().unwrap();
    assert_eq!(storage.spines.spine_count(), 0);
    assert_eq!(storage.entries.entry_count(), 0);
    assert_eq!(storage.certificates.certificate_count(), 0);
}

#[tokio::test]
async fn sqlite_storage_open_creates_persistent_storage() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("storage.db");
    let storage = SqliteStorage::open(&db_path).unwrap();
    assert_eq!(storage.spines.spine_count(), 0);
    assert_eq!(storage.entries.entry_count(), 0);
    assert_eq!(storage.certificates.certificate_count(), 0);
}

#[tokio::test]
async fn sqlite_storage_flush_succeeds_after_operations() {
    let storage = SqliteStorage::temporary().unwrap();
    let spine = create_test_spine();
    let owner = Did::new("did:key:z6MkOwner");
    let entry = create_test_entry(&owner, spine.id);
    let cert = create_test_certificate(&owner, spine.id);

    storage.spines.save_spine(&spine).await.unwrap();
    storage.entries.save_entry(&entry).await.unwrap();
    storage
        .certificates
        .save_certificate(&cert, spine.id)
        .await
        .unwrap();

    let result = storage.flush();
    assert!(result.is_ok());
}

#[tokio::test]
async fn sqlite_storage_all_components_accessible() {
    let storage = SqliteStorage::temporary().unwrap();
    let spine = create_test_spine();
    let owner = Did::new("did:key:z6MkOwner");
    let entry = create_test_entry(&owner, spine.id);
    let cert = create_test_certificate(&owner, spine.id);

    storage.spines.save_spine(&spine).await.unwrap();
    let hash = storage.entries.save_entry(&entry).await.unwrap();
    storage
        .certificates
        .save_certificate(&cert, spine.id)
        .await
        .unwrap();

    let retrieved_spine = storage.spines.get_spine(spine.id).await.unwrap();
    assert!(retrieved_spine.is_some());
    assert_eq!(retrieved_spine.unwrap().id, spine.id);

    let retrieved_entry = storage.entries.get_entry(hash).await.unwrap();
    assert!(retrieved_entry.is_some());
    assert_eq!(retrieved_entry.unwrap().spine_id, spine.id);

    let retrieved_cert = storage.certificates.get_certificate(cert.id).await.unwrap();
    assert!(retrieved_cert.is_some());
    let (retrieved_cert_inner, retrieved_spine_id) = retrieved_cert.unwrap();
    assert_eq!(retrieved_cert_inner.id, cert.id);
    assert_eq!(retrieved_spine_id, spine.id);
}

#[tokio::test]
async fn sqlite_storage_combined_operations() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("storage.db");
    let storage = SqliteStorage::open(&db_path).unwrap();

    let owner = Did::new("did:key:z6MkOwner");
    let spine = create_test_spine();
    storage.spines.save_spine(&spine).await.unwrap();

    let entry = create_test_entry(&owner, spine.id);
    let hash = storage.entries.save_entry(&entry).await.unwrap();

    let cert = create_test_certificate(&owner, spine.id);
    storage
        .certificates
        .save_certificate(&cert, spine.id)
        .await
        .unwrap();

    storage.flush().unwrap();

    assert_eq!(storage.spines.spine_count(), 1);
    assert_eq!(storage.entries.entry_count(), 1);
    assert_eq!(storage.certificates.certificate_count(), 1);

    let ids = storage.spines.list_spines().await.unwrap();
    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&spine.id));

    assert!(storage.entries.entry_exists(hash).await.unwrap());

    let cert_ids = storage.certificates.list_certificates().await.unwrap();
    assert_eq!(cert_ids.len(), 1);
    assert!(cert_ids.contains(&cert.id));
}

// ============================================================================
// Corrupt data and error path tests (coverage push)
// ============================================================================

#[tokio::test]
async fn sqlite_get_entry_corrupt_data_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("corrupt_entry.db");
    let storage = SqliteEntryStorage::open(&db_path).unwrap();

    let owner = Did::new("did:key:z6MkCorrupt");
    let spine_id = SpineId::now_v7();
    let entry = create_test_entry(&owner, spine_id);
    let hash = storage.save_entry(&entry).await.unwrap();

    {
        let conn = storage
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute(
            "UPDATE entries SET data = ? WHERE hash = ?",
            rusqlite::params![b"not valid json", hash.as_slice()],
        )
        .unwrap();
    }

    let result = storage.get_entry(hash).await;
    assert!(result.is_err(), "corrupt entry data should error");
}

#[tokio::test]
async fn sqlite_get_spine_corrupt_data_returns_error() {
    let (_temp_dir, storage) = spine_storage_from_tempdir();
    let spine = create_test_spine();
    storage.save_spine(&spine).await.unwrap();

    {
        let conn = storage
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute(
            "UPDATE spines SET data = ? WHERE id = ?",
            rusqlite::params![b"garbage", spine.id.to_string()],
        )
        .unwrap();
    }

    let result = storage.get_spine(spine.id).await;
    assert!(result.is_err(), "corrupt spine data should error");
}

#[tokio::test]
async fn sqlite_list_spines_skips_invalid_ids() {
    let (_temp_dir, storage) = spine_storage_from_tempdir();
    let spine = create_test_spine();
    storage.save_spine(&spine).await.unwrap();

    {
        let conn = storage
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute(
            "INSERT INTO spines (id, data) VALUES (?, ?)",
            rusqlite::params!["not-a-valid-uuid", b"{}"],
        )
        .unwrap();
    }

    let ids = storage.list_spines().await.unwrap();
    assert_eq!(ids.len(), 1, "invalid UUID should be skipped");
    assert!(ids.contains(&spine.id));
}

#[tokio::test]
async fn sqlite_get_certificate_corrupt_data_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("corrupt_cert.db");
    let cert_storage = SqliteCertificateStorage::open(&db_path).unwrap();

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let cert = create_test_certificate(&owner, spine_id);
    cert_storage
        .save_certificate(&cert, spine_id)
        .await
        .unwrap();

    {
        let conn = cert_storage
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute(
            "UPDATE certificates SET data = ? WHERE id = ?",
            rusqlite::params![b"broken", cert.id.to_string()],
        )
        .unwrap();
    }

    let result = cert_storage.get_certificate(cert.id).await;
    assert!(result.is_err(), "corrupt certificate data should error");
}

#[tokio::test]
async fn sqlite_list_certificates_skips_invalid_ids() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("bad_cert_id.db");
    let cert_storage = SqliteCertificateStorage::open(&db_path).unwrap();

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let cert = create_test_certificate(&owner, spine_id);
    cert_storage
        .save_certificate(&cert, spine_id)
        .await
        .unwrap();

    {
        let conn = cert_storage
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute(
            "INSERT INTO certificates (id, spine_id, data) VALUES (?, ?, ?)",
            rusqlite::params!["not-uuid", spine_id.to_string(), b"{}"],
        )
        .unwrap();
    }

    let ids = cert_storage.list_certificates().await.unwrap();
    assert_eq!(ids.len(), 1, "invalid certificate UUID should be skipped");
    assert!(ids.contains(&cert.id));
}

#[tokio::test]
async fn sqlite_get_entries_for_spine_corrupt_entry_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("corrupt_entries.db");
    let storage = SqliteEntryStorage::open(&db_path).unwrap();

    let owner = Did::new("did:key:z6MkCorrupt");
    let spine_id = SpineId::now_v7();
    let entry = create_test_entry(&owner, spine_id);
    let hash = storage.save_entry(&entry).await.unwrap();

    {
        let conn = storage
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute(
            "UPDATE entries SET data = ? WHERE hash = ?",
            rusqlite::params![b"corrupted json", hash.as_slice()],
        )
        .unwrap();
    }

    let result = storage.get_entries_for_spine(spine_id, 0, 100).await;
    assert!(result.is_err(), "corrupt entry in spine should error");
}

#[test]
fn sqlite_spine_count_returns_zero_on_empty_db() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("empty.db");
    let storage = SqliteSpineStorage::open(&db_path).unwrap();
    assert_eq!(storage.spine_count(), 0);
}

#[test]
fn sqlite_entry_count_returns_zero_on_empty_db() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("empty.db");
    let storage = SqliteEntryStorage::open(&db_path).unwrap();
    assert_eq!(storage.entry_count(), 0);
}

#[test]
fn sqlite_certificate_count_returns_zero_on_empty_db() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("empty.db");
    let storage = SqliteCertificateStorage::open(&db_path).unwrap();
    assert_eq!(storage.certificate_count(), 0);
}

// ========================================================================
// temporary() constructors (exercises in-memory path)
// ========================================================================

#[tokio::test]
async fn sqlite_spine_storage_temporary_crud() {
    let storage = SqliteSpineStorage::temporary().unwrap();
    let spine = create_test_spine();
    storage.save_spine(&spine).await.unwrap();
    assert_eq!(storage.spine_count(), 1);
    let retrieved = storage.get_spine(spine.id).await.unwrap();
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn sqlite_entry_storage_temporary_crud() {
    let storage = SqliteEntryStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let entry = create_test_entry(&owner, spine_id);
    let hash = storage.save_entry(&entry).await.unwrap();
    assert!(storage.entry_exists(hash).await.unwrap());
    assert_eq!(storage.entry_count(), 1);
}

#[tokio::test]
async fn sqlite_certificate_storage_temporary_crud() {
    let storage = SqliteCertificateStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let cert = create_test_certificate(&owner, spine_id);
    storage.save_certificate(&cert, spine_id).await.unwrap();
    assert_eq!(storage.certificate_count(), 1);
    let retrieved = storage.get_certificate(cert.id).await.unwrap();
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn sqlite_combined_storage_temporary_crud() {
    let storage = SqliteStorage::temporary().unwrap();
    let spine = create_test_spine();
    storage.spines.save_spine(&spine).await.unwrap();

    let owner = Did::new("did:key:z6MkOwner");
    let entry = create_test_entry(&owner, spine.id);
    storage.entries.save_entry(&entry).await.unwrap();

    let cert = create_test_certificate(&owner, spine.id);
    storage
        .certificates
        .save_certificate(&cert, spine.id)
        .await
        .unwrap();

    assert_eq!(storage.spines.spine_count(), 1);
    assert_eq!(storage.entries.entry_count(), 1);
    assert_eq!(storage.certificates.certificate_count(), 1);
    storage.flush().unwrap();
}

// ========================================================================
// flush() exercises
// ========================================================================

#[test]
fn sqlite_spine_storage_flush_via_open() {
    let (_temp_dir, storage) = spine_storage_from_tempdir();
    assert!(storage.flush().is_ok());
}

#[test]
fn sqlite_entry_storage_flush_via_open() {
    let (_temp_dir, storage) = entry_storage_from_tempdir();
    assert!(storage.flush().is_ok());
}

#[test]
fn sqlite_certificate_storage_flush_via_open() {
    let (_temp_dir, storage) = certificate_storage_from_tempdir();
    assert!(storage.flush().is_ok());
}

// ========================================================================
// get_entry None path
// ========================================================================

#[tokio::test]
async fn sqlite_get_entry_nonexistent_returns_none() {
    let storage = SqliteEntryStorage::temporary().unwrap();
    let result = storage.get_entry([0u8; 32]).await.unwrap();
    assert!(result.is_none());
}
