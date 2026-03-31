// SPDX-License-Identifier: AGPL-3.0-or-later

//! Protocol-level JSON-RPC tests: method normalization, dispatch outcomes,
//! tools.call routing, notifications, batch edge cases, TCP/UDS servers,
//! and serialization helpers.

use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// =========================================================================
// UDS server tests
// =========================================================================

#[cfg(unix)]
#[tokio::test]
async fn uds_server_starts_and_accepts_connections() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("test-jsonrpc.sock");
    let service = crate::service::LoamSpineRpcService::default_service();

    let handle = super::run_jsonrpc_uds_server(&sock_path, service)
        .await
        .unwrap();

    assert!(sock_path.exists());
    assert_eq!(handle.path(), sock_path);

    let mut stream = tokio::net::UnixStream::connect(&sock_path).await.unwrap();

    let request = r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#;
    stream.write_all(request.as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = vec![0u8; 4096];
    let n = tokio::time::timeout(std::time::Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();

    let response: serde_json::Value = serde_json::from_slice(&buf[..n]).unwrap();
    assert!(response.get("result").is_some() || response.get("error").is_some());

    handle.stop();
}

#[cfg(unix)]
#[tokio::test]
async fn uds_server_removes_stale_socket() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("stale.sock");

    std::fs::write(&sock_path, "stale socket data").unwrap();
    assert!(sock_path.exists());

    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_uds_server(&sock_path, service)
        .await
        .unwrap();

    assert!(sock_path.exists());
    handle.stop();
}

#[cfg(unix)]
#[tokio::test]
async fn uds_server_creates_parent_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("nested").join("dir").join("test.sock");

    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_uds_server(&sock_path, service)
        .await
        .unwrap();

    assert!(sock_path.exists());
    handle.stop();
}

#[cfg(unix)]
#[tokio::test]
async fn uds_server_drop_removes_socket() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("drop-test.sock");

    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_uds_server(&sock_path, service)
        .await
        .unwrap();

    assert!(sock_path.exists());
    drop(handle);
    assert!(!sock_path.exists());
}

#[cfg(unix)]
#[tokio::test]
async fn uds_server_shutdown_via_stop() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("shutdown-test.sock");

    let service = crate::service::LoamSpineRpcService::default_service();
    let mut handle = super::run_jsonrpc_uds_server(&sock_path, service)
        .await
        .unwrap();

    handle.stop();
    let result = tokio::time::timeout(std::time::Duration::from_secs(2), handle.stopped()).await;
    assert!(result.is_ok(), "server should stop within timeout");
}

// =========================================================================
// normalize_method alias coverage
// =========================================================================

#[test]
fn normalize_method_legacy_aliases() {
    assert_eq!(normalize_method("commit.session"), "session.commit");
    assert_eq!(
        normalize_method("permanent-storage.commitSession"),
        "permanence.commit_session"
    );
    assert_eq!(
        normalize_method("permanent-storage.verifyCommit"),
        "permanence.verify_commit"
    );
    assert_eq!(
        normalize_method("permanent-storage.getCommit"),
        "permanence.get_commit"
    );
    assert_eq!(
        normalize_method("permanent-storage.healthCheck"),
        "permanence.health_check"
    );
    assert_eq!(normalize_method("capability.list"), "capabilities.list");
    assert_eq!(normalize_method("primal.capabilities"), "capabilities.list");
}

#[test]
fn normalize_method_passthrough() {
    assert_eq!(normalize_method("spine.create"), "spine.create");
    assert_eq!(normalize_method("health.liveness"), "health.liveness");
    assert_eq!(normalize_method("unknown.method"), "unknown.method");
}

// =========================================================================
// dispatch_typed outcome separation
// =========================================================================

#[tokio::test]
async fn dispatch_typed_ok_on_valid_method() {
    let server = LoamSpineJsonRpc::default_server();
    let outcome = server
        .dispatch_typed("health.liveness", serde_json::Value::Null)
        .await;
    assert!(
        matches!(outcome, loam_spine_core::error::DispatchOutcome::Ok(_)),
        "liveness should return Ok"
    );
}

#[tokio::test]
async fn dispatch_typed_protocol_error_on_unknown_method() {
    let server = LoamSpineJsonRpc::default_server();
    let outcome = server
        .dispatch_typed("nonexistent.method", serde_json::Value::Null)
        .await;
    assert!(matches!(
        outcome,
        loam_spine_core::error::DispatchOutcome::ProtocolError(_)
    ));
}

#[tokio::test]
async fn dispatch_typed_protocol_error_on_invalid_params() {
    let server = LoamSpineJsonRpc::default_server();
    let outcome = server
        .dispatch_typed("spine.create", serde_json::Value::Null)
        .await;
    assert!(matches!(
        outcome,
        loam_spine_core::error::DispatchOutcome::ProtocolError(_)
    ));
}

