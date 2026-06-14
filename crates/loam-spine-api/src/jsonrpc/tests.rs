// SPDX-License-Identifier: AGPL-3.0-or-later

//! Shared test helpers and core server/discovery dispatch tests.
//!
//! Domain-specific tests live in sibling modules:
//! - `tests_spine_entry` — spine lifecycle, entry CRUD, certificates, slices, health probes
//! - `tests_session` — session dehydrate/commit, braid commit
//! - `tests_proof_anchor` — inclusion proofs, public chain anchor dispatch
//! - `tests_wire_errors` — JSON-RPC 2.0 protocol edge cases (parse errors, batches, etc.)
//! - `tests_bond_ledger` — ionic bond ledger operations
//! - `tests_permanence_cert` — permanence compat layer, certificate transfer/loan
//! - `tests_btsp_gate` — BTSP Phase 3 + JH-0 method gate
//! - `tests_validation` — validation-focused tests
//! - `tests_protocol_wire` — protocol wire format
//! - `tests_protocol_trio` — provenance trio protocol
//! - `tests_protocol_transport` — transport protocol
//! - `tests_phase3_transport` — Phase 3 transport

use super::*;

/// Helper: build a JSON-RPC request and dispatch through the handler.
pub(super) async fn rpc_call<Req: serde::Serialize + Sync, Resp: serde::de::DeserializeOwned>(
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
pub(super) async fn rpc_call_no_params<Resp: serde::de::DeserializeOwned>(
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
    let _server =
        LoamSpineJsonRpc::new(service, super::MethodGate::new(super::AuthMode::Permissive));
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
async fn capability_list_method() {
    let server = LoamSpineJsonRpc::default_server();
    let value: serde_json::Value = rpc_call_no_params(&server, "capability.list")
        .await
        .unwrap();
    assert_eq!(value["primal"], "loamspine");
    assert!(value["version"].is_string());
    assert!(value["capabilities"].is_array());
    assert!(
        value["methods"].is_array(),
        "methods must be flat string array per Wire Standard L2"
    );
    let methods = value["methods"].as_array().unwrap();
    assert!(
        methods.iter().all(serde_json::Value::is_string),
        "all methods must be strings"
    );
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
    let value: serde_json::Value = rpc_call_no_params(&server, "identity.get").await.unwrap();
    assert_eq!(value["primal"], "loamspine");
    assert!(value["version"].is_string());
    assert_eq!(value["domain"], "permanence");
    assert_eq!(value["license"], "AGPL-3.0-or-later");
}

#[tokio::test]
async fn bare_health_method_returns_status_primal_version() {
    let server = LoamSpineJsonRpc::default_server();
    let value: serde_json::Value = rpc_call_no_params(&server, "health").await.unwrap();
    assert_eq!(value["status"], "ok");
    assert_eq!(value["primal"], "loamspine");
    assert!(value["version"].is_string());
    assert!(!value["version"].as_str().unwrap().is_empty());
}
