// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP Phase 3 negotiation + JH-0 method gate tests.
//!
//! Split from `tests.rs` to keep individual test files under 800 lines.

use super::*;

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

// ========================================================================
// BTSP Phase 3 negotiation (JSON-RPC wire level)
// ========================================================================

#[tokio::test]
async fn jsonrpc_btsp_negotiate_returns_null_cipher() {
    let server = LoamSpineJsonRpc::default_server();
    let resp: crate::types::BtspNegotiateResponse = rpc_call(
        &server,
        "btsp.negotiate",
        &crate::types::BtspNegotiateRequest {
            session_id: "test-session-001".into(),
            preferred_cipher: "chacha20-poly1305".into(),
            ciphers: vec!["chacha20-poly1305".into()],
            client_nonce: Some("dGVzdA==".into()),
            bond_type: Some("Covalent".into()),
        },
    )
    .await
    .unwrap();

    assert_eq!(resp.cipher, "null");
    assert!(resp.server_nonce.is_none());
}

#[tokio::test]
async fn jsonrpc_btsp_negotiate_minimal_params() {
    let server = LoamSpineJsonRpc::default_server();
    let resp: crate::types::BtspNegotiateResponse = rpc_call(
        &server,
        "btsp.negotiate",
        &serde_json::json!({ "session_id": "minimal-session" }),
    )
    .await
    .unwrap();

    assert_eq!(resp.cipher, "null");
}

#[tokio::test]
async fn jsonrpc_btsp_negotiate_returns_chacha20_with_registered_key() {
    let server = LoamSpineJsonRpc::default_server();
    server
        .service
        .register_btsp_session("keyed-session".to_string(), [0xAA; 32])
        .await;

    let resp: crate::types::BtspNegotiateResponse = rpc_call(
        &server,
        "btsp.negotiate",
        &crate::types::BtspNegotiateRequest {
            session_id: "keyed-session".into(),
            preferred_cipher: "chacha20-poly1305".into(),
            ciphers: vec!["chacha20-poly1305".into()],
            client_nonce: Some("dGVzdA==".into()),
            bond_type: Some("Ionic".into()),
        },
    )
    .await
    .unwrap();

    assert_eq!(resp.cipher, "chacha20-poly1305");
    assert!(resp.server_nonce.is_some());
}

// ============================================================================
// JH-0 Method Gate tests
// ============================================================================

#[tokio::test]
async fn auth_mode_returns_current_mode() {
    let server = LoamSpineJsonRpc::default_server();
    let resp: serde_json::Value = rpc_call_no_params(&server, "auth.mode").await.unwrap();
    assert_eq!(resp["mode"], "permissive");
    assert!(resp["public_prefixes"].is_array());
    assert!(resp["public_methods"].is_array());
}

#[tokio::test]
async fn auth_check_public_method() {
    let server = LoamSpineJsonRpc::default_server();
    let resp: serde_json::Value = rpc_call(
        &server,
        "auth.check",
        &serde_json::json!({"method": "health.check"}),
    )
    .await
    .unwrap();
    assert_eq!(resp["access"], "public");
    assert_eq!(resp["allowed"], true);
}

#[tokio::test]
async fn auth_check_protected_method_permissive() {
    let server = LoamSpineJsonRpc::default_server();
    let resp: serde_json::Value = rpc_call(
        &server,
        "auth.check",
        &serde_json::json!({"method": "spine.create"}),
    )
    .await
    .unwrap();
    assert_eq!(resp["access"], "protected");
    assert_eq!(resp["allowed"], true);
    assert_eq!(resp["mode"], "permissive");
}

#[tokio::test]
async fn auth_check_protected_method_enforced() {
    let service = LoamSpineRpcService::default_service();
    let server = LoamSpineJsonRpc::new(service, super::MethodGate::new(super::AuthMode::Enforced));
    let resp: serde_json::Value = rpc_call(
        &server,
        "auth.check",
        &serde_json::json!({"method": "spine.create"}),
    )
    .await
    .unwrap();
    assert_eq!(resp["access"], "protected");
    assert_eq!(resp["allowed"], false);
    assert_eq!(resp["mode"], "enforced");
}

#[tokio::test]
async fn auth_peer_info_returns_unauthenticated() {
    let server = LoamSpineJsonRpc::default_server();
    let resp: serde_json::Value = rpc_call_no_params(&server, "auth.peer_info").await.unwrap();
    assert_eq!(resp["authenticated"], false);
    assert!(resp["peer_id"].is_null());
}

#[tokio::test]
async fn enforced_gate_blocks_protected_method() {
    let service = LoamSpineRpcService::default_service();
    let server = LoamSpineJsonRpc::new(service, super::MethodGate::new(super::AuthMode::Enforced));
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "spine.create".to_string(),
        params: serde_json::json!({"owner": {"value": "did:key:z6Mk"}, "name": "test"}),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    let err = resp.error.unwrap();
    assert_eq!(err.code, super::method_gate::AUTH_UNAUTHORIZED);
    assert!(err.message.contains("requires authentication"));
}

#[tokio::test]
async fn enforced_gate_allows_health_check() {
    let service = LoamSpineRpcService::default_service();
    let server = LoamSpineJsonRpc::new(service, super::MethodGate::new(super::AuthMode::Enforced));
    let resp: serde_json::Value = rpc_call_no_params(&server, "health.check").await.unwrap();
    assert!(resp["status"].is_string());
}

#[tokio::test]
async fn enforced_gate_allows_auth_methods() {
    let service = LoamSpineRpcService::default_service();
    let server = LoamSpineJsonRpc::new(service, super::MethodGate::new(super::AuthMode::Enforced));
    let resp: serde_json::Value = rpc_call_no_params(&server, "auth.mode").await.unwrap();
    assert_eq!(resp["mode"], "enforced");
}

#[tokio::test]
async fn enforced_gate_allows_capabilities_list() {
    let service = LoamSpineRpcService::default_service();
    let server = LoamSpineJsonRpc::new(service, super::MethodGate::new(super::AuthMode::Enforced));
    let resp: serde_json::Value = rpc_call_no_params(&server, "capabilities.list")
        .await
        .unwrap();
    assert!(resp["methods"].is_array());
}

#[tokio::test]
async fn enforced_gate_blocks_entry_append() {
    let service = LoamSpineRpcService::default_service();
    let server = LoamSpineJsonRpc::new(service, super::MethodGate::new(super::AuthMode::Enforced));
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "entry.append".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    let err = resp.error.unwrap();
    assert_eq!(err.code, -32001);
}

#[tokio::test]
async fn enforced_gate_blocks_session_commit() {
    let service = LoamSpineRpcService::default_service();
    let server = LoamSpineJsonRpc::new(service, super::MethodGate::new(super::AuthMode::Enforced));
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "session.commit".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    let err = resp.error.unwrap();
    assert_eq!(err.code, -32001);
}
