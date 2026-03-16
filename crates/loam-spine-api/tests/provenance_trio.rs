// SPDX-License-Identifier: AGPL-3.0-or-later

//! Provenance Trio integration tests.
//!
//! Exercises the full coordination flows between the three provenance primals:
//!
//! - **rhizoCrypt** (ephemeral DAG workspace) → `loamSpine` via dehydration
//! - **sweetGrass** (semantic attribution) → `loamSpine` via braid anchoring
//!
//! These tests simulate what rhizoCrypt and sweetGrass send on the wire,
//! validating that loamSpine's `permanent-storage.*` compatibility layer
//! and native `session.commit` / `braid.commit` APIs work end-to-end.

#![allow(missing_docs)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use loam_spine_api::service::LoamSpineRpcService;
use loam_spine_api::types::*;
use uuid::Uuid;

fn test_did() -> Did {
    Did::new(format!("did:key:z6Mk{}", Uuid::now_v7().simple()))
}

fn blake3_hash(data: &[u8]) -> ContentHash {
    *blake3::hash(data).as_bytes()
}

fn hash_to_hex(hash: &ContentHash) -> String {
    use std::fmt::Write;
    hash.iter().fold(String::with_capacity(64), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

// ============================================================================
// Dehydration Flow: rhizoCrypt → loamSpine
// ============================================================================

#[tokio::test]
async fn dehydration_flow_native_session_commit() {
    let service = LoamSpineRpcService::default_service();
    let rhizocrypt_did = test_did();

    let spine = service
        .create_spine(CreateSpineRequest {
            owner: rhizocrypt_did.clone(),
            name: "rhizoCrypt permanence spine".to_string(),
            config: None,
        })
        .await
        .expect("create spine");

    let session_id = Uuid::now_v7();
    let merkle_root = blake3_hash(b"session-dag-root-42-vertices");

    let commit_resp = service
        .commit_session(CommitSessionRequest {
            spine_id: spine.spine_id,
            session_id,
            session_hash: merkle_root,
            vertex_count: 42,
            committer: rhizocrypt_did.clone(),
        })
        .await
        .expect("session commit");

    assert!(
        commit_resp.commit_hash.iter().any(|&b| b != 0),
        "commit hash should be non-zero"
    );
    assert!(commit_resp.index > 0, "should be after genesis");

    let entry_resp = service
        .get_entry(GetEntryRequest {
            spine_id: spine.spine_id,
            entry_hash: commit_resp.commit_hash,
        })
        .await
        .expect("get entry");

    assert!(entry_resp.found, "committed session entry should exist");
    let entry = entry_resp.entry.expect("entry present");
    assert!(
        matches!(entry.entry_type, EntryType::SessionCommit { .. }),
        "entry type should be SessionCommit"
    );

    if let EntryType::SessionCommit {
        session_id: stored_sid,
        merkle_root: stored_root,
        vertex_count,
        ..
    } = &entry.entry_type
    {
        assert_eq!(*stored_sid, session_id, "session ID should match");
        assert_eq!(*stored_root, merkle_root, "merkle root should match");
        assert_eq!(*vertex_count, 42, "vertex count should match");
    }
}

#[tokio::test]
async fn dehydration_flow_permanent_storage_compat() {
    let service = LoamSpineRpcService::default_service();
    let rhizocrypt_did = test_did();

    let session_id = Uuid::now_v7();
    let merkle_root = blake3_hash(b"dehydrated-session-100-vertices");

    let compat_resp = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: session_id.to_string(),
            merkle_root: hash_to_hex(&merkle_root),
            summary: PermanentStorageDehydrationSummary {
                session_type: "game".to_string(),
                vertex_count: 100,
                leaf_count: 5,
                started_at: 1_710_000_000_000_000_000,
                ended_at: 1_710_000_060_000_000_000,
                outcome: "Success".to_string(),
            },
            committer_did: Some(rhizocrypt_did.as_str().to_string()),
        })
        .await
        .expect("permanent-storage.commitSession");

    assert!(compat_resp.accepted, "commit should be accepted");
    assert!(compat_resp.commit_id.is_some(), "should return commit_id");
    assert!(
        compat_resp.spine_entry_hash.is_some(),
        "should return spine_entry_hash"
    );
    assert!(
        compat_resp.entry_index.is_some(),
        "should return entry_index"
    );
    assert!(compat_resp.spine_id.is_some(), "should return spine_id");
    assert!(compat_resp.error.is_none(), "should have no error");

    let spine_id: Uuid = compat_resp
        .spine_id
        .as_ref()
        .expect("spine_id")
        .parse()
        .expect("valid UUID");

    let verify_resp = service
        .permanent_storage_verify_commit(PermanentStorageVerifyRequest {
            spine_id: spine_id.to_string(),
            entry_hash: compat_resp.spine_entry_hash.clone().expect("hash"),
            index: compat_resp.entry_index.expect("index"),
        })
        .await
        .expect("permanent-storage.verifyCommit");

    assert!(verify_resp, "committed session should verify");
}

