// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration ops tests: `permanent_storage.*` and `commit_session`.

use super::*;
use crate::types::{
    PermanentStorageCommitRequest, PermanentStorageDehydrationSummary,
    PermanentStorageGetCommitRequest, PermanentStorageVerifyRequest,
};

fn make_permanent_storage_summary() -> PermanentStorageDehydrationSummary {
    PermanentStorageDehydrationSummary {
        session_type: "game".to_string(),
        vertex_count: 10,
        leaf_count: 5,
        started_at: 0,
        ended_at: 1000,
        outcome: "Success".to_string(),
    }
}

#[tokio::test]
async fn test_permanent_storage_commit_session_invalid_hex_merkle_root() {
    let service = LoamSpineRpcService::default_service();
    let result = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: uuid::Uuid::now_v7().to_string(),
            merkle_root: "not-64-hex-chars".to_string(),
            summary: make_permanent_storage_summary(),
            committer_did: Some("did:key:test".to_string()),
        })
        .await;
    assert!(result.is_err());
    let err = result.expect_err("expected error");
    assert!(err.to_string().contains("invalid merkle_root hex"));
}

#[tokio::test]
async fn test_permanent_storage_commit_session_missing_committer_did() {
    let service = LoamSpineRpcService::default_service();
    let valid_hex = "a".repeat(64);
    let result = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: uuid::Uuid::now_v7().to_string(),
            merkle_root: valid_hex,
            summary: make_permanent_storage_summary(),
            committer_did: None,
        })
        .await;
    assert!(result.is_err());
    let err = result.expect_err("expected error");
    assert!(err.to_string().contains("committer_did is required"));
}

#[tokio::test]
async fn test_permanent_storage_commit_session_invalid_session_id() {
    let service = LoamSpineRpcService::default_service();
    let valid_hex = "a".repeat(64);
    let result = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: "not-a-uuid".to_string(),
            merkle_root: valid_hex,
            summary: make_permanent_storage_summary(),
            committer_did: Some("did:key:test".to_string()),
        })
        .await;
    assert!(result.is_err());
    let err = result.expect_err("expected error");
    assert!(err.to_string().contains("invalid session_id UUID"));
}

#[tokio::test]
async fn test_permanent_storage_commit_session_success() {
    let service = LoamSpineRpcService::default_service();
    let valid_hex = "a".repeat(64);
    let result = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: uuid::Uuid::now_v7().to_string(),
            merkle_root: valid_hex,
            summary: make_permanent_storage_summary(),
            committer_did: Some("did:key:committer".to_string()),
        })
        .await;
    let resp = result.expect("commit should succeed");
    assert!(resp.accepted);
    assert!(resp.commit_id.is_some());
    assert!(resp.spine_entry_hash.is_some());
    assert!(resp.entry_index.is_some());
    assert!(resp.spine_id.is_some());
    assert!(resp.error.is_none());
}

#[tokio::test]
async fn test_commit_session_success() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:session-owner");
    let create_resp = service
        .create_spine(CreateSpineRequest {
            name: "session-spine".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create should succeed");

    let session_id = uuid::Uuid::now_v7();
    let merkle_root = [1u8; 32];

    let result = service
        .commit_session(CommitSessionRequest {
            spine_id: create_resp.spine_id,
            session_id,
            session_hash: merkle_root,
            vertex_count: 100,
            committer: owner.clone(),
        })
        .await;
    let resp = result.expect("commit_session should succeed");

    assert_ne!(resp.commit_hash, [0u8; 32]);
    assert!(resp.index >= 1);
    assert_eq!(resp.spine_id, create_resp.spine_id);
    assert!(resp.committed_at.as_nanos() > 0);

    assert_eq!(resp.session_id, session_id);
    assert_eq!(resp.merkle_root, merkle_root);
    assert_eq!(resp.vertex_count, 100);
    assert_eq!(resp.committer, owner);
    assert!(resp.tower_signature.is_none());
}

#[tokio::test]
async fn test_permanent_storage_verify_commit_invalid_spine_id() {
    let service = LoamSpineRpcService::default_service();
    let valid_hex = "a".repeat(64);
    let result = service
        .permanent_storage_verify_commit(PermanentStorageVerifyRequest {
            spine_id: "not-a-valid-uuid".to_string(),
            entry_hash: valid_hex.clone(),
            index: 0,
        })
        .await;
    assert!(result.is_err());
    let err = result.expect_err("expected error");
    assert!(err.to_string().contains("invalid spine_id UUID"));
}

#[tokio::test]
async fn test_permanent_storage_verify_commit_invalid_entry_hash() {
    let service = LoamSpineRpcService::default_service();
    let result = service
        .permanent_storage_verify_commit(PermanentStorageVerifyRequest {
            spine_id: uuid::Uuid::now_v7().to_string(),
            entry_hash: "bad-hex".to_string(),
            index: 0,
        })
        .await;
    assert!(result.is_err());
    let err = result.expect_err("expected error");
    assert!(err.to_string().contains("invalid entry_hash hex"));
}

#[tokio::test]
async fn test_permanent_storage_verify_commit_valid_found() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:verify-owner");
    let commit_resp = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: uuid::Uuid::now_v7().to_string(),
            merkle_root: "b".repeat(64),
            summary: make_permanent_storage_summary(),
            committer_did: Some(owner.to_string()),
        })
        .await
        .expect("commit should succeed");
    let spine_id = commit_resp.spine_id.expect("spine_id");
    let entry_hash = commit_resp.commit_id.expect("commit_id");

    let found = service
        .permanent_storage_verify_commit(PermanentStorageVerifyRequest {
            spine_id,
            entry_hash,
            index: 0,
        })
        .await
        .expect("verify should succeed");
    assert!(found);
}

