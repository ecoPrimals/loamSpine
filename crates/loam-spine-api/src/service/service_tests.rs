// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::types::{
    PermanentStorageCommitRequest, PermanentStorageDehydrationSummary,
    PermanentStorageGetCommitRequest, PermanentStorageVerifyRequest,
};
use loam_spine_core::KB;

#[tokio::test]
async fn test_service_creation() {
    let service = LoamSpineRpcService::default_service();
    let result = service
        .health_check(HealthCheckRequest {
            include_details: true,
        })
        .await;
    assert!(result.is_ok());
    let resp = result.expect("health check should succeed");
    assert!(matches!(resp.status, HealthStatus::Healthy));
}

#[tokio::test]
async fn test_create_and_get_spine() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:test");

    let create_resp = service
        .create_spine(CreateSpineRequest {
            name: "test".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create should succeed");

    let get_resp = service
        .get_spine(GetSpineRequest {
            spine_id: create_resp.spine_id,
        })
        .await
        .expect("get should succeed");

    assert!(get_resp.found);
    assert!(get_resp.spine.is_some());
}

#[tokio::test]
async fn test_mint_certificate() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:test-owner");

    // Create spine first
    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "cert-test".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create spine should succeed");

    // Mint a certificate
    let mint_resp = service
        .mint_certificate(MintCertificateRequest {
            spine_id: spine_resp.spine_id,
            cert_type: CertificateType::DigitalGame {
                platform: "steam".into(),
                game_id: "hl3".into(),
                edition: None,
            },
            owner: owner.clone(),
            metadata: None,
        })
        .await
        .expect("mint should succeed");

    assert_ne!(mint_resp.mint_hash, [0u8; 32]);

    // Get the certificate
    let get_resp = service
        .get_certificate(GetCertificateRequest {
            certificate_id: mint_resp.certificate_id,
        })
        .await
        .expect("get certificate should succeed");

    assert!(get_resp.found);
    assert!(get_resp.certificate.is_some());
}

#[tokio::test]
async fn test_certificate_transfer() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:owner");
    let new_owner = Did::new("did:key:new-owner");

    // Create spine and mint certificate
    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "transfer-test".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create should succeed");

    let mint_resp = service
        .mint_certificate(MintCertificateRequest {
            spine_id: spine_resp.spine_id,
            cert_type: CertificateType::SoftwareLicense {
                software_id: "cursor".into(),
                license_type: "pro".into(),
                seats: Some(1),
                expires: None,
            },
            owner: owner.clone(),
            metadata: None,
        })
        .await
        .expect("mint should succeed");

    // Transfer
    let transfer_resp = service
        .transfer_certificate(TransferCertificateRequest {
            certificate_id: mint_resp.certificate_id,
            from: owner,
            to: new_owner.clone(),
        })
        .await
        .expect("transfer should succeed");

    assert!(transfer_resp.success);
    assert!(transfer_resp.transfer_hash.is_some());

    // Verify new owner
    let get_resp = service
        .get_certificate(GetCertificateRequest {
            certificate_id: mint_resp.certificate_id,
        })
        .await
        .expect("get should succeed");

    assert!(get_resp.found);
    let cert = get_resp.certificate.expect("certificate should exist");
    assert_eq!(cert.owner, new_owner);
}

#[tokio::test]
async fn test_certificate_loan_and_return() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:lender");
    let borrower = Did::new("did:key:borrower");

    // Create spine and mint certificate
    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "loan-test".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create spine should succeed");

    let mint_resp = service
        .mint_certificate(MintCertificateRequest {
            spine_id: spine_resp.spine_id,
            cert_type: CertificateType::DigitalCollectible {
                collection_id: "cards".into(),
                item_number: Some(42),
                total_supply: Some(1000),
                rarity: None,
            },
            owner: owner.clone(),
            metadata: None,
        })
        .await
        .expect("mint should succeed");

    // Loan
    let loan_resp = service
        .loan_certificate(LoanCertificateRequest {
            certificate_id: mint_resp.certificate_id,
            lender: owner.clone(),
            borrower: borrower.clone(),
            terms: LoanTerms::default(),
        })
        .await
        .expect("loan should succeed");

    assert!(loan_resp.success);
    assert!(loan_resp.loan_hash.is_some());

    // Return
    let return_resp = service
        .return_certificate(ReturnCertificateRequest {
            certificate_id: mint_resp.certificate_id,
            returner: borrower,
        })
        .await
        .expect("return should succeed");

    assert!(return_resp.success);
    assert!(return_resp.return_hash.is_some());

    // Verify certificate is back with owner
    let get_resp = service
        .get_certificate(GetCertificateRequest {
            certificate_id: mint_resp.certificate_id,
        })
        .await
        .expect("get should succeed");

    let cert = get_resp.certificate.expect("certificate should exist");
    assert_eq!(cert.owner, owner);
    assert!(!cert.is_loaned());
}

