// SPDX-License-Identifier: AGPL-3.0-only

#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::types::{Did, SpineId};

#[cfg(feature = "redb-storage")]
use crate::storage::RedbStorage;
#[cfg(feature = "sled-storage")]
use crate::storage::SledStorage;

// ========================================================================
// Certificate Storage Tests
// ========================================================================

#[tokio::test]
async fn certificate_storage_crud() {
    use crate::certificate::{Certificate, CertificateType, MintInfo};
    use crate::storage::{CertificateStorage, InMemoryCertificateStorage};
    use crate::types::{CertificateId, Timestamp};

    let storage = InMemoryCertificateStorage::new();
    let cert_id = CertificateId::now_v7();
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
            game_id: "hl3".into(),
            edition: None,
        },
        &owner,
        &mint_info,
    );

    // Save
    storage.save_certificate(&cert, spine_id).await.unwrap();
    assert_eq!(storage.certificate_count().await, 1);

    // Get
    let retrieved = storage.get_certificate(cert_id).await.unwrap();
    assert!(retrieved.is_some());
    let (retrieved_cert, retrieved_spine) = retrieved.unwrap();
    assert_eq!(retrieved_cert.id, cert_id);
    assert_eq!(retrieved_spine, spine_id);

    // List
    let ids = storage.list_certificates().await.unwrap();
    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&cert_id));

    // Delete
    storage.delete_certificate(cert_id).await.unwrap();
    assert_eq!(storage.certificate_count().await, 0);
    assert!(storage.get_certificate(cert_id).await.unwrap().is_none());
}

#[tokio::test]
async fn certificate_storage_upsert() {
    use crate::certificate::{Certificate, CertificateType, MintInfo};
    use crate::storage::{CertificateStorage, InMemoryCertificateStorage};
    use crate::types::{CertificateId, Timestamp};

    let storage = InMemoryCertificateStorage::new();
    let cert_id = CertificateId::now_v7();
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
            game_id: "hl3".into(),
            edition: None,
        },
        &owner,
        &mint_info,
    );

    // Save twice (upsert)
    storage.save_certificate(&cert, spine_id).await.unwrap();
    storage.save_certificate(&cert, spine_id).await.unwrap();

    // Still only one
    assert_eq!(storage.certificate_count().await, 1);
}

#[tokio::test]
async fn certificate_storage_delete_idempotent() {
    use crate::storage::{CertificateStorage, InMemoryCertificateStorage};
    use crate::types::CertificateId;

    let storage = InMemoryCertificateStorage::new();
    let fake_id = CertificateId::now_v7();

    // Delete non-existent is OK
    storage.delete_certificate(fake_id).await.unwrap();
}

#[tokio::test]
async fn certificate_storage_get_nonexistent() {
    use crate::storage::{CertificateStorage, InMemoryCertificateStorage};
    use crate::types::CertificateId;

    let storage = InMemoryCertificateStorage::new();
    let fake_id = CertificateId::now_v7();

    assert!(storage.get_certificate(fake_id).await.unwrap().is_none());
}

#[tokio::test]
async fn combined_storage_has_certificates() {
    use crate::storage::InMemoryStorage;

    let storage = InMemoryStorage::new();
    assert_eq!(storage.certificates.certificate_count().await, 0);
}

// ========================================================================
// Sled Certificate Storage Tests
// ========================================================================

