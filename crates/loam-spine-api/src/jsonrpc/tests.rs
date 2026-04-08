// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::types::{
    AnchorSliceRequest, AppendEntryRequest, CommitBraidRequest, CommitSessionRequest,
    CreateSpineRequest, GenerateInclusionProofRequest, GetCertificateRequest, GetEntryRequest,
    GetSpineRequest, GetTipRequest, MintCertificateRequest, SealSpineRequest,
    VerifyInclusionProofRequest,
};
use crate::types::{CertificateType, Did, EntryType};

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
    let _server = LoamSpineJsonRpc::new(service);
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
    assert_eq!(liveness.status, "alive");

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

// Permanence operations, certificate transfer/loan lifecycle, slice checkout,
// and legacy method aliases live in `tests_permanence_cert.rs` (domain-focused split).

#[tokio::test]
async fn capability_list_method() {
    let server = LoamSpineJsonRpc::default_server();
    let value: serde_json::Value = rpc_call_no_params(&server, "capability.list")
        .await
        .unwrap();
    assert_eq!(value["primal"], "loamspine");
    assert!(value["version"].is_string());
    assert!(value["capabilities"].is_array());
    assert!(value["methods"].is_array(), "methods must be flat string array per Wire Standard L2");
    let methods = value["methods"].as_array().unwrap();
    assert!(methods.iter().all(serde_json::Value::is_string), "all methods must be strings");
    let method_strs: Vec<&str> = methods.iter().filter_map(|v| v.as_str()).collect();
    assert!(method_strs.contains(&"spine.create"));
    assert!(method_strs.contains(&"identity.get"));
    assert!(value["provided_capabilities"].is_array());
    assert!(value["consumed_capabilities"].is_array());
    assert!(value["cost_estimates"].is_object());
    assert!(value["operation_dependencies"].is_object());
}

#[tokio::test]
async fn identity_get_method() {
    let server = LoamSpineJsonRpc::default_server();
    let value: serde_json::Value = rpc_call_no_params(&server, "identity.get")
        .await
        .unwrap();
    assert_eq!(value["primal"], "loamspine");
    assert!(value["version"].is_string());
    assert_eq!(value["domain"], "permanence");
    assert_eq!(value["license"], "AGPL-3.0-or-later");
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

// ========================================================================
// Additional coverage: JsonRpcResponse constructors, error paths, dispatch
// ========================================================================

#[test]
fn json_rpc_response_success_constructor() {
    let id = serde_json::Value::Number(99.into());
    let result = serde_json::json!({"ok": true});
    let resp = JsonRpcResponse::success(id.clone(), result.clone());
    assert_eq!(resp.jsonrpc, "2.0");
    assert!(resp.error.is_none());
    assert_eq!(resp.result.unwrap(), result);
    assert_eq!(resp.id, id);
}

#[test]
fn json_rpc_response_error_constructor() {
    let id = serde_json::Value::Null;
    let resp = JsonRpcResponse::error(id, -32601, "method not found: foo".to_string());
    assert_eq!(resp.jsonrpc, "2.0");
    assert!(resp.result.is_none());
    assert!(resp.error.is_some());
    let err = resp.error.unwrap();
    assert_eq!(err.code, -32601);
    assert_eq!(err.message, "method not found: foo");
}

#[tokio::test]
async fn invalid_method_returns_method_not_found() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "invalid.method.name".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32601);
    assert!(
        resp.error
            .as_ref()
            .unwrap()
            .message
            .contains("method not found")
    );
}

#[tokio::test]
async fn invalid_json_returns_parse_error() {
    let server = LoamSpineJsonRpc::default_server();
    let body = b"{ invalid json }";
    let response = process_request(&server, body).await;
    let parsed: JsonRpcResponse = serde_json::from_slice(&response).unwrap();
    assert!(parsed.error.is_some());
    assert_eq!(parsed.error.as_ref().unwrap().code, -32700);
    assert!(
        parsed
            .error
            .as_ref()
            .unwrap()
            .message
            .contains("parse error")
    );
}

#[tokio::test]
async fn invalid_jsonrpc_version_string_accepted() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "1.0".to_string(),
        method: "health.liveness".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_none());
    let liveness: crate::health::LivenessProbe =
        serde_json::from_value(resp.result.unwrap()).unwrap();
    assert_eq!(liveness.status, "alive");
}

#[tokio::test]
async fn null_params_on_method_requiring_params_returns_invalid_params() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "spine.create".to_string(),
        params: serde_json::Value::Null,
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
async fn batch_request_returns_batch_response() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#"[{"jsonrpc":"2.0","method":"health.liveness","id":1},{"jsonrpc":"2.0","method":"health.liveness","id":2}]"#;
    let response = process_request(&server, body).await;
    let parsed: Vec<JsonRpcResponse> = serde_json::from_slice(&response).unwrap();
    assert_eq!(parsed.len(), 2);
    assert!(parsed[0].result.is_some());
    assert!(parsed[1].result.is_some());
}