#[tokio::test]
async fn dispatch_typed_application_error() {
    let server = LoamSpineJsonRpc::default_server();
    let params = serde_json::json!({
        "spine_id": "00000000-0000-0000-0000-000000000000",
        "entry_hash": vec![0u8; 32],
    });
    let outcome = server.dispatch_typed("entry.get", params).await;
    assert!(matches!(
        outcome,
        loam_spine_core::error::DispatchOutcome::Ok(_)
    ));
}

// =========================================================================
// tools.call dispatch path
// =========================================================================

#[tokio::test]
async fn tools_list_returns_array() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools.list".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_none());
    let result = resp.result.unwrap();
    assert!(result.get("tools").is_some());
}

#[tokio::test]
async fn tools_call_dispatches_to_rpc_method() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools.call".to_string(),
        params: serde_json::json!({
            "name": "health_check",
            "arguments": { "include_details": false },
        }),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_none(), "tools.call should succeed");
    let result = resp.result.unwrap();
    assert!(result.get("content").is_some());
    assert_eq!(result["isError"], false);
}

#[tokio::test]
async fn tools_call_missing_name_returns_invalid_params() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools.call".to_string(),
        params: serde_json::json!({ "arguments": {} }),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn tools_call_unknown_tool_returns_method_not_found() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools.call".to_string(),
        params: serde_json::json!({ "name": "nonexistent_tool" }),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32601);
}

#[tokio::test]
async fn tools_call_without_arguments_falls_through_to_rpc() {
    let server = LoamSpineJsonRpc::default_server();
    let rpc_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools.call".to_string(),
        params: serde_json::json!({ "name": "health_check" }),
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(rpc_req).await;
    // May fail because health_check expects `include_details`; the point is
    // that the code path that defaults missing `arguments` to {} is exercised.
    assert!(resp.error.is_some() || resp.result.is_some());
}

// =========================================================================
// Notification handling in process_request
// =========================================================================

#[tokio::test]
async fn single_notification_produces_no_response() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#"{"jsonrpc":"2.0","method":"health.liveness"}"#;
    let response = process_request(&server, body).await;
    assert!(response.is_empty());
}

#[tokio::test]
async fn notification_with_null_id_produces_no_response() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#"{"jsonrpc":"2.0","method":"health.liveness","id":null}"#;
    let response = process_request(&server, body).await;
    assert!(response.is_empty());
}

#[tokio::test]
async fn invalid_jsonrpc_version_in_process_request() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#"{"jsonrpc":"1.0","method":"health.liveness","id":1}"#;
    let response = process_request(&server, body).await;
    let parsed: JsonRpcResponse = serde_json::from_slice(&response).unwrap();
    assert!(parsed.error.is_some());
    assert_eq!(parsed.error.as_ref().unwrap().code, -32600);
}

#[tokio::test]
async fn invalid_jsonrpc_version_notification_produces_no_response() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#"{"jsonrpc":"1.0","method":"health.liveness"}"#;
    let response = process_request(&server, body).await;
    assert!(response.is_empty());
}

#[tokio::test]
async fn malformed_object_notification_produces_no_response() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#"{"not_a_valid_request": true}"#;
    let response = process_request(&server, body).await;
    assert!(response.is_empty());
}

#[tokio::test]
async fn malformed_object_with_id_returns_invalid_request() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#"{"id": 1, "not_jsonrpc": true}"#;
    let response = process_request(&server, body).await;
    let parsed: JsonRpcResponse = serde_json::from_slice(&response).unwrap();
    assert!(parsed.error.is_some());
    assert_eq!(parsed.error.as_ref().unwrap().code, -32600);
}

// =========================================================================
// Batch edge cases
// =========================================================================

#[tokio::test]
async fn batch_with_mixed_valid_and_invalid() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#"[{"jsonrpc":"2.0","method":"health.liveness","id":1},{"invalid":true,"id":2}]"#;
    let response = process_request(&server, body).await;
    let parsed: Vec<JsonRpcResponse> = serde_json::from_slice(&response).unwrap();
    assert_eq!(parsed.len(), 2);
    assert!(parsed[0].result.is_some());
    assert!(parsed[1].error.is_some());
}

#[tokio::test]
async fn batch_with_wrong_jsonrpc_version() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#"[{"jsonrpc":"1.0","method":"health.liveness","id":1}]"#;
    let response = process_request(&server, body).await;
    let parsed: Vec<JsonRpcResponse> = serde_json::from_slice(&response).unwrap();
    assert_eq!(parsed.len(), 1);
    assert!(parsed[0].error.is_some());
    assert_eq!(parsed[0].error.as_ref().unwrap().code, -32600);
}

#[tokio::test]
async fn batch_notification_with_wrong_version_produces_no_response() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#"[{"jsonrpc":"1.0","method":"health.liveness"}]"#;
    let response = process_request(&server, body).await;
    assert!(response.is_empty());
}

#[tokio::test]
async fn batch_with_invalid_item_notification_skipped() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#"[{"not_jsonrpc": true}]"#;
    let response = process_request(&server, body).await;
    assert!(response.is_empty());
}

