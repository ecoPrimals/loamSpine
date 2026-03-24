// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use std::collections::HashSet;

#[test]
fn default_waypoint_config() {
    let config = WaypointConfig::default();
    assert!(config.accept_anchors);
    assert_eq!(config.max_anchored_slices, Some(100));
    assert_eq!(config.max_anchor_depth, Some(2));
    assert!(config.auto_return_expired);
    assert_eq!(config.propagation_policy, PropagationPolicy::SummaryOnly);
    assert_eq!(config.operation_attestation, AttestationRequirement::None);
}

#[test]
fn attestation_requirement_default_is_none() {
    let req = AttestationRequirement::default();
    assert!(!req.is_required());
    assert!(!req.requires_for_operation("anchor"));
}

#[test]
fn attestation_requirement_boundary_only() {
    let req = AttestationRequirement::BoundaryOnly;
    assert!(req.is_required());
    assert!(req.requires_for_operation("anchor"));
    assert!(req.requires_for_operation("depart"));
    assert!(!req.requires_for_operation("use"));
}

#[test]
fn attestation_requirement_all_operations() {
    let req = AttestationRequirement::AllOperations;
    assert!(req.is_required());
    assert!(req.requires_for_operation("anchor"));
    assert!(req.requires_for_operation("use"));
    assert!(req.requires_for_operation("anything"));
}

#[test]
fn attestation_requirement_selective() {
    let req = AttestationRequirement::Selective {
        operation_types: vec!["transfer".into(), "export".into()],
    };
    assert!(req.is_required());
    assert!(req.requires_for_operation("transfer"));
    assert!(req.requires_for_operation("export"));
    assert!(!req.requires_for_operation("view"));
}

#[test]
fn attestation_requirement_serde_roundtrip() {
    let req = AttestationRequirement::Selective {
        operation_types: vec!["anchor".into()],
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let restored: AttestationRequirement = serde_json::from_str(&json).expect("deserialize");
    assert!(restored.requires_for_operation("anchor"));
    assert!(!restored.requires_for_operation("view"));
}

#[test]
fn attestation_result_serde_roundtrip() {
    let result = AttestationResult {
        attested: true,
        attester: crate::types::Did::new("did:key:z6MkAttest"),
        timestamp: Timestamp::now(),
        token: vec![1, 2, 3, 4],
        denial_reason: None,
    };
    let json = serde_json::to_string(&result).expect("serialize");
    let restored: AttestationResult = serde_json::from_str(&json).expect("deserialize");
    assert!(restored.attested);
    assert_eq!(restored.token, vec![1, 2, 3, 4]);
}

#[test]
fn waypoint_config_with_attestation_serde() {
    let config = WaypointConfig {
        operation_attestation: AttestationRequirement::BoundaryOnly,
        ..WaypointConfig::default()
    };
    let json = serde_json::to_string(&config).expect("serialize");
    let restored: WaypointConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(
        restored.operation_attestation,
        AttestationRequirement::BoundaryOnly
    );
}

#[test]
fn propagation_policy_default() {
    let policy = PropagationPolicy::default();
    assert_eq!(policy, PropagationPolicy::SummaryOnly);
}

#[test]
fn departure_reason_display() {
    assert_eq!(DepartureReason::Expired.to_string(), "expired");
    assert_eq!(DepartureReason::ManualReturn.to_string(), "manual_return");
    assert_eq!(DepartureReason::OwnerRecall.to_string(), "owner_recall");
    assert_eq!(
        DepartureReason::Administrative {
            reason: "cleanup".into()
        }
        .to_string(),
        "admin:cleanup"
    );
}

#[test]
fn slice_operation_type_name() {
    assert_eq!(
        SliceOperationType::Use {
            action: "play".into(),
            duration_secs: None,
        }
        .name(),
        "use"
    );
    assert_eq!(
        SliceOperationType::Custom {
            operation_name: "forge".into(),
        }
        .name(),
        "forge"
    );
}

#[test]
fn slice_terms_operation_allowed() {
    let terms = SliceTerms {
        forbidden_operations: HashSet::from(["export".to_string()]),
        ..SliceTerms::default()
    };
    assert!(terms.is_operation_allowed("use"));
    assert!(!terms.is_operation_allowed("export"));
}

#[test]
fn slice_terms_allowed_list() {
    let terms = SliceTerms {
        allowed_operations: Some(HashSet::from(["read".to_string(), "view".to_string()])),
        ..SliceTerms::default()
    };
    assert!(terms.is_operation_allowed("read"));
    assert!(!terms.is_operation_allowed("edit"));
}

#[test]
fn waypoint_summary_serde_roundtrip() {
    let summary = WaypointSummary {
        slice_id: crate::types::SliceId::now_v7(),
        duration_nanos: 1_000_000_000,
        operation_count: 5,
        operation_types: vec!["use".into(), "view".into()],
        first_operation: Some(Timestamp::now()),
        last_operation: Some(Timestamp::now()),
        operations_hash: [0u8; 32],
        was_relent: false,
        max_relend_depth: 0,
    };
    let json = serde_json::to_string(&summary).expect("serialize");
    let restored: WaypointSummary = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.operation_count, 5);
    assert_eq!(restored.operation_types.len(), 2);
}

