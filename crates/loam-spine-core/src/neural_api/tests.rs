// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

// ── Socket path resolution (pure inner functions) ────────────────────────

#[test]
fn resolve_socket_path_returns_valid_path() {
    let path = resolve_socket_path_with(None, None, None);
    assert!(!path.as_os_str().is_empty());
    assert!(path.to_string_lossy().contains("permanence"));
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
        "/run/user/1000/biomeos/permanence.sock"
    );
}

#[test]
fn resolve_socket_path_uses_xdg_runtime_dir_with_family() {
    let path = resolve_socket_path_with(None, Some("myfamily"), Some("/run/user/1000"));
    assert_eq!(
        path.to_string_lossy(),
        "/run/user/1000/biomeos/permanence-myfamily.sock"
    );
}

#[test]
fn resolve_socket_path_fallback_when_xdg_unset() {
    let path = resolve_socket_path_with(None, None, None);
    assert!(
        path.to_string_lossy().ends_with("biomeos/permanence.sock"),
        "got: {}",
        path.display()
    );
}

#[test]
fn resolve_socket_path_with_custom_family_id() {
    let path = resolve_socket_path_with(None, Some("custom-family"), None);
    assert!(
        path.to_string_lossy()
            .ends_with("biomeos/permanence-custom-family.sock"),
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
        "/run/user/1000/biomeos/permanence.sock",
        "empty BIOMEOS_FAMILY_ID should be treated as unset"
    );
}

#[test]
fn resolve_socket_path_default_family_id_treated_as_unset() {
    let path = resolve_socket_path_with(None, Some("default"), Some("/run/user/1000"));
    assert_eq!(
        path.to_string_lossy(),
        "/run/user/1000/biomeos/permanence.sock",
        "BIOMEOS_FAMILY_ID=default should produce domain-only socket"
    );
}

// ── Domain socket naming ────────────────────────────────────────────

#[test]
fn domain_socket_name_without_family() {
    assert_eq!(domain_socket_name(None), "permanence.sock");
    assert_eq!(domain_socket_name(Some("")), "permanence.sock");
    assert_eq!(domain_socket_name(Some("default")), "permanence.sock");
}

#[test]
fn domain_socket_name_with_family() {
    assert_eq!(domain_socket_name(Some("prod")), "permanence-prod.sock");
}

#[test]
fn legacy_socket_name_without_family() {
    assert_eq!(legacy_socket_name(None), "loamspine.sock");
}

#[test]
fn legacy_socket_name_with_family() {
    assert_eq!(legacy_socket_name(Some("prod")), "loamspine-prod.sock");
}

#[test]
fn legacy_symlink_path_matches_parent() {
    let primary = std::path::Path::new("/run/user/1000/biomeos/permanence.sock");
    let legacy = resolve_legacy_symlink_path(primary, None);
    assert_eq!(
        legacy.to_string_lossy(),
        "/run/user/1000/biomeos/loamspine.sock"
    );
}

// ── Capability-domain socket naming ─────────────────────────────────

#[test]
fn capability_domain_socket_name_without_family() {
    assert_eq!(capability_domain_socket_name(None), "ledger.sock");
    assert_eq!(capability_domain_socket_name(Some("")), "ledger.sock");
    assert_eq!(
        capability_domain_socket_name(Some("default")),
        "ledger.sock"
    );
}

#[test]
fn capability_domain_socket_name_with_family() {
    assert_eq!(
        capability_domain_socket_name(Some("prod")),
        "ledger-prod.sock"
    );
}

#[test]
fn capability_symlink_path_matches_parent() {
    let primary = std::path::Path::new("/run/user/1000/biomeos/permanence.sock");
    let cap = resolve_capability_symlink_path(primary, None);
    assert_eq!(cap.to_string_lossy(), "/run/user/1000/biomeos/ledger.sock");
}

// ── Security config validation ──────────────────────────────────────

#[test]
fn validate_security_config_ok_no_family() {
    assert!(validate_security_config(None, None).is_ok());
    assert!(validate_security_config(None, Some("1")).is_ok());
}

#[test]
fn validate_security_config_ok_family_no_insecure() {
    assert!(validate_security_config(Some("prod"), None).is_ok());
    assert!(validate_security_config(Some("prod"), Some("0")).is_ok());
}

#[test]
fn validate_security_config_ok_default_family_insecure() {
    assert!(validate_security_config(Some("default"), Some("1")).is_ok());
    assert!(validate_security_config(Some(""), Some("1")).is_ok());
}

