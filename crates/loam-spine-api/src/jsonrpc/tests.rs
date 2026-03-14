// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
use crate::types::{
    AnchorSliceRequest, AppendEntryRequest, CommitBraidRequest, CommitSessionRequest,
    CreateSpineRequest, GenerateInclusionProofRequest, GetCertificateRequest, GetEntryRequest,
    GetSpineRequest, GetTipRequest, MintCertificateRequest, PermanentStorageCommitRequest,
    PermanentStorageGetCommitRequest, PermanentStorageVerifyRequest, SealSpineRequest,
    VerifyInclusionProofRequest,
};
use crate::types::{CertificateType, Did, EntryType};

/// Helper: build a JSON-RPC request and dispatch through the handler.
async fn rpc_call<Req: serde::Serialize, Resp: serde::de::DeserializeOwned>(
    server: &LoamSpineJsonRpc,
    method: &str,
    request: &Req,
) -> Result<Resp, String> {
    let params = serde_json::to_value(request).unwrap();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: serde_json::Value::Number(1.into()),
    };
    let rpc_resp = server.handle_request(rpc_req).await;
    if let Some(err) = rpc_resp.error {
        return Err(err.message);
    }
    serde_json::from_value(rpc_resp.result.unwrap_or_default()).map_err(|e| e.to_string())
}

/// Helper: call a no-params method.
async fn rpc_call_no_params<Resp: serde::de::DeserializeOwned>(
    server: &LoamSpineJsonRpc,
    method: &str,
) -> Result<Resp, String> {
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let rpc_resp = server.handle_request(rpc_req).await;
    if let Some(err) = rpc_resp.error {
        return Err(err.message);
    }
    serde_json::from_value(rpc_resp.result.unwrap_or_default()).map_err(|e| e.to_string())
}

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
    let request = crate::types::HealthCheckRequest {
        include_details: false,
    };

    let response: crate::types::HealthCheckResponse =
        rpc_call(&server, "health.check", &request).await.unwrap();
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

    let response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &request).await.unwrap();
    assert!(!response.spine_id.is_nil());
}

#[tokio::test]
async fn test_jsonrpc_get_nonexistent_spine() {
    let server = LoamSpineJsonRpc::default_server();
    let request = GetSpineRequest {
        spine_id: uuid::Uuid::nil(),
    };

    let response: crate::types::GetSpineResponse =
        rpc_call(&server, "spine.get", &request).await.unwrap();
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
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let seal_request = SealSpineRequest {
        spine_id: create_response.spine_id,
        sealer: owner,
    };

    let result: Result<crate::types::SealSpineResponse, _> =
        rpc_call(&server, "spine.seal", &seal_request).await;
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
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

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

    let mint_response: crate::types::MintCertificateResponse =
        rpc_call(&server, "certificate.mint", &mint_request)
            .await
            .unwrap();

    let get_request = GetCertificateRequest {
        certificate_id: mint_response.certificate_id,
    };

    let result: Result<crate::types::GetCertificateResponse, _> =
        rpc_call(&server, "certificate.get", &get_request).await;
    assert!(result.is_ok());
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
async fn test_jsonrpc_append_entry() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Entry Test".to_string(),
        config: None,
    };
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

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
    let response: crate::types::AppendEntryResponse =
        rpc_call(&server, "entry.append", &append_request)
            .await
            .unwrap();
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
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

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
    let append_response: crate::types::AppendEntryResponse =
        rpc_call(&server, "entry.append", &append_request)
            .await
            .unwrap();

    let get_entry_request = GetEntryRequest {
        spine_id: create_response.spine_id,
        entry_hash: append_response.entry_hash,
    };
    let response: crate::types::GetEntryResponse =
        rpc_call(&server, "entry.get", &get_entry_request)
            .await
            .unwrap();
    assert!(response.found);

    let get_tip_request = GetTipRequest {
        spine_id: create_response.spine_id,
    };
    let result: Result<crate::types::GetTipResponse, _> =
        rpc_call(&server, "entry.get_tip", &get_tip_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jsonrpc_liveness_and_readiness() {
    let server = LoamSpineJsonRpc::default_server();

    let liveness: crate::health::LivenessProbe = rpc_call_no_params(&server, "health.liveness")
        .await
        .unwrap();
    assert!(liveness.alive);

    let readiness: crate::health::ReadinessProbe = rpc_call_no_params(&server, "health.readiness")
        .await
        .unwrap();
    assert!(readiness.ready);
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

#[tokio::test]
async fn test_jsonrpc_anchor_slice() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTest");

    let waypoint_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Waypoint".to_string(),
        config: None,
    };
    let waypoint_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &waypoint_request)
            .await
            .unwrap();

    let origin_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Origin".to_string(),
        config: None,
    };
    let origin_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &origin_request)
            .await
            .unwrap();

    let anchor_request = AnchorSliceRequest {
        waypoint_spine_id: waypoint_response.spine_id,
        slice_id: uuid::Uuid::now_v7(),
        origin_spine_id: origin_response.spine_id,
        committer: owner,
    };
    let result: Result<crate::types::AnchorSliceResponse, _> =
        rpc_call(&server, "slice.anchor", &anchor_request).await;
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
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

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
    let append_response: crate::types::AppendEntryResponse =
        rpc_call(&server, "entry.append", &append_request)
            .await
            .unwrap();

    let gen_request = GenerateInclusionProofRequest {
        spine_id: create_response.spine_id,
        entry_hash: append_response.entry_hash,
    };
    let proof: crate::types::GenerateInclusionProofResponse =
        rpc_call(&server, "proof.generate_inclusion", &gen_request)
            .await
            .unwrap();

    let verify_request = VerifyInclusionProofRequest { proof: proof.proof };
    let response: crate::types::VerifyInclusionProofResponse =
        rpc_call(&server, "proof.verify_inclusion", &verify_request)
            .await
            .unwrap();
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

    let response: crate::types::PermanentStorageCommitResponse =
        rpc_call(&server, "permanence.commit_session", &commit_request)
            .await
            .unwrap();
    assert!(response.accepted);
    assert!(response.commit_id.is_some());
    assert!(response.spine_id.is_some());

    let spine_id_str = response.spine_id.clone().unwrap();
    let entry_hash_str = response.spine_entry_hash.clone().unwrap();
    let index = response.entry_index.unwrap_or(0);

    let verify_request = PermanentStorageVerifyRequest {
        spine_id: spine_id_str.clone(),
        entry_hash: entry_hash_str.clone(),
        index,
    };

    let verified: bool = rpc_call(&server, "permanence.verify_commit", &verify_request)
        .await
        .unwrap();
    assert!(verified);

    let get_request = PermanentStorageGetCommitRequest {
        spine_id: spine_id_str,
        entry_hash: entry_hash_str,
        index,
    };

    let value: serde_json::Value = rpc_call(&server, "permanence.get_commit", &get_request)
        .await
        .unwrap();
    assert!(!value.is_null());
}