#[tokio::test]
async fn dehydration_flow_compat_auto_creates_spine() {
    let service = LoamSpineRpcService::default_service();
    let new_did = test_did();

    let resp1 = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: Uuid::now_v7().to_string(),
            merkle_root: hash_to_hex(&blake3_hash(b"session-1")),
            summary: PermanentStorageDehydrationSummary {
                session_type: "transaction".to_string(),
                vertex_count: 10,
                leaf_count: 2,
                started_at: 0,
                ended_at: 1,
                outcome: "Success".to_string(),
            },
            committer_did: Some(new_did.as_str().to_string()),
        })
        .await
        .expect("first commit");

    assert!(resp1.accepted);
    let first_spine = resp1.spine_id.clone().expect("spine_id");

    let resp2 = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: Uuid::now_v7().to_string(),
            merkle_root: hash_to_hex(&blake3_hash(b"session-2")),
            summary: PermanentStorageDehydrationSummary {
                session_type: "transaction".to_string(),
                vertex_count: 20,
                leaf_count: 3,
                started_at: 2,
                ended_at: 3,
                outcome: "Success".to_string(),
            },
            committer_did: Some(new_did.as_str().to_string()),
        })
        .await
        .expect("second commit");

    assert!(resp2.accepted);
    assert_eq!(
        resp2.spine_id.as_ref().expect("spine_id"),
        &first_spine,
        "should reuse same spine for same committer"
    );
}

#[tokio::test]
async fn dehydration_flow_compat_invalid_hex_rejected() {
    let service = LoamSpineRpcService::default_service();

    let resp = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: Uuid::now_v7().to_string(),
            merkle_root: "not-valid-hex".to_string(),
            summary: PermanentStorageDehydrationSummary {
                session_type: "test".to_string(),
                vertex_count: 0,
                leaf_count: 0,
                started_at: 0,
                ended_at: 0,
                outcome: "Success".to_string(),
            },
            committer_did: None,
        })
        .await;

    assert!(resp.is_err(), "invalid hex should be rejected");
}

#[tokio::test]
async fn dehydration_flow_compat_invalid_session_id_rejected() {
    let service = LoamSpineRpcService::default_service();

    let resp = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: "not-a-uuid".to_string(),
            merkle_root: hash_to_hex(&[0u8; 32]),
            summary: PermanentStorageDehydrationSummary {
                session_type: "test".to_string(),
                vertex_count: 0,
                leaf_count: 0,
                started_at: 0,
                ended_at: 0,
                outcome: "Success".to_string(),
            },
            committer_did: None,
        })
        .await;

    assert!(resp.is_err(), "invalid session_id should be rejected");
}

// ============================================================================
// Braid Anchoring Flow: sweetGrass → loamSpine
// ============================================================================