#[test]
fn waypoint_config_serde_roundtrip() {
    let config = WaypointConfig::default();
    let json = serde_json::to_string(&config).expect("serialize");
    let restored: WaypointConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.max_anchor_depth, Some(2));
}

#[test]
fn slice_terms_serde_roundtrip() {
    let terms = SliceTerms::default();
    let json = serde_json::to_string(&terms).expect("serialize");
    let restored: SliceTerms = serde_json::from_str(&json).expect("deserialize");
    assert!(!restored.allow_relend);
}

#[test]
fn relending_chain_initial() {
    let did_a = crate::types::Did::new("did:key:z6MkA");
    let chain = RelendingChain::with_initial(did_a.clone(), [1u8; 32]);
    assert_eq!(chain.depth(), 0);
    assert_eq!(chain.current_holder(), Some(&did_a));
    assert_eq!(chain.root_borrower(), Some(&did_a));
    assert!(chain.contains(&did_a));
}

#[test]
fn relending_chain_sublend_validation() {
    let did_a = crate::types::Did::new("did:key:z6MkA");
    let mut chain = RelendingChain::with_initial(did_a, [1u8; 32]);

    // allow_sublend=false -> cannot sublend
    assert!(chain.can_sublend(false, Some(2)).is_err());

    // allow_sublend=true, max_depth=1 -> can add one more (depth 0 -> 1)
    assert!(chain.can_sublend(true, Some(1)).is_ok());

    chain
        .sublend(
            crate::types::Did::new("did:key:z6MkB"),
            [2u8; 32],
            true,
            Some(1),
        )
        .expect("sublend");

    assert_eq!(chain.depth(), 1);

    // Now at max depth, cannot sublend further
    assert!(chain.can_sublend(true, Some(1)).is_err());
}

#[test]
fn relending_chain_return_at() {
    let did_a = crate::types::Did::new("did:key:z6MkA");
    let did_b = crate::types::Did::new("did:key:z6MkB");
    let did_c = crate::types::Did::new("did:key:z6MkC");

    let mut chain = RelendingChain::with_initial(did_a.clone(), [1u8; 32]);
    chain
        .sublend(did_b.clone(), [2u8; 32], true, Some(2))
        .expect("sublend");
    chain
        .sublend(did_c.clone(), [3u8; 32], true, Some(2))
        .expect("sublend");

    assert_eq!(chain.depth(), 2);
    assert_eq!(chain.current_holder(), Some(&did_c));

    // Return at B - unwinds B and C
    let unwound = chain.return_at(&did_b).expect("return_at");
    assert_eq!(unwound.len(), 2); // B and C entries
    assert_eq!(chain.depth(), 0);
    assert_eq!(chain.current_holder(), Some(&did_a));
}

#[test]
fn relending_chain_return_at_not_found() {
    let did_a = crate::types::Did::new("did:key:z6MkA");
    let mut chain = RelendingChain::with_initial(did_a, [1u8; 32]);
    let did_x = crate::types::Did::new("did:key:z6MkX");
    assert!(chain.return_at(&did_x).is_err());
}

#[test]
fn relending_chain_serde_roundtrip() {
    let did_a = crate::types::Did::new("did:key:z6MkA");
    let chain = RelendingChain::with_initial(did_a, [1u8; 32]);
    let json = serde_json::to_string(&chain).expect("serialize");
    let restored: RelendingChain = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.depth(), chain.depth());
}

#[test]
fn relending_chain_new_empty() {
    let chain = RelendingChain::new();
    assert_eq!(chain.depth(), 0);
    assert!(chain.current_holder().is_none());
}

#[test]
fn departure_reason_display_relend() {
    let wp_id = crate::types::SpineId::now_v7();
    let reason = DepartureReason::Relend {
        target_waypoint: wp_id,
    };
    let display = reason.to_string();
    assert!(display.starts_with("relend:"));
    assert!(display.contains(&wp_id.to_string()));
}

#[test]
fn slice_operation_type_names() {
    assert_eq!(SliceOperationType::View { viewport: None }.name(), "view");
    assert_eq!(SliceOperationType::Read { pages: None }.name(), "read");
    assert_eq!(
        SliceOperationType::Edit {
            operation_type: "insert".into()
        }
        .name(),
        "edit"
    );
    assert_eq!(
        SliceOperationType::Export {
            format: "json".into()
        }
        .name(),
        "export"
    );
    assert_eq!(
        SliceOperationType::Custom {
            operation_name: "special".into(),
        }
        .name(),
        "special"
    );
}
