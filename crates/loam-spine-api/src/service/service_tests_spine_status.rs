// SPDX-License-Identifier: AGPL-3.0-or-later

//! Spine status RPC tests.
//!
//! Split from `service_tests.rs` for navigability.

use super::*;

#[tokio::test]
async fn spine_status_returns_basic_info() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:z6MkStatus");

    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "status-test".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create spine");

    let status = service
        .spine_status(SpineStatusRequest {
            spine_id: spine_resp.spine_id,
        })
        .await
        .expect("status");

    assert_eq!(status.spine_id, spine_resp.spine_id);
    assert_eq!(status.name.as_deref(), Some("status-test"));
    assert_eq!(status.owner, owner);
    assert!(matches!(status.state, SpineState::Active));
    assert_eq!(status.entry_count, 1);
    assert_eq!(status.genesis_hash, spine_resp.genesis_hash);
    assert_eq!(status.tip_hash, spine_resp.genesis_hash);
    assert_eq!(status.session_count, 0);
    assert!(status.sessions.is_empty());
}

#[tokio::test]
async fn spine_status_reports_sessions() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:z6MkStatusSession");

    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "status-sessions".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create spine");

    let session_id = uuid::Uuid::now_v7();
    let session_hash = [0xABu8; 32];

    service
        .commit_session(CommitSessionRequest {
            spine_id: spine_resp.spine_id,
            session_id,
            session_hash,
            vertex_count: 42,
            committer: owner.clone(),
        })
        .await
        .expect("commit session");

    let status = service
        .spine_status(SpineStatusRequest {
            spine_id: spine_resp.spine_id,
        })
        .await
        .expect("status after session");

    assert_eq!(status.entry_count, 2);
    assert_eq!(status.session_count, 1);
    assert_eq!(status.sessions.len(), 1);

    let sess = &status.sessions[0];
    assert_eq!(sess.session_id, session_id);
    assert_eq!(sess.merkle_root, session_hash);
    assert_eq!(sess.vertex_count, 42);
    assert_eq!(sess.committer, owner);
    assert_eq!(sess.entry_index, 1);
}

#[tokio::test]
async fn spine_status_not_found_returns_error() {
    let service = LoamSpineRpcService::default_service();
    let result = service
        .spine_status(SpineStatusRequest {
            spine_id: uuid::Uuid::nil(),
        })
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn spine_status_after_seal_reports_sealed() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:z6MkStatusSeal");

    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "status-seal".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create spine");

    service
        .seal_spine(SealSpineRequest {
            spine_id: spine_resp.spine_id,
            sealer: owner.clone(),
            reason: Some("archive".into()),
        })
        .await
        .expect("seal spine");

    let status = service
        .spine_status(SpineStatusRequest {
            spine_id: spine_resp.spine_id,
        })
        .await
        .expect("status after seal");

    assert!(matches!(status.state, SpineState::Sealed { .. }));
    assert_eq!(status.entry_count, 2);
}

#[tokio::test]
async fn spine_status_multiple_sessions_reverse_order() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:z6MkStatusMulti");

    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "status-multi".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create spine");

    for i in 0..3u8 {
        service
            .commit_session(CommitSessionRequest {
                spine_id: spine_resp.spine_id,
                session_id: uuid::Uuid::now_v7(),
                session_hash: [i; 32],
                vertex_count: u64::from(i) + 10,
                committer: owner.clone(),
            })
            .await
            .expect("commit session");
    }

    let status = service
        .spine_status(SpineStatusRequest {
            spine_id: spine_resp.spine_id,
        })
        .await
        .expect("status");

    assert_eq!(status.session_count, 3);
    assert!(
        status.sessions[0].entry_index > status.sessions[1].entry_index,
        "sessions should be most-recent-first"
    );
    assert!(
        status.sessions[1].entry_index > status.sessions[2].entry_index,
        "sessions should be most-recent-first"
    );
}
