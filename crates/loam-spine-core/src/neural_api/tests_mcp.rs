// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

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
        "lifecycle.status",
        "capabilities.list",
        "tools.list",
        "tools.call",
        "identity.get",
        "auth.check",
        "auth.mode",
        "auth.peer_info",
        "btsp.negotiate",
        "btsp.capabilities",
        "primal.announce",
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
        ("slice_anchor", "slice.anchor"),
        ("slice_checkout", "slice.checkout"),
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