fn create_test_certificate(owner: &Did, spine_id: SpineId) -> crate::certificate::Certificate {
    use crate::certificate::{Certificate, CertificateType, MintInfo};
    use crate::types::{CertificateId, Timestamp};

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

#[cfg(feature = "sled-storage")]
#[tokio::test]
async fn sled_certificate_storage_crud() {
    use crate::storage::{CertificateStorage, SledCertificateStorage};

    let storage = SledCertificateStorage::temporary().unwrap();
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

#[cfg(feature = "sled-storage")]
#[tokio::test]
async fn sled_certificate_storage_upsert() {
    use crate::storage::{CertificateStorage, SledCertificateStorage};

    let storage = SledCertificateStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let cert = create_test_certificate(&owner, spine_id);

    storage.save_certificate(&cert, spine_id).await.unwrap();
    storage.save_certificate(&cert, spine_id).await.unwrap();
    assert_eq!(storage.certificate_count(), 1);
}

#[cfg(feature = "sled-storage")]
#[tokio::test]
async fn sled_certificate_storage_delete_nonexistent() {
    use crate::storage::{CertificateStorage, SledCertificateStorage};
    use crate::types::CertificateId;

    let storage = SledCertificateStorage::temporary().unwrap();
    storage
        .delete_certificate(CertificateId::now_v7())
        .await
        .unwrap();
}

#[cfg(feature = "sled-storage")]
#[tokio::test]
async fn sled_combined_storage_includes_certificates() {
    use crate::storage::CertificateStorage;

    let storage = SledStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let cert = create_test_certificate(&owner, spine_id);
    let cert_id = cert.id;

    storage
        .certificates
        .save_certificate(&cert, spine_id)
        .await
        .unwrap();
    assert_eq!(storage.certificates.certificate_count(), 1);

    let retrieved = storage.certificates.get_certificate(cert_id).await.unwrap();
    assert!(retrieved.is_some());

    storage.flush().unwrap();
}

#[cfg(feature = "sled-storage")]
#[tokio::test]
async fn sled_certificate_storage_multiple() {
    use crate::storage::{CertificateStorage, SledCertificateStorage};

    let storage = SledCertificateStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");

    for _ in 0..10 {
        let spine_id = SpineId::now_v7();
        let cert = create_test_certificate(&owner, spine_id);
        storage.save_certificate(&cert, spine_id).await.unwrap();
    }

    assert_eq!(storage.certificate_count(), 10);
    let ids = storage.list_certificates().await.unwrap();
    assert_eq!(ids.len(), 10);
}

// ========================================================================
// redb Certificate Storage Tests
// ========================================================================

#[cfg(feature = "redb-storage")]
#[tokio::test]
async fn redb_certificate_storage_crud() {
    use crate::storage::{CertificateStorage, RedbCertificateStorage};

    let storage = RedbCertificateStorage::temporary().unwrap();
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

#[cfg(feature = "redb-storage")]
#[tokio::test]
async fn redb_certificate_storage_upsert() {
    use crate::storage::{CertificateStorage, RedbCertificateStorage};

    let storage = RedbCertificateStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let cert = create_test_certificate(&owner, spine_id);

    storage.save_certificate(&cert, spine_id).await.unwrap();
    storage.save_certificate(&cert, spine_id).await.unwrap();
    assert_eq!(storage.certificate_count(), 1);
}

#[cfg(feature = "redb-storage")]
#[tokio::test]
async fn redb_certificate_storage_delete_nonexistent() {
    use crate::storage::{CertificateStorage, RedbCertificateStorage};
    use crate::types::CertificateId;

    let storage = RedbCertificateStorage::temporary().unwrap();
    storage
        .delete_certificate(CertificateId::now_v7())
        .await
        .unwrap();
}

#[cfg(feature = "redb-storage")]
#[tokio::test]
async fn redb_combined_storage_includes_certificates() {
    use crate::storage::CertificateStorage;

    let storage = RedbStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = SpineId::now_v7();
    let cert = create_test_certificate(&owner, spine_id);
    let cert_id = cert.id;

    storage
        .certificates
        .save_certificate(&cert, spine_id)
        .await
        .unwrap();
    assert_eq!(storage.certificates.certificate_count(), 1);

    let retrieved = storage.certificates.get_certificate(cert_id).await.unwrap();
    assert!(retrieved.is_some());

    storage.flush().unwrap();
}

#[cfg(feature = "redb-storage")]
#[tokio::test]
async fn redb_certificate_storage_multiple() {
    use crate::storage::{CertificateStorage, RedbCertificateStorage};

    let storage = RedbCertificateStorage::temporary().unwrap();
    let owner = Did::new("did:key:z6MkOwner");

    for _ in 0..10 {
        let spine_id = SpineId::now_v7();
        let cert = create_test_certificate(&owner, spine_id);
        storage.save_certificate(&cert, spine_id).await.unwrap();
    }

    assert_eq!(storage.certificate_count(), 10);
    let ids = storage.list_certificates().await.unwrap();
    assert_eq!(ids.len(), 10);
}