#[tokio::test]
async fn batch_empty_returns_invalid_request() {
    let server = LoamSpineJsonRpc::default_server();
    let body = b"[]";
    let response = process_request(&server, body).await;
    let parsed: JsonRpcResponse = serde_json::from_slice(&response).unwrap();
    assert!(parsed.error.is_some());
    assert_eq!(parsed.error.as_ref().unwrap().code, -32600);
}

#[tokio::test]
async fn batch_notification_produces_no_response() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#"[{"jsonrpc":"2.0","method":"health.liveness"}]"#;
    let response = process_request(&server, body).await;
    assert!(response.is_empty());
}

#[tokio::test]
async fn empty_body_returns_parse_error() {
    let server = LoamSpineJsonRpc::default_server();
    let body = b"";
    let response = process_request(&server, body).await;
    let parsed: JsonRpcResponse = serde_json::from_slice(&response).unwrap();
    assert!(parsed.error.is_some());
    assert_eq!(parsed.error.as_ref().unwrap().code, -32700);
}

#[tokio::test]
async fn non_object_non_array_returns_parse_error() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#""just a string""#;
    let response = process_request(&server, body).await;
    let parsed: JsonRpcResponse = serde_json::from_slice(&response).unwrap();
    assert!(parsed.error.is_some());
    assert_eq!(parsed.error.as_ref().unwrap().code, -32700);
}

#[tokio::test]
async fn numeric_json_value_returns_parse_error() {
    let server = LoamSpineJsonRpc::default_server();
    let body = b"42";
    let response = process_request(&server, body).await;
    let parsed: JsonRpcResponse = serde_json::from_slice(&response).unwrap();
    assert!(parsed.error.is_some());
    assert_eq!(parsed.error.as_ref().unwrap().code, -32700);
}

// ============================================================================
// Public Chain Anchor dispatch tests
// ============================================================================

#[tokio::test]
async fn anchor_publish_and_verify_dispatch() {
    use crate::types::{
        AnchorPublishRequest, AnchorPublishResponse, AnchorVerifyRequest, AnchorVerifyResponse,
    };
    use loam_spine_core::entry::AnchorTarget;

    let server = LoamSpineJsonRpc::default_server();

    let create_resp: crate::types::CreateSpineResponse = rpc_call(
        &server,
        "spine.create",
        &CreateSpineRequest {
            name: "anchor-dispatch-test".into(),
            owner: Did::new("did:key:z6MkAnchor"),
            config: None,
        },
    )
    .await
    .unwrap();

    let publish_resp: AnchorPublishResponse = rpc_call(
        &server,
        "anchor.publish",
        &AnchorPublishRequest {
            spine_id: create_resp.spine_id,
            anchor_target: AnchorTarget::DataCommons {
                commons_id: "test-commons".into(),
            },
            tx_ref: "bafytest123".into(),
            block_height: 0,
            anchor_timestamp: loam_spine_core::types::Timestamp::now(),
        },
    )
    .await
    .unwrap();

    assert_ne!(publish_resp.entry_hash, [0u8; 32]);
    assert_ne!(publish_resp.state_hash, [0u8; 32]);

    let verify_resp: AnchorVerifyResponse = rpc_call(
        &server,
        "anchor.verify",
        &AnchorVerifyRequest {
            spine_id: create_resp.spine_id,
            anchor_entry_hash: Some(publish_resp.entry_hash),
        },
    )
    .await
    .unwrap();

    assert!(verify_resp.verified);
    assert_eq!(verify_resp.tx_ref, "bafytest123");
    assert_eq!(verify_resp.state_hash, publish_resp.state_hash);
}

#[tokio::test]
async fn anchor_verify_latest_dispatch() {
    use crate::types::{AnchorPublishRequest, AnchorVerifyRequest, AnchorVerifyResponse};
    use loam_spine_core::entry::AnchorTarget;

    let server = LoamSpineJsonRpc::default_server();

    let create_resp: crate::types::CreateSpineResponse = rpc_call(
        &server,
        "spine.create",
        &CreateSpineRequest {
            name: "anchor-latest-test".into(),
            owner: Did::new("did:key:z6MkLatest"),
            config: None,
        },
    )
    .await
    .unwrap();

    let _: crate::types::AnchorPublishResponse = rpc_call(
        &server,
        "anchor.publish",
        &AnchorPublishRequest {
            spine_id: create_resp.spine_id,
            anchor_target: AnchorTarget::Ethereum,
            tx_ref: "0xdeadbeef".into(),
            block_height: 42,
            anchor_timestamp: loam_spine_core::types::Timestamp::now(),
        },
    )
    .await
    .unwrap();

    let verify_resp: AnchorVerifyResponse = rpc_call(
        &server,
        "anchor.verify",
        &AnchorVerifyRequest {
            spine_id: create_resp.spine_id,
            anchor_entry_hash: None,
        },
    )
    .await
    .unwrap();

    assert!(verify_resp.verified);
    assert_eq!(verify_resp.tx_ref, "0xdeadbeef");
    assert_eq!(verify_resp.block_height, 42);
}

// Protocol-level, UDS, TCP, and infrastructure tests split into tests_protocol.rs
