// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::discovery::DynAttestationProvider;
use crate::traits::SpineQuery;
use crate::types::{Did, Timestamp};
use crate::waypoint::{
    AttestationContext, AttestationRequirement, AttestationResult, WaypointConfig,
};
use std::sync::Arc;

/// Stub attestation provider for tests.
struct TestAttestationProvider;

impl DynAttestationProvider for TestAttestationProvider {
    fn request_attestation(
        &self,
        _context: AttestationContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = LoamSpineResult<AttestationResult>> + Send + '_>,
    > {
        Box::pin(async move {
            Ok(AttestationResult {
                attested: true,
                attester: Did::new("did:key:z6MkTestAttester"),
                timestamp: Timestamp::now(),
                token: vec![],
                denial_reason: None,
            })
        })
    }
}

async fn service_with_attestation_provider() -> LoamSpineService {
    use crate::discovery::CapabilityRegistry;

    let registry = CapabilityRegistry::new();
    registry
        .register_attestation_provider(Arc::new(TestAttestationProvider))
        .await;
    LoamSpineService::with_capabilities(registry)
}

#[tokio::test]
async fn test_anchor_slice() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let slice_id = SliceId::now_v7();
    let origin_spine_id = spine_id; // Use same spine for simplicity
    let origin_entry = [1u8; 32];

    let result = service
        .anchor_slice(spine_id, slice_id, origin_spine_id, origin_entry)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_generate_inclusion_proof() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Test".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    // Get a valid entry hash from the spine (genesis)
    let spine_result = service.get_spine(spine_id).await;
    if let Ok(Some(spine)) = spine_result
        && let Some(genesis) = spine.genesis_entry()
    {
        let entry_hash = genesis.compute_hash().expect("compute_hash");

        let result = service.generate_inclusion_proof(spine_id, entry_hash).await;
        assert!(result.is_ok());

        if let Ok(proof) = result {
            assert!(proof.verify().expect("verify"));
        }
    }
}

#[tokio::test]
async fn test_record_operation() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Waypoint".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let slice_id = SliceId::now_v7();

    // Anchor a slice first
    service
        .anchor_slice(spine_id, slice_id, spine_id, [1u8; 32])
        .await
        .unwrap_or_else(|_| unreachable!());

    // Record an operation
    let result = service
        .record_operation(spine_id, slice_id, "use".into())
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_depart_slice() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Waypoint".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let slice_id = SliceId::now_v7();

    // Anchor first
    service
        .anchor_slice(spine_id, slice_id, spine_id, [1u8; 32])
        .await
        .unwrap_or_else(|_| unreachable!());

    // Record an operation
    service
        .record_operation(spine_id, slice_id, "view".into())
        .await
        .unwrap_or_else(|_| unreachable!());

    // Depart
    let result = service
        .depart_slice(spine_id, slice_id, DepartureReason::ManualReturn)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_depart_slice_expired() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Waypoint".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let slice_id = SliceId::now_v7();

    service
        .anchor_slice(spine_id, slice_id, spine_id, [1u8; 32])
        .await
        .unwrap_or_else(|_| unreachable!());

    let result = service
        .depart_slice(spine_id, slice_id, DepartureReason::Expired)
        .await;
    assert!(result.is_ok());
}

// ========================================================================
// Attestation enforcement tests
// ========================================================================

