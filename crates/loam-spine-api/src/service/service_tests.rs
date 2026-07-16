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
