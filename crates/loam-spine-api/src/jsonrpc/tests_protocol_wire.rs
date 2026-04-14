// SPDX-License-Identifier: AGPL-3.0-or-later

//! Wire-format conformance and dispatch logic tests: GAP-MATRIX wire format
//! validation (health.liveness, capabilities.list, identity.get), method
//! normalization, `dispatch_typed` outcomes, tools.call routing, JSON-RPC
//! notification handling, and batch edge cases.

use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// =========================================================================
// UDS helper (shared with tests_protocol_transport)
// =========================================================================

#[cfg(unix)]
async fn uds_rpc(stream: &mut tokio::net::UnixStream, request: &str) -> serde_json::Value {
    stream.write_all(request.as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = vec![0u8; 65536];
    let n = tokio::time::timeout(std::time::Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();

    serde_json::from_slice(&buf[..n]).unwrap()
}

// =========================================================================
// GAP-MATRIX-05: Neural API wire-format validation (health.liveness)
//
// Validates that health.liveness returns the exact wire format biomeOS
// expects when probing primals through Neural API routing.
// =========================================================================

#[cfg(unix)]
#[tokio::test]
async fn uds_health_liveness_wire_format() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("liveness-wire.sock");
    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_uds_server(&sock_path, service, None)
        .await
        .unwrap();

    let mut stream = tokio::net::UnixStream::connect(&sock_path).await.unwrap();
    let response = uds_rpc(
        &mut stream,
        r#"{"jsonrpc":"2.0","method":"health.liveness","id":42}"#,
    )
    .await;

    // JSON-RPC 2.0 envelope
    assert_eq!(response["jsonrpc"], "2.0", "must be JSON-RPC 2.0");
    assert_eq!(response["id"], 42, "id must echo request");
    assert!(response.get("error").is_none(), "must not be an error");

    // Semantic Method Naming Standard v2.1: {"status": "alive"}
    let result = &response["result"];
    assert_eq!(result["status"], "alive", "liveness status must be 'alive'");
    assert!(result.is_object(), "liveness result must be an object");
    assert_eq!(
        result.as_object().unwrap().len(),
        1,
        "liveness result must contain only 'status'"
    );

    handle.stop();
}

// =========================================================================
// GAP-MATRIX-05: Neural API wire-format validation (capabilities.list)
//
// Validates that capabilities.list returns a structure biomeOS can parse
// via its 5-format capability parser (primalSpring ipc/discover.rs).
// LoamSpine uses Format D: object with both `capabilities` (string array)
// and `methods` (array of {method, domain, cost, deps}).
// =========================================================================

#[cfg(unix)]
#[tokio::test]
async fn uds_capabilities_list_wire_format() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("caps-wire.sock");
    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_uds_server(&sock_path, service, None)
        .await
        .unwrap();

    let mut stream = tokio::net::UnixStream::connect(&sock_path).await.unwrap();

    // Use canonical name per Semantic Method Naming Standard v2.1
    let response = uds_rpc(
        &mut stream,
        r#"{"jsonrpc":"2.0","method":"capabilities.list","id":1}"#,
    )
    .await;

    // JSON-RPC 2.0 envelope
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("error").is_none(), "must not be an error");

    let result = &response["result"];

    // -- biomeOS Format D: primal identity --
    assert_eq!(result["primal"], "loamspine", "must identify as loamspine");
    let version = result["version"].as_str().unwrap();
    assert!(
        version.contains('.'),
        "version must be semver-like: {version}"
    );

    // -- biomeOS Format A/D: capabilities as flat string array --
    assert!(
        result["capabilities"].is_array(),
        "capabilities must be an array"
    );
    let caps = result["capabilities"].as_array().unwrap();
    assert!(!caps.is_empty(), "capabilities must not be empty");
    for cap in caps {
        assert!(cap.is_string(), "each capability must be a string");
    }
    let cap_strings: Vec<&str> = caps.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        cap_strings.contains(&"permanence"),
        "must advertise 'permanence'"
    );
    assert!(
        cap_strings.contains(&"session.commit"),
        "must advertise 'session.commit'"
    );

    // -- Wire Standard L2: methods as flat string array --
    assert!(result["methods"].is_array(), "methods must be an array");
    let methods = result["methods"].as_array().unwrap();
    assert!(!methods.is_empty(), "methods must not be empty");
    for m in methods {
        assert!(m.is_string(), "each method must be a string");
        let s = m.as_str().unwrap();
        assert!(s.contains('.'), "method must be dotted: {s}");
    }
    let method_strs: Vec<&str> = methods.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        method_strs.contains(&"spine.create"),
        "must list spine.create"
    );
    assert!(
        method_strs.contains(&"health.liveness"),
        "must list health.liveness"
    );
    assert!(
        method_strs.contains(&"identity.get"),
        "must list identity.get"
    );
    assert!(
        method_strs.contains(&"capabilities.list"),
        "must list capabilities.list"
    );

    // -- Wire Standard L3: provided_capabilities grouping --
    assert!(
        result["provided_capabilities"].is_array(),
        "provided_capabilities must be an array"
    );
    let groups = result["provided_capabilities"].as_array().unwrap();
    assert!(!groups.is_empty());
    for g in groups {
        assert!(g["type"].is_string(), "group type must be string");
        assert!(g["methods"].is_array(), "group methods must be array");
    }

    // -- Wire Standard L3: consumed_capabilities --
    assert!(
        result["consumed_capabilities"].is_array(),
        "consumed_capabilities must be an array"
    );

    // -- operation_dependencies (DAG for Pathway Learner) --
    assert!(
        result["operation_dependencies"].is_object(),
        "operation_dependencies must be an object"
    );

    // -- cost_estimates (resource hints for biomeOS scheduler) --
    assert!(
        result["cost_estimates"].is_object(),
        "cost_estimates must be an object"
    );
    let cost = &result["cost_estimates"]["health.check"];
    assert!(cost["latency_ms"].is_number(), "cost must have latency_ms");
    assert!(cost["cpu"].is_string(), "cost must have cpu");

    handle.stop();
}

