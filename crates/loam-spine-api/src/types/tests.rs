// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

use loam_spine_core::trio_types::WireDehydrationSummary;

#[test]
fn create_spine_request_roundtrip() {
    let req = CreateSpineRequest {
        name: "test-spine".to_string(),
        owner: Did::new("did:key:test"),
        config: None,
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let parsed: CreateSpineRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.name, "test-spine");
}

#[test]
fn create_spine_response_roundtrip() {
    let resp = CreateSpineResponse {
        spine_id: uuid::Uuid::nil(),
        genesis_hash: [0u8; 32],
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    assert!(json.contains("spine_id"));
    let parsed: CreateSpineResponse = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.spine_id, uuid::Uuid::nil());
}

#[test]
fn get_spine_request_roundtrip() {
    let req = GetSpineRequest {
        spine_id: uuid::Uuid::now_v7(),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let parsed: GetSpineRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.spine_id, req.spine_id);
}

#[test]
fn seal_spine_request_roundtrip() {
    let req = SealSpineRequest {
        spine_id: uuid::Uuid::now_v7(),
        sealer: Did::new("did:key:sealer"),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let parsed: SealSpineRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.spine_id, req.spine_id);
}

#[test]
fn append_entry_request_roundtrip() {
    let req = AppendEntryRequest {
        spine_id: uuid::Uuid::now_v7(),
        entry_type: EntryType::SpineSealed { reason: None },
        committer: Did::new("did:key:z6Mk1"),
        payload: None,
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let parsed: AppendEntryRequest = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(parsed.entry_type, EntryType::SpineSealed { .. }));
}

#[test]
fn mint_certificate_request_roundtrip() {
    let req = MintCertificateRequest {
        spine_id: uuid::Uuid::now_v7(),
        cert_type: CertificateType::DigitalCollectible {
            collection_id: "test-collection".into(),
            item_number: Some(1),
            total_supply: Some(100),
            rarity: Some(Rarity::Rare),
        },
        owner: Did::new("did:key:owner"),
        metadata: None,
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let parsed: MintCertificateRequest = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(
        parsed.cert_type,
        CertificateType::DigitalCollectible { .. }
    ));
}

#[test]
fn health_check_response_roundtrip() {
    let resp = HealthCheckResponse {
        status: HealthStatus::Healthy,
        report: None,
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    let parsed: HealthCheckResponse = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(parsed.status, HealthStatus::Healthy));
}

#[test]
fn commit_session_request_roundtrip() {
    let req = CommitSessionRequest {
        spine_id: uuid::Uuid::now_v7(),
        session_id: uuid::Uuid::now_v7(),
        session_hash: [1u8; 32],
        vertex_count: 42,
        committer: Did::new("did:key:committer"),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let parsed: CommitSessionRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.vertex_count, 42);
}

#[test]
fn permanent_storage_commit_request_roundtrip() {
    let req = PermanentStorageCommitRequest {
        session_id: uuid::Uuid::now_v7().to_string(),
        merkle_root: "a".repeat(64),
        committer_did: Some("did:key:z6MkTest".into()),
        summary: PermanentStorageDehydrationSummary {
            session_type: "game".into(),
            vertex_count: 10,
            leaf_count: 5,
            started_at: 1_000_000,
            ended_at: 2_000_000,
            outcome: "committed".to_string(),
        },
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let parsed: PermanentStorageCommitRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.summary.vertex_count, 10);
}

