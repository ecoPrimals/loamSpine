// SPDX-License-Identifier: AGPL-3.0-or-later

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "test assertions use expect/panic for failure clarity"
)]

use super::*;

#[test]
fn test_server_creation() {
    let _server = LoamSpineTarpcServer::default_server();
}

#[test]
fn test_server_with_service() {
    let service = LoamSpineRpcService::default_service();
    let server = LoamSpineTarpcServer::new(service);
    assert!(Arc::strong_count(&server.service) >= 1);
}

#[test]
fn test_server_clone() {
    let server = LoamSpineTarpcServer::default_server();
    let cloned = server.clone();

    assert!(Arc::ptr_eq(&server.service, &cloned.service));
}

#[tokio::test]
async fn test_tarpc_health_check() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let request = HealthCheckRequest {
        include_details: false,
    };

    let result = LoamSpineRpc::health_check(server, ctx, request).await;
    assert!(result.is_ok());

    let response = result.unwrap_or_else(|_| panic!("unexpected error"));
    assert!(response.status.is_healthy());
}

#[tokio::test]
async fn test_tarpc_create_spine() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let request = CreateSpineRequest {
        owner: Did::new("did:key:z6MkTest"),
        name: "Test Spine".to_string(),
        config: None,
    };

    let result = LoamSpineRpc::create_spine(server, ctx, request).await;
    assert!(result.is_ok());

    let response = result.unwrap_or_else(|_| panic!("unexpected error"));
    assert!(!response.spine_id.is_nil());
}

#[tokio::test]
async fn test_tarpc_get_nonexistent_spine() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let request = GetSpineRequest {
        spine_id: uuid::Uuid::nil(),
    };

    let result = LoamSpineRpc::get_spine(server, ctx, request).await;
    assert!(result.is_ok());

    let response = result.unwrap_or_else(|_| panic!("unexpected error"));
    assert!(response.spine.is_none());
}

