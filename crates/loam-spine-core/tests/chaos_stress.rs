// SPDX-License-Identifier: AGPL-3.0-or-later

//! Stress and concurrency tests for `LoamSpine`.
//!
//! High-volume, concurrent, and endurance tests exercising system behavior
//! under heavy load. See `chaos.rs` for sequential fault injection tests.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "stress tests use unwrap/expect for failure clarity"
)]
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "stress test data uses small controlled values"
)]

use loam_spine_core::{
    CertificateType, LoanTerms, SECONDS_PER_HOUR,
    service::LoamSpineService,
    traits::{CommitAcceptor, DehydrationSummary, SpineQuery},
    types::{Did, SessionId, SpineId},
};

/// Test massive concurrent spine creation (stress test).
#[tokio::test]
async fn massive_concurrent_spine_creation() {
    let service = LoamSpineService::new();
    let mut tasks = vec![];

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

    assert!(success_count >= 990);
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

    assert_eq!(spine.height, 1001);
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

    assert!(success_count >= 95);
}

/// Test memory efficiency under high load.
#[tokio::test]
async fn memory_efficiency_test() {
    let service = LoamSpineService::new();

    for i in 0..200 {
        let owner = Did::new(format!("did:key:mem{i}"));
        let spine_id = service
            .ensure_spine(owner.clone(), Some(format!("Mem {i}")))
            .await
            .expect("Failed to create spine");

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

    let invalid_spine = SpineId::nil();
    let result = service.get_spine(invalid_spine).await;
    assert!(result.is_ok());

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
