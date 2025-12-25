//! End-to-end integration tests for `LoamSpine`.
//!
//! These tests exercise the full flow from spine creation through
//! certificate lifecycle, slice operations, and braid commits.

// Allow various patterns in tests that would be problematic in production
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use loam_spine_core::{
    service::LoamSpineService,
    traits::{
        BraidAcceptor, BraidSummary, CommitAcceptor, DehydrationSummary, SliceManager, SpineQuery,
    },
    types::{BraidId, Did, SessionId},
    CertificateType, LoanTerms, SECONDS_PER_HOUR,
};

/// Helper to create a test certificate type.
fn test_cert_type() -> CertificateType {
    CertificateType::DigitalCollectible {
        collection_id: "test-collection".to_string(),
        item_number: Some(1),
        total_supply: Some(100),
        rarity: None,
    }
}

/// Test a full spine lifecycle from creation to seal.
#[tokio::test]
async fn full_spine_lifecycle() {
    let service = LoamSpineService::new();

    // 1. Create spine
    let owner = Did::new("did:key:z6MkOwnerE2E");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("E2E Test Spine".into()))
        .await
        .expect("Failed to create spine");

    // 2. Commit multiple sessions
    for i in 0..5 {
        let summary = DehydrationSummary::new(
            SessionId::now_v7(),
            format!("e2e-session-{i}"),
            [i as u8; 32],
        )
        .with_vertex_count(100 + i);

        let commit_ref = service
            .commit_session(spine_id, owner.clone(), summary)
            .await
            .expect("Failed to commit session");

        // Verify each commit
        assert!(service
            .verify_commit(&commit_ref)
            .await
            .expect("Failed to verify commit"));
    }

    // 3. Check spine state
    let spine = service
        .get_spine(spine_id)
        .await
        .expect("Failed to get spine")
        .expect("Spine should exist");
    assert_eq!(spine.height, 6); // genesis + 5 commits

    // 4. Seal the spine
    let seal_hash = service
        .seal_spine(spine_id, Some("Test completed".into()))
        .await
        .expect("Failed to seal spine");
    assert!(!seal_hash.iter().all(|&b| b == 0));
}

/// Test certificate minting, transfer, loan, and return.
#[tokio::test]
async fn full_certificate_lifecycle() {
    let service = LoamSpineService::new();

    // Setup: Create spine
    let owner = Did::new("did:key:z6MkOwnerCert");
    let borrower = Did::new("did:key:z6MkBorrowerCert");
    let recipient = Did::new("did:key:z6MkRecipientCert");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Certificate Test".into()))
        .await
        .expect("Failed to create spine");

    // 1. Mint certificate
    let (cert_id, _mint_hash) = service
        .mint_certificate(spine_id, test_cert_type(), owner.clone(), None)
        .await
        .expect("Failed to mint certificate");

    // 2. Verify certificate exists
    let cert = service
        .get_certificate(cert_id)
        .await
        .expect("Certificate should exist");
    assert_eq!(cert.owner, owner);

    // 3. Loan certificate
    let terms = LoanTerms::new().with_duration(SECONDS_PER_HOUR);
    let _loan_hash = service
        .loan_certificate(cert_id, owner.clone(), borrower.clone(), terms)
        .await
        .expect("Failed to loan certificate");

    // Verify loaned state
    let cert = service
        .get_certificate(cert_id)
        .await
        .expect("Certificate should exist");
    assert!(cert.is_loaned());
    assert_eq!(cert.holder, Some(borrower.clone()));

    // 4. Return certificate
    let _return_hash = service
        .return_certificate(cert_id, borrower.clone())
        .await
        .expect("Failed to return certificate");

    // Verify returned state
    let cert = service
        .get_certificate(cert_id)
        .await
        .expect("Certificate should exist");
    assert!(!cert.is_loaned());

    // 5. Transfer certificate
    let _transfer_hash = service
        .transfer_certificate(cert_id, owner.clone(), recipient.clone())
        .await
        .expect("Failed to transfer certificate");

    // Verify new owner
    let cert = service
        .get_certificate(cert_id)
        .await
        .expect("Certificate should exist");
    assert_eq!(cert.owner, recipient);
    assert_eq!(cert.transfer_count, 1);
}

