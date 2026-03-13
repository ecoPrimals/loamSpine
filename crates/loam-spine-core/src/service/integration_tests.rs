// SPDX-License-Identifier: AGPL-3.0-only

//! Tests for integration trait implementations.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::types::Timestamp;

#[tokio::test]
async fn test_slice_checkout_and_resolve() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test".into()))
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
    let origin = service
        .checkout_slice(spine_id, entry_hash, owner.clone(), session_id)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_eq!(origin.spine_id, spine_id);
}

#[tokio::test]
async fn test_slice_resolve_merged() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Resolve Test".into()))
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
    let origin = service
        .checkout_slice(spine_id, entry_hash, owner.clone(), session_id)
        .await
        .unwrap_or_else(|_| unreachable!());

    let slice_id = {
        let slices = service.active_slices.read().await;
        slices
            .keys()
            .next()
            .copied()
            .unwrap_or_else(|| unreachable!())
    };

    let resolution = SliceResolution::Merged {
        summary: [0xABu8; 32],
    };
    let result_hash = service
        .resolve_slice(slice_id, resolution)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_ne!(result_hash, [0u8; 32]);
    assert_eq!(origin.spine_id, spine_id);
}

#[tokio::test]
async fn test_slice_resolve_abandoned() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Abandon Test".into()))
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

    let slice_id = {
        let slices = service.active_slices.read().await;
        slices
            .keys()
            .next()
            .copied()
            .unwrap_or_else(|| unreachable!())
    };

    let resolution = SliceResolution::Abandoned {
        reason: "test".to_string(),
    };
    let result_hash = service
        .resolve_slice(slice_id, resolution)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_ne!(result_hash, [0u8; 32]);
}

#[tokio::test]
async fn test_get_entries() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let entries = service
        .get_entries(spine_id, 0, 10)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(!entries.is_empty());
}

#[tokio::test]
async fn test_commit_session_and_verify() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Session Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let session_id = SessionId::now_v7();
    let summary = DehydrationSummary {
        session_id,
        session_type: "test-session".to_string(),
        merkle_root: [0xABu8; 32],
        vertex_count: 42,
        started_at: Timestamp::now(),
        ended_at: Timestamp::now(),
        result_entries: Vec::new(),
        metadata: std::collections::HashMap::new(),
    };

    let commit_ref = service
        .commit_session(spine_id, owner.clone(), summary)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_eq!(commit_ref.spine_id, spine_id);
    assert!(commit_ref.index > 0);

    let verified = service
        .verify_commit(&commit_ref)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(verified);

    let entry = service
        .get_commit(&commit_ref)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(entry.is_some());
}

#[tokio::test]
async fn test_braid_commit_and_verify() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Braid Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let braid_id = BraidId::now_v7();
    let subject_hash = [0xCDu8; 32];
    let braid = BraidSummary {
        braid_id,
        braid_type: "attribution".to_string(),
        subject_hash,
        braid_hash: [0xEFu8; 32],
        agents: vec![owner.clone()],
        created_at: Timestamp::now(),
        signature: None,
    };

    let entry_hash = service
        .commit_braid(spine_id, owner.clone(), braid)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_ne!(entry_hash, [0u8; 32]);

    let exists = service
        .verify_braid(braid_id)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(exists);

    let not_exists = service
        .verify_braid(BraidId::now_v7())
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(!not_exists);

    let braids = service
        .get_braids_for_subject(subject_hash)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(braids.len(), 1);
    assert_eq!(braids[0], entry_hash);

    let no_braids = service
        .get_braids_for_subject([0x00u8; 32])
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(no_braids.is_empty());
}

#[tokio::test]
async fn test_get_tip() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Tip Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let tip = service
        .get_tip(spine_id)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(tip.is_some());

    let no_tip = service
        .get_tip(SpineId::now_v7())
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(no_tip.is_none());
}

#[tokio::test]
async fn test_get_entry() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Entry Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let spine = service
        .get_spine(spine_id)
        .await
        .unwrap_or_else(|_| unreachable!())
        .unwrap_or_else(|| unreachable!());

    let genesis = spine.genesis_entry().unwrap_or_else(|| unreachable!());
    let entry_hash = genesis.compute_hash().expect("compute_hash");

    let entry = service
        .get_entry(entry_hash)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(entry.is_some());

    let no_entry = service
        .get_entry([0u8; 32])
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(no_entry.is_none());
}

// ============================================================================
// SliceManager extended operations
// ============================================================================

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

// ============================================================================
// ProvenanceSource tests
// ============================================================================

#[tokio::test]
async fn test_get_entries_for_data() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("DataTest".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let data_hash = [0xFEu8; 32];
    service
        .append_entry(
            spine_id,
            EntryType::DataAnchor {
                data_hash,
                mime_type: Some("application/json".into()),
                size: 1024,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let entries = service
        .get_entries_for_data(data_hash)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(entries.len(), 1);

    let no_entries = service
        .get_entries_for_data([0u8; 32])
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(no_entries.is_empty());
}

#[tokio::test]
async fn test_get_certificate_history() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("CertHistory".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert_id = crate::types::CertificateId::now_v7();

    service
        .append_entry(
            spine_id,
            EntryType::CertificateMint {
                cert_id,
                cert_type: "game".into(),
                initial_owner: owner.clone(),
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let new_owner = Did::new("did:key:z6MkNewOwner");
    service
        .append_entry(
            spine_id,
            EntryType::CertificateTransfer {
                cert_id,
                from: owner.clone(),
                to: new_owner,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let history = service
        .get_certificate_history(cert_id)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(history.len(), 2);

    let no_history = service
        .get_certificate_history(crate::types::CertificateId::now_v7())
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(no_history.is_empty());
}

#[tokio::test]
async fn test_get_attribution() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkCreator");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("AttrTest".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let data_hash = [0xBBu8; 32];
    service
        .append_entry(
            spine_id,
            EntryType::DataAnchor {
                data_hash,
                mime_type: Some("text/plain".into()),
                size: 256,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let attr = service
        .get_attribution(data_hash)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(attr.is_some());

    let record = attr.unwrap_or_else(|| unreachable!());
    assert_eq!(record.content_hash, data_hash);
    assert_eq!(record.creator, owner);

    let no_attr = service
        .get_attribution([0u8; 32])
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(no_attr.is_none());
}

#[tokio::test]
async fn test_get_provenance_chain() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("ProvenanceTest".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let content_hash = [0xCCu8; 32];

    service
        .append_entry(
            spine_id,
            EntryType::DataAnchor {
                data_hash: content_hash,
                mime_type: Some("text/plain".into()),
                size: 128,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    service
        .append_entry(
            spine_id,
            EntryType::BraidCommit {
                braid_id: BraidId::now_v7(),
                braid_hash: [0xDDu8; 32],
                subject_hash: content_hash,
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let chain = service
        .get_provenance_chain(content_hash)
        .await
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(chain.len(), 2);
    assert_eq!(chain[0].relationship, "anchored-by");
    assert_eq!(chain[1].relationship, "attributed-to");

    let empty_chain = service
        .get_provenance_chain([0u8; 32])
        .await
        .unwrap_or_else(|_| unreachable!());
    assert!(empty_chain.is_empty());
}
