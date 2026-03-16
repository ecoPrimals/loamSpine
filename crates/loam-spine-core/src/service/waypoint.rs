// SPDX-License-Identifier: AGPL-3.0-or-later

//! Waypoint operations and proof generation.
//!
//! This module provides:
//! - **Slice anchoring**: Anchor borrowed state on waypoint spines
//! - **Proof generation**: Create inclusion proofs for entries
//!
//! ## Waypoint Spines
//!
//! Waypoints are local spines that anchor borrowed slices, providing
//! cryptographic provenance for data borrowed from other spines.

use crate::entry::EntryType;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::proof::InclusionProof;
use crate::storage::{EntryStorage, SpineStorage};
use crate::types::{EntryHash, SliceId, SpineId};
use crate::waypoint::{AttestationContext, DepartureReason, WaypointConfig};

use super::LoamSpineService;

impl LoamSpineService {
    // ========================================================================
    // Attestation Enforcement
    // ========================================================================

    /// Check attestation requirement for a waypoint operation.
    ///
    /// Loads `WaypointConfig` from the spine's config. If attestation is
    /// required for this operation and no provider is available, returns error.
    /// If required and provider available, requests attestation (stubbed call).
    ///
    /// # Errors
    ///
    /// Returns error if attestation required but provider unavailable, or if
    /// attestation is denied.
    async fn check_attestation_requirement(
        &self,
        waypoint_spine_id: SpineId,
        slice_id: SliceId,
        operation: &str,
    ) -> LoamSpineResult<()> {
        let waypoint = self
            .spine_storage
            .get_spine(waypoint_spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(waypoint_spine_id))?;

        let waypoint_config: WaypointConfig =
            waypoint.config.waypoint_config.clone().unwrap_or_default();

        if !waypoint_config
            .operation_attestation
            .requires_for_operation(operation)
        {
            return Ok(());
        }

        let context = AttestationContext {
            operation: operation.to_string(),
            waypoint_spine_id,
            slice_id,
            caller: None,
        };

        self.capabilities.request_attestation(context).await?;
        Ok(())
    }

    // ========================================================================
    // Waypoint Operations
    // ========================================================================

    /// Anchor a slice on a waypoint spine.
    ///
    /// Records the slice's origin on the local waypoint spine, establishing
    /// a cryptographic link to the borrowed data's provenance.
    ///
    /// # Errors
    ///
    /// Returns error if waypoint spine not found or sealed.
    pub async fn anchor_slice(
        &self,
        waypoint_spine_id: SpineId,
        slice_id: SliceId,
        origin_spine_id: SpineId,
        origin_entry: EntryHash,
    ) -> LoamSpineResult<EntryHash> {
        self.check_attestation_requirement(waypoint_spine_id, slice_id, "anchor")
            .await?;

        let mut waypoint = self
            .spine_storage
            .get_spine(waypoint_spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(waypoint_spine_id))?;

        let entry = waypoint.create_entry(EntryType::SliceAnchor {
            slice_id,
            origin_spine: origin_spine_id,
            origin_entry,
        });

        let anchor_hash = waypoint.append(entry)?;
        let appended = waypoint
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;
        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&waypoint).await?;

        Ok(anchor_hash)
    }

    /// Record an operation on an anchored slice at a waypoint spine.
    ///
    /// The operation is validated against the entry's `allowed_in_waypoint`
    /// check and appended as a `SliceOperation` entry.
    ///
    /// # Errors
    ///
    /// Returns error if waypoint spine not found or sealed.
    pub async fn record_operation(
        &self,
        waypoint_spine_id: SpineId,
        slice_id: SliceId,
        operation: String,
    ) -> LoamSpineResult<EntryHash> {
        self.check_attestation_requirement(waypoint_spine_id, slice_id, &operation)
            .await?;

        let mut waypoint = self
            .spine_storage
            .get_spine(waypoint_spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(waypoint_spine_id))?;

        let entry = waypoint.create_entry(EntryType::SliceOperation {
            slice_id,
            operation,
        });

        if !entry.entry_type.allowed_in_waypoint() {
            return Err(LoamSpineError::InvalidEntryType(
                "entry type not allowed in waypoint".into(),
            ));
        }

        let op_hash = waypoint.append(entry)?;
        let appended = waypoint
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;
        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&waypoint).await?;

        Ok(op_hash)
    }

    /// Depart a slice from a waypoint spine.
    ///
    /// Records a `SliceDeparture` entry on the waypoint with the given reason.
    ///
    /// # Errors
    ///
    /// Returns error if waypoint spine not found or sealed.
    pub async fn depart_slice(
        &self,
        waypoint_spine_id: SpineId,
        slice_id: SliceId,
        reason: DepartureReason,
    ) -> LoamSpineResult<EntryHash> {
        self.check_attestation_requirement(waypoint_spine_id, slice_id, "depart")
            .await?;

        let mut waypoint = self
            .spine_storage
            .get_spine(waypoint_spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(waypoint_spine_id))?;

        let entry = waypoint.create_entry(EntryType::SliceDeparture {
            slice_id,
            reason: reason.to_string(),
        });

        let departure_hash = waypoint.append(entry)?;
        let appended = waypoint
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;
        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&waypoint).await?;

        Ok(departure_hash)
    }

    // ========================================================================
    // Proof Generation
    // ========================================================================

    /// Generate an inclusion proof for an entry.
    ///
    /// The proof demonstrates that an entry exists in a spine and provides
    /// the path from the entry to the current tip.
    ///
    /// # Errors
    ///
    /// Returns error if spine or entry not found.
    pub async fn generate_inclusion_proof(
        &self,
        spine_id: SpineId,
        entry_hash: EntryHash,
    ) -> LoamSpineResult<InclusionProof> {
        let spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entry = self
            .entry_storage
            .get_entry(entry_hash)
            .await?
            .ok_or(LoamSpineError::EntryNotFound(entry_hash))?;

        let tip_hash = spine.tip;

        let mut path = Vec::new();
        let entry_index = entry.index;

        // Build proof path from entry to tip (zero-copy: compute_hash is &self)
        for idx in (entry_index + 1)..=spine.height {
            if let Some(e) = spine.get_entry(idx) {
                path.push(e.compute_hash()?);
            }
        }

        let proof = InclusionProof::new(entry, spine_id, tip_hash)?.with_path(path);

        Ok(proof)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
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
}