#[tokio::test]
async fn test_seal_spine() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:sealer");

    // Create spine
    let create_resp = service
        .create_spine(CreateSpineRequest {
            name: "seal-test".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create should succeed");

    // Seal the spine
    let seal_resp = service
        .seal_spine(SealSpineRequest {
            spine_id: create_resp.spine_id,
            sealer: owner,
        })
        .await
        .expect("seal should succeed");

    assert!(seal_resp.success);
    assert!(seal_resp.seal_hash.is_some());

    // Verify spine is sealed (attempting to seal again should fail)
    let seal_again = service
        .seal_spine(SealSpineRequest {
            spine_id: create_resp.spine_id,
            sealer: Did::new("did:key:other"),
        })
        .await;

    assert!(seal_again.is_err());
}

#[tokio::test]
async fn test_append_entry() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:appender");

    // Create spine
    let create_resp = service
        .create_spine(CreateSpineRequest {
            name: "append-test".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create should succeed");

    // Append a data anchor entry
    let append_resp = service
        .append_entry(AppendEntryRequest {
            spine_id: create_resp.spine_id,
            entry_type: EntryType::DataAnchor {
                data_hash: [42u8; 32],
                mime_type: Some("application/json".into()),
                size: KB,
            },
            committer: owner.clone(),
            payload: None,
        })
        .await
        .expect("append should succeed");

    assert_ne!(append_resp.entry_hash, [0u8; 32]);
    assert_eq!(append_resp.index, 1); // After genesis

    // Get the entry
    let get_resp = service
        .get_entry(GetEntryRequest {
            spine_id: create_resp.spine_id,
            entry_hash: append_resp.entry_hash,
        })
        .await
        .expect("get should succeed");

    assert!(get_resp.found);
    assert!(get_resp.entry.is_some());
}

#[tokio::test]
async fn test_anchor_slice() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:waypoint-owner");

    // Create a waypoint spine
    let waypoint_resp = service
        .create_spine(CreateSpineRequest {
            name: "waypoint-spine".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create waypoint should succeed");

    // Create an origin spine
    let origin_resp = service
        .create_spine(CreateSpineRequest {
            name: "origin-spine".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create origin should succeed");

    // Anchor a slice
    let slice_id = loam_spine_core::types::SliceId::now_v7();
    let anchor_resp = service
        .anchor_slice(AnchorSliceRequest {
            waypoint_spine_id: waypoint_resp.spine_id,
            slice_id,
            origin_spine_id: origin_resp.spine_id,
            committer: owner,
        })
        .await
        .expect("anchor should succeed");

    assert_ne!(anchor_resp.anchor_hash, [0u8; 32]);
}

#[tokio::test]
async fn test_generate_inclusion_proof() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:prover");

    // Create spine
    let create_resp = service
        .create_spine(CreateSpineRequest {
            name: "proof-test".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create should succeed");

    // Append an entry
    let append_resp = service
        .append_entry(AppendEntryRequest {
            spine_id: create_resp.spine_id,
            entry_type: EntryType::DataAnchor {
                data_hash: [99u8; 32],
                mime_type: Some("text/plain".into()),
                size: 512,
            },
            committer: owner,
            payload: None,
        })
        .await
        .expect("append should succeed");

    // Generate inclusion proof for the entry
    let proof_resp = service
        .generate_inclusion_proof(GenerateInclusionProofRequest {
            spine_id: create_resp.spine_id,
            entry_hash: append_resp.entry_hash,
        })
        .await
        .expect("proof generation should succeed");

    // Verify the proof
    assert!(proof_resp.proof.verify().expect("verify"));
    assert_eq!(proof_resp.proof.spine_id, create_resp.spine_id);
}

// ========================================================================
// integration_ops: permanent_storage.* and commit_session
// ========================================================================

fn make_permanent_storage_summary() -> PermanentStorageDehydrationSummary {
    PermanentStorageDehydrationSummary {
        session_type: "game".to_string(),
        vertex_count: 10,
        leaf_count: 5,
        started_at: 0,
        ended_at: 1000,
        outcome: "Success".to_string(),
    }
}

#[tokio::test]
async fn test_permanent_storage_commit_session_invalid_hex_merkle_root() {
    let service = LoamSpineRpcService::default_service();
    let result = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: uuid::Uuid::now_v7().to_string(),
            merkle_root: "not-64-hex-chars".to_string(),
            summary: make_permanent_storage_summary(),
            committer_did: Some("did:key:test".to_string()),
        })
        .await;
    assert!(result.is_err());
    let err = result.expect_err("expected error");
    assert!(err.to_string().contains("invalid merkle_root hex"));
}

#[tokio::test]
async fn test_permanent_storage_commit_session_missing_committer_did() {
    let service = LoamSpineRpcService::default_service();
    let valid_hex = "a".repeat(64);
    let result = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: uuid::Uuid::now_v7().to_string(),
            merkle_root: valid_hex,
            summary: make_permanent_storage_summary(),
            committer_did: None,
        })
        .await;
    assert!(result.is_err());
    let err = result.expect_err("expected error");
    assert!(err.to_string().contains("committer_did is required"));
}

#[tokio::test]
async fn test_permanent_storage_commit_session_invalid_session_id() {
    let service = LoamSpineRpcService::default_service();
    let valid_hex = "a".repeat(64);
    let result = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: "not-a-uuid".to_string(),
            merkle_root: valid_hex,
            summary: make_permanent_storage_summary(),
            committer_did: Some("did:key:test".to_string()),
        })
        .await;
    assert!(result.is_err());
    let err = result.expect_err("expected error");
    assert!(err.to_string().contains("invalid session_id UUID"));
}

