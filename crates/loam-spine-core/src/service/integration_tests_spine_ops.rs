// SPDX-License-Identifier: AGPL-3.0-or-later

//! Core spine operation tests: ensure, entries, commits, braids, tips.

use super::*;

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
