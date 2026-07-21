// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::traits::CommitAcceptor;
use crate::traits::{BraidAcceptor, BraidSummary, DehydrationSummary};
use crate::types::SessionId;

#[tokio::test]
async fn service_basic() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_eq!(service.spine_count().await, 1);

    // Ensure idempotent
    let spine_id2 = service
        .ensure_spine(owner.clone(), None)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(spine_id, spine_id2);
    assert_eq!(service.spine_count().await, 1);
}

#[tokio::test]
async fn test_service_creation() {
    let service = LoamSpineService::new();
    assert_eq!(service.spine_count().await, 0);
    assert_eq!(service.entry_count().await, 0);
    assert_eq!(service.certificate_count().await, 0);
}

#[tokio::test]
async fn test_create_and_get_spine() {
    use crate::traits::SpineQuery;

    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test Spine".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    // Spine should exist
    let spine = service.get_spine(spine_id).await;
    assert!(spine.is_ok());

    // Entry count should include genesis
    assert!(service.entry_count().await >= 1);
}

#[tokio::test]
async fn test_seal_spine() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    // Seal the spine
    let result = service.seal_spine(spine_id, Some("Done".into())).await;
    assert!(result.is_ok());

    // Trying to append should fail (sealed)
    let entry_type = crate::entry::EntryType::MetadataUpdate {
        field: "test".into(),
        value: "value".into(),
    };
    let result = service.append_entry(spine_id, entry_type).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_append_entry() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    // Append a metadata update entry
    let entry_type = crate::entry::EntryType::MetadataUpdate {
        field: "test".into(),
        value: "value".into(),
    };

    let result = service.append_entry(spine_id, entry_type).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn commit_session() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let summary =
        DehydrationSummary::new(SessionId::now_v7(), "game", [0u8; 32]).with_vertex_count(100);

    let commit_ref = service
        .commit_session(spine_id, owner.clone(), summary)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_eq!(commit_ref.spine_id, spine_id);
    assert_eq!(commit_ref.index, 1);

    let exists = service
        .verify_commit(&commit_ref)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(exists);
}

#[tokio::test]
async fn commit_braid() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let braid = BraidSummary::new(
        crate::types::BraidId::now_v7(),
        "attribution",
        [1u8; 32],
        [2u8; 32],
    )
    .with_agent(owner.clone());

    let _entry_hash = service
        .commit_braid(spine_id, owner.clone(), braid.clone())
        .await
        .unwrap_or_else(|_| unreachable!());

    let exists = service
        .verify_braid(braid.braid_id)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(exists);
}

#[tokio::test]
async fn test_with_capabilities() {
    let caps = CapabilityRegistry::new();
    let service = LoamSpineService::with_capabilities(caps);
    assert_eq!(service.spine_count().await, 0);
}

#[tokio::test]
async fn prepare_entry_missing_spine() {
    let service = LoamSpineService::new();
    let bogus = SpineId::nil();
    let entry_type = crate::entry::EntryType::MetadataUpdate {
        field: "f".into(),
        value: "v".into(),
    };
    let result = service.prepare_entry(bogus, entry_type).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        format!("{err}").contains("not found") || matches!(err, LoamSpineError::SpineNotFound(_)),
    );
}

#[tokio::test]
async fn prepare_entry_sealed_spine() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkSealTest");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Seal".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    service
        .seal_spine(spine_id, Some("done".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let entry_type = crate::entry::EntryType::MetadataUpdate {
        field: "f".into(),
        value: "v".into(),
    };
    let result = service.prepare_entry(spine_id, entry_type).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LoamSpineError::SpineSealed(_)
    ));
}

#[tokio::test]
async fn prepare_and_append_entry_roundtrip() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkPrepare");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Prep".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let entry_type = crate::entry::EntryType::MetadataUpdate {
        field: "key".into(),
        value: "val".into(),
    };
    let entry = service
        .prepare_entry(spine_id, entry_type)
        .await
        .unwrap_or_else(|_| unreachable!());

    let entry_with_meta = entry.with_metadata("custom_sig", "abc123");

    let hash = service
        .append_prepared_entry(spine_id, entry_with_meta)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_ne!(hash, [0u8; 32]);
}

#[tokio::test]
async fn append_prepared_entry_missing_spine() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkAppendGhost");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Ghost".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let entry_type = crate::entry::EntryType::MetadataUpdate {
        field: "f".into(),
        value: "v".into(),
    };
    let entry = service
        .prepare_entry(spine_id, entry_type)
        .await
        .unwrap_or_else(|_| unreachable!());

    let bogus = SpineId::nil();
    let result = service.append_prepared_entry(bogus, entry).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LoamSpineError::SpineNotFound(_)
    ));
}

#[tokio::test]
async fn append_prepared_entry_sealed_spine() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkAppendSeal");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Seal".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let entry_type = crate::entry::EntryType::MetadataUpdate {
        field: "f".into(),
        value: "v".into(),
    };
    let entry = service
        .prepare_entry(spine_id, entry_type)
        .await
        .unwrap_or_else(|_| unreachable!());

    service
        .seal_spine(spine_id, Some("done".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let result = service.append_prepared_entry(spine_id, entry).await;
    assert!(result.is_err());
}