#[tokio::test]
async fn test_permanent_storage_verify_commit_valid_not_found() {
    let service = LoamSpineRpcService::default_service();
    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "verify-spine".to_string(),
            owner: Did::new("did:key:verify-spine-owner"),
            config: None,
        })
        .await
        .expect("create should succeed");

    let nonexistent_hex = "c".repeat(64);
    let found = service
        .permanent_storage_verify_commit(PermanentStorageVerifyRequest {
            spine_id: spine_resp.spine_id.to_string(),
            entry_hash: nonexistent_hex,
            index: 0,
        })
        .await
        .expect("verify should succeed");
    assert!(!found);
}

#[tokio::test]
async fn test_permanent_storage_get_commit_invalid_spine_id() {
    let service = LoamSpineRpcService::default_service();
    let valid_hex = "a".repeat(64);
    let result = service
        .permanent_storage_get_commit(PermanentStorageGetCommitRequest {
            spine_id: "not-a-uuid".to_string(),
            entry_hash: valid_hex,
            index: 0,
        })
        .await;
    assert!(result.is_err());
    let err = result.expect_err("expected error");
    assert!(err.to_string().contains("invalid spine_id UUID"));
}

#[tokio::test]
async fn test_permanent_storage_get_commit_invalid_entry_hash() {
    let service = LoamSpineRpcService::default_service();
    let result = service
        .permanent_storage_get_commit(PermanentStorageGetCommitRequest {
            spine_id: uuid::Uuid::now_v7().to_string(),
            entry_hash: "invalid-hex-zz".to_string(),
            index: 0,
        })
        .await;
    assert!(result.is_err());
    let err = result.expect_err("expected error");
    assert!(err.to_string().contains("invalid entry_hash hex"));
}

#[tokio::test]
async fn test_permanent_storage_get_commit_found() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:get-commit-owner");
    let commit_resp = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: uuid::Uuid::now_v7().to_string(),
            merkle_root: "d".repeat(64),
            summary: make_permanent_storage_summary(),
            committer_did: Some(owner.to_string()),
        })
        .await
        .expect("commit should succeed");
    let spine_id = commit_resp.spine_id.expect("spine_id");
    let entry_hash = commit_resp.commit_id.expect("commit_id");

    let value = service
        .permanent_storage_get_commit(PermanentStorageGetCommitRequest {
            spine_id,
            entry_hash,
            index: 0,
        })
        .await
        .expect("get_commit should succeed");
    assert!(!value.is_null());
}

#[tokio::test]
async fn test_permanent_storage_get_commit_not_found() {
    let service = LoamSpineRpcService::default_service();
    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "get-commit-spine".to_string(),
            owner: Did::new("did:key:get-spine-owner"),
            config: None,
        })
        .await
        .expect("create should succeed");

    let value = service
        .permanent_storage_get_commit(PermanentStorageGetCommitRequest {
            spine_id: spine_resp.spine_id.to_string(),
            entry_hash: "e".repeat(64),
            index: 0,
        })
        .await
        .expect("get_commit should succeed");
    assert!(value.is_null());
}
