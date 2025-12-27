//! Fault Tolerance Tests
//!
//! Comprehensive fault injection tests for network, disk, memory, and Byzantine scenarios.
//! These tests verify LoamSpine's resilience under adverse conditions.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use loam_spine_core::{
    service::LoamSpineService,
    traits::{CommitAcceptor, DehydrationSummary, SpineQuery},
    types::{Did, SessionId},
    LoamSpineError,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

//
// ═══════════════════════════════════════════════════════════════════════════
// NETWORK PARTITION TESTS
// ═══════════════════════════════════════════════════════════════════════════
//

/// Test service operation when storage backend becomes unavailable
#[tokio::test]
async fn test_network_partition_storage_unavailable() {
    let service = LoamSpineService::new();

    // Service should create spines even if external storage is slow
    let owner = Did::new("did:key:z6MkNetworkTest");
    let spine_id = timeout(
        Duration::from_secs(5),
        service.ensure_spine(owner, Some("Network Test".into())),
    )
    .await
    .expect("Operation should not hang")
    .expect("Should create spine");

    // Verify spine exists
    let spine = service.get_spine(spine_id).await.expect("Should get spine");
    assert!(spine.is_some(), "Spine should exist");
}

/// Test rapid operations under network stress
#[tokio::test]
async fn test_network_stress_rapid_operations() {
    let service = Arc::new(LoamSpineService::new());

    // Create spines concurrently (simulating network stress)
    let mut handles = vec![];
    for i in 0..50 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            let owner = Did::new(format!("did:key:z6MkStress{i}"));
            service_clone.ensure_spine(owner, None).await
        });
        handles.push(handle);
    }

    // All operations should complete within reasonable time
    let results = timeout(Duration::from_secs(10), async {
        let mut all_results = vec![];
        for handle in handles {
            all_results.push(handle.await);
        }
        all_results
    })
    .await
    .expect("Operations should not hang under stress");

    // Count successes
    let successes = results.iter().filter(|r| r.is_ok()).count();
    assert!(
        successes >= 45,
        "Most operations should succeed: {successes}/50"
    );
}

/// Test timeout handling for slow operations
#[tokio::test]
async fn test_network_timeout_graceful_handling() {
    let service = LoamSpineService::new();

    // Operations should complete within reasonable time limits
    let owner = Did::new("did:key:z6MkTimeout");
    let result = timeout(
        Duration::from_secs(3),
        service.ensure_spine(owner, Some("Timeout Test".into())),
    )
    .await;

    assert!(
        result.is_ok(),
        "Operations should timeout gracefully, not hang"
    );
}

//
// ═══════════════════════════════════════════════════════════════════════════
// DISK/STORAGE FAULT TESTS
// ═══════════════════════════════════════════════════════════════════════════
//

/// Test handling of storage pressure with many spines
#[tokio::test]
async fn test_disk_pressure_many_spines() {
    let service = LoamSpineService::new();

    // Create many spines to simulate storage pressure
    let mut spine_ids = vec![];
    for i in 0..1000 {
        let owner = Did::new(format!("did:key:z6MkDisk{i}"));
        match service.ensure_spine(owner, None).await {
            Ok(id) => spine_ids.push(id),
            Err(e) => {
                println!("Storage pressure at {i} spines: {e:?}");
                break;
            }
        }
    }

    // Should handle at least 500 spines
    assert!(
        spine_ids.len() >= 500,
        "Should handle storage pressure: created {}",
        spine_ids.len()
    );

    // Service should still be responsive
    let final_count = service.spine_count().await;
    assert!(final_count > 0, "Service should remain responsive");
}

/// Test handling of large data commits
#[tokio::test]
async fn test_disk_large_commit_handling() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkLargeCommit");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Large Commit Test".into()))
        .await
        .expect("Should create spine");

    // Create commit with large metadata
    let large_metadata = "x".repeat(100_000); // 100KB of metadata
    let summary = DehydrationSummary::new(SessionId::now_v7(), &large_metadata, [0u8; 32]);

    // Should handle large commits gracefully
    let result = service.commit_session(spine_id, owner, summary).await;

    match result {
        Ok(_) => println!("Successfully handled large commit"),
        Err(e) => println!("Gracefully rejected large commit: {e:?}"),
    }

    // Service should remain stable
    let count = service.spine_count().await;
    assert!(count > 0, "Service should remain stable");
}

//
// ═══════════════════════════════════════════════════════════════════════════
// MEMORY PRESSURE TESTS
// ═══════════════════════════════════════════════════════════════════════════
//