#[tokio::test]
async fn test_tarpc_seal_spine() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let seal_request = SealSpineRequest {
        spine_id: create_response.spine_id,
        sealer: owner,
    };

    let result = LoamSpineRpc::seal_spine(server, ctx, seal_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tarpc_append_entry() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Entry Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let append_request = AppendEntryRequest {
        spine_id: create_response.spine_id,
        entry_type: EntryType::DataAnchor {
            data_hash: [0u8; 32],
            mime_type: Some("text/plain".to_string()),
            size: 100,
        },
        committer: owner,
        payload: None,
    };

    let result = LoamSpineRpc::append_entry(server, ctx, append_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tarpc_mint_certificate() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Cert Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let mint_request = MintCertificateRequest {
        spine_id: create_response.spine_id,
        cert_type: CertificateType::DigitalGame {
            platform: "steam".to_string(),
            game_id: "hl3".to_string(),
            edition: None,
        },
        owner,
        metadata: None,
    };

    let result = LoamSpineRpc::mint_certificate(server, ctx, mint_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tarpc_commit_session() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Session Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let commit_request = CommitSessionRequest {
        spine_id: create_response.spine_id,
        committer: owner,
        session_id: uuid::Uuid::now_v7(),
        session_hash: [0u8; 32],
        vertex_count: 42,
    };

    let result = LoamSpineRpc::commit_session(server, ctx, commit_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tarpc_commit_braid() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Braid Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let braid_request = CommitBraidRequest {
        spine_id: create_response.spine_id,
        committer: owner,
        braid_id: uuid::Uuid::now_v7(),
        braid_hash: [2u8; 32],
        subjects: vec![],
    };

    let result = LoamSpineRpc::commit_braid(server, ctx, braid_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tarpc_get_entry() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Entry Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let append_request = AppendEntryRequest {
        spine_id: create_response.spine_id,
        entry_type: EntryType::DataAnchor {
            data_hash: [1u8; 32],
            mime_type: Some("text/plain".to_string()),
            size: 50,
        },
        committer: owner.clone(),
        payload: None,
    };
    let append_response = LoamSpineRpc::append_entry(server.clone(), ctx, append_request)
        .await
        .unwrap_or_else(|_| panic!("append failed"));

    let get_request = GetEntryRequest {
        spine_id: create_response.spine_id,
        entry_hash: append_response.entry_hash,
    };
    let result = LoamSpineRpc::get_entry(server.clone(), ctx, get_request).await;
    assert!(result.is_ok());
    let response = result.unwrap_or_else(|_| panic!("get_entry failed"));
    assert!(response.found);

    let get_nonexistent = GetEntryRequest {
        spine_id: create_response.spine_id,
        entry_hash: [99u8; 32],
    };
    let result = LoamSpineRpc::get_entry(server, ctx, get_nonexistent).await;
    assert!(result.is_ok());
    let response = result.unwrap_or_else(|_| panic!("get_entry failed"));
    assert!(!response.found);
}

#[tokio::test]
async fn test_tarpc_get_tip() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Tip Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let get_tip_request = GetTipRequest {
        spine_id: create_response.spine_id,
    };
    let result = LoamSpineRpc::get_tip(server, ctx, get_tip_request).await;
    assert!(result.is_ok());
    let response = result.unwrap_or_else(|_| panic!("get_tip failed"));
    assert!(!response.tip_hash.iter().all(|&b| b == 0));
}

#[tokio::test]
async fn test_tarpc_transfer_certificate() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");
    let new_owner = Did::new("did:key:z6MkNewOwner");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Transfer Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let mint_request = MintCertificateRequest {
        spine_id: create_response.spine_id,
        cert_type: CertificateType::DigitalGame {
            platform: "steam".to_string(),
            game_id: "hl3".to_string(),
            edition: None,
        },
        owner: owner.clone(),
        metadata: None,
    };
    let mint_response = LoamSpineRpc::mint_certificate(server.clone(), ctx, mint_request)
        .await
        .unwrap_or_else(|_| panic!("mint failed"));

    let transfer_request = TransferCertificateRequest {
        certificate_id: mint_response.certificate_id,
        from: owner,
        to: new_owner,
    };
    let result = LoamSpineRpc::transfer_certificate(server, ctx, transfer_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tarpc_loan_and_return_certificate() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");
    let borrower = Did::new("did:key:z6MkBorrower");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Loan Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let mint_request = MintCertificateRequest {
        spine_id: create_response.spine_id,
        cert_type: CertificateType::DigitalGame {
            platform: "steam".to_string(),
            game_id: "hl3".to_string(),
            edition: None,
        },
        owner: owner.clone(),
        metadata: None,
    };
    let mint_response = LoamSpineRpc::mint_certificate(server.clone(), ctx, mint_request)
        .await
        .unwrap_or_else(|_| panic!("mint failed"));

    let loan_request = LoanCertificateRequest {
        certificate_id: mint_response.certificate_id,
        lender: owner.clone(),
        borrower: borrower.clone(),
        terms: loam_spine_core::LoanTerms::new().with_duration(3600),
    };
    let result = LoamSpineRpc::loan_certificate(server.clone(), ctx, loan_request).await;
    assert!(result.is_ok());

    let return_request = ReturnCertificateRequest {
        certificate_id: mint_response.certificate_id,
        returner: borrower,
    };
    let result = LoamSpineRpc::return_certificate(server, ctx, return_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tarpc_anchor_slice() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let waypoint_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Waypoint".to_string(),
        config: None,
    };
    let waypoint_response = LoamSpineRpc::create_spine(server.clone(), ctx, waypoint_request)
        .await
        .unwrap_or_else(|_| panic!("create waypoint failed"));

    let origin_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Origin".to_string(),
        config: None,
    };
    let origin_response = LoamSpineRpc::create_spine(server.clone(), ctx, origin_request)
        .await
        .unwrap_or_else(|_| panic!("create origin failed"));

    let anchor_request = AnchorSliceRequest {
        waypoint_spine_id: waypoint_response.spine_id,
        slice_id: uuid::Uuid::now_v7(),
        origin_spine_id: origin_response.spine_id,
        committer: owner,
    };
    let result = LoamSpineRpc::anchor_slice(server, ctx, anchor_request).await;
    assert!(result.is_ok());
    let response = result.unwrap_or_else(|_| panic!("anchor failed"));
    assert!(!response.anchor_hash.iter().all(|&b| b == 0));
}

#[tokio::test]
async fn test_tarpc_checkout_slice_nonexistent() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let waypoint_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Waypoint Checkout".to_string(),
        config: None,
    };
    let waypoint_response = LoamSpineRpc::create_spine(server.clone(), ctx, waypoint_request)
        .await
        .unwrap_or_else(|_| panic!("create waypoint failed"));

    let checkout_request = CheckoutSliceRequest {
        waypoint_spine_id: waypoint_response.spine_id,
        slice_id: uuid::Uuid::now_v7(),
        requester: owner,
    };
    let result = LoamSpineRpc::checkout_slice(server, ctx, checkout_request).await;
    assert!(result.is_err(), "checkout of nonexistent slice should fail");
}