/// Test slice checkout and resolve flow.
#[tokio::test]
async fn slice_checkout_resolve_flow() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwnerSlice");
    let holder = Did::new("did:key:z6MkHolderSlice");

    // Create spine
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Slice Test".into()))
        .await
        .expect("Failed to create spine");

    // Get genesis hash
    let spine = service
        .get_spine(spine_id)
        .await
        .expect("Failed to get spine")
        .expect("Spine should exist");
    let genesis_hash = spine.genesis;

    // Checkout slice
    let session_id = SessionId::now_v7();
    let origin = service
        .checkout_slice(spine_id, genesis_hash, holder.clone(), session_id)
        .await
        .expect("Failed to checkout slice");

    assert_eq!(origin.spine_id, spine_id);
    assert_eq!(origin.owner, owner);
}

/// Test braid commit and verification.
#[tokio::test]
async fn braid_commit_and_verify() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwnerBraid");
    let contributor = Did::new("did:key:z6MkContributor");

    // Create spine
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Braid Test".into()))
        .await
        .expect("Failed to create spine");

    // Create and commit braid
    let braid_id = BraidId::now_v7();
    let subject_hash = [1u8; 32];
    let braid = BraidSummary::new(braid_id, "attribution", subject_hash, [2u8; 32])
        .with_agent(owner.clone())
        .with_agent(contributor.clone());

    let entry_hash = service
        .commit_braid(spine_id, owner.clone(), braid)
        .await
        .expect("Failed to commit braid");

    // Verify braid exists
    assert!(service
        .verify_braid(braid_id)
        .await
        .expect("Failed to verify braid"));

    // Get braids for subject
    let braids = service
        .get_braids_for_subject(subject_hash)
        .await
        .expect("Failed to get braids");
    assert_eq!(braids.len(), 1);
    assert_eq!(braids[0], entry_hash);
}

/// Test concurrent operations on the same spine.
#[tokio::test]
async fn concurrent_spine_operations() {
    use std::sync::Arc;

    let service = Arc::new(LoamSpineService::new());
    let owner = Did::new("did:key:z6MkOwnerConcurrent");

    // Create spine
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Concurrent Test".into()))
        .await
        .expect("Failed to create spine");

    // Spawn multiple concurrent commits
    let mut handles = vec![];
    for i in 0..10 {
        let svc = Arc::clone(&service);
        let own = owner.clone();
        let handle = tokio::spawn(async move {
            let summary = DehydrationSummary::new(
                SessionId::now_v7(),
                format!("concurrent-session-{i}"),
                [i as u8; 32],
            );
            svc.commit_session(spine_id, own, summary).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let mut success_count = 0;
    for handle in handles {
        if handle.await.expect("Task panicked").is_ok() {
            success_count += 1;
        }
    }

    // All should succeed (sequential due to locks)
    assert_eq!(success_count, 10);

    // Verify final spine state
    let spine = service
        .get_spine(spine_id)
        .await
        .expect("Failed to get spine")
        .expect("Spine should exist");
    assert_eq!(spine.height, 11); // genesis + 10 concurrent commits
}

/// Test capability registry integration.
#[tokio::test]
async fn capability_registry_integration() {
    use loam_spine_core::discovery::CapabilityStatus;
    use loam_spine_core::CapabilityRegistry;

    // Create service with shared registry
    let registry = CapabilityRegistry::new();
    let service = LoamSpineService::with_capabilities(registry.clone());

    // Initially no capabilities
    assert_eq!(
        service.capabilities().signer_status().await,
        CapabilityStatus::Unavailable
    );
    assert_eq!(
        service.capabilities().verifier_status().await,
        CapabilityStatus::Unavailable
    );

    // Note: MockSigner is only available with the "testing" feature.
    // To test registration, run with: cargo test --features testing
}
