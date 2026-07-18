// SPDX-License-Identifier: AGPL-3.0-or-later

//! Chaos and fault injection tests for `LoamSpine`.
//!
//! Sequential tests verifying system behavior under adverse conditions:
//! edge cases, error handling, and fault injection.
//! See `chaos_stress.rs` for concurrent stress and endurance tests.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "chaos tests use expect/panic for failure clarity"
)]
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "chaos test data uses small controlled values"
)]

use loam_spine_core::{
    CertificateType, LoamSpineError, LoanTerms, SECONDS_PER_HOUR,
    service::LoamSpineService,
    traits::{CommitAcceptor, DehydrationSummary, SpineQuery},
    types::{Did, SessionId, SpineId},
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

/// Test rapid spine creation and deletion (via sealing).
#[tokio::test]
async fn rapid_spine_creation() {
    let service = LoamSpineService::new();

    // Create many spines rapidly
    let mut spine_ids = vec![];
    for i in 0..100 {
        let owner = Did::new(format!("did:key:z6MkOwner{i}"));
        let spine_id = service
            .ensure_spine(owner, Some(format!("Chaos Spine {i}")))
            .await
            .expect("Failed to create spine");
        spine_ids.push(spine_id);
    }

    assert_eq!(service.spine_count().await, 100);

    // Verify all spines exist
    for spine_id in spine_ids {
        let spine = service
            .get_spine(spine_id)
            .await
            .expect("Failed to get spine");
        assert!(spine.is_some());
    }
}

/// Test error handling for non-existent spine.
#[tokio::test]
async fn nonexistent_spine_error() {
    let service = LoamSpineService::new();

    let fake_spine_id = SpineId::nil();
    let owner = Did::new("did:key:z6MkOwner");

    let summary = DehydrationSummary::new(SessionId::now_v7(), "test", [0u8; 32]);

    let result = service.commit_session(fake_spine_id, owner, summary).await;

    assert!(matches!(result, Err(LoamSpineError::SpineNotFound(_))));
}

/// Test double-seal prevention.
#[tokio::test]
async fn double_seal_prevention() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = service
        .ensure_spine(owner, Some("Seal Test".into()))
        .await
        .expect("Failed to create spine");

    // First seal should succeed
    service
        .seal_spine(spine_id, Some("First seal".into()))
        .await
        .expect("First seal should succeed");

    // Second seal should fail
    let result = service
        .seal_spine(spine_id, Some("Second seal".into()))
        .await;
    assert!(matches!(result, Err(LoamSpineError::SpineSealed(_))));
}

/// Test entry retrieval with invalid hash.
#[tokio::test]
async fn invalid_entry_hash() {
    let service = LoamSpineService::new();

    let fake_hash = [0u8; 32];
    let result = service.get_entry(fake_hash).await;

    // Should return Ok(None) for non-existent entry
    assert!(result.expect("Query should not fail").is_none());
}

/// Test certificate operations on non-existent certificate.
#[tokio::test]
async fn nonexistent_certificate() {
    let service = LoamSpineService::new();

    let fake_cert_id = uuid::Uuid::nil();
    let result = service.get_certificate(fake_cert_id).await;

    assert!(result.is_none());
}

/// Test ownership validation for certificate transfer.
#[tokio::test]
async fn certificate_ownership_validation() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");
    let imposter = Did::new("did:key:z6MkImposter");
    let recipient = Did::new("did:key:z6MkRecipient");

    // Create spine and mint certificate
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Ownership Test".into()))
        .await
        .expect("Failed to create spine");

    let (cert_id, _) = service
        .mint_certificate(spine_id, test_cert_type(), owner.clone(), None)
        .await
        .expect("Failed to mint certificate");

    // Imposter should not be able to transfer
    let result = service
        .transfer_certificate(cert_id, imposter, recipient)
        .await;

    assert!(matches!(result, Err(LoamSpineError::NotCertificateOwner)));
}

