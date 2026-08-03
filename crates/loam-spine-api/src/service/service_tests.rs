// SPDX-License-Identifier: AGPL-3.0-or-later

//! Core service tests: spine CRUD, certificate lifecycle, entry append, proof.

use super::*;
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
async fn health_check_reports_storage_details() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:z6MkHealth");
    service
        .create_spine(CreateSpineRequest {
            name: "health-test".to_string(),
            owner,
            config: None,
        })
        .await
        .expect("create should succeed");

    let resp = service
        .health_check(HealthCheckRequest {
            include_details: true,
        })
        .await
        .expect("health check should succeed");
    assert!(matches!(resp.status, HealthStatus::Healthy));
    let report = resp.report.expect("details should be present");
    assert!(!report.components.is_empty());
    let component_text = format!("{:?}", report.components[0]);
    assert!(component_text.contains("1 spines"));
}

#[tokio::test]
async fn readiness_probe_returns_storage_count() {
    let service = LoamSpineRpcService::default_service();
    let probe = service.readiness().await.expect("readiness should succeed");
    assert!(probe.ready);
    assert!(probe.reason.is_some());
    let reason = probe.reason.expect("reason should be set");
    assert!(reason.contains("storage accessible"));
}

#[tokio::test]
async fn liveness_probe_returns_alive() {
    let service = LoamSpineRpcService::default_service();
    let probe = service.liveness().await;
    assert_eq!(probe.status, "alive");
}

