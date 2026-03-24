// SPDX-License-Identifier: AGPL-3.0-or-later

//! Certificate-focused sled storage tests.
//!
//! Extracted from `sled_tests.rs` for domain cohesion. Covers
//! `SledCertificateStorage` CRUD, edge cases, corruption, and
//! malformed key resilience.

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap for failure clarity"
)]

use crate::certificate::{Certificate, CertificateType, MintInfo};
use crate::entry::{Entry, SpineConfig};
use crate::spine::Spine;
use crate::storage::{
    CertificateStorage, EntryStorage, SledCertificateStorage, SledStorage, SpineStorage,
};
use crate::types::{CertificateId, Did, SpineId, Timestamp};
use serial_test::serial;

fn create_sled_test_spine() -> Spine {
    Spine::new(
        Did::new("did:key:z6MkSledCert"),
        Some("Sled Cert Test".into()),
        SpineConfig::default(),
    )
    .unwrap()
}

fn test_mint_info(owner: &Did, spine_id: SpineId) -> MintInfo {
    MintInfo {
        minter: owner.clone(),
        spine: spine_id,
        entry: [0u8; 32],
        timestamp: Timestamp::now(),
        authority: None,
    }
}

fn test_certificate(cert_id: CertificateId, owner: &Did, mint_info: &MintInfo) -> Certificate {
    Certificate::new(
        cert_id,
        CertificateType::DigitalGame {
            platform: "steam".into(),
            game_id: "test".into(),
            edition: None,
        },
        owner,
        mint_info,
    )
}

#[tokio::test]
async fn sled_certificate_storage_crud() {
    let storage = SledCertificateStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkCertOwner");
    let spine_id = SpineId::now_v7();

    let cert_id = CertificateId::now_v7();
    let mint_info = test_mint_info(&owner, spine_id);
    let cert = test_certificate(cert_id, &owner, &mint_info);

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
async fn sled_certificate_list_empty() {
    let storage = SledCertificateStorage::temporary().unwrap();
    let ids = storage.list_certificates().await.unwrap();
    assert!(ids.is_empty());
}

#[tokio::test]
async fn sled_certificate_get_nonexistent() {
    let storage = SledCertificateStorage::temporary().unwrap();
    let result = storage
        .get_certificate(CertificateId::now_v7())
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn sled_certificate_delete_nonexistent() {
    let storage = SledCertificateStorage::temporary().unwrap();
    storage
        .delete_certificate(CertificateId::now_v7())
        .await
        .unwrap();
}

#[tokio::test]
async fn sled_certificate_count_on_fresh_db() {
    let storage = SledCertificateStorage::temporary().unwrap();
    assert_eq!(storage.certificate_count(), 0);
}

#[tokio::test]
async fn sled_combined_flush_all_components() {
    let storage = SledStorage::temporary().unwrap();
    let spine = create_sled_test_spine();
    storage.spines.save_spine(&spine).await.unwrap();

    let owner = Did::new("did:key:z6MkOwner");
    let entry = Entry::genesis(owner.clone(), spine.id, SpineConfig::default());
    storage.entries.save_entry(&entry).await.unwrap();

    let cert_id = CertificateId::now_v7();
    let mint_info = test_mint_info(&owner, spine.id);
    let cert = test_certificate(cert_id, &owner, &mint_info);
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

#[tokio::test]
#[serial]
async fn sled_get_certificate_corrupted_data_returns_error() {
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
async fn sled_get_certificate_corrupted_bincode_returns_deserialize_error() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("cert-corrupt-bc");

    let cert_id = CertificateId::now_v7();
    {
        let db = sled::open(&path).unwrap();
        let tree = db.open_tree("certificates").unwrap();
        tree.insert(cert_id.as_bytes().as_slice(), b"corrupt-data" as &[u8])
            .unwrap();
        db.flush().unwrap();
    }

    let storage = SledCertificateStorage::open(&path).unwrap();
    let result = storage.get_certificate(cert_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("deserialize"));
}

#[tokio::test]
#[serial]
async fn sled_list_certificates_with_malformed_keys_skips_invalid() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("certs-malformed");

    let cert_id = CertificateId::now_v7();
    {
        let db = sled::open(&path).unwrap();
        let tree = db.open_tree("certificates").unwrap();

        let owner = Did::new("did:key:z6MkOwner");
        let spine_id = SpineId::now_v7();
        let mint_info = test_mint_info(&owner, spine_id);
        let cert = test_certificate(cert_id, &owner, &mint_info);
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
