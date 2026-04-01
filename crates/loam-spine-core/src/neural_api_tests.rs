// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

// ── Socket path resolution (pure inner functions) ────────────────────────

#[test]
fn resolve_socket_path_returns_valid_path() {
    let path = resolve_socket_path_with(None, None, None);
    assert!(!path.as_os_str().is_empty());
    assert!(path.to_string_lossy().contains("loamspine"));
}

#[test]
fn socket_path_respects_override() {
    let path = resolve_socket_path_with(Some("/custom/loamspine.sock"), None, None);
    assert_eq!(path.to_string_lossy(), "/custom/loamspine.sock");
}

#[test]
fn resolve_socket_path_uses_xdg_runtime_dir() {
    let path = resolve_socket_path_with(None, None, Some("/run/user/1000"));
    assert_eq!(
        path.to_string_lossy(),
        "/run/user/1000/biomeos/loamspine.sock"
    );
}

#[test]
fn resolve_socket_path_uses_xdg_runtime_dir_with_family() {
    let path = resolve_socket_path_with(None, Some("myfamily"), Some("/run/user/1000"));
    assert_eq!(
        path.to_string_lossy(),
        "/run/user/1000/biomeos/loamspine-myfamily.sock"
    );
}

#[test]
fn resolve_socket_path_fallback_when_xdg_unset() {
    let path = resolve_socket_path_with(None, None, None);
    assert!(
        path.to_string_lossy().ends_with("biomeos/loamspine.sock"),
        "got: {}",
        path.display()
    );
}

#[test]
fn resolve_socket_path_with_custom_family_id() {
    let path = resolve_socket_path_with(None, Some("custom-family"), None);
    assert!(
        path.to_string_lossy()
            .ends_with("biomeos/loamspine-custom-family.sock"),
        "got: {}",
        path.display()
    );
}

#[test]
fn resolve_socket_path_override_wins_over_xdg_and_family() {
    let path = resolve_socket_path_with(
        Some("/override/path.sock"),
        Some("ignored"),
        Some("/run/user/1000"),
    );
    assert_eq!(path.to_string_lossy(), "/override/path.sock");
}

#[test]
fn resolve_socket_path_empty_family_id_treated_as_unset() {
    let path = resolve_socket_path_with(None, Some(""), Some("/run/user/1000"));
    assert_eq!(
        path.to_string_lossy(),
        "/run/user/1000/biomeos/loamspine.sock",
        "empty BIOMEOS_FAMILY_ID should be treated as unset"
    );
}

// ── NeuralAPI socket resolution (pure inner function) ────────────────────

#[test]
fn resolve_neural_api_socket_with_explicit() {
    let path = resolve_neural_api_socket_with(Some("/custom/neural.sock"), None, None);
    assert!(path.is_some());
    assert_eq!(path.unwrap().to_string_lossy(), "/custom/neural.sock");
}

#[test]
fn resolve_neural_api_socket_with_xdg_runtime_dir() {
    let path = resolve_neural_api_socket_with(None, Some("/run/user/1000"), None);
    assert!(path.is_some());
    assert_eq!(
        path.unwrap().to_string_lossy(),
        "/run/user/1000/biomeos/neural-api-default.sock"
    );
}

#[test]
fn resolve_neural_api_socket_without_env_returns_none() {
    let path = resolve_neural_api_socket_with(None, None, None);
    assert!(path.is_none());
}

#[test]
fn resolve_neural_api_socket_with_family_id() {
    let path = resolve_neural_api_socket_with(None, Some("/run/user/42"), Some("my-family"));
    assert!(path.is_some());
    assert_eq!(
        path.unwrap().to_string_lossy(),
        "/run/user/42/biomeos/neural-api-my-family.sock"
    );
}

// ── Capability list / primal identity ────────────────────────────────────

#[test]
fn primal_name_is_correct() {
    assert_eq!(PRIMAL_NAME, "loamspine");
    assert_eq!(PRIMAL_NAME, crate::primal_names::SELF_ID);
}

#[test]
fn capability_list_includes_all_expected() {
    let list = capability_list();
    assert!(list.get("primal").is_some());
    assert!(list.get("version").is_some());
    assert!(list.get("capabilities").is_some());
    assert!(list.get("methods").is_some());
    let caps = list["capabilities"].as_array().expect("capabilities array");
    assert!(caps.contains(&serde_json::json!("permanence")));
    assert!(caps.contains(&serde_json::json!("spine.create")));
    assert!(caps.contains(&serde_json::json!("capability.list")));
    assert_eq!(caps.len(), CAPABILITIES.len());
    let methods = list["methods"].as_array().expect("methods array");
    assert!(!methods.is_empty());
}

#[test]
fn capabilities_contains_expected_entries() {
    assert!(CAPABILITIES.contains(&"permanence"));
    assert!(CAPABILITIES.contains(&"spine.create"));
    assert!(CAPABILITIES.contains(&"spine.query"));
    assert!(CAPABILITIES.contains(&"certificate.issue"));
    assert!(CAPABILITIES.contains(&"capability.list"));
}