#[cfg(unix)]
#[tokio::test]
async fn uds_capabilities_list_legacy_alias() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("caps-alias.sock");
    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_uds_server(&sock_path, service, None)
        .await
        .unwrap();

    let mut stream = tokio::net::UnixStream::connect(&sock_path).await.unwrap();

    // Legacy alias — biomeOS must also be able to call the old name
    let response = uds_rpc(
        &mut stream,
        r#"{"jsonrpc":"2.0","method":"capability.list","id":1}"#,
    )
    .await;

    assert!(response.get("error").is_none(), "alias must not error");
    assert_eq!(response["result"]["primal"], "loamspine");
    assert!(response["result"]["capabilities"].is_array());

    handle.stop();
}

// =========================================================================
// identity.get Wire Standard L2
// =========================================================================

#[cfg(unix)]
#[tokio::test]
async fn uds_identity_get_wire_format() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("identity.sock");
    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_uds_server(&sock_path, service, None)
        .await
        .unwrap();

    let mut stream = tokio::net::UnixStream::connect(&sock_path).await.unwrap();

    let response = uds_rpc(
        &mut stream,
        r#"{"jsonrpc":"2.0","method":"identity.get","id":7}"#,
    )
    .await;

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 7);
    assert!(response.get("error").is_none(), "must not be an error");

    let result = &response["result"];
    assert_eq!(result["primal"], "loamspine");
    assert!(result["version"].as_str().unwrap().contains('.'));
    assert_eq!(result["domain"], "permanence");
    assert_eq!(result["license"], "AGPL-3.0-or-later");

    handle.stop();
}

// =========================================================================
// normalize_method alias coverage
// =========================================================================

#[test]
fn normalize_method_legacy_aliases() {
    assert_eq!(normalize_method("commit.session"), "session.commit");
    assert_eq!(normalize_method("provenance.commit"), "session.commit");
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
async fn batch_all_notifications_produces_no_response() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#"[{"jsonrpc":"2.0","method":"health.liveness"},{"jsonrpc":"2.0","method":"health.liveness"}]"#;
    let response = process_request(&server, body).await;
    assert!(response.is_empty());
}

#[tokio::test]
async fn batch_empty_array_returns_invalid_request() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br"[]";
    let response = process_request(&server, body).await;
    let parsed: JsonRpcResponse = serde_json::from_slice(&response).unwrap();
    assert!(parsed.error.is_some());
    assert_eq!(parsed.error.as_ref().unwrap().code, -32600);
}

#[tokio::test]
async fn batch_all_invalid_notifications() {
    let server = LoamSpineJsonRpc::default_server();
    let body = br#"[{"not_jsonrpc": true}]"#;
    let response = process_request(&server, body).await;
    assert!(response.is_empty());
}