#[tokio::test]
async fn braid_anchoring_flow() {
    let service = LoamSpineRpcService::default_service();
    let sweetgrass_did = test_did();
    let subject_did = test_did();

    let spine = service
        .create_spine(CreateSpineRequest {
            owner: sweetgrass_did.clone(),
            name: "sweetGrass attribution spine".to_string(),
            config: None,
        })
        .await
        .expect("create spine");

    let braid_id = Uuid::now_v7();
    let braid_hash = blake3_hash(b"braid-content-hash-attribution");

    let commit_resp = service
        .commit_braid(CommitBraidRequest {
            spine_id: spine.spine_id,
            braid_id,
            braid_hash,
            subjects: vec![subject_did.clone(), sweetgrass_did.clone()],
            committer: sweetgrass_did.clone(),
        })
        .await
        .expect("braid commit");

    assert!(
        commit_resp.commit_hash.iter().any(|&b| b != 0),
        "braid commit hash should be non-zero"
    );

    let entry_resp = service
        .get_entry(GetEntryRequest {
            spine_id: spine.spine_id,
            entry_hash: commit_resp.commit_hash,
        })
        .await
        .expect("get braid entry");

    assert!(entry_resp.found, "committed braid entry should exist");
    let entry = entry_resp.entry.expect("entry present");
    assert!(
        matches!(entry.entry_type, EntryType::BraidCommit { .. }),
        "entry type should be BraidCommit"
    );

    if let EntryType::BraidCommit {
        braid_id: stored_bid,
        braid_hash: stored_hash,
        ..
    } = &entry.entry_type
    {
        assert_eq!(*stored_bid, braid_id, "braid ID should match");
        assert_eq!(*stored_hash, braid_hash, "braid hash should match");
    }
}

#[tokio::test]
async fn braid_anchoring_with_proof() {
    let service = LoamSpineRpcService::default_service();
    let sweetgrass_did = test_did();

    let spine = service
        .create_spine(CreateSpineRequest {
            owner: sweetgrass_did.clone(),
            name: "braid proof spine".to_string(),
            config: None,
        })
        .await
        .expect("create spine");

    let commit_resp = service
        .commit_braid(CommitBraidRequest {
            spine_id: spine.spine_id,
            braid_id: Uuid::now_v7(),
            braid_hash: blake3_hash(b"braid-for-proof"),
            subjects: vec![sweetgrass_did.clone()],
            committer: sweetgrass_did,
        })
        .await
        .expect("braid commit");

    let proof_resp = service
        .generate_inclusion_proof(GenerateInclusionProofRequest {
            spine_id: spine.spine_id,
            entry_hash: commit_resp.commit_hash,
        })
        .await
        .expect("generate proof");

    let verify_resp = service
        .verify_inclusion_proof(VerifyInclusionProofRequest {
            proof: proof_resp.proof,
        })
        .await
        .expect("verify proof");

    assert!(
        verify_resp.valid,
        "inclusion proof for braid commit should be valid"
    );
}

// ============================================================================
// Full Provenance Trio: rhizoCrypt → loamSpine ← sweetGrass
// ============================================================================

#[tokio::test]
async fn full_provenance_trio_flow() {
    let service = LoamSpineRpcService::default_service();

    let rhizocrypt_did = Did::new("did:key:z6MkRhizoCryptAgent001");
    let sweetgrass_did = Did::new("did:key:z6MkSweetGrassAgent001");
    let player_did = Did::new("did:key:z6MkPlayer001");

    // Step 1: rhizoCrypt dehydrates a game session and commits to loamSpine
    let permanence_spine = service
        .create_spine(CreateSpineRequest {
            owner: rhizocrypt_did.clone(),
            name: "game-permanence".to_string(),
            config: None,
        })
        .await
        .expect("permanence spine");

    let session_id = Uuid::now_v7();
    let session_dag_root = blake3_hash(b"game-session-42-moves-player-won");

    let session_commit = service
        .commit_session(CommitSessionRequest {
            spine_id: permanence_spine.spine_id,
            session_id,
            session_hash: session_dag_root,
            vertex_count: 42,
            committer: rhizocrypt_did.clone(),
        })
        .await
        .expect("session commit");

    // Step 2: sweetGrass creates an attribution braid for the session
    //         and anchors it on the same permanence spine
    let braid_id = Uuid::now_v7();
    let braid_content = format!(
        "attribution:session={session_id}:player={}:outcome=victory",
        player_did.as_str()
    );
    let braid_hash = blake3_hash(braid_content.as_bytes());

    let braid_commit = service
        .commit_braid(CommitBraidRequest {
            spine_id: permanence_spine.spine_id,
            braid_id,
            braid_hash,
            subjects: vec![player_did.clone(), rhizocrypt_did.clone()],
            committer: sweetgrass_did.clone(),
        })
        .await
        .expect("braid commit");

    // Step 3: Verify both commits exist and are provable
    let session_proof = service
        .generate_inclusion_proof(GenerateInclusionProofRequest {
            spine_id: permanence_spine.spine_id,
            entry_hash: session_commit.commit_hash,
        })
        .await
        .expect("session proof");

    let braid_proof = service
        .generate_inclusion_proof(GenerateInclusionProofRequest {
            spine_id: permanence_spine.spine_id,
            entry_hash: braid_commit.commit_hash,
        })
        .await
        .expect("braid proof");

    assert!(
        service
            .verify_inclusion_proof(VerifyInclusionProofRequest {
                proof: session_proof.proof,
            })
            .await
            .expect("verify session")
            .valid,
        "session commit proof should be valid"
    );

    assert!(
        service
            .verify_inclusion_proof(VerifyInclusionProofRequest {
                proof: braid_proof.proof,
            })
            .await
            .expect("verify braid")
            .valid,
        "braid commit proof should be valid"
    );

    // Step 4: Verify the spine contains both entries (session + braid + genesis)
    let tip = service
        .get_tip(GetTipRequest {
            spine_id: permanence_spine.spine_id,
        })
        .await
        .expect("get tip");

    assert!(
        tip.height >= 3,
        "spine should have genesis + session + braid = at least 3 entries"
    );
}

