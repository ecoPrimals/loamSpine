// SPDX-License-Identifier: AGPL-3.0-only

//! Extended JSON-RPC tests: notification handling, response structure,
//! error paths, param validation, and edge cases.

use super::*;
use crate::types::Did;
use crate::types::{
    CreateSpineRequest, GetCertificateRequest, GetTipRequest, PermanentStorageCommitRequest,
    PermanentStorageGetCommitRequest, PermanentStorageVerifyRequest, SealSpineRequest,
    VerifyInclusionProofRequest,
};

/// Helper: build a JSON-RPC request and dispatch through the handler.
async fn rpc_call<Req: serde::Serialize + Sync, Resp: serde::de::DeserializeOwned>(
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

// ========================================================================
// Notification handling (id: null), response structure, error paths
// ========================================================================

#[tokio::test]
async fn notification_request_returns_response_with_null_id() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "health.liveness".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Null,
    };
    let resp = server.handle_request(rpc_req).await;
    assert_eq!(resp.id, serde_json::Value::Null);
    assert_eq!(resp.jsonrpc, "2.0");
    assert!(resp.error.is_none());
    assert!(resp.result.is_some());
}

#[tokio::test]
async fn success_response_echoes_id_and_has_jsonrpc_version() {
    let server = LoamSpineJsonRpc::default_server();
    let req_id = serde_json::json!(42);
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "health.liveness".to_string(),
        params: serde_json::Value::Null,
        id: req_id.clone(),
    };
    let resp = server.handle_request(rpc_req).await;
    assert_eq!(resp.id, req_id);
    assert_eq!(resp.jsonrpc, "2.0");
    assert!(resp.error.is_none());
    assert!(resp.result.is_some());
}

#[tokio::test]
async fn error_response_echoes_id_and_has_jsonrpc_version() {
    let server = LoamSpineJsonRpc::default_server();
    let req_id = serde_json::json!("string-id");
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "nonexistent.method".to_string(),
        params: serde_json::Value::Null,
        id: req_id.clone(),
    };
    let resp = server.handle_request(rpc_req).await;
    assert_eq!(resp.id, req_id);
    assert_eq!(resp.jsonrpc, "2.0");
    assert!(resp.error.is_some());
    assert!(resp.result.is_none());
    assert_eq!(resp.error.as_ref().unwrap().code, -32601);
}

#[tokio::test]
async fn malformed_params_string_returns_invalid_params() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "spine.create".to_string(),
        params: serde_json::json!("not an object"),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
    assert!(
        resp.error
            .as_ref()
            .unwrap()
            .message
            .contains("invalid params")
    );
}

#[tokio::test]
async fn malformed_params_array_returns_invalid_params() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "spine.get".to_string(),
        params: serde_json::json!([1, 2, 3]),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn malformed_params_empty_object_returns_invalid_params() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "entry.get".to_string(),
        params: serde_json::json!({}),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn service_error_returns_loamspine_error_code() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkSealOwner");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Seal Error Test".to_string(),
        config: None,
    };
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let seal_request = SealSpineRequest {
        spine_id: create_response.spine_id,
        sealer: owner.clone(),
    };
    let _: crate::types::SealSpineResponse = rpc_call(&server, "spine.seal", &seal_request)
        .await
        .unwrap();

    let wrong_sealer = Did::new("did:key:z6MkWrongSealer");
    let seal_again_request = SealSpineRequest {
        spine_id: create_response.spine_id,
        sealer: wrong_sealer,
    };
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "spine.seal".to_string(),
        params: serde_json::to_value(&seal_again_request).unwrap(),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32000);
    assert!(resp.result.is_none());
}

#[tokio::test]
async fn permanence_invalid_merkle_root_returns_loamspine_error() {
    let server = LoamSpineJsonRpc::default_server();
    let commit_request = serde_json::json!({
        "session_id": uuid::Uuid::now_v7().to_string(),
        "merkle_root": "not-64-hex-chars",
        "committer_did": "did:key:z6MkTest",
        "summary": {
            "session_type": "test",
            "vertex_count": 10,
            "leaf_count": 5,
            "started_at": 0,
            "ended_at": 1,
            "outcome": "success"
        }
    });

    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "permanence.commit_session".to_string(),
        params: commit_request,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32000);
}