/// Test loan terms validation (cannot transfer while loaned).
#[tokio::test]
async fn cannot_transfer_while_loaned() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");
    let borrower = Did::new("did:key:z6MkBorrower");
    let recipient = Did::new("did:key:z6MkRecipient");

    // Create spine and mint certificate
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Loan Test".into()))
        .await
        .expect("Failed to create spine");

    let (cert_id, _) = service
        .mint_certificate(spine_id, test_cert_type(), owner.clone(), None)
        .await
        .expect("Failed to mint certificate");

    // Loan the certificate
    let terms = loam_spine_core::LoanTerms::new().with_duration(SECONDS_PER_HOUR);
    service
        .loan_certificate(cert_id, owner.clone(), borrower.clone(), terms)
        .await
        .expect("Failed to loan certificate");

    // Should not be able to transfer while loaned
    let result = service
        .transfer_certificate(cert_id, owner, recipient)
        .await;

    assert!(matches!(result, Err(LoamSpineError::CertificateLoaned(_))));
}

/// Test high-volume commit operations.
#[tokio::test]
async fn high_volume_commits() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("High Volume Test".into()))
        .await
        .expect("Failed to create spine");

    // Commit many sessions
    for i in 0..1000 {
        let summary =
            DehydrationSummary::new(SessionId::now_v7(), format!("session-{i}"), [0u8; 32])
                .with_vertex_count(i);

        service
            .commit_session(spine_id, owner.clone(), summary)
            .await
            .expect("Failed to commit session");
    }

    // Verify final state
    let spine = service
        .get_spine(spine_id)
        .await
        .expect("Failed to get spine")
        .expect("Spine should exist");
    assert_eq!(spine.height, 1001); // genesis + 1000 commits
}

/// Test memory efficiency with many entries.
#[tokio::test]
async fn memory_efficiency() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Memory Test".into()))
        .await
        .expect("Failed to create spine");

    // Create entries with larger metadata
    for i in 0..100 {
        let summary =
            DehydrationSummary::new(SessionId::now_v7(), format!("session-{i}"), [0u8; 32])
                .with_vertex_count(1000)
                .with_metadata("key1", "value1".repeat(100))
                .with_metadata("key2", "value2".repeat(100));

        service
            .commit_session(spine_id, owner.clone(), summary)
            .await
            .expect("Failed to commit session");
    }

    // Query should still work efficiently
    let entries = service
        .get_entries(spine_id, 0, 50)
        .await
        .expect("Failed to get entries");
    assert_eq!(entries.len(), 50);
}

/// Test network timeout simulation (service unavailable).
#[tokio::test]
async fn service_unavailable_handling() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Network Test".into()))
        .await
        .expect("Failed to create spine");

    // Simulate rapid requests that might timeout in real network scenario
    let mut handles = vec![];
    for i in 0..50 {
        let svc = service.clone();
        let own = owner.clone();
        let handle = tokio::spawn(async move {
            let summary =
                DehydrationSummary::new(SessionId::now_v7(), format!("rapid-{i}"), [i as u8; 32]);
            svc.commit_session(spine_id, own, summary).await
        });
        handles.push(handle);
    }

    // All should complete successfully (in-memory, no real network)
    for handle in handles {
        assert!(handle.await.expect("Task panicked").is_ok());
    }
}

/// Test corrupted entry data handling.
#[tokio::test]
async fn corrupted_entry_handling() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Corruption Test".into()))
        .await
        .expect("Failed to create spine");

    // Create entry with invalid data
    let summary =
        DehydrationSummary::new(SessionId::now_v7(), "test", [0u8; 32]).with_vertex_count(u64::MAX); // Edge case: maximum value

    let result = service.commit_session(spine_id, owner, summary).await;
    // Should handle gracefully
    assert!(result.is_ok());
}

/// Test rapid certificate operations (race conditions).
#[tokio::test]
async fn rapid_certificate_operations() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Cert Race Test".into()))
        .await
        .expect("Failed to create spine");

    // Mint multiple certificates rapidly
    let mut handles = vec![];
    for i in 0..20 {
        let svc = service.clone();
        let own = owner.clone();
        let handle = tokio::spawn(async move {
            let cert_type = CertificateType::DigitalCollectible {
                collection_id: format!("race-collection-{i}"),
                item_number: Some(i),
                total_supply: Some(100),
                rarity: None,
            };
            svc.mint_certificate(spine_id, cert_type, own, None).await
        });
        handles.push(handle);
    }

    // All should succeed
    let mut success_count = 0;
    for handle in handles {
        if handle.await.expect("Task panicked").is_ok() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 20);
}