#[tokio::test]
async fn test_tarpc_generate_and_verify_inclusion_proof() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        owner: owner.clone(),
        name: "Proof Test".to_string(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .unwrap_or_else(|_| panic!("create failed"));

    let append_request = AppendEntryRequest {
        spine_id: create_response.spine_id,
        entry_type: EntryType::DataAnchor {
            data_hash: [5u8; 32],
            mime_type: Some("text/plain".to_string()),
            size: 10,
        },
        committer: owner.clone(),
        payload: None,
    };
    let append_response = LoamSpineRpc::append_entry(server.clone(), ctx, append_request)
        .await
        .unwrap_or_else(|_| panic!("append failed"));

    let gen_request = GenerateInclusionProofRequest {
        spine_id: create_response.spine_id,
        entry_hash: append_response.entry_hash,
    };
    let gen_result = LoamSpineRpc::generate_inclusion_proof(server.clone(), ctx, gen_request)
        .await
        .unwrap_or_else(|_| panic!("generate proof failed"));

    let verify_request = VerifyInclusionProofRequest {
        proof: gen_result.proof,
    };
    let result = LoamSpineRpc::verify_inclusion_proof(server, ctx, verify_request).await;
    assert!(result.is_ok());
    let response = result.unwrap_or_else(|_| panic!("verify failed"));
    assert!(response.valid);
}

#[tokio::test]
async fn test_tarpc_get_spine_existing() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        name: "Get Spine Test".to_string(),
        owner: owner.clone(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .expect("create failed");

    let get_request = GetSpineRequest {
        spine_id: create_response.spine_id,
    };
    let result = LoamSpineRpc::get_spine(server, ctx, get_request).await;
    assert!(result.is_ok());
    let response = result.expect("get_spine failed");
    assert!(response.found);
    assert!(response.spine.is_some());
}

#[tokio::test]
async fn test_tarpc_health_check_with_details() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let request = HealthCheckRequest {
        include_details: true,
    };

    let result = LoamSpineRpc::health_check(server, ctx, request).await;
    assert!(result.is_ok());
    let response = result.expect("health_check failed");
    assert!(response.status.is_healthy());
    assert!(response.report.is_some());
}