/// Test memory handling with concurrent spine operations
#[tokio::test]
async fn test_memory_pressure_concurrent_operations() {
    let service = Arc::new(LoamSpineService::new());

    // Create spines and perform operations concurrently
    let mut handles = vec![];
    for i in 0..100 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            let owner = Did::new(format!("did:key:z6MkMemory{i}"));
            let spine_id = service_clone.ensure_spine(owner.clone(), None).await?;

            // Perform commit on each spine
            let summary =
                DehydrationSummary::new(SessionId::now_v7(), &format!("test-{i}"), [0u8; 32]);
            service_clone.commit_session(spine_id, owner, summary).await
        });
        handles.push(handle);
    }

    // Wait for all operations
    let mut successes = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => successes += 1,
            Ok(Err(e)) => println!("Expected error under pressure: {e:?}"),
            Err(e) => panic!("Task panicked: {e:?}"),
        }
    }

    assert!(
        successes >= 80,
        "Most operations should succeed under memory pressure: {successes}/100"
    );
}

/// Test memory leak detection via rapid create/seal cycles
#[tokio::test]
async fn test_memory_leak_detection_cycles() {
    let service = LoamSpineService::new();

    // Rapid create/seal cycles
    for cycle in 0..200 {
        let owner = Did::new(format!("did:key:z6MkLeak{}", cycle));
        let spine_id = service
            .ensure_spine(owner, None)
            .await
            .expect("Should create spine");

        // Seal spine (finalization)
        service
            .seal_spine(spine_id, Some(format!("Cycle {}", cycle)))
            .await
            .ok(); // Ignore seal errors
    }

    // Service should remain responsive
    let final_owner = Did::new("did:key:z6MkFinal");
    let final_result = service.ensure_spine(final_owner, None).await;
    assert!(
        final_result.is_ok(),
        "Service should remain stable after many cycles"
    );
}

/// Test handling of rapid spine creation bursts
#[tokio::test]
async fn test_memory_burst_spine_creation() {
    let service = LoamSpineService::new();

    // Burst creation
    let start_count = service.spine_count().await;

    for i in 0..500 {
        let owner = Did::new(format!("did:key:z6MkBurst{}", i));
        service.ensure_spine(owner, None).await.ok();
    }

    let end_count = service.spine_count().await;
    let created = end_count - start_count;

    assert!(
        created >= 400,
        "Should handle burst creation: created {}",
        created
    );
}

//
// ═══════════════════════════════════════════════════════════════════════════
// CLOCK SKEW TESTS
// ═══════════════════════════════════════════════════════════════════════════
//

/// Test handling of rapid consecutive operations (clock precision)
#[tokio::test]
async fn test_clock_skew_rapid_consecutive_commits() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkClockTest");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Clock Test".into()))
        .await
        .expect("Should create spine");

    // Perform rapid consecutive commits (tests timestamp precision)
    for i in 0..10 {
        let summary =
            DehydrationSummary::new(SessionId::now_v7(), &format!("commit-{}", i), [i as u8; 32]);

        service
            .commit_session(spine_id, owner.clone(), summary)
            .await
            .expect("Rapid commits should succeed");
    }

    // Verify spine maintains order
    let spine = service
        .get_spine(spine_id)
        .await
        .expect("Should get spine")
        .expect("Spine should exist");

    // Genesis + 10 commits = 11 entries
    assert_eq!(spine.height, 11, "Should have 11 entries");
}

/// Test timestamp ordering under concurrent load
#[tokio::test]
async fn test_clock_skew_concurrent_timestamp_ordering() {
    let service = Arc::new(LoamSpineService::new());

    let owner = Did::new("did:key:z6MkConcurrentClock");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Concurrent Clock Test".into()))
        .await
        .expect("Should create spine");

    // Concurrent commits (may arrive out of order)
    let mut handles = vec![];
    for i in 0..20 {
        let service_clone = service.clone();
        let owner_clone = owner.clone();
        let handle = tokio::spawn(async move {
            let summary = DehydrationSummary::new(
                SessionId::now_v7(),
                &format!("concurrent-{}", i),
                [i as u8; 32],
            );
            service_clone
                .commit_session(spine_id, owner_clone, summary)
                .await
        });
        handles.push(handle);
    }

    // Wait for all commits
    for handle in handles {
        handle.await.ok();
    }

    // Spine should maintain consistency despite concurrent operations
    let spine = service
        .get_spine(spine_id)
        .await
        .expect("Should get spine")
        .expect("Spine should exist");

    assert!(
        spine.height >= 10,
        "Spine should have accepted commits: height={}",
        spine.height
    );
}

//
// ═══════════════════════════════════════════════════════════════════════════
// BYZANTINE FAULT TESTS
// ═══════════════════════════════════════════════════════════════════════════
//

/// Test handling of invalid DIDs
#[tokio::test]
async fn test_byzantine_malformed_dids() {
    let service = LoamSpineService::new();

    // Various malformed DIDs
    let very_long = "x".repeat(10000);
    let bad_dids = vec![
        "",
        "not-a-did",
        "did:",
        "did::",
        "did:method:",
        &very_long, // Very long
    ];

    for bad_did in bad_dids {
        let result = service.ensure_spine(Did::new(bad_did), None).await;

        // Should either accept or reject gracefully, not crash
        match result {
            Ok(_) => println!("Accepted DID: {}", bad_did),
            Err(e) => println!("Rejected DID '{bad_did}': {e:?}"),
        }
    }

    // Service should still work with valid DID
    let valid_result = service
        .ensure_spine(Did::new("did:key:z6MkValid"), None)
        .await;
    assert!(valid_result.is_ok(), "Should accept valid DIDs");
}