#[test]
fn permanent_storage_commit_response_roundtrip() {
    let resp = PermanentStorageCommitResponse {
        accepted: true,
        commit_id: Some("abc123".into()),
        spine_entry_hash: Some("def456".into()),
        entry_index: Some(5),
        spine_id: Some(uuid::Uuid::nil().to_string()),
        error: None,
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    let parsed: PermanentStorageCommitResponse = serde_json::from_str(&json).expect("deserialize");
    assert!(parsed.accepted);
    assert_eq!(parsed.entry_index, Some(5));
}

#[test]
fn serde_opt_bytes_none_roundtrip() {
    let req = AppendEntryRequest {
        spine_id: uuid::Uuid::now_v7(),
        entry_type: EntryType::SpineSealed { reason: None },
        committer: Did::new("did:key:z6Mk1"),
        payload: None,
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let parsed: AppendEntryRequest = serde_json::from_str(&json).expect("deserialize");
    assert!(parsed.payload.is_none());
}

#[test]
fn serde_opt_bytes_some_roundtrip() {
    let payload_data = vec![1u8, 2, 3, 4, 5];
    let req = AppendEntryRequest {
        spine_id: uuid::Uuid::now_v7(),
        entry_type: EntryType::SpineSealed { reason: None },
        committer: Did::new("did:key:z6Mk1"),
        payload: Some(ByteBuffer::from(payload_data)),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let parsed: AppendEntryRequest = serde_json::from_str(&json).expect("deserialize");
    assert!(parsed.payload.is_some());
}

#[test]
fn anchor_slice_request_roundtrip() {
    let req = AnchorSliceRequest {
        waypoint_spine_id: uuid::Uuid::now_v7(),
        origin_spine_id: uuid::Uuid::now_v7(),
        slice_id: uuid::Uuid::now_v7(),
        committer: Did::new("did:key:z6MkWaypoint"),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let parsed: AnchorSliceRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.waypoint_spine_id, req.waypoint_spine_id);
}

#[test]
fn checkout_slice_request_roundtrip() {
    let req = CheckoutSliceRequest {
        waypoint_spine_id: uuid::Uuid::now_v7(),
        slice_id: uuid::Uuid::now_v7(),
        requester: Did::new("did:key:z6MkRequester"),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let parsed: CheckoutSliceRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.slice_id, req.slice_id);
}

#[test]
fn commit_braid_request_roundtrip() {
    let req = CommitBraidRequest {
        spine_id: uuid::Uuid::now_v7(),
        braid_id: uuid::Uuid::now_v7(),
        braid_hash: [2u8; 32],
        subjects: vec![Did::new("did:key:agent1"), Did::new("did:key:agent2")],
        committer: Did::new("did:key:z6MkCommitter"),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let parsed: CommitBraidRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.subjects.len(), 2);
}

#[test]
fn permanent_storage_verify_request_roundtrip() {
    let req = PermanentStorageVerifyRequest {
        spine_id: uuid::Uuid::nil().to_string(),
        entry_hash: "b".repeat(64),
        index: 7,
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let parsed: PermanentStorageVerifyRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.index, 7);
}

#[test]
fn permanent_storage_dehydration_summary_from_wire() {
    let wire = WireDehydrationSummary {
        source_primal: "rhizoCrypt".into(),
        session_id: "sess-1".into(),
        merkle_root: "c".repeat(64),
        vertex_count: 10,
        branch_count: 3,
        payload_bytes: 0,
        agents: vec!["did:key:agent".into()],
        session_start: 100,
        dehydrated_at: 200,
        session_type: "experiment".into(),
        outcome: "Success".into(),
        agent_summaries: vec![],
        witnesses: vec![],
        operations: vec![],
        frontier: vec![],
        niche: None,
        compression_ratio: None,
    };
    let summary = PermanentStorageDehydrationSummary::from(&wire);
    assert_eq!(summary.session_type, "experiment");
    assert_eq!(summary.vertex_count, 10);
    assert_eq!(summary.leaf_count, 3);
    assert_eq!(summary.started_at, 100);
    assert_eq!(summary.ended_at, 200);
    assert_eq!(summary.outcome, "Success");
}

#[test]
fn permanent_storage_commit_request_from_wire() {
    let wire = WireDehydrationSummary {
        source_primal: "rhizoCrypt".into(),
        session_id: "550e8400-e29b-41d4-a716-446655440000".into(),
        merkle_root: "d".repeat(64),
        vertex_count: 5,
        branch_count: 2,
        payload_bytes: 0,
        agents: vec!["did:key:first".into(), "did:key:second".into()],
        session_start: 1,
        dehydrated_at: 2,
        session_type: "game".into(),
        outcome: "Failure { reason: test }".into(),
        agent_summaries: vec![],
        witnesses: vec![],
        operations: vec![],
        frontier: vec![],
        niche: None,
        compression_ratio: None,
    };
    let req = PermanentStorageCommitRequest::from(&wire);
    assert_eq!(req.session_id, wire.session_id);
    assert_eq!(req.merkle_root, wire.merkle_root);
    assert_eq!(req.committer_did.as_deref(), Some("did:key:first"));
    assert_eq!(req.summary.leaf_count, wire.branch_count);
    assert_eq!(req.summary.outcome, wire.outcome);
}

#[test]
fn permanent_storage_get_commit_request_roundtrip() {
    let req = PermanentStorageGetCommitRequest {
        spine_id: uuid::Uuid::now_v7().to_string(),
        entry_hash: "e".repeat(64),
        index: 42,
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let parsed: PermanentStorageGetCommitRequest =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.spine_id, req.spine_id);
    assert_eq!(parsed.entry_hash, req.entry_hash);
    assert_eq!(parsed.index, 42);
}

#[test]
fn permanent_storage_commit_response_rejected_roundtrip() {
    let resp = PermanentStorageCommitResponse {
        accepted: false,
        commit_id: None,
        spine_entry_hash: None,
        entry_index: None,
        spine_id: None,
        error: Some("commit rejected: bad merkle root".into()),
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    let parsed: PermanentStorageCommitResponse = serde_json::from_str(&json).expect("deserialize");
    assert!(!parsed.accepted);
    assert_eq!(
        parsed.error.as_deref(),
        Some("commit rejected: bad merkle root")
    );
    assert!(parsed.commit_id.is_none());
}