#[tokio::test]
async fn permanence_healthy_reports_counts() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:z6MkPerm");
    service
        .create_spine(CreateSpineRequest {
            name: "perm-test".to_string(),
            owner,
            config: None,
        })
        .await
        .expect("create should succeed");

    let val = service.permanence_healthy().await;
    assert_eq!(val["healthy"], true);
    assert_eq!(val["spine_count"], 1);
    assert!(val["entry_count"].as_u64().expect("entry count") >= 1);
    assert!(val["uptime_s"].as_u64().is_some());
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

    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "cert-test".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create spine should succeed");

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

    let return_resp = service
        .return_certificate(ReturnCertificateRequest {
            certificate_id: mint_resp.certificate_id,
            returner: borrower,
        })
        .await
        .expect("return should succeed");

    assert!(return_resp.success);
    assert!(return_resp.return_hash.is_some());

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

    let create_resp = service
        .create_spine(CreateSpineRequest {
            name: "seal-test".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create should succeed");

    let seal_resp = service
        .seal_spine(SealSpineRequest {
            spine_id: create_resp.spine_id,
            sealer: owner,
            reason: Some("test-seal".into()),
        })
        .await
        .expect("seal should succeed");

    assert!(seal_resp.success);
    assert!(seal_resp.seal_hash.is_some());

    let seal_again = service
        .seal_spine(SealSpineRequest {
            spine_id: create_resp.spine_id,
            sealer: Did::new("did:key:other"),
            reason: None,
        })
        .await;

    assert!(seal_again.is_err());
}

#[tokio::test]
async fn test_append_entry() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:appender");

    let create_resp = service
        .create_spine(CreateSpineRequest {
            name: "append-test".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create should succeed");

    let append_resp = service
        .append_entry(AppendEntryRequest {
            spine_id: create_resp.spine_id,
            entry_type: EntryType::DataAnchor {
                data_hash: [42u8; 32],
                mime_type: Some("application/json".into()),
                size: KB,
            },
            committer: Some(owner.clone()),
            payload: None,
        })
        .await
        .expect("append should succeed");

    assert_ne!(append_resp.entry_hash, [0u8; 32]);
    assert_eq!(append_resp.index, 1);

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

    let waypoint_resp = service
        .create_spine(CreateSpineRequest {
            name: "waypoint-spine".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create waypoint should succeed");

    let origin_resp = service
        .create_spine(CreateSpineRequest {
            name: "origin-spine".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create origin should succeed");

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

    let create_resp = service
        .create_spine(CreateSpineRequest {
            name: "proof-test".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create should succeed");

    let append_resp = service
        .append_entry(AppendEntryRequest {
            spine_id: create_resp.spine_id,
            entry_type: EntryType::DataAnchor {
                data_hash: [99u8; 32],
                mime_type: Some("text/plain".into()),
                size: 512,
            },
            committer: Some(owner),
            payload: None,
        })
        .await
        .expect("append should succeed");

    let proof_resp = service
        .generate_inclusion_proof(GenerateInclusionProofRequest {
            spine_id: create_resp.spine_id,
            entry_hash: append_resp.entry_hash,
        })
        .await
        .expect("proof generation should succeed");

    assert!(proof_resp.proof.verify().expect("verify"));
    assert_eq!(proof_resp.proof.spine_id, create_resp.spine_id);
}

#[tokio::test]
async fn verify_certificate_rpc_returns_semantic_checks() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:test-verify-rpc");

    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "verify-rpc".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create spine");

    let mint_resp = service
        .mint_certificate(MintCertificateRequest {
            spine_id: spine_resp.spine_id,
            cert_type: CertificateType::DigitalGame {
                platform: "steam".into(),
                game_id: "verify-test".into(),
                edition: None,
            },
            owner: owner.clone(),
            metadata: None,
        })
        .await
        .expect("mint");

    let verify_resp = service
        .verify_certificate(VerifyCertificateRequest {
            certificate_id: mint_resp.certificate_id,
        })
        .await
        .expect("verify");

    assert!(verify_resp.exists);
    assert!(verify_resp.valid);
    assert_eq!(verify_resp.checks_passed.len(), 6);
}

#[tokio::test]
async fn certificate_lifecycle_rpc_returns_events() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:test-lifecycle-rpc");
    let buyer = Did::new("did:key:test-lifecycle-buyer");

    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "lifecycle-rpc".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create spine");

    let mint_resp = service
        .mint_certificate(MintCertificateRequest {
            spine_id: spine_resp.spine_id,
            cert_type: CertificateType::ArtworkProvenance {
                artist: "test".into(),
                title: "lifecycle-art".into(),
                medium: "digital".into(),
                year_created: None,
            },
            owner: owner.clone(),
            metadata: None,
        })
        .await
        .expect("mint");

    service
        .transfer_certificate(TransferCertificateRequest {
            certificate_id: mint_resp.certificate_id,
            from: owner.clone(),
            to: buyer.clone(),
        })
        .await
        .expect("transfer");

    let lifecycle_resp = service
        .certificate_lifecycle(CertificateLifecycleRequest {
            certificate_id: mint_resp.certificate_id,
        })
        .await
        .expect("lifecycle");

    assert_eq!(lifecycle_resp.count, 2);
    assert_eq!(lifecycle_resp.entries.len(), 2);
}

#[tokio::test]
async fn certificate_history_rpc_returns_typed_records() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:test-history-rpc");
    let buyer = Did::new("did:key:test-history-buyer");

    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "history-rpc".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create spine");

    let mint_resp = service
        .mint_certificate(MintCertificateRequest {
            spine_id: spine_resp.spine_id,
            cert_type: CertificateType::SoftwareLicense {
                software_id: "hist-rpc".into(),
                license_type: "perpetual".into(),
                seats: Some(1),
                expires: None,
            },
            owner: owner.clone(),
            metadata: None,
        })
        .await
        .expect("mint");

    service
        .transfer_certificate(TransferCertificateRequest {
            certificate_id: mint_resp.certificate_id,
            from: owner.clone(),
            to: buyer.clone(),
        })
        .await
        .expect("transfer");

    let hist_resp = service
        .certificate_history(CertificateHistoryRequest {
            certificate_id: mint_resp.certificate_id,
        })
        .await
        .expect("history");

    assert_eq!(hist_resp.certificate.id, mint_resp.certificate_id);
    assert_eq!(hist_resp.ownership_records.len(), 2);
    assert!(hist_resp.loan_records.is_empty());
}

#[tokio::test]
async fn append_entry_batch_rpc_returns_ordered_results() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:test-batch-entry-rpc");

    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "batch-entry-rpc".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create spine");

    let entries: Vec<_> = (0..5)
        .map(|i| loam_spine_core::entry::EntryType::MetadataUpdate {
            field: format!("field_{i}"),
            value: format!("value_{i}"),
        })
        .collect();

    let batch_resp = service
        .append_entry_batch(AppendEntryBatchRequest {
            spine_id: spine_resp.spine_id,
            entries,
        })
        .await
        .expect("batch append");

    assert_eq!(batch_resp.count, 5);
    assert_eq!(batch_resp.results.len(), 5);

    for i in 0..4 {
        assert!(
            batch_resp.results[i].index < batch_resp.results[i + 1].index,
            "indices must be monotonically increasing"
        );
    }
}

#[tokio::test]
async fn mint_certificate_batch_rpc_creates_all() {
    let service = LoamSpineRpcService::default_service();
    let owner = Did::new("did:key:test-batch-mint-rpc");

    let spine_resp = service
        .create_spine(CreateSpineRequest {
            name: "batch-mint-rpc".to_string(),
            owner: owner.clone(),
            config: None,
        })
        .await
        .expect("create spine");

    let items: Vec<_> = (0..5)
        .map(|i| BatchMintItem {
            cert_type: CertificateType::SoftwareLicense {
                software_id: format!("batch-rpc-sw-{i}"),
                license_type: "perpetual".into(),
                seats: Some(1),
                expires: None,
            },
            owner: owner.clone(),
            metadata: None,
        })
        .collect();

    let batch_resp = service
        .mint_certificate_batch(MintCertificateBatchRequest {
            spine_id: spine_resp.spine_id,
            items,
        })
        .await
        .expect("batch mint");

    assert_eq!(batch_resp.count, 5);
    assert_eq!(batch_resp.results.len(), 5);

    let mut ids: Vec<_> = batch_resp
        .results
        .iter()
        .map(|r| r.certificate_id)
        .collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 5, "all certificate IDs should be unique");
}
