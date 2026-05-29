// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::types::{
    AppendEntryRequest, CommitBraidRequest, CommitSessionRequest, CreateSpineRequest,
    DehydrateSessionRequest,
};
use crate::types::{Did, EntryType};
use tests::rpc_call;

#[tokio::test]
async fn test_jsonrpc_commit_session() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Session Test".to_string(),
        config: None,
    };
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let commit_request = CommitSessionRequest {
        spine_id: create_response.spine_id,
        committer: owner,
        session_id: uuid::Uuid::now_v7(),
        session_hash: [0u8; 32],
        vertex_count: 42,
    };

    let result: Result<crate::types::CommitSessionResponse, _> =
        rpc_call(&server, "session.commit", &commit_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jsonrpc_dehydrate_session() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Dehydrate Test".to_string(),
        config: None,
    };
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let session_id = uuid::Uuid::now_v7();
    let dehydrate_request = DehydrateSessionRequest {
        spine_id: create_response.spine_id,
        session_id,
        committer: owner.clone(),
        session_type: None,
    };

    let result: Result<crate::types::DehydrateSessionResponse, _> =
        rpc_call(&server, "session.dehydrate", &dehydrate_request).await;
    let response = result.unwrap();

    assert_eq!(response.spine_id, create_response.spine_id);
    assert_eq!(response.session_id, session_id);
    assert_eq!(response.session_type, "session");
    assert_eq!(response.committer, owner);
    assert!(!response.session_hash.iter().all(|&b| b == 0));
}

#[tokio::test]
async fn test_jsonrpc_dehydrate_then_commit() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Dehydrate+Commit Test".to_string(),
        config: None,
    };
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let append_request = AppendEntryRequest {
        spine_id: create_response.spine_id,
        entry_type: EntryType::DataAnchor {
            data_hash: [7u8; 32],
            mime_type: Some("text/plain".to_string()),
            size: 100,
        },
        committer: Some(owner.clone()),
        payload: None,
    };
    rpc_call::<_, crate::types::AppendEntryResponse>(&server, "entry.append", &append_request)
        .await
        .unwrap();

    let session_id = uuid::Uuid::now_v7();
    let dehydrate_request = DehydrateSessionRequest {
        spine_id: create_response.spine_id,
        session_id,
        committer: owner.clone(),
        session_type: Some("computation".to_string()),
    };
    let dehydrate_response: crate::types::DehydrateSessionResponse =
        rpc_call(&server, "session.dehydrate", &dehydrate_request)
            .await
            .unwrap();

    assert_eq!(dehydrate_response.session_type, "computation");
    assert!(dehydrate_response.entry_count > 0);

    let commit_request = CommitSessionRequest {
        spine_id: create_response.spine_id,
        committer: owner,
        session_id,
        session_hash: dehydrate_response.session_hash,
        vertex_count: dehydrate_response.entry_count,
    };
    let commit_response: crate::types::CommitSessionResponse =
        rpc_call(&server, "session.commit", &commit_request)
            .await
            .unwrap();

    assert_eq!(commit_response.session_id, session_id);
    assert_eq!(commit_response.merkle_root, dehydrate_response.session_hash);
}

#[tokio::test]
async fn test_jsonrpc_dehydrate_idempotent() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Idempotent Dehydrate".to_string(),
        config: None,
    };
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let session_id = uuid::Uuid::now_v7();
    let dehydrate_request = DehydrateSessionRequest {
        spine_id: create_response.spine_id,
        session_id,
        committer: owner,
        session_type: None,
    };

    let r1: crate::types::DehydrateSessionResponse =
        rpc_call(&server, "session.dehydrate", &dehydrate_request)
            .await
            .unwrap();
    let r2: crate::types::DehydrateSessionResponse =
        rpc_call(&server, "session.dehydrate", &dehydrate_request)
            .await
            .unwrap();

    assert_eq!(r1.session_hash, r2.session_hash);
    assert_eq!(r1.entry_count, r2.entry_count);
}

#[tokio::test]
async fn test_jsonrpc_commit_braid() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Braid Test".to_string(),
        config: None,
    };
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let braid_request = CommitBraidRequest {
        spine_id: create_response.spine_id,
        committer: owner,
        braid_id: uuid::Uuid::now_v7(),
        braid_hash: [3u8; 32],
        subjects: vec![],
    };
    let result: Result<crate::types::CommitBraidResponse, _> =
        rpc_call(&server, "braid.commit", &braid_request).await;
    assert!(result.is_ok());
}
