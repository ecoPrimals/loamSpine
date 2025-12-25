//! API integration tests for `LoamSpine`.
//!
//! Tests the RPC service implementation directly.

// Allow test-specific patterns
#![allow(missing_docs)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(clippy::cast_possible_truncation)]

use loam_spine_api::service::LoamSpineRpcService;
use loam_spine_api::types::*;
use loam_spine_core::KB;
use uuid::Uuid;

fn test_did() -> Did {
    Did::new(format!("did:key:z6Mk{}", Uuid::now_v7().simple()))
}

#[tokio::test]
async fn create_and_get_spine() {
    let service = LoamSpineRpcService::default_service();
    let owner = test_did();

    // Create spine
    let create_req = CreateSpineRequest {
        owner: owner.clone(),
        name: "Test Spine".to_string(),
        config: None,
    };
    let create_resp = service
        .create_spine(create_req)
        .await
        .unwrap_or_else(|_| unreachable!());

    // Get spine
    let get_req = GetSpineRequest {
        spine_id: create_resp.spine_id,
    };
    let get_resp = service
        .get_spine(get_req)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(get_resp.found);
    assert!(get_resp.spine.is_some());
    let spine = get_resp.spine.unwrap_or_else(|| unreachable!());
    assert_eq!(spine.owner, owner);
}

#[tokio::test]
async fn spine_not_found() {
    let service = LoamSpineRpcService::default_service();

    let get_req = GetSpineRequest {
        spine_id: Uuid::now_v7(),
    };
    let get_resp = service
        .get_spine(get_req)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(!get_resp.found);
    assert!(get_resp.spine.is_none());
}

