// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn ephemeral_session_id_roundtrip() {
    let uuid = uuid::Uuid::now_v7();
    let ephemeral: EphemeralSessionId = uuid.try_into().unwrap();
    let back: uuid::Uuid = ephemeral.try_into().unwrap();
    assert_eq!(uuid, back);
}

#[test]
fn ephemeral_session_id_from_invalid_hex_fails() {
    let id = EphemeralSessionId::new("not-a-valid-uuid");
    let result: Result<uuid::Uuid, _> = id.try_into();
    assert!(result.is_err());
}

#[test]
fn ephemeral_content_hash_roundtrip() {
    let hash: EntryHash = blake3::hash(b"test").into();
    let ephemeral: EphemeralContentHash = hash.try_into().unwrap();
    let back: EntryHash = ephemeral.try_into().unwrap();
    assert_eq!(hash, back);
}

#[test]
fn ephemeral_content_hash_from_invalid_hex_fails() {
    let h = EphemeralContentHash::new("not-valid-hex");
    let result: Result<EntryHash, _> = h.try_into();
    assert!(result.is_err());
}

#[test]
fn ephemeral_content_hash_wrong_length_fails() {
    let h = EphemeralContentHash::new("abc"); // too short
    let result: Result<EntryHash, _> = h.try_into();
    assert!(result.is_err());
}

#[test]
fn braid_ref_roundtrip() {
    let uuid = uuid::Uuid::now_v7();
    let braid: BraidRef = uuid.try_into().unwrap();
    assert!(braid.as_str().starts_with(BraidRef::URN_PREFIX));
    let back: uuid::Uuid = braid.try_into().unwrap();
    assert_eq!(uuid, back);
}

#[test]
fn braid_ref_wrong_prefix_fails() {
    let br = BraidRef::new("urn:other:12345678-1234-1234-1234-123456789012");
    let result: Result<uuid::Uuid, _> = br.try_into();
    assert!(result.is_err());
}

#[test]
fn ephemeral_session_id_display() {
    let id = EphemeralSessionId::new("abc123");
    assert_eq!(id.to_string(), "abc123");
}

#[test]
fn braid_ref_display() {
    let br = BraidRef::new("urn:braid:12345678-1234-1234-1234-123456789012");
    assert!(br.to_string().contains("urn:braid:"));
}

#[test]
fn ephemeral_content_hash_display() {
    let h = EphemeralContentHash::new("deadbeef");
    assert_eq!(h.to_string(), "deadbeef");
}

#[test]
fn trio_commit_request_serialization() {
    let req = TrioCommitRequest {
        session_id: EphemeralSessionId::new(uuid::Uuid::now_v7().as_simple().to_string()),
        content_hash: EphemeralContentHash::new("a".repeat(64)),
        committer: Did::new("did:key:z6MkTest"),
        braid_ref: Some(BraidRef::new(format!("urn:braid:{}", uuid::Uuid::now_v7()))),
        signature: None,
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let _: TrioCommitRequest = serde_json::from_str(&json).expect("deserialize");
}

#[test]
fn trio_commit_receipt_serialization() {
    let hash: EntryHash = blake3::hash(b"receipt").into();
    let receipt = TrioCommitReceipt {
        spine_id: uuid::Uuid::now_v7(),
        entry_hash: hash,
        entry_index: 42,
        committed_at: Timestamp::now(),
    };
    let json = serde_json::to_string(&receipt).expect("serialize");
    let _: TrioCommitReceipt = serde_json::from_str(&json).expect("deserialize");
}

#[test]
fn wire_dehydration_summary_deserializes() {
    let payload = serde_json::json!({
        "session_id": "sess-1",
        "source_primal": "rhizoCrypt",
        "merkle_root": "sha256:abc",
        "vertex_count": 10,
        "agents": ["did:key:z6MkAlice"],
        "session_type": "experiment",
        "outcome": "Success"
    });
    let s: WireDehydrationSummary = serde_json::from_value(payload).unwrap();
    assert_eq!(s.session_id, "sess-1");
    assert_eq!(s.vertex_count, 10);
    assert_eq!(s.branch_count, 0); // serde default
}

#[test]
fn trio_receipt_to_pipeline_result() {
    let hash: EntryHash = blake3::hash(b"test").into();
    let receipt = TrioCommitReceipt {
        spine_id: uuid::Uuid::now_v7(),
        entry_hash: hash,
        entry_index: 1,
        committed_at: Timestamp::now(),
    };
    let result = receipt.to_pipeline_result("sha256:abc", Some("urn:braid:123".into()));
    assert_eq!(result.dehydration_merkle_root, "sha256:abc");
    assert!(!result.commit_ref.is_empty());
    assert_eq!(result.braid_ref.as_deref(), Some("urn:braid:123"));
}

#[test]
fn ephemeral_session_id_as_str_accessor() {
    let id = EphemeralSessionId::new("session-42");
    assert_eq!(id.as_str(), "session-42");
}

#[test]
fn ephemeral_content_hash_as_str_accessor() {
    let h = EphemeralContentHash::new("deadbeef");
    assert_eq!(h.as_str(), "deadbeef");
}

#[test]
fn wire_agent_contribution_default_weight() {
    let json = r#"{"agent_did":"did:key:z6MkTest","description":"test agent"}"#;
    let agent: WireAgentContribution = serde_json::from_str(json).unwrap();
    assert!((agent.weight - 1.0).abs() < f64::EPSILON);
}
