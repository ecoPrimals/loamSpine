// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
use crate::types::{CertificateType, Did, EntryType};

#[test]
fn test_jsonrpc_creation() {
    let _server = LoamSpineJsonRpc::default_server();
}

#[test]
fn test_jsonrpc_with_service() {
    let service = LoamSpineRpcService::default_service();
    let server = LoamSpineJsonRpc::new(service);
    assert!(Arc::strong_count(&server.service) >= 1);
}

#[tokio::test]
async fn test_jsonrpc_health_check() {
    let server = LoamSpineJsonRpc::default_server();
    let request = HealthCheckRequest {
        include_details: false,
    };

    let result = LoamSpineJsonRpcApiServer::health_check(&server, request).await;
    assert!(result.is_ok());

    let response = result.unwrap_or_else(|_| panic!("unexpected error"));
    assert!(response.status.is_healthy());
}

#[tokio::test]
async fn test_jsonrpc_create_spine() {
    let server = LoamSpineJsonRpc::default_server();
    let request = CreateSpineRequest {
        owner: Did::new("did:key:z6MkTest"),
        name: "Test Spine".to_string(),
        config: None,
    };

    let result = LoamSpineJsonRpcApiServer::create_spine(&server, request).await;
    assert!(result.is_ok());

    let response = result.unwrap_or_else(|_| panic!("unexpected error"));
    assert!(!response.spine_id.is_nil());
}

#[tokio::test]
async fn test_jsonrpc_get_nonexistent_spine() {
    let server = LoamSpineJsonRpc::default_server();
    let request = GetSpineRequest {
        spine_id: uuid::Uuid::nil(),
    };

    let result = LoamSpineJsonRpcApiServer::get_spine(&server, request).await;
    assert!(result.is_ok());

    let response = result.unwrap_or_else(|_| panic!("unexpected error"));
    assert!(response.spine.is_none());
}