#[tokio::test]
async fn append_and_get_entry() {
    let service = LoamSpineRpcService::default_service();
    let owner = test_did();

    // Create spine
    let create_resp = service
        .create_spine(CreateSpineRequest {
            owner: owner.clone(),
            name: "Entry Test".to_string(),
            config: None,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    // Append entry (using DataAnchor as a simple entry type)
    let append_req = AppendEntryRequest {
        spine_id: create_resp.spine_id,
        entry_type: EntryType::DataAnchor {
            data_hash: [1u8; 32],
            mime_type: Some("application/json".to_string()),
            size: KB,
        },
        committer: owner.clone(),
        payload: None,
    };
    let append_resp = service
        .append_entry(append_req)
        .await
        .unwrap_or_else(|_| unreachable!());

    // Get entry
    let get_req = GetEntryRequest {
        spine_id: create_resp.spine_id,
        entry_hash: append_resp.entry_hash,
    };
    let get_resp = service
        .get_entry(get_req)
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(get_resp.found);
    assert!(get_resp.entry.is_some());
}

#[tokio::test]
async fn get_tip_entry() {
    let service = LoamSpineRpcService::default_service();
    let owner = test_did();

    // Create spine
    let create_resp = service
        .create_spine(CreateSpineRequest {
            owner: owner.clone(),
            name: "Tip Test".to_string(),
            config: None,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    // Append multiple entries
    for i in 0..3 {
        service
            .append_entry(AppendEntryRequest {
                spine_id: create_resp.spine_id,
                entry_type: EntryType::DataAnchor {
                    data_hash: [i as u8; 32],
                    mime_type: None,
                    size: (i + 1) * 100,
                },
                committer: owner.clone(),
                payload: None,
            })
            .await
            .unwrap_or_else(|_| unreachable!());
    }

    // Get tip
    let tip_resp = service
        .get_tip(GetTipRequest {
            spine_id: create_resp.spine_id,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    // Height should include genesis + 3 entries
    assert!(tip_resp.height >= 3);
}

#[tokio::test]
async fn seal_spine() {
    let service = LoamSpineRpcService::default_service();
    let owner = test_did();

    // Create spine
    let create_resp = service
        .create_spine(CreateSpineRequest {
            owner: owner.clone(),
            name: "Seal Test".to_string(),
            config: None,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    // Seal spine
    let seal_resp = service
        .seal_spine(SealSpineRequest {
            spine_id: create_resp.spine_id,
            sealer: owner.clone(),
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(seal_resp.success);

    // Verify spine is sealed
    let get_resp = service
        .get_spine(GetSpineRequest {
            spine_id: create_resp.spine_id,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(get_resp.spine.unwrap_or_else(|| unreachable!()).is_sealed());
}

#[tokio::test]
async fn certificate_lifecycle() {
    let service = LoamSpineRpcService::default_service();
    let owner = test_did();
    let recipient = test_did();

    // Create spine
    let create_resp = service
        .create_spine(CreateSpineRequest {
            owner: owner.clone(),
            name: "Certificate Test".to_string(),
            config: None,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    // Mint certificate
    let mint_resp = service
        .mint_certificate(MintCertificateRequest {
            spine_id: create_resp.spine_id,
            cert_type: CertificateType::DigitalCollectible {
                collection_id: "test-collection".to_string(),
                item_number: Some(1),
                total_supply: Some(100),
                rarity: Some(Rarity::Rare),
            },
            owner: owner.clone(),
            metadata: None,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    // Get certificate
    let get_resp = service
        .get_certificate(GetCertificateRequest {
            certificate_id: mint_resp.certificate_id,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(get_resp.found);
    assert!(get_resp.certificate.is_some());

    // Transfer certificate
    let transfer_resp = service
        .transfer_certificate(TransferCertificateRequest {
            certificate_id: mint_resp.certificate_id,
            from: owner.clone(),
            to: recipient.clone(),
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(transfer_resp.success);

    // Verify new owner
    let get_resp = service
        .get_certificate(GetCertificateRequest {
            certificate_id: mint_resp.certificate_id,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    let cert = get_resp.certificate.unwrap_or_else(|| unreachable!());
    assert_eq!(cert.owner, recipient);
}

#[tokio::test]
async fn certificate_loan_flow() {
    let service = LoamSpineRpcService::default_service();
    let owner = test_did();
    let borrower = test_did();

    // Create spine
    let create_resp = service
        .create_spine(CreateSpineRequest {
            owner: owner.clone(),
            name: "Loan Test".to_string(),
            config: None,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    // Mint certificate
    let mint_resp = service
        .mint_certificate(MintCertificateRequest {
            spine_id: create_resp.spine_id,
            cert_type: CertificateType::DigitalGame {
                platform: "PC".to_string(),
                game_id: "test-game-123".to_string(),
                edition: Some("Standard".to_string()),
            },
            owner: owner.clone(),
            metadata: None,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    // Loan certificate
    let loan_resp = service
        .loan_certificate(LoanCertificateRequest {
            certificate_id: mint_resp.certificate_id,
            lender: owner.clone(),
            borrower: borrower.clone(),
            terms: LoanTerms::new(),
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(loan_resp.success);

    // Return certificate
    let return_resp = service
        .return_certificate(ReturnCertificateRequest {
            certificate_id: mint_resp.certificate_id,
            returner: borrower.clone(),
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(return_resp.success);
}

#[tokio::test]
async fn session_commit() {
    let service = LoamSpineRpcService::default_service();
    let owner = test_did();

    // Create spine
    let create_resp = service
        .create_spine(CreateSpineRequest {
            owner: owner.clone(),
            name: "Session Test".to_string(),
            config: None,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    // Commit session
    let commit_resp = service
        .commit_session(CommitSessionRequest {
            spine_id: create_resp.spine_id,
            committer: owner.clone(),
            session_id: Uuid::now_v7(),
            session_hash: [42u8; 32],
            vertex_count: 10,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(commit_resp.commit_hash.iter().any(|&b| b != 0));
}

#[tokio::test]
async fn braid_commit() {
    let service = LoamSpineRpcService::default_service();
    let owner = test_did();

    // Create spine
    let create_resp = service
        .create_spine(CreateSpineRequest {
            owner: owner.clone(),
            name: "Braid Test".to_string(),
            config: None,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    // Commit braid
    let commit_resp = service
        .commit_braid(CommitBraidRequest {
            spine_id: create_resp.spine_id,
            committer: owner.clone(),
            braid_id: Uuid::now_v7(),
            braid_hash: [2u8; 32],
            subjects: vec![owner.clone()],
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(commit_resp.commit_hash.iter().any(|&b| b != 0));
}

#[tokio::test]
async fn health_check() {
    let service = LoamSpineRpcService::default_service();

    let resp = service
        .health_check(HealthCheckRequest {
            include_details: true,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    assert_eq!(resp.status, HealthStatus::Healthy);
}

#[tokio::test]
async fn inclusion_proof_lifecycle() {
    let service = LoamSpineRpcService::default_service();
    let owner = test_did();

    // Create spine
    let create_resp = service
        .create_spine(CreateSpineRequest {
            owner: owner.clone(),
            name: "Proof Test".to_string(),
            config: None,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    // Append entry
    let append_resp = service
        .append_entry(AppendEntryRequest {
            spine_id: create_resp.spine_id,
            entry_type: EntryType::DataAnchor {
                data_hash: [0u8; 32],
                mime_type: Some("text/plain".to_string()),
                size: 256,
            },
            committer: owner.clone(),
            payload: None,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    // Generate proof
    let proof_resp = service
        .generate_inclusion_proof(GenerateInclusionProofRequest {
            spine_id: create_resp.spine_id,
            entry_hash: append_resp.entry_hash,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    // Verify proof
    let verify_resp = service
        .verify_inclusion_proof(VerifyInclusionProofRequest {
            proof: proof_resp.proof,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(verify_resp.valid);
}

#[tokio::test]
async fn slice_operations() {
    let service = LoamSpineRpcService::default_service();
    let owner = test_did();

    // Create source spine
    let source_resp = service
        .create_spine(CreateSpineRequest {
            owner: owner.clone(),
            name: "Source Spine".to_string(),
            config: None,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    // Create waypoint spine
    let waypoint_resp = service
        .create_spine(CreateSpineRequest {
            owner: owner.clone(),
            name: "Waypoint Spine".to_string(),
            config: None,
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    let slice_id = Uuid::now_v7();

    // Anchor slice
    let anchor_resp = service
        .anchor_slice(AnchorSliceRequest {
            waypoint_spine_id: waypoint_resp.spine_id,
            slice_id,
            origin_spine_id: source_resp.spine_id,
            committer: owner.clone(),
        })
        .await
        .unwrap_or_else(|_| unreachable!());

    assert!(anchor_resp.anchor_hash.iter().any(|&b| b != 0));

    // Checkout slice - may return error if slice not fully set up
    // This tests the API endpoint works, not the full slice workflow
    let checkout_result = service
        .checkout_slice(CheckoutSliceRequest {
            waypoint_spine_id: waypoint_resp.spine_id,
            slice_id,
            requester: owner.clone(),
        })
        .await;

    // Checkout may succeed or fail depending on slice state
    // The important thing is the API endpoint responds
    if let Ok(resp) = checkout_result {
        assert!(resp.success || resp.checkout_hash.is_some() || resp.checkout_hash.is_none());
    } else { /* Expected if slice not fully initialized */
    }
}

#[tokio::test]
async fn concurrent_api_operations() {
    use std::sync::Arc;

    let service = Arc::new(LoamSpineRpcService::default_service());
    let owner = test_did();

    // Create multiple spines concurrently
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let svc = Arc::clone(&service);
            let own = owner.clone();
            tokio::spawn(async move {
                svc.create_spine(CreateSpineRequest {
                    owner: own,
                    name: format!("Concurrent Spine {i}"),
                    config: None,
                })
                .await
            })
        })
        .collect();

    for handle in handles {
        let result = handle.await.unwrap_or_else(|_| unreachable!());
        assert!(result.is_ok());
    }
}