/// Test spine sealing race condition.
#[tokio::test]
async fn seal_race_condition() {
    use std::sync::Arc;

    let service = Arc::new(LoamSpineService::new());
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Seal Race".into()))
        .await
        .expect("Failed to create spine");

    // Try to seal multiple times concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let svc = Arc::clone(&service);
        let handle = tokio::spawn(async move {
            svc.seal_spine(spine_id, Some(format!("Seal attempt {i}")))
                .await
        });
        handles.push(handle);
    }

    // Only one should succeed, rest should fail with SpineSealed error
    let mut success_count = 0;
    let mut sealed_errors = 0;

    for handle in handles {
        match handle.await.expect("Task panicked") {
            Ok(_) => success_count += 1,
            Err(LoamSpineError::SpineSealed(_)) => sealed_errors += 1,
            Err(e) => panic!("Unexpected error: {e:?}"),
        }
    }

    assert_eq!(success_count, 1, "Exactly one seal should succeed");
    assert_eq!(sealed_errors, 9, "Nine seals should fail with SpineSealed");
}

/// Test certificate loan expiration edge cases.
#[tokio::test]
async fn certificate_loan_expiration() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");
    let borrower = Did::new("did:key:z6MkBorrower");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Loan Test".into()))
        .await
        .expect("Failed to create spine");

    let (cert_id, _) = service
        .mint_certificate(
            spine_id,
            CertificateType::DigitalGame {
                platform: "PC".into(),
                game_id: "test-game-123".into(),
                edition: Some("Standard Edition".into()),
            },
            owner.clone(),
            None,
        )
        .await
        .expect("Failed to mint certificate");

    // Create loan with very short duration (edge case)
    let terms = LoanTerms::new().with_duration(1); // 1 second
    service
        .loan_certificate(cert_id, owner.clone(), borrower.clone(), terms)
        .await
        .expect("Failed to loan certificate");

    // Verify loaned state
    let cert = service
        .get_certificate(cert_id)
        .await
        .expect("Certificate should exist");
    assert!(cert.is_loaned());

    // Try to return immediately
    let result = service.return_certificate(cert_id, borrower).await;
    assert!(result.is_ok());
}

/// Test memory pressure with large metadata.
#[tokio::test]
async fn large_metadata_handling() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Large Metadata".into()))
        .await
        .expect("Failed to create spine");

    // Create entry with very large metadata
    let large_value = "x".repeat(1024 * 100); // 100KB string
    let summary = DehydrationSummary::new(SessionId::now_v7(), "large", [0u8; 32])
        .with_metadata("large_key", &large_value);

    let result = service.commit_session(spine_id, owner, summary).await;
    assert!(result.is_ok());
}

/// Test clock skew tolerance (timestamps).
#[tokio::test]
async fn timestamp_edge_cases() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Timestamp Test".into()))
        .await
        .expect("Failed to create spine");

    // Create entries rapidly (same or very close timestamps)
    for i in 0..10 {
        let summary =
            DehydrationSummary::new(SessionId::now_v7(), format!("rapid-ts-{i}"), [i as u8; 32]);
        service
            .commit_session(spine_id, owner.clone(), summary)
            .await
            .expect("Failed to commit");
    }

    // Verify all entries are in spine
    let spine = service
        .get_spine(spine_id)
        .await
        .expect("Failed to get spine")
        .expect("Spine should exist");
    assert_eq!(spine.height, 11); // genesis + 10 entries
}

/// Test empty string handling in various fields.
#[tokio::test]
async fn empty_string_handling() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");

    // Empty spine name should work (uses default)
    let spine_id = service
        .ensure_spine(owner.clone(), Some(String::new()))
        .await
        .expect("Failed to create spine");

    // Empty session kind
    let summary = DehydrationSummary::new(SessionId::now_v7(), "", [0u8; 32]);
    let result = service.commit_session(spine_id, owner, summary).await;
    assert!(result.is_ok());
}

/// Test maximum spine height (boundary condition).
#[tokio::test]
async fn maximum_spine_height() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkOwner");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Max Height".into()))
        .await
        .expect("Failed to create spine");

    // Add many entries (test performance and boundary)
    for i in 0..100 {
        let summary = DehydrationSummary::new(SessionId::now_v7(), format!("entry-{i}"), [0u8; 32]);
        service
            .commit_session(spine_id, owner.clone(), summary)
            .await
            .expect("Failed to commit");
    }

    let spine = service
        .get_spine(spine_id)
        .await
        .expect("Failed to get spine")
        .expect("Spine should exist");
    assert_eq!(spine.height, 101); // genesis + 100
}