#[tokio::test]
async fn permanence_verify_commit_alias_works() {
    use crate::types::PermanentStorageDehydrationSummary;

    let server = LoamSpineJsonRpc::default_server();
    let commit_request = PermanentStorageCommitRequest {
        session_id: uuid::Uuid::now_v7().to_string(),
        merkle_root: "ab".repeat(32),
        committer_did: Some("did:key:z6MkPermanenceVerify".to_string()),
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
        rpc_call(&server, "permanence.commit_session", &commit_request)
            .await
            .unwrap();
    let spine_id_str = response.spine_id.unwrap();
    let entry_hash_str = response.spine_entry_hash.unwrap();
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
}

#[tokio::test]
async fn permanence_get_commit_alias_works() {
    use crate::types::PermanentStorageDehydrationSummary;

    let server = LoamSpineJsonRpc::default_server();
    let commit_request = PermanentStorageCommitRequest {
        session_id: uuid::Uuid::now_v7().to_string(),
        merkle_root: "cd".repeat(32),
        committer_did: Some("did:key:z6MkPermanenceGet".to_string()),
        summary: PermanentStorageDehydrationSummary {
            session_type: "test".to_string(),
            vertex_count: 3,
            leaf_count: 1,
            started_at: 0,
            ended_at: 1,
            outcome: "success".to_string(),
        },
    };

    let response: crate::types::PermanentStorageCommitResponse =
        rpc_call(&server, "permanence.commit_session", &commit_request)
            .await
            .unwrap();
    let spine_id_str = response.spine_id.unwrap();
    let entry_hash_str = response.spine_entry_hash.unwrap();
    let index = response.entry_index.unwrap_or(0);

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
async fn process_request_valid_json_returns_response() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#"{"jsonrpc":"2.0","method":"health.liveness","params":null,"id":1}"#;
    let response = process_request(&server, body).await;
    let parsed: JsonRpcResponse = serde_json::from_slice(&response).unwrap();
    assert_eq!(parsed.jsonrpc, "2.0");
    assert!(parsed.error.is_none());
    assert!(parsed.result.is_some());
    assert_eq!(parsed.id, serde_json::json!(1));
}

#[tokio::test]
async fn process_request_malformed_json_returns_parse_error() {
    let server = LoamSpineJsonRpc::default_server();
    let body = b"{ broken json ";
    let response = process_request(&server, body).await;
    let parsed: JsonRpcResponse = serde_json::from_slice(&response).unwrap();
    assert!(parsed.error.is_some());
    assert_eq!(parsed.error.as_ref().unwrap().code, -32700);
}

#[tokio::test]
async fn invalid_params_entry_append_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "entry.append".to_string(),
        params: serde_json::json!({"wrong": "structure"}),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn invalid_params_certificate_mint_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "certificate.mint".to_string(),
        params: serde_json::json!({"foo": "bar"}),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn invalid_params_proof_generate_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "proof.generate_inclusion".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn invalid_params_braid_commit_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "braid.commit".to_string(),
        params: serde_json::json!({}),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn invalid_params_slice_anchor_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "slice.anchor".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn invalid_params_slice_checkout_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "slice.checkout".to_string(),
        params: serde_json::json!({"missing": "fields"}),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn health_check_with_details() {
    let server = LoamSpineJsonRpc::default_server();
    let request = serde_json::json!({"include_details": true});
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "health.check".to_string(),
        params: request,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_none());
    let result = resp.result.unwrap();
    assert!(result.get("status").is_some());
    assert!(result.get("report").is_some());
}

