// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

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