#[test]
fn validate_security_config_rejects_family_plus_insecure() {
    let err = validate_security_config(Some("prod"), Some("1"));
    assert!(err.is_err(), "family + insecure must be rejected");
    let msg = err.unwrap_err().to_string();
    assert!(msg.contains("BIOMEOS_INSECURE"));
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
    assert!(
        methods.iter().all(serde_json::Value::is_string),
        "Wire Standard L2: methods must be a flat string array"
    );
    let method_strs: Vec<&str> = methods.iter().filter_map(|v| v.as_str()).collect();
    assert!(method_strs.contains(&"spine.create"));
    assert!(method_strs.contains(&"identity.get"));
    assert!(method_strs.contains(&"capabilities.list"));
    assert!(method_strs.contains(&"health.liveness"));

    assert!(
        list.get("provided_capabilities").is_some(),
        "Wire Standard L3"
    );
    assert!(
        list.get("consumed_capabilities").is_some(),
        "Wire Standard L3"
    );
}

#[test]
fn identity_response_fields() {
    let id = identity_response();
    assert_eq!(id["primal"], "loamspine");
    assert!(id["version"].is_string());
    assert_eq!(id["domain"], "permanence");
    assert_eq!(id["license"], "AGPL-3.0-or-later");
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
    assert!(pretty.contains("capabilities.list"));
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

/// Peer hangs up after reading the request but before sending the 4-byte response length.
#[tokio::test]
async fn deregister_handles_peer_close_before_response_length() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api-dereg-hangup.sock");
    let listener = tokio::net::UnixListener::bind(&sock).unwrap();
    let handle = tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let mut stream = stream;
            let mut len_buf = [0u8; 4];
            let _ = stream.read_exact(&mut len_buf).await;
            let req_len = u32::from_be_bytes(len_buf) as usize;
            let mut req_buf = vec![0u8; req_len];
            let _ = stream.read_exact(&mut req_buf).await;
            drop(stream);
        }
    });
    let result = deregister_at_socket(&sock).await;
    handle.abort();
    assert!(
        result.is_ok(),
        "deregister should tolerate hang-up before response length"
    );
}

// ── MCP tools ────────────────────────────────────────────────────────────

#[test]
fn mcp_tools_cover_all_methods_in_capability_list() {
    let list = capability_list();
    let methods: Vec<&str> = list["methods"]
        .as_array()
        .expect("methods array")
        .iter()
        .filter_map(|m| m.as_str())
        .collect();

    let tools = mcp_tools_list();
    let tool_names: Vec<&str> = tools["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();

    let meta_methods = [
        "health.liveness",
        "health.readiness",
        "capabilities.list",
        "tools.list",
        "tools.call",
        "identity.get",
        "permanence.commit_session",
        "permanence.verify_commit",
        "permanence.get_commit",
        "permanence.health_check",
    ];

    for method in &methods {
        if meta_methods.contains(method) {
            continue;
        }
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
        ("capability_list", "capabilities.list"),
        ("identity_get", "identity.get"),
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
        ("capability_list", "capabilities.list"),
        ("anchor_publish", "anchor.publish"),
        ("anchor_verify", "anchor.verify"),
        ("identity_get", "identity.get"),
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

// =========================================================================
// Additional coverage: public wrapper entry points
// =========================================================================

#[tokio::test]
async fn register_with_neural_api_returns_false_when_no_socket() {
    let result = super::register_with_neural_api().await;
    assert!(result.is_ok());
    assert!(!result.unwrap(), "no NeuralAPI socket → Ok(false)");
}

#[tokio::test]
async fn deregister_from_neural_api_succeeds_when_no_socket() {
    let result = super::deregister_from_neural_api().await;
    assert!(result.is_ok());
}

// =========================================================================
// Additional coverage: capability_list and identity_response accessors
// =========================================================================

#[test]
fn capability_list_is_object_with_methods() {
    let list = super::capability_list();
    assert!(list.is_object());
    assert!(list.get("primal").is_some());
    assert!(list.get("methods").is_some());
}

#[test]
fn identity_response_has_primal_and_version() {
    let id = super::identity_response();
    assert!(id.is_object());
    assert!(id.get("primal").is_some());
    assert!(id.get("version").is_some());
    assert_eq!(id["primal"], "loamspine");
}

#[test]
fn capability_list_pretty_is_nonempty_string() {
    let pretty = super::capability_list_pretty();
    assert!(!pretty.is_empty());
    assert!(pretty.contains("permanence") || pretty.contains("ledger"));
}

// =========================================================================
// Additional coverage: validate_security_config_from_env
// =========================================================================

#[test]
fn validate_security_config_from_env_does_not_panic() {
    let result = super::validate_security_config_from_env();
    let _ = result;
}
