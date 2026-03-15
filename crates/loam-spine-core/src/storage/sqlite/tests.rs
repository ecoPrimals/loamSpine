// SPDX-License-Identifier: AGPL-3.0-only

#[cfg(test)]
#[cfg(feature = "sqlite")]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use crate::certificate::{Certificate, CertificateType, MintInfo};
    use crate::entry::{Entry, EntryType, SpineConfig};
    use crate::spine::Spine;
    use crate::storage::{
        CertificateStorage, EntryStorage, SpineStorage, SqliteCertificateStorage,
        SqliteEntryStorage, SqliteSpineStorage,
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
}
