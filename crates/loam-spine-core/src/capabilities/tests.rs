// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn test_capability_identifiers() {
    assert_eq!(loamspine::PERMANENT_LEDGER, "permanent-ledger");
    assert_eq!(external::SIGNING, "cryptographic-signing");
    assert!(!loamspine::PERMANENT_LEDGER.contains(' '));
    assert!(!loamspine::PERMANENT_LEDGER.contains('_'));
}

#[test]
fn test_introspection() {
    let capabilities = LoamSpineCapability::introspect();
    assert!(!capabilities.is_empty());
    assert!(capabilities.len() >= 6);
    assert!(
        capabilities
            .iter()
            .any(|c| matches!(c, LoamSpineCapability::PermanentLedger { .. }))
    );
    assert!(
        capabilities
            .iter()
            .any(|c| matches!(c, LoamSpineCapability::TemporalTracking { .. }))
    );
}

#[test]
fn test_capability_identifier_extraction() {
    let cap = LoamSpineCapability::PermanentLedger {
        entry_types: vec![],
        max_spine_size: None,
        supports_sealing: true,
    };
    assert_eq!(cap.identifier(), "permanent-ledger");
}

#[test]
fn test_service_health_default() {
    let health = ServiceHealth::default();
    assert_eq!(health, ServiceHealth::Unknown);
}

#[test]
fn extract_flat_capabilities() {
    let response = serde_json::json!({
        "primal": "rhizoCrypt",
        "version": "0.13.0",
        "capabilities": ["signing", "verification", "key-management"],
    });
    let parsed = extract_capabilities(&response);
    assert_eq!(parsed.primal.as_deref(), Some("rhizoCrypt"));
    assert_eq!(parsed.version.as_deref(), Some("0.13.0"));
    assert_eq!(
        parsed.capabilities,
        vec!["signing", "verification", "key-management"]
    );
    assert!(parsed.methods.is_empty());
}

#[test]
fn extract_methods_with_domain_cost_deps() {
    let response = serde_json::json!({
        "primal": "loamSpine",
        "methods": [
            { "method": "spine.create", "domain": "spine", "cost": "low", "deps": [] },
            { "method": "entry.append", "domain": "entry", "cost": "low", "deps": ["spine.create"] },
        ],
    });
    let parsed = extract_capabilities(&response);
    assert_eq!(parsed.methods.len(), 2);
    assert_eq!(parsed.methods[1].method, "entry.append");
    assert_eq!(parsed.methods[1].domain.as_deref(), Some("entry"));
    assert_eq!(parsed.methods[1].deps, vec!["spine.create"]);
}

#[test]
fn extract_nested_domains() {
    let response = serde_json::json!({
        "domains": {
            "spine": ["create", "get", "seal"],
            "entry": ["append", "get"],
        },
    });
    let parsed = extract_capabilities(&response);
    assert!(parsed.capabilities.contains(&"spine.create".to_string()));
    assert!(parsed.capabilities.contains(&"entry.append".to_string()));
    assert_eq!(parsed.capabilities.len(), 5);
}

#[test]
fn extract_combined_format() {
    let response = serde_json::json!({
        "primal": "sweetGrass",
        "version": "0.7.19",
        "capabilities": ["attribution", "braid.create"],
        "methods": [
            { "method": "braid.create", "domain": "braid", "cost": "medium", "deps": [] },
        ],
    });
    let parsed = extract_capabilities(&response);
    assert_eq!(parsed.capabilities.len(), 2);
    assert_eq!(parsed.methods.len(), 1);
    assert_eq!(parsed.primal.as_deref(), Some("sweetGrass"));
}

#[test]
fn extract_empty_response() {
    let response = serde_json::json!({});
    let parsed = extract_capabilities(&response);
    assert!(parsed.primal.is_none());
    assert!(parsed.capabilities.is_empty());
    assert!(parsed.methods.is_empty());
}