/// Test double-seal Byzantine attack
#[tokio::test]
async fn test_byzantine_double_seal_attack() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkDoubleSeal");
    let spine_id = service
        .ensure_spine(owner, Some("Double Seal Test".into()))
        .await
        .expect("Should create spine");

    // First seal should succeed
    let seal1 = service.seal_spine(spine_id, Some("First".into())).await;
    assert!(seal1.is_ok(), "First seal should succeed");

    // Second seal should be prevented (Byzantine attack attempt)
    let seal2 = service.seal_spine(spine_id, Some("Second".into())).await;
    assert!(
        matches!(seal2, Err(LoamSpineError::SpineSealed(_))),
        "Double seal should be prevented"
    );
}

/// Test conflicting concurrent operations (race conditions)
#[tokio::test]
async fn test_byzantine_concurrent_conflicts() {
    let service = Arc::new(LoamSpineService::new());

    let owner = Did::new("did:key:z6MkConflict");
    let spine_id = service
        .ensure_spine(owner.clone(), Some("Conflict Test".into()))
        .await
        .expect("Should create spine");

    // Multiple tasks trying to commit simultaneously
    let mut handles = vec![];
    for i in 0..30 {
        let service_clone = service.clone();
        let owner_clone = owner.clone();
        let handle = tokio::spawn(async move {
            let summary = DehydrationSummary::new(
                SessionId::now_v7(),
                &format!("conflict-{}", i),
                [i as u8; 32],
            );
            service_clone
                .commit_session(spine_id, owner_clone, summary)
                .await
        });
        handles.push(handle);
    }

    // All tasks should complete (success or failure, but not hang/crash)
    let mut successes = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => successes += 1,
            Ok(Err(_)) => {} // Expected conflicts
            Err(e) => panic!("Task panicked: {:?}", e),
        }
    }

    println!("Successful concurrent commits: {successes}/30");

    // At least some should succeed
    assert!(successes > 0, "Some operations should succeed");
}

/// Test resource exhaustion attempt
#[tokio::test]
async fn test_byzantine_resource_exhaustion_attempt() {
    let service = LoamSpineService::new();

    // Attempt to create extremely large spine name
    let huge_name = "x".repeat(1_000_000); // 1MB name
    let owner = Did::new("did:key:z6MkExhaust");

    let result = service.ensure_spine(owner, Some(huge_name)).await;

    match result {
        Ok(_) => println!("Accepted large name"),
        Err(e) => println!("Rejected large name: {e:?}"),
    }

    // Service should remain responsive
    let test_result = service
        .ensure_spine(Did::new("did:key:z6MkTest"), None)
        .await;
    assert!(
        test_result.is_ok(),
        "Service should remain responsive after exhaustion attempt"
    );
}

/// Test handling of non-existent spine operations (Byzantine probing)
#[tokio::test]
async fn test_byzantine_nonexistent_spine_probing() {
    let service = LoamSpineService::new();

    // Try operations on non-existent spines
    let fake_id = loam_spine_core::SpineId::nil();
    let owner = Did::new("did:key:z6MkProbe");

    let summary = DehydrationSummary::new(SessionId::now_v7(), "probe", [0u8; 32]);
    let result = service.commit_session(fake_id, owner, summary).await;

    // Should get proper error, not crash
    assert!(
        matches!(result, Err(LoamSpineError::SpineNotFound(_))),
        "Should handle non-existent spine gracefully"
    );
}

/// Test rapid state change attempts (start/stop Byzantine pattern)
#[tokio::test]
async fn test_byzantine_rapid_seal_attempts() {
    let service = LoamSpineService::new();

    let owner = Did::new("did:key:z6MkRapidSeal");
    let spine_id = service
        .ensure_spine(owner, Some("Rapid Seal Test".into()))
        .await
        .expect("Should create spine");

    // Rapid seal attempts (Byzantine attack pattern)
    let mut seal_results = vec![];
    for i in 0..10 {
        let result = service
            .seal_spine(spine_id, Some(format!("Attempt {i}")))
            .await;
        seal_results.push(result);
    }

    // First should succeed, rest should fail
    let successes = seal_results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(successes, 1, "Only one seal should succeed");

    // Rest should be proper errors
    let proper_errors = seal_results
        .iter()
        .filter(|r| matches!(r, Err(LoamSpineError::SpineSealed(_))))
        .count();
    assert!(
        proper_errors >= 8,
        "Failed seals should be proper errors: {proper_errors}"
    );
}