#[tokio::test]
async fn attestation_none_operations_succeed() {
    // WaypointConfig::default() has operation_attestation: None
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_spine(owner.clone(), Some("Waypoint".into()))
        .await
        .unwrap_or_else(|_| unreachable!());

    let slice_id = SliceId::now_v7();

    // All operations should succeed without attestation provider
    assert!(
        service
            .anchor_slice(spine_id, slice_id, spine_id, [1u8; 32])
            .await
            .is_ok()
    );
    assert!(
        service
            .record_operation(spine_id, slice_id, "use".into())
            .await
            .is_ok()
    );
    assert!(
        service
            .depart_slice(spine_id, slice_id, DepartureReason::ManualReturn)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn attestation_boundary_only_no_provider_anchor_fails() {
    let service = LoamSpineService::new(); // No attestation provider
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_waypoint_spine(
            owner.clone(),
            Some("Waypoint".into()),
            WaypointConfig {
                operation_attestation: AttestationRequirement::BoundaryOnly,
                ..WaypointConfig::default()
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let slice_id = SliceId::now_v7();

    let result = service
        .anchor_slice(spine_id, slice_id, spine_id, [1u8; 32])
        .await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .to_lowercase()
            .contains("attestation")
    );
}

#[tokio::test]
async fn attestation_boundary_only_with_provider_succeeds() {
    let service = service_with_attestation_provider().await;
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_waypoint_spine(
            owner.clone(),
            Some("Waypoint".into()),
            WaypointConfig {
                operation_attestation: AttestationRequirement::BoundaryOnly,
                ..WaypointConfig::default()
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let slice_id = SliceId::now_v7();

    // Anchor and depart require attestation - should succeed with provider
    assert!(
        service
            .anchor_slice(spine_id, slice_id, spine_id, [1u8; 32])
            .await
            .is_ok()
    );
    assert!(
        service
            .depart_slice(spine_id, slice_id, DepartureReason::ManualReturn)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn attestation_boundary_only_record_operation_no_attestation_needed() {
    // BoundaryOnly: anchor and depart need attestation; "use" does not.
    let service = service_with_attestation_provider().await;
    let owner = Did::new("did:key:z6MkOwner");
    let slice_id = SliceId::now_v7();
    let spine_id = service
        .ensure_waypoint_spine(
            owner.clone(),
            Some("Waypoint".into()),
            WaypointConfig {
                operation_attestation: AttestationRequirement::BoundaryOnly,
                ..WaypointConfig::default()
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    service
        .anchor_slice(spine_id, slice_id, spine_id, [1u8; 32])
        .await
        .unwrap_or_else(|_| unreachable!());

    // "use" does not require attestation for BoundaryOnly
    assert!(
        service
            .record_operation(spine_id, slice_id, "use".into())
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn attestation_all_operations_no_provider_record_fails() {
    let service = LoamSpineService::new();
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_waypoint_spine(
            owner.clone(),
            Some("Waypoint".into()),
            WaypointConfig {
                operation_attestation: AttestationRequirement::AllOperations,
                ..WaypointConfig::default()
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let slice_id = SliceId::now_v7();

    // Anchor and record_operation both need attestation for AllOperations
    assert!(
        service
            .anchor_slice(spine_id, slice_id, spine_id, [1u8; 32])
            .await
            .is_err()
    );
}

#[tokio::test]
async fn attestation_selective_with_provider() {
    let service = service_with_attestation_provider().await;
    let owner = Did::new("did:key:z6MkOwner");

    let spine_id = service
        .ensure_waypoint_spine(
            owner.clone(),
            Some("Waypoint".into()),
            WaypointConfig {
                operation_attestation: AttestationRequirement::Selective {
                    operation_types: vec!["anchor".into(), "export".into()],
                },
                ..WaypointConfig::default()
            },
        )
        .await
        .unwrap_or_else(|_| unreachable!());

    let slice_id = SliceId::now_v7();

    // Anchor and export need attestation; use does not
    assert!(
        service
            .anchor_slice(spine_id, slice_id, spine_id, [1u8; 32])
            .await
            .is_ok()
    );
    assert!(
        service
            .record_operation(spine_id, slice_id, "use".into())
            .await
            .is_ok()
    );
    assert!(
        service
            .record_operation(spine_id, slice_id, "export".into())
            .await
            .is_ok()
    );
    assert!(
        service
            .depart_slice(spine_id, slice_id, DepartureReason::ManualReturn)
            .await
            .is_ok()
    );
}
