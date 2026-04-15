// SPDX-License-Identifier: AGPL-3.0-or-later

#![expect(
    clippy::panic,
    reason = "test assertions use unwrap_or_else(panic) for failure clarity"
)]

use super::*;

#[test]
fn test_server_creation() {
    let _server = LoamSpineTarpcServer::default_server();
}

#[test]
fn test_server_with_service() {
    let service = LoamSpineRpcService::default_service();
    let server = LoamSpineTarpcServer::new(service);
    assert!(Arc::strong_count(&server.service) >= 1);
}

#[test]
fn test_server_clone() {
    let server = LoamSpineTarpcServer::default_server();
    let cloned = server.clone();

    assert!(Arc::ptr_eq(&server.service, &cloned.service));
}

#[tokio::test]
async fn test_tarpc_health_check() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let request = HealthCheckRequest {
        include_details: false,
    };

    let result = LoamSpineRpc::health_check(server, ctx, request).await;
    assert!(result.is_ok());

    let response = result.unwrap_or_else(|_| panic!("unexpected error"));
    assert!(response.status.is_healthy());
}

#[tokio::test]
async fn test_tarpc_create_spine() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let request = CreateSpineRequest {
        owner: Did::new("did:key:z6MkTest"),
        name: "Test Spine".to_string(),
        config: None,
    };

    let result = LoamSpineRpc::create_spine(server, ctx, request).await;
    assert!(result.is_ok());

    let response = result.unwrap_or_else(|_| panic!("unexpected error"));
    assert!(!response.spine_id.is_nil());
}

#[tokio::test]
async fn test_tarpc_get_nonexistent_spine() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let request = GetSpineRequest {
        spine_id: uuid::Uuid::nil(),
    };

    let result = LoamSpineRpc::get_spine(server, ctx, request).await;
    assert!(result.is_ok());

    let response = result.unwrap_or_else(|_| panic!("unexpected error"));
    assert!(response.spine.is_none());
}

#[tokio::test]
async fn test_tarpc_seal_spine() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let seal_request = SealSpineRequest {
        spine_id: create_response.spine_id,
        sealer: owner,
    };

    let result = LoamSpineRpc::seal_spine(server, ctx, seal_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tarpc_append_entry() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Entry Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let append_request = AppendEntryRequest {
        spine_id: create_response.spine_id,
        entry_type: EntryType::DataAnchor {
            data_hash: [0u8; 32],
            mime_type: Some("text/plain".to_string()),
            size: 100,
        },
        committer: owner,
        payload: None,
    };

    let result = LoamSpineRpc::append_entry(server, ctx, append_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tarpc_mint_certificate() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Cert Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let mint_request = MintCertificateRequest {
        spine_id: create_response.spine_id,
        cert_type: CertificateType::DigitalGame {
            platform: "steam".to_string(),
            game_id: "hl3".to_string(),
            edition: None,
        },
        owner,
        metadata: None,
    };

    let result = LoamSpineRpc::mint_certificate(server, ctx, mint_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tarpc_commit_session() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Session Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let commit_request = CommitSessionRequest {
        spine_id: create_response.spine_id,
        committer: owner,
        session_id: uuid::Uuid::now_v7(),
        session_hash: [0u8; 32],
        vertex_count: 42,
    };

    let result = LoamSpineRpc::commit_session(server, ctx, commit_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tarpc_commit_braid() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Braid Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let braid_request = CommitBraidRequest {
        spine_id: create_response.spine_id,
        committer: owner,
        braid_id: uuid::Uuid::now_v7(),
        braid_hash: [2u8; 32],
        subjects: vec![],
    };

    let result = LoamSpineRpc::commit_braid(server, ctx, braid_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tarpc_get_entry() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Entry Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let append_request = AppendEntryRequest {
        spine_id: create_response.spine_id,
        entry_type: EntryType::DataAnchor {
            data_hash: [1u8; 32],
            mime_type: Some("text/plain".to_string()),
            size: 50,
        },
        committer: owner.clone(),
        payload: None,
    };
    let append_response = LoamSpineRpc::append_entry(server.clone(), ctx, append_request)
        .await
        .unwrap_or_else(|_| panic!("append failed"));

    let get_request = GetEntryRequest {
        spine_id: create_response.spine_id,
        entry_hash: append_response.entry_hash,
    };
    let result = LoamSpineRpc::get_entry(server.clone(), ctx, get_request).await;
    assert!(result.is_ok());
    let response = result.unwrap_or_else(|_| panic!("get_entry failed"));
    assert!(response.found);

    let get_nonexistent = GetEntryRequest {
        spine_id: create_response.spine_id,
        entry_hash: [99u8; 32],
    };
    let result = LoamSpineRpc::get_entry(server, ctx, get_nonexistent).await;
    assert!(result.is_ok());
    let response = result.unwrap_or_else(|_| panic!("get_entry failed"));
    assert!(!response.found);
}

#[tokio::test]
async fn test_tarpc_get_tip() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Tip Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let get_tip_request = GetTipRequest {
        spine_id: create_response.spine_id,
    };
    let result = LoamSpineRpc::get_tip(server, ctx, get_tip_request).await;
    assert!(result.is_ok());
    let response = result.unwrap_or_else(|_| panic!("get_tip failed"));
    assert!(!response.tip_hash.iter().all(|&b| b == 0));
}

// Compound multi-step tarpc tests (transfers, loans, waypoints, proofs,
// error paths) extracted to tarpc_server_tests_compound.rs by domain.
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "test assertions use expect/panic for failure clarity"
)]
#[path = "tarpc_server_tests_compound.rs"]
mod compound;

// Server lifecycle, config, and TCP integration tests extracted to
// tarpc_server_tests_lifecycle.rs for 1000-line compliance.
#[expect(
    clippy::expect_used,
    reason = "test assertions use expect for failure clarity"
)]
#[path = "tarpc_server_tests_lifecycle.rs"]
mod lifecycle;