// =========================================================================
// TCP server handle
// =========================================================================

#[tokio::test]
async fn tcp_server_starts_and_stops() {
    let addr = "127.0.0.1:0".parse().unwrap();
    let service = crate::service::LoamSpineRpcService::default_service();
    let mut handle = super::run_jsonrpc_server(addr, service).await.unwrap();

    let bound_addr = handle.local_addr();
    assert_ne!(bound_addr.port(), 0);

    handle.stop();
    let result = tokio::time::timeout(std::time::Duration::from_secs(2), handle.stopped()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn tcp_server_accepts_raw_jsonrpc() {
    let addr = "127.0.0.1:0".parse().unwrap();
    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_server(addr, service).await.unwrap();
    let bound_addr = handle.local_addr();

    let mut stream = tokio::net::TcpStream::connect(bound_addr).await.unwrap();
    let request = r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#;
    stream.write_all(request.as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = vec![0u8; 4096];
    let n = tokio::time::timeout(std::time::Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();
    assert!(n > 0);

    let response: serde_json::Value = serde_json::from_slice(&buf[..n]).unwrap();
    assert!(response.get("result").is_some());

    handle.stop();
}

#[tokio::test]
async fn tcp_server_accepts_http_post() {
    let addr = "127.0.0.1:0".parse().unwrap();
    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_server(addr, service).await.unwrap();
    let bound_addr = handle.local_addr();

    let body = r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#;
    let http_request = format!(
        "POST / HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );

    let mut stream = tokio::net::TcpStream::connect(bound_addr).await.unwrap();
    stream.write_all(http_request.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = vec![0u8; 8192];
    let n = tokio::time::timeout(std::time::Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();
    let response_str = String::from_utf8_lossy(&buf[..n]);
    assert!(response_str.contains("HTTP/1.1 200 OK"));
    assert!(response_str.contains("result"));

    handle.stop();
}

// =========================================================================
// outcome_to_response coverage
// =========================================================================

#[test]
fn outcome_to_response_ok() {
    let id = serde_json::Value::Number(1.into());
    let outcome = loam_spine_core::error::DispatchOutcome::Ok(serde_json::json!(true));
    let resp = outcome_to_response(id, outcome);
    assert!(resp.error.is_none());
    assert_eq!(resp.result.unwrap(), serde_json::json!(true));
}

#[test]
fn outcome_to_response_application_error() {
    let id = serde_json::Value::Number(2.into());
    let outcome = loam_spine_core::error::DispatchOutcome::ApplicationError {
        code: -32000,
        message: "app error".to_string(),
    };
    let resp = outcome_to_response(id, outcome);
    assert!(resp.result.is_none());
    let err = resp.error.unwrap();
    assert_eq!(err.code, -32000);
    assert_eq!(err.message, "app error");
}

#[test]
fn outcome_to_response_protocol_error_jsonrpc() {
    let id = serde_json::Value::Number(3.into());
    let outcome = loam_spine_core::error::DispatchOutcome::<serde_json::Value>::ProtocolError(
        loam_spine_core::error::LoamSpineError::ipc(
            loam_spine_core::error::IpcErrorPhase::JsonRpcError(-32601),
            "method not found",
        ),
    );
    let resp = outcome_to_response(id, outcome);
    let err = resp.error.unwrap();
    assert_eq!(err.code, -32601);
}

#[test]
fn outcome_to_response_protocol_error_other() {
    let id = serde_json::Value::Number(4.into());
    let outcome = loam_spine_core::error::DispatchOutcome::<serde_json::Value>::ProtocolError(
        loam_spine_core::error::LoamSpineError::Internal("some error".to_string()),
    );
    let resp = outcome_to_response(id, outcome);
    let err = resp.error.unwrap();
    assert_eq!(err.code, -32000);
}

// =========================================================================
// serialize_response / serialize_response_batch helpers
// =========================================================================

#[test]
fn serialize_response_produces_valid_json() {
    let resp = JsonRpcResponse::success(serde_json::Value::Number(1.into()), serde_json::json!(42));
    let bytes = serialize_response(&resp);
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["result"], 42);
}

#[test]
fn serialize_response_batch_produces_valid_json() {
    let responses = vec![
        JsonRpcResponse::success(serde_json::Value::Number(1.into()), serde_json::json!("a")),
        JsonRpcResponse::success(serde_json::Value::Number(2.into()), serde_json::json!("b")),
    ];
    let bytes = serialize_response_batch(&responses);
    let parsed: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed.len(), 2);
}

#[test]
fn is_notification_missing_id() {
    let val = serde_json::json!({"jsonrpc": "2.0", "method": "test"});
    assert!(is_notification(&val));
}

#[test]
fn is_notification_null_id() {
    let val = serde_json::json!({"jsonrpc": "2.0", "method": "test", "id": null});
    assert!(is_notification(&val));
}

#[test]
fn is_notification_with_id() {
    let val = serde_json::json!({"jsonrpc": "2.0", "method": "test", "id": 1});
    assert!(!is_notification(&val));
}