#[tokio::test]
async fn test_jsonrpc_seal_spine() {
    let server = LoamSpineJsonRpc::default_server();

    let owner = Did::new("did:key:z6MkTest");
    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineJsonRpcApiServer::create_spine(&server, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let seal_request = SealSpineRequest {
        spine_id: create_response.spine_id,
        sealer: owner,
    };

    let result = LoamSpineJsonRpcApiServer::seal_spine(&server, seal_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jsonrpc_mint_and_get_certificate() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Cert Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineJsonRpcApiServer::create_spine(&server, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let mint_request = MintCertificateRequest {
        spine_id: create_response.spine_id,
        cert_type: CertificateType::DigitalGame {
            platform: "steam".to_string(),
            game_id: "hl3".to_string(),
            edition: None,
        },
        owner: owner.clone(),
        metadata: None,
    };

    let mint_result = LoamSpineJsonRpcApiServer::mint_certificate(&server, mint_request).await;
    assert!(mint_result.is_ok());

    let mint_response = mint_result.unwrap_or_else(|_| panic!("mint failed"));

    let get_request = GetCertificateRequest {
        certificate_id: mint_response.certificate_id,
    };

    let get_result = LoamSpineJsonRpcApiServer::get_certificate(&server, get_request).await;
    assert!(get_result.is_ok());
}

#[tokio::test]
async fn test_jsonrpc_commit_session() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Session Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineJsonRpcApiServer::create_spine(&server, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let commit_request = CommitSessionRequest {
        spine_id: create_response.spine_id,
        committer: owner,
        session_id: uuid::Uuid::now_v7(),
        session_hash: [0u8; 32],
        vertex_count: 42,
    };

    let result = LoamSpineJsonRpcApiServer::commit_session(&server, commit_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jsonrpc_append_entry() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Entry Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineJsonRpcApiServer::create_spine(&server, create_request)
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
    let result = LoamSpineJsonRpcApiServer::append_entry(&server, append_request).await;
    assert!(result.is_ok());
    let response = result.unwrap_or_else(|_| panic!("append failed"));
    assert!(!response.entry_hash.iter().all(|&b| b == 0));
}

#[tokio::test]
async fn test_jsonrpc_get_entry_and_tip() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Get Entry Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineJsonRpcApiServer::create_spine(&server, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let append_request = AppendEntryRequest {
        spine_id: create_response.spine_id,
        entry_type: EntryType::DataAnchor {
            data_hash: [2u8; 32],
            mime_type: Some("text/plain".to_string()),
            size: 10,
        },
        committer: owner.clone(),
        payload: None,
    };
    let append_response = LoamSpineJsonRpcApiServer::append_entry(&server, append_request)
        .await
        .unwrap_or_else(|_| panic!("append failed"));

    let get_entry_request = GetEntryRequest {
        spine_id: create_response.spine_id,
        entry_hash: append_response.entry_hash,
    };
    let result = LoamSpineJsonRpcApiServer::get_entry(&server, get_entry_request).await;
    assert!(result.is_ok());
    let response = result.unwrap_or_else(|_| panic!("get_entry failed"));
    assert!(response.found);

    let get_tip_request = GetTipRequest {
        spine_id: create_response.spine_id,
    };
    let result = LoamSpineJsonRpcApiServer::get_tip(&server, get_tip_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jsonrpc_liveness_and_readiness() {
    let server = LoamSpineJsonRpc::default_server();

    let liveness = LoamSpineJsonRpcApiServer::liveness(&server).await;
    assert!(liveness.is_ok());

    let readiness = LoamSpineJsonRpcApiServer::readiness(&server).await;
    assert!(readiness.is_ok());
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
    let create_response = LoamSpineJsonRpcApiServer::create_spine(&server, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let braid_request = CommitBraidRequest {
        spine_id: create_response.spine_id,
        committer: owner,
        braid_id: uuid::Uuid::now_v7(),
        braid_hash: [3u8; 32],
        subjects: vec![],
    };
    let result = LoamSpineJsonRpcApiServer::commit_braid(&server, braid_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jsonrpc_anchor_slice() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let waypoint_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Waypoint".to_string(),
        config: None,
    };
    let waypoint_response = LoamSpineJsonRpcApiServer::create_spine(&server, waypoint_request)
        .await
        .unwrap_or_else(|_| panic!("create waypoint failed"));

    let origin_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Origin".to_string(),
        config: None,
    };
    let origin_response = LoamSpineJsonRpcApiServer::create_spine(&server, origin_request)
        .await
        .unwrap_or_else(|_| panic!("create origin failed"));

    let anchor_request = AnchorSliceRequest {
        waypoint_spine_id: waypoint_response.spine_id,
        slice_id: uuid::Uuid::now_v7(),
        origin_spine_id: origin_response.spine_id,
        committer: owner,
    };
    let result = LoamSpineJsonRpcApiServer::anchor_slice(&server, anchor_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jsonrpc_generate_and_verify_inclusion_proof() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Proof Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineJsonRpcApiServer::create_spine(&server, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let append_request = AppendEntryRequest {
        spine_id: create_response.spine_id,
        entry_type: EntryType::DataAnchor {
            data_hash: [4u8; 32],
            mime_type: Some("text/plain".to_string()),
            size: 20,
        },
        committer: owner,
        payload: None,
    };
    let append_response = LoamSpineJsonRpcApiServer::append_entry(&server, append_request)
        .await
        .unwrap_or_else(|_| panic!("append failed"));

    let gen_request = GenerateInclusionProofRequest {
        spine_id: create_response.spine_id,
        entry_hash: append_response.entry_hash,
    };
    let gen_result =
        LoamSpineJsonRpcApiServer::generate_inclusion_proof(&server, gen_request).await;
    assert!(gen_result.is_ok());

    let proof = gen_result.unwrap_or_else(|_| panic!("generate failed"));
    let verify_request = VerifyInclusionProofRequest { proof: proof.proof };
    let verify_result =
        LoamSpineJsonRpcApiServer::verify_inclusion_proof(&server, verify_request).await;
    assert!(verify_result.is_ok());
    let response = verify_result.unwrap_or_else(|_| panic!("verify failed"));
    assert!(response.valid);
}

// ========================================================================
// Permanence JSON-RPC tests
// ========================================================================

#[tokio::test]
async fn test_jsonrpc_permanence_commit_and_verify() {
    use crate::types::PermanentStorageDehydrationSummary;

    let server = LoamSpineJsonRpc::default_server();

    let commit_request = PermanentStorageCommitRequest {
        session_id: uuid::Uuid::now_v7().to_string(),
        merkle_root: "ab".repeat(32),
        committer_did: Some("did:key:z6MkTest".to_string()),
        summary: PermanentStorageDehydrationSummary {
            session_type: "test".to_string(),
            vertex_count: 10,
            leaf_count: 5,
            started_at: 0,
            ended_at: 1,
            outcome: "success".to_string(),
        },
    };

    let result =
        LoamSpineJsonRpcApiServer::permanence_commit_session(&server, commit_request.clone()).await;
    assert!(result.is_ok());

    let response = result.unwrap_or_else(|_| panic!("commit failed"));
    assert!(response.accepted);
    assert!(response.commit_id.is_some());
    assert!(response.spine_id.is_some());

    let spine_id_str = response
        .spine_id
        .clone()
        .unwrap_or_else(|| panic!("no spine_id"));
    let entry_hash_str = response
        .spine_entry_hash
        .clone()
        .unwrap_or_else(|| panic!("no entry hash"));
    let index = response.entry_index.unwrap_or(0);

    let verify_request = PermanentStorageVerifyRequest {
        spine_id: spine_id_str.clone(),
        entry_hash: entry_hash_str.clone(),
        index,
    };

    let verify_result =
        LoamSpineJsonRpcApiServer::permanence_verify_commit(&server, verify_request).await;
    assert!(verify_result.is_ok());
    assert!(verify_result.unwrap_or_else(|_| panic!("verify failed")));

    let get_request = PermanentStorageGetCommitRequest {
        spine_id: spine_id_str,
        entry_hash: entry_hash_str,
        index,
    };

    let get_result = LoamSpineJsonRpcApiServer::permanence_get_commit(&server, get_request).await;
    assert!(get_result.is_ok());
    let value = get_result.unwrap_or_else(|_| panic!("get failed"));
    assert!(!value.is_null());
}

#[tokio::test]
async fn test_jsonrpc_permanence_health_check() {
    let server = LoamSpineJsonRpc::default_server();

    let result = LoamSpineJsonRpcApiServer::permanence_health_check(&server).await;
    assert!(result.is_ok());
    assert!(result.unwrap_or_else(|_| panic!("health check failed")));
}

#[tokio::test]
async fn test_jsonrpc_legacy_permanence_delegates() {
    use crate::types::PermanentStorageDehydrationSummary;

    let server = LoamSpineJsonRpc::default_server();

    let commit_request = PermanentStorageCommitRequest {
        session_id: uuid::Uuid::now_v7().to_string(),
        merkle_root: "cd".repeat(32),
        committer_did: Some("did:key:z6MkLegacy".to_string()),
        summary: PermanentStorageDehydrationSummary {
            session_type: "test".to_string(),
            vertex_count: 5,
            leaf_count: 2,
            started_at: 0,
            ended_at: 1,
            outcome: "success".to_string(),
        },
    };

    let result =
        LoamSpineJsonRpcApiServer::permanent_storage_commit_session(&server, commit_request).await;
    assert!(result.is_ok());
    let response = result.unwrap_or_else(|_| panic!("legacy commit failed"));
    assert!(response.accepted);

    let health_result = LoamSpineJsonRpcApiServer::permanent_storage_health_check(&server).await;
    assert!(health_result.is_ok());
    assert!(health_result.unwrap_or_else(|_| panic!("legacy health failed")));
}

#[tokio::test]
async fn semantic_commit_session_alias() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkSemanticTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Semantic Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineJsonRpcApiServer::create_spine(&server, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let request = CommitSessionRequest {
        spine_id: create_response.spine_id,
        session_id: uuid::Uuid::now_v7(),
        session_hash: [0u8; 32],
        vertex_count: 10,
        committer: owner,
    };
    let result = LoamSpineJsonRpcApiServer::commit_session_semantic(&server, request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn capability_list_method() {
    let server = LoamSpineJsonRpc::default_server();
    let result = LoamSpineJsonRpcApiServer::capability_list(&server).await;
    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.get("capabilities").is_some());
    assert!(value.get("primal").is_some());
    assert_eq!(value["primal"], "loamspine");
}