#[tokio::test]
async fn test_permanent_storage_commit_session_success() {
    let service = LoamSpineRpcService::default_service();
    let valid_hex = "a".repeat(64);
    let result = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: uuid::Uuid::now_v7().to_string(),
            merkle_root: valid_hex,
            summary: make_permanent_storage_summary(),
            committer_did: Some("did:key:committer".to_string()),
        })
        .await;
    let resp = result.expect("commit should succeed");
    assert!(resp.accepted);
    assert!(resp.commit_id.is_some());
    assert!(resp.spine_entry_hash.is_some());
    assert!(resp.entry_index.is_some());
    assert!(resp.spine_id.is_some());
    assert!(resp.error.is_none());
}

#[tokio::test]
async fn test_commit_session_success() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:session-owner");
    let create_resp = service
        .create_spine(CreateSpineRequest {
            name: "session-spine".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create should succeed");

    let session_id = uuid::Uuid::now_v7();
    let merkle_root = [1u8; 32];

    let result = service
        .commit_session(CommitSessionRequest {
            spine_id: create_resp.spine_id,
            session_id,
            session_hash: merkle_root,
            vertex_count: 100,
            committer: owner.clone(),
        })
        .await;
    let resp = result.expect("commit_session should succeed");

    // Ledger anchor
    assert_ne!(resp.commit_hash, [0u8; 32]);
    assert!(resp.index >= 1);
    assert_eq!(resp.spine_id, create_resp.spine_id);
    assert!(resp.committed_at.as_nanos() > 0);

    // Session binding (provenance receipt)
    assert_eq!(resp.session_id, session_id);
    assert_eq!(resp.merkle_root, merkle_root);
    assert_eq!(resp.vertex_count, 100);
    assert_eq!(resp.committer, owner);
    assert!(resp.tower_signature.is_none());
}

#[tokio::test]
async fn test_permanent_storage_verify_commit_invalid_spine_id() {
    let service = LoamSpineRpcService::default_service();
    let valid_hex = "a".repeat(64);
    let result = service
        .permanent_storage_verify_commit(PermanentStorageVerifyRequest {
            spine_id: "not-a-valid-uuid".to_string(),
            entry_hash: valid_hex.clone(),
            index: 0,
        })
        .await;
    assert!(result.is_err());
    let err = result.expect_err("expected error");
    assert!(err.to_string().contains("invalid spine_id UUID"));
}

#[tokio::test]
async fn test_permanent_storage_verify_commit_invalid_entry_hash() {
    let service = LoamSpineRpcService::default_service();
    let result = service
        .permanent_storage_verify_commit(PermanentStorageVerifyRequest {
            spine_id: uuid::Uuid::now_v7().to_string(),
            entry_hash: "bad-hex".to_string(),
            index: 0,
        })
        .await;
    assert!(result.is_err());
    let err = result.expect_err("expected error");
    assert!(err.to_string().contains("invalid entry_hash hex"));
}

#[tokio::test]
async fn test_permanent_storage_verify_commit_valid_found() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:verify-owner");
    let commit_resp = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: uuid::Uuid::now_v7().to_string(),
            merkle_root: "b".repeat(64),
            summary: make_permanent_storage_summary(),
            committer_did: Some(owner.to_string()),
        })
        .await
        .expect("commit should succeed");
    let spine_id = commit_resp.spine_id.expect("spine_id");
    let entry_hash = commit_resp.commit_id.expect("commit_id");

    let found = service
        .permanent_storage_verify_commit(PermanentStorageVerifyRequest {
            spine_id,
            entry_hash,
            index: 0,
        })
        .await
        .expect("verify should succeed");
    assert!(found);
}