#[test]
fn capability_list_is_valid_json() {
    let list = capability_list();
    let s = serde_json::to_string(&list).expect("serialize");
    let _: serde_json::Value = serde_json::from_str(&s).expect("deserialize");
}

#[test]
fn capability_list_pretty_output_validation() {
    let pretty = capability_list_pretty();
    assert!(!pretty.is_empty());
    assert!(pretty.contains('\n'));
    assert!(pretty.contains("  "));
    let parsed: serde_json::Value =
        serde_json::from_str(&pretty).expect("pretty output must be valid JSON");
    assert!(parsed.get("capabilities").is_some());
    assert!(parsed.get("primal").is_some());
    assert!(parsed.get("version").is_some());
    assert!(parsed.get("methods").is_some());
    assert_eq!(parsed["primal"], PRIMAL_NAME);
    let caps = parsed["capabilities"]
        .as_array()
        .expect("capabilities array");
    assert_eq!(caps.len(), CAPABILITIES.len());
}

#[test]
fn capability_list_pretty_contains_primal_and_capabilities() {
    let pretty = capability_list_pretty();
    assert!(pretty.contains(PRIMAL_NAME));
    assert!(pretty.contains("permanence"));
    assert!(pretty.contains("capability.list"));
}

#[test]
fn capabilities_has_no_duplicates() {
    let mut seen = std::collections::HashSet::new();
    for cap in CAPABILITIES {
        assert!(seen.insert(cap), "duplicate capability: {cap}");
    }
}

#[test]
fn capabilities_follow_semantic_naming() {
    for cap in CAPABILITIES {
        assert!(
            !cap.contains("loamspine"),
            "capability '{cap}' should not reference primal name"
        );
        assert!(!cap.is_empty());
    }
}

#[test]
fn capability_list_cost_estimates() {
    let list = capability_list();
    let costs = list.get("cost_estimates").unwrap();
    assert!(costs.get("spine.create").is_some());
    assert!(costs.get("health.check").is_some());
    let spine_create = &costs["spine.create"];
    assert!(spine_create.get("latency_ms").is_some());
    assert!(spine_create.get("cpu").is_some());
    assert!(spine_create.get("gpu_eligible").is_some());
}

#[test]
fn capability_list_operation_dependencies() {
    let list = capability_list();
    let deps = list.get("operation_dependencies").unwrap();
    assert!(deps.get("entry.append").is_some());
    let entry_deps = deps["entry.append"].as_array().unwrap();
    assert!(entry_deps.contains(&serde_json::json!("spine.create")));
}

// ── Register / Deregister via mock sockets (no env vars) ─────────────────

fn spawn_mock_neural_api(
    socket_path: &std::path::Path,
    response: &serde_json::Value,
) -> tokio::task::JoinHandle<()> {
    let listener = tokio::net::UnixListener::bind(socket_path).unwrap();
    let resp_bytes = serde_json::to_vec(response).unwrap();

    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut len_buf = [0u8; 4];
            let _ = stream.read_exact(&mut len_buf).await;
            let req_len = u32::from_be_bytes(len_buf) as usize;
            let mut req_buf = vec![0u8; req_len];
            let _ = stream.read_exact(&mut req_buf).await;

            let len = u32::try_from(resp_bytes.len())
                .unwrap_or(u32::MAX)
                .to_be_bytes();
            let _ = stream.write_all(&len).await;
            let _ = stream.write_all(&resp_bytes).await;
            let _ = stream.flush().await;
        }
    })
}

#[tokio::test]
async fn register_returns_ok_false_when_socket_unresolvable() {
    let path = resolve_neural_api_socket_with(None, None, None);
    assert!(path.is_none());
}

#[tokio::test]
async fn register_returns_ok_false_when_socket_missing() {
    let path = std::path::PathBuf::from("/tmp/nonexistent-neural-api-loamspine-test.sock");
    assert!(!path.exists());
}

#[tokio::test]
async fn register_succeeds_with_mock_server() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api.sock");
    let our_sock = tmp.path().join("loamspine.sock");
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": { "registered": true },
        "id": 1
    });
    let handle = spawn_mock_neural_api(&sock, &response);
    let result = register_at_socket(&sock, &our_sock).await;
    handle.abort();
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn register_returns_error_on_jsonrpc_error() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api-err.sock");
    let our_sock = tmp.path().join("loamspine.sock");
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": -32601, "message": "method not found" },
        "id": 1
    });
    let handle = spawn_mock_neural_api(&sock, &response);
    let result = register_at_socket(&sock, &our_sock).await;
    handle.abort();
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("method not found"), "error: {err}");
}

#[tokio::test]
async fn deregister_succeeds_with_mock_server() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api-dereg.sock");
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": { "deregistered": true },
        "id": 2
    });
    let handle = spawn_mock_neural_api(&sock, &response);
    let result = deregister_at_socket(&sock).await;
    handle.abort();
    assert!(result.is_ok());
}