#[tokio::test]
async fn test_tarpc_get_certificate() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        name: "Get Cert Test".to_string(),
        owner: owner.clone(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .expect("create failed");

    let mint_request = MintCertificateRequest {
        spine_id: create_response.spine_id,
        cert_type: CertificateType::DigitalGame {
            platform: "steam".to_string(),
            game_id: "hl3".to_string(),
            edition: None,
        },
        owner: owner.clone(),
        metadata: None,
    };
    let mint_response = LoamSpineRpc::mint_certificate(server.clone(), ctx, mint_request)
        .await
        .expect("mint failed");

    let get_request = GetCertificateRequest {
        certificate_id: mint_response.certificate_id,
    };
    let result = LoamSpineRpc::get_certificate(server, ctx, get_request).await;
    assert!(result.is_ok());
    let response = result.expect("get_certificate failed");
    assert!(response.found);
    assert!(response.certificate.is_some());
}

#[tokio::test]
async fn test_tarpc_get_certificate_nonexistent() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();

    let get_request = GetCertificateRequest {
        certificate_id: uuid::Uuid::nil(),
    };
    let result = LoamSpineRpc::get_certificate(server, ctx, get_request).await;
    assert!(result.is_ok());
    let response = result.expect("get_certificate failed");
    assert!(!response.found);
    assert!(response.certificate.is_none());
}

#[tokio::test]
async fn test_tarpc_seal_nonexistent_spine() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let request = SealSpineRequest {
        spine_id: uuid::Uuid::nil(),
        sealer: Did::new("did:key:z6MkTest"),
    };

    let result = LoamSpineRpc::seal_spine(server, ctx, request).await;
    assert!(result.is_err(), "seal of nonexistent spine should fail");
}

#[tokio::test]
async fn test_tarpc_append_to_nonexistent_spine() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let request = AppendEntryRequest {
        spine_id: uuid::Uuid::nil(),
        entry_type: EntryType::DataAnchor {
            data_hash: [0u8; 32],
            mime_type: Some("text/plain".to_string()),
            size: 10,
        },
        committer: Did::new("did:key:z6MkTest"),
        payload: None,
    };

    let result = LoamSpineRpc::append_entry(server, ctx, request).await;
    assert!(result.is_err(), "append to nonexistent spine should fail");
}

#[tokio::test]
async fn test_tarpc_mint_to_nonexistent_spine() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let request = MintCertificateRequest {
        spine_id: uuid::Uuid::nil(),
        cert_type: CertificateType::DigitalGame {
            platform: "steam".to_string(),
            game_id: "hl3".to_string(),
            edition: None,
        },
        owner: Did::new("did:key:z6MkTest"),
        metadata: None,
    };

    let result = LoamSpineRpc::mint_certificate(server, ctx, request).await;
    assert!(result.is_err(), "mint to nonexistent spine should fail");
}

#[tokio::test]
async fn test_tarpc_get_tip_nonexistent_spine() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let request = GetTipRequest {
        spine_id: uuid::Uuid::nil(),
    };

    let result = LoamSpineRpc::get_tip(server, ctx, request).await;
    assert!(result.is_err(), "get_tip of nonexistent spine should fail");
}

#[tokio::test]
async fn test_tarpc_generate_inclusion_proof_nonexistent_entry() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        name: "Proof Error Test".to_string(),
        owner: owner.clone(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .expect("create failed");

    let gen_request = GenerateInclusionProofRequest {
        spine_id: create_response.spine_id,
        entry_hash: [99u8; 32],
    };
    let result = LoamSpineRpc::generate_inclusion_proof(server, ctx, gen_request).await;
    assert!(
        result.is_err(),
        "generate proof for nonexistent entry should fail"
    );
}