#[tokio::test]
async fn test_permanent_storage_verify_commit_valid_not_found() {
    let service = LoamSpineRpcService::default_service();
    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "verify-spine".to_string(),
            owner: Did::new("did:key:verify-spine-owner"),
            config: None,
        })
        .await
        .expect("create should succeed");

    let nonexistent_hex = "c".repeat(64);
    let found = service
        .permanent_storage_verify_commit(PermanentStorageVerifyRequest {
            spine_id: spine_resp.spine_id.to_string(),
            entry_hash: nonexistent_hex,
            index: 0,
        })
        .await
        .expect("verify should succeed");
    assert!(!found);
}

#[tokio::test]
async fn test_permanent_storage_get_commit_invalid_spine_id() {
    let service = LoamSpineRpcService::default_service();
    let valid_hex = "a".repeat(64);
    let result = service
        .permanent_storage_get_commit(PermanentStorageGetCommitRequest {
            spine_id: "not-a-uuid".to_string(),
            entry_hash: valid_hex,
            index: 0,
        })
        .await;
    assert!(result.is_err());
    let err = result.expect_err("expected error");
    assert!(err.to_string().contains("invalid spine_id UUID"));
}

#[tokio::test]
async fn test_permanent_storage_get_commit_invalid_entry_hash() {
    let service = LoamSpineRpcService::default_service();
    let result = service
        .permanent_storage_get_commit(PermanentStorageGetCommitRequest {
            spine_id: uuid::Uuid::now_v7().to_string(),
            entry_hash: "invalid-hex-zz".to_string(),
            index: 0,
        })
        .await;
    assert!(result.is_err());
    let err = result.expect_err("expected error");
    assert!(err.to_string().contains("invalid entry_hash hex"));
}

#[tokio::test]
async fn test_permanent_storage_get_commit_found() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:get-commit-owner");
    let commit_resp = service
        .permanent_storage_commit_session(PermanentStorageCommitRequest {
            session_id: uuid::Uuid::now_v7().to_string(),
            merkle_root: "d".repeat(64),
            summary: make_permanent_storage_summary(),
            committer_did: Some(owner.to_string()),
        })
        .await
        .expect("commit should succeed");
    let spine_id = commit_resp.spine_id.expect("spine_id");
    let entry_hash = commit_resp.commit_id.expect("commit_id");

    let value = service
        .permanent_storage_get_commit(PermanentStorageGetCommitRequest {
            spine_id,
            entry_hash,
            index: 0,
        })
        .await
        .expect("get_commit should succeed");
    assert!(!value.is_null());
}

#[tokio::test]
async fn test_permanent_storage_get_commit_not_found() {
    let service = LoamSpineRpcService::default_service();
    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "get-commit-spine".to_string(),
            owner: Did::new("did:key:get-spine-owner"),
            config: None,
        })
        .await
        .expect("create should succeed");

    let value = service
        .permanent_storage_get_commit(PermanentStorageGetCommitRequest {
            spine_id: spine_resp.spine_id.to_string(),
            entry_hash: "e".repeat(64),
            index: 0,
        })
        .await
        .expect("get_commit should succeed");
    assert!(value.is_null());
}