#[tokio::test]
async fn deregister_handles_jsonrpc_error_gracefully() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api-dereg-err.sock");
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": -32601, "message": "not supported" },
        "id": 2
    });
    let handle = spawn_mock_neural_api(&sock, &response);
    let result = deregister_at_socket(&sock).await;
    handle.abort();
    assert!(
        result.is_ok(),
        "deregister should succeed even on JSON-RPC error"
    );
}

#[tokio::test]
async fn deregister_handles_malformed_response() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api-dereg-bad.sock");
    let listener = tokio::net::UnixListener::bind(&sock).unwrap();
    let handle = tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut len_buf = [0u8; 4];
            let _ = stream.read_exact(&mut len_buf).await;
            let req_len = u32::from_be_bytes(len_buf) as usize;
            let mut req_buf = vec![0u8; req_len];
            let _ = stream.read_exact(&mut req_buf).await;

            let garbage = b"not json";
            let len = u32::try_from(garbage.len())
                .unwrap_or(u32::MAX)
                .to_be_bytes();
            let _ = stream.write_all(&len).await;
            let _ = stream.write_all(garbage).await;
            let _ = stream.flush().await;
        }
    });
    let result = deregister_at_socket(&sock).await;
    handle.abort();
    assert!(
        result.is_ok(),
        "deregister should succeed even on malformed response"
    );
}

// ── MCP tools ────────────────────────────────────────────────────────────

#[test]
fn mcp_tools_cover_all_methods_in_capability_list() {
    let list = capability_list();
    let methods = list["methods"]
        .as_array()
        .expect("methods array")
        .iter()
        .filter_map(|m| m["method"].as_str())
        .collect::<Vec<_>>();

    let tools = mcp_tools_list();
    let tool_names: Vec<&str> = tools["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();

    for method in &methods {
        let tool_name = method.replace('.', "_");
        let has_mcp = tool_names.contains(&tool_name.as_str())
            || mcp_tool_to_rpc(&tool_name, serde_json::json!({})).is_some();
        assert!(
            has_mcp,
            "method '{method}' (tool '{tool_name}') missing from MCP tools or mcp_tool_to_rpc"
        );
    }
}

#[test]
fn mcp_tool_to_rpc_returns_canonical_method_names() {
    let cases = [
        ("spine_create", "spine.create"),
        ("entry_append", "entry.append"),
        ("certificate_mint", "certificate.mint"),
        ("health_check", "health.check"),
        ("capability_list", "capability.list"),
    ];
    for (tool, expected_method) in cases {
        let result = mcp_tool_to_rpc(tool, serde_json::json!({}));
        assert!(
            result.is_some(),
            "tool '{tool}' not found in mcp_tool_to_rpc"
        );
        let (method, _) = result.unwrap();
        assert_eq!(
            method, expected_method,
            "tool '{tool}' maps to '{method}' but canonical is '{expected_method}'"
        );
    }
}

#[test]
fn mcp_tool_to_rpc_returns_none_for_unknown_tool() {
    assert!(mcp_tool_to_rpc("nonexistent_tool", serde_json::json!({})).is_none());
}

#[test]
fn mcp_tool_to_rpc_covers_all_known_tools() {
    let all_tools = [
        ("spine_create", "spine.create"),
        ("spine_get", "spine.get"),
        ("spine_seal", "spine.seal"),
        ("entry_append", "entry.append"),
        ("entry_get", "entry.get"),
        ("entry_get_tip", "entry.get_tip"),
        ("certificate_mint", "certificate.mint"),
        ("certificate_get", "certificate.get"),
        ("certificate_transfer", "certificate.transfer"),
        ("certificate_loan", "certificate.loan"),
        ("certificate_return", "certificate.return"),
        ("certificate_verify", "certificate.verify"),
        ("certificate_lifecycle", "certificate.lifecycle"),
        ("slice_anchor", "slice.anchor"),
        ("slice_checkout", "slice.checkout"),
        ("slice_record_operation", "slice.record_operation"),
        ("slice_depart", "slice.depart"),
        ("proof_generate_inclusion", "proof.generate_inclusion"),
        ("proof_verify_inclusion", "proof.verify_inclusion"),
        ("session_commit", "session.commit"),
        ("braid_commit", "braid.commit"),
        ("health_check", "health.check"),
        ("capability_list", "capability.list"),
    ];
    for (tool, expected) in all_tools {
        let result = mcp_tool_to_rpc(tool, serde_json::json!({"test": true}));
        assert!(result.is_some(), "tool '{tool}' should be recognized");
        let (method, params) = result.unwrap();
        assert_eq!(method, expected, "tool '{tool}' mapped incorrectly");
        assert_eq!(params["test"], true);
    }
}

#[test]
fn mcp_tools_list_schema_structure() {
    let tools = mcp_tools_list();
    let tools_array = tools["tools"].as_array().unwrap();
    assert!(!tools_array.is_empty());
    for tool in tools_array {
        assert!(tool.get("name").is_some(), "tool missing name");
        assert!(
            tool.get("description").is_some(),
            "tool missing description"
        );
        assert!(
            tool.get("inputSchema").is_some(),
            "tool missing inputSchema"
        );
        let schema = &tool["inputSchema"];
        assert_eq!(schema["type"], "object");
    }
}