#[tokio::test]
async fn test_tarpc_checkout_slice_success() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");
    let slice_id = uuid::Uuid::now_v7();

    let waypoint_request = CreateSpineRequest {
        name: "Waypoint Checkout Success".to_string(),
        owner: owner.clone(),
        config: None,
    };
    let waypoint_response = LoamSpineRpc::create_spine(server.clone(), ctx, waypoint_request)
        .await
        .expect("create waypoint failed");

    let origin_request = CreateSpineRequest {
        name: "Origin Checkout Success".to_string(),
        owner: owner.clone(),
        config: None,
    };
    let origin_response = LoamSpineRpc::create_spine(server.clone(), ctx, origin_request)
        .await
        .expect("create origin failed");

    let anchor_request = AnchorSliceRequest {
        waypoint_spine_id: waypoint_response.spine_id,
        slice_id,
        origin_spine_id: origin_response.spine_id,
        committer: owner.clone(),
    };
    let _anchor_response = LoamSpineRpc::anchor_slice(server.clone(), ctx, anchor_request)
        .await
        .expect("anchor failed");

    let checkout_request = CheckoutSliceRequest {
        waypoint_spine_id: waypoint_response.spine_id,
        slice_id,
        requester: owner,
    };
    let result = LoamSpineRpc::checkout_slice(server, ctx, checkout_request).await;
    assert!(result.is_ok(), "checkout of anchored slice should succeed");
    let response = result.expect("checkout failed");
    assert!(response.success);
    assert!(response.checkout_hash.is_some());
}

#[tokio::test]
async fn test_tarpc_verify_invalid_proof() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkTest");

    let create_request = CreateSpineRequest {
        name: "Invalid Proof Test".to_string(),
        owner: owner.clone(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .expect("create failed");

    let append_request = AppendEntryRequest {
        spine_id: create_response.spine_id,
        entry_type: EntryType::DataAnchor {
            data_hash: [7u8; 32],
            mime_type: Some("text/plain".to_string()),
            size: 10,
        },
        committer: owner.clone(),
        payload: None,
    };
    let append_response = LoamSpineRpc::append_entry(server.clone(), ctx, append_request)
        .await
        .expect("append failed");

    let gen_request = GenerateInclusionProofRequest {
        spine_id: create_response.spine_id,
        entry_hash: append_response.entry_hash,
    };
    let gen_result = LoamSpineRpc::generate_inclusion_proof(server.clone(), ctx, gen_request)
        .await
        .expect("generate proof failed");

    let mut tampered_proof = gen_result.proof;
    tampered_proof.tip = [0xFF; 32];
    let verify_request = VerifyInclusionProofRequest {
        proof: tampered_proof,
    };
    let result = LoamSpineRpc::verify_inclusion_proof(server, ctx, verify_request).await;
    assert!(result.is_ok());
    let response = result.expect("verify failed");
    assert!(!response.valid);
    assert_eq!(response.message, "Proof verification failed");
}

#[tokio::test]
async fn test_tarpc_transfer_wrong_owner() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();
    let owner = Did::new("did:key:z6MkOwner");
    let wrong_owner = Did::new("did:key:z6MkWrong");
    let new_owner = Did::new("did:key:z6MkNew");

    let create_request = CreateSpineRequest {
        name: "Transfer Wrong Owner Test".to_string(),
        owner: owner.clone(),
        config: None,
    };
    let create_response = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .expect("create failed");

    let mint_request = MintCertificateRequest {
        spine_id: create_response.spine_id,
        cert_type: CertificateType::DigitalGame {
            platform: "steam".to_string(),
            game_id: "hl3".to_string(),
            edition: None,
        },
        owner: owner.clone(),
        metadata: None,
    };
    let mint_response = LoamSpineRpc::mint_certificate(server.clone(), ctx, mint_request)
        .await
        .expect("mint failed");

    let transfer_request = TransferCertificateRequest {
        certificate_id: mint_response.certificate_id,
        from: wrong_owner,
        to: new_owner,
    };
    let result = LoamSpineRpc::transfer_certificate(server, ctx, transfer_request).await;
    assert!(result.is_err(), "transfer with wrong owner should fail");
}

// Server lifecycle, config, and TCP integration tests extracted to
// tarpc_server_tests_lifecycle.rs for 1000-line compliance.
#[expect(
    clippy::expect_used,
    reason = "test assertions use expect for failure clarity"
)]
#[path = "tarpc_server_tests_lifecycle.rs"]
mod lifecycle;