#[tokio::test]
async fn test_jsonrpc_permanence_health_check() {
    let server = LoamSpineJsonRpc::default_server();

    let healthy: bool = rpc_call_no_params(&server, "permanence.health_check")
        .await
        .unwrap();
    assert!(healthy);
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

    let response: crate::types::PermanentStorageCommitResponse =
        rpc_call(&server, "permanent-storage.commitSession", &commit_request)
            .await
            .unwrap();
    assert!(response.accepted);

    let healthy: bool = rpc_call_no_params(&server, "permanent-storage.healthCheck")
        .await
        .unwrap();
    assert!(healthy);
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
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let request = CommitSessionRequest {
        spine_id: create_response.spine_id,
        session_id: uuid::Uuid::now_v7(),
        session_hash: [0u8; 32],
        vertex_count: 10,
        committer: owner,
    };
    let result: Result<crate::types::CommitSessionResponse, _> =
        rpc_call(&server, "commit.session", &request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn capability_list_method() {
    let server = LoamSpineJsonRpc::default_server();
    let value: serde_json::Value = rpc_call_no_params(&server, "capability.list")
        .await
        .unwrap();
    assert!(value.get("capabilities").is_some());
    assert!(value.get("primal").is_some());
    assert_eq!(value["primal"], "loamspine");
}

#[tokio::test]
async fn method_not_found_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "nonexistent.method".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.unwrap().code, -32601);
}

#[test]
fn json_rpc_types_serde_roundtrip() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "test.method".to_string(),
        params: serde_json::json!({"key": "value"}),
        id: serde_json::Value::Number(42.into()),
    };
    let json = serde_json::to_string(&req).unwrap();
    let parsed: JsonRpcRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.method, "test.method");

    let resp =
        JsonRpcResponse::success(serde_json::Value::Number(1.into()), serde_json::json!(true));
    let json = serde_json::to_string(&resp).unwrap();
    let parsed: JsonRpcResponse = serde_json::from_str(&json).unwrap();
    assert!(parsed.error.is_none());
    assert_eq!(parsed.result.unwrap(), serde_json::Value::Bool(true));
}
