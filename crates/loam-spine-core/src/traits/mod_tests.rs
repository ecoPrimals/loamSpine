// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::types::{BraidId, Did, Signature};

#[test]
fn braid_summary_creation() {
    let braid_id = BraidId::now_v7();
    let summary = BraidSummary::new(braid_id, "attribution", [1u8; 32], [2u8; 32]);

    assert_eq!(summary.braid_type, "attribution");
    assert_eq!(summary.subject_hash, [1u8; 32]);
    assert_eq!(summary.braid_hash, [2u8; 32]);
    assert!(summary.agents.is_empty());
    assert!(summary.signature.is_none());
}

#[test]
fn braid_summary_with_agent() {
    let braid_id = BraidId::now_v7();
    let agent = Did::new("did:key:z6MkAgent");
    let summary = BraidSummary::new(braid_id, "derivation", [0u8; 32], [0u8; 32]).with_agent(agent);

    assert_eq!(summary.agents.len(), 1);
}

#[test]
fn braid_summary_with_multiple_agents() {
    let braid_id = BraidId::now_v7();
    let agent1 = Did::new("did:key:z6MkAgent1");
    let agent2 = Did::new("did:key:z6MkAgent2");

    let summary = BraidSummary::new(braid_id, "collab", [0u8; 32], [0u8; 32])
        .with_agent(agent1)
        .with_agent(agent2);

    assert_eq!(summary.agents.len(), 2);
}

#[test]
fn braid_summary_with_signature() {
    let braid_id = BraidId::now_v7();
    let sig = Signature::from_vec(vec![1, 2, 3, 4]);
    let summary = BraidSummary::new(braid_id, "signed", [0u8; 32], [0u8; 32]).with_signature(sig);

    assert!(summary.signature.is_some());
}

#[test]
fn braid_summary_chained_builders() {
    let braid_id = BraidId::now_v7();
    let agent = Did::new("did:key:z6MkAgent");
    let sig = Signature::from_vec(vec![5, 6, 7]);

    let summary = BraidSummary::new(braid_id, "full", [3u8; 32], [4u8; 32])
        .with_agent(agent)
        .with_signature(sig);

    assert_eq!(summary.agents.len(), 1);
    assert!(summary.signature.is_some());
}

#[test]
fn braid_summary_debug_and_clone() {
    let braid_id = BraidId::now_v7();
    let summary = BraidSummary::new(braid_id, "test", [0u8; 32], [0u8; 32]);

    let cloned = summary.clone();
    assert_eq!(summary.braid_type, cloned.braid_type);

    let debug_str = format!("{summary:?}");
    assert!(debug_str.contains("BraidSummary"));
}

#[test]
fn attribution_record_creation() {
    let record = AttributionRecord {
        content_hash: [1u8; 32],
        creator: Did::new("did:key:z6MkCreator"),
        contributors: vec![Did::new("did:key:z6MkContrib1")],
        certificate_id: None,
        recorded_at: crate::types::Timestamp::now(),
    };

    assert_eq!(record.content_hash, [1u8; 32]);
    assert_eq!(record.contributors.len(), 1);
    assert!(record.certificate_id.is_none());
}

#[test]
fn provenance_link_creation() {
    let link = ProvenanceLink {
        entry_hash: [2u8; 32],
        spine_id: crate::types::SpineId::now_v7(),
        index: 42,
        agent: Did::new("did:key:z6MkAgent"),
        timestamp: crate::types::Timestamp::now(),
        relationship: "derived-from".to_string(),
    };

    assert_eq!(link.index, 42);
    assert_eq!(link.relationship, "derived-from");
}

#[test]
fn sync_result_creation() {
    let result = SyncResult {
        accepted: 10,
        rejected: 2,
        rejection_reasons: vec!["duplicate".to_string()],
    };

    assert_eq!(result.accepted, 10);
    assert_eq!(result.rejected, 2);
    assert_eq!(result.rejection_reasons.len(), 1);
}

#[test]
fn sync_status_variants() {
    assert_eq!(SyncStatus::InSync, SyncStatus::InSync);
    assert_ne!(SyncStatus::InSync, SyncStatus::Unknown);

    let ahead = SyncStatus::LocalAhead { entries_ahead: 5 };
    assert!(matches!(ahead, SyncStatus::LocalAhead { entries_ahead: 5 }));

    let behind = SyncStatus::RemoteAhead { entries_behind: 3 };
    assert!(matches!(
        behind,
        SyncStatus::RemoteAhead { entries_behind: 3 }
    ));
}

#[test]
fn integration_types_debug_clone() {
    let record = AttributionRecord {
        content_hash: [0u8; 32],
        creator: Did::new("did:key:test"),
        contributors: vec![],
        certificate_id: None,
        recorded_at: crate::types::Timestamp::now(),
    };
    let cloned = record.clone();
    assert_eq!(record.content_hash, cloned.content_hash);
    let _ = format!("{record:?}");

    let link = ProvenanceLink {
        entry_hash: [0u8; 32],
        spine_id: crate::types::SpineId::now_v7(),
        index: 0,
        agent: Did::new("did:key:test"),
        timestamp: crate::types::Timestamp::now(),
        relationship: "test".to_string(),
    };
    let cloned = link.clone();
    assert_eq!(link.relationship, cloned.relationship);
    let _ = format!("{link:?}");

    let result = SyncResult {
        accepted: 0,
        rejected: 0,
        rejection_reasons: vec![],
    };
    let cloned = result.clone();
    assert_eq!(result.accepted, cloned.accepted);
    let _ = format!("{result:?}");
}