#[tokio::test]
async fn provenance_trio_via_compat_layer() {
    let service = LoamSpineRpcService::default_service();
    let rhizocrypt_did = test_did();
    let sweetgrass_did = test_did();

    // Step 1: rhizoCrypt commits via permanent-storage.commitSession (compat layer)
    let session_id = Uuid::now_v7();
    let merkle_root = blake3_hash(b"compat-session-dag");

    let ps_resp = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: session_id.to_string(),
            merkle_root: hash_to_hex(&merkle_root),
            summary: PermanentStorageDehydrationSummary {
                session_type: "game".to_string(),
                vertex_count: 50,
                leaf_count: 8,
                started_at: 1_710_000_000_000_000_000,
                ended_at: 1_710_000_120_000_000_000,
                outcome: "Success".to_string(),
            },
            committer_did: Some(rhizocrypt_did.as_str().to_string()),
        })
        .await
        .expect("compat commit");

    assert!(ps_resp.accepted);
    let spine_id: Uuid = ps_resp
        .spine_id
        .as_ref()
        .expect("spine_id")
        .parse()
        .expect("valid UUID");

    // Step 2: sweetGrass anchors a braid on the same spine
    let braid_commit = service
        .commit_braid(CommitBraidRequest {
            spine_id,
            braid_id: Uuid::now_v7(),
            braid_hash: blake3_hash(b"attribution-for-compat-session"),
            subjects: vec![rhizocrypt_did, sweetgrass_did.clone()],
            committer: sweetgrass_did,
        })
        .await
        .expect("braid commit on compat spine");

    assert!(
        braid_commit.commit_hash.iter().any(|&b| b != 0),
        "braid commit should succeed on auto-created spine"
    );

    // Step 3: Verify via compat layer
    let verified = service
        .permanent_storage_verify_commit(PermanentStorageVerifyRequest {
            spine_id: spine_id.to_string(),
            entry_hash: ps_resp.spine_entry_hash.expect("hash"),
            index: ps_resp.entry_index.expect("index"),
        })
        .await
        .expect("verify via compat");

    assert!(verified, "session commit should be verifiable via compat");
}

// ============================================================================
// Health check compatibility
// ============================================================================

#[tokio::test]
async fn permanent_storage_health_check() {
    use loam_spine_api::jsonrpc::{JsonRpcRequest, LoamSpineJsonRpc};

    let server = LoamSpineJsonRpc::default_server();
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "permanence.health_check".to_string(),
        params: serde_json::Value::Null,
        id: serde_json::Value::Number(1.into()),
    };
    let resp = server.handle_request(req).await;
    assert!(resp.error.is_none());
    assert_eq!(resp.result.unwrap(), serde_json::Value::Bool(true));
}
