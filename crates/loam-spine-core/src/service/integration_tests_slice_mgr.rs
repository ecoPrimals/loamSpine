// SPDX-License-Identifier: AGPL-3.0-or-later

//! SliceManager extended operation tests: mark, clear, status, record, list.

use super::*;

async fn setup_slice(service: &LoamSpineService) -> (SpineId, SliceId, EntryHash) {
    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("SliceTest".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let spine = service
        .get_spine(spine_id)
        .await
        .unwrap_or_else(|_| unreachable!())
        .unwrap_or_else(|| unreachable!());
    let genesis = spine.genesis_entry().unwrap_or_else(|| unreachable!());
    let entry_hash = genesis.compute_hash().expect("compute_hash");

    let session_id = SessionId::now_v7();
    let _origin = service
        .checkout_slice(spine_id, entry_hash, owner, session_id)
        .await
        .unwrap_or_else(|_| unreachable!());

    let slice_id = {
        let slices = service.active_slices.read().await;
        *slices.keys().next().unwrap_or_else(|| unreachable!())
    };

    (spine_id, slice_id, entry_hash)
}

#[tokio::test]
async fn test_mark_sliced() {
    let service = LoamSpineService::new();
    let (_, slice_id, _) = setup_slice(&service).await;

    let new_holder = Did::new("did:key:z6MkNewHolder");
    service
        .mark_sliced(slice_id, new_holder.clone())
        .await
        .unwrap_or_else(|_| unreachable!());

    let status = service
        .get_slice_status(slice_id)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(status, SliceStatus::Active { holder: new_holder });
}

#[tokio::test]
async fn test_clear_slice_mark() {
    let service = LoamSpineService::new();
    let (_, slice_id, _) = setup_slice(&service).await;

    service
        .clear_slice_mark(slice_id)
        .await
        .unwrap_or_else(|_| unreachable!());

    let status = service
        .get_slice_status(slice_id)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(status, SliceStatus::Unknown);
}

#[tokio::test]
async fn test_get_slice_status_active() {
    let service = LoamSpineService::new();
    let (_, slice_id, _) = setup_slice(&service).await;

    let status = service
        .get_slice_status(slice_id)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(matches!(status, SliceStatus::Active { .. }));
}

#[tokio::test]
async fn test_get_slice_status_unknown() {
    let service = LoamSpineService::new();
    let status = service
        .get_slice_status(SliceId::now_v7())
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(status, SliceStatus::Unknown);
}

#[tokio::test]
async fn test_record_slice_checkout() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("RecordCheckout".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let origin = SliceOrigin {
        spine_id,
        entry_hash: [1u8; 32],
        entry_index: 0,
        certificate_id: None,
        owner: owner.clone(),
    };

    let slice_id = SliceId::now_v7();
    let holder = Did::new("did:key:z6MkHolder");

    let entry_hash = service
        .record_slice_checkout(spine_id, slice_id, holder, &origin)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_ne!(entry_hash, [0u8; 32]);

    let entries = service
        .get_entries(spine_id, 0, 100)
        .await
        .unwrap_or_else(|_| unreachable!());
    let has_checkout = entries.iter().any(|e| {
        matches!(&e.entry_type, EntryType::SliceCheckout { slice_id: sid, .. } if *sid == slice_id)
    });
    assert!(has_checkout);
}

#[tokio::test]
async fn test_record_slice_return_merged() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("RecordReturn".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let slice_id = SliceId::now_v7();
    let resolution = SliceResolution::Merged {
        summary: [0xAAu8; 32],
    };

    let entry_hash = service
        .record_slice_return(spine_id, slice_id, &resolution)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_ne!(entry_hash, [0u8; 32]);

    let entries = service
        .get_entries(spine_id, 0, 100)
        .await
        .unwrap_or_else(|_| unreachable!());
    let has_return = entries.iter().any(|e| {
        matches!(&e.entry_type, EntryType::SliceReturn { slice_id: sid, success, .. } if *sid == slice_id && *success)
    });
    assert!(has_return);
}

#[tokio::test]
async fn test_record_slice_return_abandoned() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("ReturnAbandon".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let slice_id = SliceId::now_v7();
    let resolution = SliceResolution::Abandoned {
        reason: "changed mind".into(),
    };

    let entry_hash = service
        .record_slice_return(spine_id, slice_id, &resolution)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_ne!(entry_hash, [0u8; 32]);
}

#[tokio::test]
async fn test_list_active_slices() {
    let service = LoamSpineService::new();
    let (spine_id, _slice_id, _) = setup_slice(&service).await;

    let active = service
        .list_active_slices(spine_id)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_eq!(active.len(), 1);
    assert_eq!(active[0].origin.spine_id, spine_id);
}

#[tokio::test]
async fn test_list_active_slices_empty() {
    let service = LoamSpineService::new();
    let active = service
        .list_active_slices(SpineId::now_v7())
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(active.is_empty());
}

#[tokio::test]
async fn test_list_active_slices_metadata_accuracy() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("MetadataTest".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let spine = service
        .get_spine(spine_id)
        .await
        .unwrap_or_else(|_| unreachable!())
        .unwrap_or_else(|| unreachable!());
    let genesis = spine.genesis_entry().unwrap_or_else(|| unreachable!());
    let entry_hash = genesis.compute_hash().expect("compute_hash");

    let session_id = SessionId::now_v7();
    let _origin = service
        .checkout_slice(spine_id, entry_hash, owner.clone(), session_id)
        .await
        .unwrap_or_else(|_| unreachable!());

    let active = service
        .list_active_slices(spine_id)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_eq!(active.len(), 1);
    let slice = &active[0];
    assert_eq!(slice.origin.entry_hash, entry_hash);
    assert_eq!(slice.origin.owner, owner);
    assert_eq!(slice.holder, owner);
    assert_eq!(slice.session_id, session_id);
}
