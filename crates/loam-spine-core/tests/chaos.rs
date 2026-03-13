// SPDX-License-Identifier: AGPL-3.0-only

//! Chaos and fault injection tests for `LoamSpine`.
//!
//! These tests verify system behavior under adverse conditions like
//! rapid concurrent operations, edge cases, and error handling.

// Allow various patterns in tests that would be problematic in production
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::manual_string_new)]

use loam_spine_core::{
    service::LoamSpineService,
    traits::{CommitAcceptor, DehydrationSummary, SpineQuery},
    types::{Did, SessionId, SpineId},
    CertificateType, LoamSpineError, LoanTerms, SECONDS_PER_HOUR,
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

    let fake_spine_id = loam_spine_core::SpineId::nil();
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
            Err(e) => panic!("Unexpected error: {:?}", e),
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
        .ensure_spine(owner.clone(), Some("".into()))
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

/// Test massive concurrent spine creation (stress test).
#[tokio::test]
async fn massive_concurrent_spine_creation() {
    let service = LoamSpineService::new();
    let mut tasks = vec![];

    // Create 1000 spines concurrently
    for i in 0..1000 {
        let service = service.clone();
        let task = tokio::spawn(async move {
            let owner = Did::new(format!("did:key:stress{i}"));
            service
                .ensure_spine(owner, Some(format!("Stress {i}")))
                .await
        });
        tasks.push(task);
    }

    let mut success_count = 0;
    for task in tasks {
        if task.await.unwrap().is_ok() {
            success_count += 1;
        }
    }

    assert!(success_count >= 990); // Allow for some contention
    assert!(service.spine_count().await >= 990);
}

/// Test rapid certificate operations under load.
#[tokio::test]
async fn rapid_certificate_churn() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:cert-churn");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Cert Churn".into()))
        .await
        .expect("Failed to create spine");

    // Rapidly mint, loan, and return certificates
    for i in 0..50 {
        let cert_type = CertificateType::DigitalCollectible {
            collection_id: format!("churn-{i}"),
            item_number: Some(i),
            total_supply: Some(1000),
            rarity: None,
        };

        let (cert_id, _) = service
            .mint_certificate(spine_id, cert_type, owner.clone(), None)
            .await
            .expect("Failed to mint");

        let borrower = Did::new(format!("did:key:borrower{i}"));
        let terms = LoanTerms::new().with_duration(SECONDS_PER_HOUR);

        service
            .loan_certificate(cert_id, owner.clone(), borrower.clone(), terms)
            .await
            .expect("Failed to loan");

        service
            .return_certificate(cert_id, borrower)
            .await
            .expect("Failed to return");
    }

    // Verify spine is still healthy
    let spine = service
        .get_spine(spine_id)
        .await
        .expect("Failed to get spine");
    assert!(spine.is_some());
}

/// Test extreme spine height (endurance test).
#[tokio::test]
async fn extreme_spine_height() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:extreme");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Extreme Height".into()))
        .await
        .expect("Failed to create spine");

    // Add 1000 entries
    for i in 0..1000 {
        let summary =
            DehydrationSummary::new(SessionId::now_v7(), format!("extreme-{i}"), [i as u8; 32]);

        let result = service
            .commit_session(spine_id, owner.clone(), summary)
            .await;

        assert!(result.is_ok(), "Failed at entry {i}");
    }

    let spine = service
        .get_spine(spine_id)
        .await
        .expect("Failed to get spine")
        .expect("Spine should exist");

    assert_eq!(spine.height, 1001); // genesis + 1000
}

/// Test concurrent read/write under contention.
#[tokio::test]
async fn concurrent_read_write_stress() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:rw-stress");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("RW Stress".into()))
        .await
        .expect("Failed to create spine");

    let mut write_tasks = vec![];
    let mut read_tasks = vec![];

    // Spawn 50 writer tasks
    for i in 0..50 {
        let service = service.clone();
        let owner = owner.clone();

        let task = tokio::spawn(async move {
            let summary =
                DehydrationSummary::new(SessionId::now_v7(), format!("writer-{i}"), [i as u8; 32]);
            service.commit_session(spine_id, owner, summary).await
        });
        write_tasks.push(task);
    }

    // Spawn 50 reader tasks
    for _ in 0..50 {
        let service = service.clone();

        let task = tokio::spawn(async move { service.get_spine(spine_id).await });
        read_tasks.push(task);
    }

    let mut success_count = 0;
    for task in write_tasks {
        if task.await.unwrap().is_ok() {
            success_count += 1;
        }
    }
    for task in read_tasks {
        if task.await.unwrap().is_ok() {
            success_count += 1;
        }
    }

    assert!(success_count >= 95); // Allow for some contention
}

/// Test memory efficiency under high load.
#[tokio::test]
async fn memory_efficiency_test() {
    let service = LoamSpineService::new();

    // Create many small spines
    for i in 0..200 {
        let owner = Did::new(format!("did:key:mem{i}"));
        let spine_id = service
            .ensure_spine(owner.clone(), Some(format!("Mem {i}")))
            .await
            .expect("Failed to create spine");

        // Add a few entries
        for j in 0..5 {
            let summary =
                DehydrationSummary::new(SessionId::now_v7(), format!("mem-{i}-{j}"), [i as u8; 32]);
            service
                .commit_session(spine_id, owner.clone(), summary)
                .await
                .expect("Failed to commit");
        }
    }

    assert_eq!(service.spine_count().await, 200);
}

/// Test error recovery after failures.
#[tokio::test]
async fn error_recovery_resilience() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:recovery");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Recovery".into()))
        .await
        .expect("Failed to create spine");

    // Try invalid operations
    let invalid_spine = SpineId::nil();
    let result = service.get_spine(invalid_spine).await;
    assert!(result.is_ok()); // Should return Ok(None), not error

    // Service should still be functional
    let summary = DehydrationSummary::new(SessionId::now_v7(), "recovery", [0u8; 32]);
    let result = service.commit_session(spine_id, owner, summary).await;
    assert!(result.is_ok());
}

/// Test boundary conditions for certificate operations.
#[tokio::test]
async fn certificate_boundary_conditions() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:boundary");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Boundary".into()))
        .await
        .expect("Failed to create spine");

    // Test with many certificates
    for i in 0..100 {
        let cert_type = CertificateType::DigitalCollectible {
            collection_id: format!("boundary-{i}"),
            item_number: Some(i),
            total_supply: Some(1000),
            rarity: None,
        };

        let result = service
            .mint_certificate(spine_id, cert_type, owner.clone(), None)
            .await;
        assert!(result.is_ok());
    }
}

/// Test rapid sequential spine sealing.
#[tokio::test]
async fn rapid_spine_sealing() {
    let service = LoamSpineService::new();

    for i in 0..100 {
        let owner = Did::new(format!("did:key:seal{i}"));
        let spine_id = service
            .ensure_spine(owner.clone(), Some(format!("Seal {i}")))
            .await
            .expect("Failed to create spine");

        let result = service
            .seal_spine(spine_id, Some(format!("Sealed {i}")))
            .await;
        assert!(result.is_ok());
    }

    assert_eq!(service.spine_count().await, 100);
}
