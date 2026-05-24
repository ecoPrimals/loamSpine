// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

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
    assert!(caps.contains(&serde_json::json!("capabilities.list")));
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
    assert!(CAPABILITIES.contains(&"ledger"));
    assert!(CAPABILITIES.contains(&"spine.create"));
    assert!(CAPABILITIES.contains(&"spine.get"));
    assert!(CAPABILITIES.contains(&"certificate.mint"));
    assert!(CAPABILITIES.contains(&"capabilities.list"));
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