#[tokio::test]
async fn invalid_params_certificate_get_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "certificate.get".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn invalid_params_certificate_loan_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "certificate.loan".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn invalid_params_certificate_return_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "certificate.return".to_string(),
        params: serde_json::json!({"wrong": "structure"}),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn invalid_params_entry_get_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "entry.get".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn invalid_params_entry_get_tip_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "entry.get_tip".to_string(),
        params: serde_json::json!({}),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn invalid_params_certificate_transfer_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "certificate.transfer".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn invalid_params_proof_verify_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "proof.verify_inclusion".to_string(),
        params: serde_json::json!({"wrong": "structure"}),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn invalid_params_permanence_commit_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "permanence.commit_session".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn invalid_params_permanence_verify_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "permanence.verify_commit".to_string(),
        params: serde_json::json!({}),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn invalid_params_permanence_get_commit_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "permanence.get_commit".to_string(),
        params: serde_json::json!({"spine_id": "x", "entry_hash": "y"}),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn spine_list_method_not_in_dispatch() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "spine.list".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32601);
    assert!(resp.error.as_ref().unwrap().message.contains("spine.list"));
}

// ========================================================================
// Explicit LoamSpineRpcService::new() / LoamSpineService::new() coverage
// ========================================================================

#[tokio::test]
async fn jsonrpc_with_explicit_service_construction() {
    use loam_spine_core::service::LoamSpineService as CoreService;

    let core = CoreService::new();
    let rpc_service = LoamSpineRpcService::new(core);
    let server = LoamSpineJsonRpc::new(rpc_service);

    let request = CreateSpineRequest {
        owner: Did::new("did:key:z6MkExplicit"),
        name: "Explicit Service Test".to_string(),
        config: None,
    };
    let response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &request).await.unwrap();
    assert!(!response.spine_id.is_nil());
}

#[tokio::test]
async fn entry_get_tip_on_new_spine_returns_genesis() {
    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkTipEmpty");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Tip Empty Test".to_string(),
        config: None,
    };
    let create_response: crate::types::CreateSpineResponse =
        rpc_call(&server, "spine.create", &create_request)
            .await
            .unwrap();

    let get_tip_request = GetTipRequest {
        spine_id: create_response.spine_id,
    };
    let tip_response: crate::types::GetTipResponse =
        rpc_call(&server, "entry.get_tip", &get_tip_request)
            .await
            .unwrap();
    assert!(!tip_response.tip_hash.iter().all(|&b| b == 0));
    assert!(tip_response.height <= 1, "new spine has genesis at most");
}

#[tokio::test]
async fn entry_get_tip_on_nonexistent_spine_returns_error() {
    let server = LoamSpineJsonRpc::default_server();
    let get_tip_request = GetTipRequest {
        spine_id: uuid::Uuid::nil(),
    };
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "entry.get_tip".to_string(),
        params: serde_json::to_value(&get_tip_request).unwrap(),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32000);
}

#[tokio::test]
async fn get_certificate_nonexistent_returns_not_found() {
    let server = LoamSpineJsonRpc::default_server();
    let get_request = GetCertificateRequest {
        certificate_id: uuid::Uuid::now_v7(),
    };
    let response: crate::types::GetCertificateResponse =
        rpc_call(&server, "certificate.get", &get_request)
            .await
            .unwrap();
    assert!(response.certificate.is_none());
}

#[tokio::test]
async fn proof_verify_inclusion_invalid_proof_returns_false() {
    use loam_spine_core::entry::{Entry, SpineConfig};
    use loam_spine_core::proof::InclusionProof;

    let server = LoamSpineJsonRpc::default_server();
    let owner = Did::new("did:key:z6MkProofInvalid");
    let spine_id = uuid::Uuid::now_v7();
    let entry = Entry::genesis(owner, spine_id, SpineConfig::default());
    let proof = InclusionProof::new(entry, spine_id, [0u8; 32]).unwrap();

    let verify_request = VerifyInclusionProofRequest { proof };
    let response: crate::types::VerifyInclusionProofResponse =
        rpc_call(&server, "proof.verify_inclusion", &verify_request)
            .await
            .unwrap();
    assert!(!response.valid);
}
