// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use super::*;

#[tokio::test]
async fn empty_registry() {
    let registry = CapabilityRegistry::new();

    assert_eq!(
        registry.signer_status().await,
        CapabilityStatus::Unavailable
    );
    assert_eq!(
        registry.verifier_status().await,
        CapabilityStatus::Unavailable
    );
    assert!(registry.get_signer().await.is_none());
    assert!(registry.get_verifier().await.is_none());
}

#[tokio::test]
async fn register_signer() {
    use crate::traits::signing::testing::MockSigner;
    use crate::types::Did;

    let registry = CapabilityRegistry::new();
    let signer = Arc::new(MockSigner::new(Did::new("did:key:test")));

    registry.register_signer(signer).await;

    assert_eq!(registry.signer_status().await, CapabilityStatus::Available);
    assert!(registry.get_signer().await.is_some());
}

#[tokio::test]
async fn register_verifier() {
    use crate::traits::signing::testing::MockVerifier;

    let registry = CapabilityRegistry::new();
    let verifier = Arc::new(MockVerifier::permissive());

    registry.register_verifier(verifier).await;

    assert_eq!(
        registry.verifier_status().await,
        CapabilityStatus::Available
    );
    assert!(registry.get_verifier().await.is_some());
}

#[tokio::test]
async fn register_and_unregister() {
    use crate::traits::signing::testing::MockSigner;
    use crate::types::Did;

    let registry = CapabilityRegistry::new();
    let signer = Arc::new(MockSigner::new(Did::new("did:key:test")));

    registry.register_signer(signer).await;
    assert_eq!(registry.signer_status().await, CapabilityStatus::Available);

    registry.unregister_signer().await;
    assert_eq!(
        registry.signer_status().await,
        CapabilityStatus::Unavailable
    );
}

#[tokio::test]
async fn unregister_verifier() {
    use crate::traits::signing::testing::MockVerifier;

    let registry = CapabilityRegistry::new();
    let verifier = Arc::new(MockVerifier::permissive());

    registry.register_verifier(verifier).await;
    assert_eq!(
        registry.verifier_status().await,
        CapabilityStatus::Available
    );

    registry.unregister_verifier().await;
    assert_eq!(
        registry.verifier_status().await,
        CapabilityStatus::Unavailable
    );
}

#[tokio::test]
async fn require_missing_capability() {
    let registry = CapabilityRegistry::new();

    let result = registry.require_signer().await;
    assert!(result.is_err());

    let result = registry.require_verifier().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn require_registered_capability() {
    use crate::traits::signing::testing::{MockSigner, MockVerifier};
    use crate::types::Did;

    let registry = CapabilityRegistry::new();
    let signer = Arc::new(MockSigner::new(Did::new("did:key:test")));
    let verifier = Arc::new(MockVerifier::permissive());

    registry.register_signer(signer).await;
    registry.register_verifier(verifier).await;

    assert!(registry.require_signer().await.is_ok());
    assert!(registry.require_verifier().await.is_ok());
}

#[tokio::test]
async fn all_statuses() {
    use crate::traits::signing::testing::MockSigner;
    use crate::types::Did;

    let registry = CapabilityRegistry::new();
    let statuses = registry.all_statuses().await;
    assert_eq!(statuses.len(), 3);

    let signer = Arc::new(MockSigner::new(Did::new("did:key:test")));
    registry.register_signer(signer).await;

    let statuses = registry.all_statuses().await;
    let signer_status = statuses.iter().find(|(name, _)| *name == "Signer");
    assert_eq!(
        signer_status.map(|(_, s)| s),
        Some(&CapabilityStatus::Available)
    );
}

#[tokio::test]
async fn all_required_available() {
    let registry = CapabilityRegistry::new();
    assert!(registry.all_required_available().await);
}

#[test]
fn registry_debug() {
    let registry = CapabilityRegistry::new();
    let debug = format!("{registry:?}");
    assert!(debug.contains("CapabilityRegistry"));
}

#[test]
fn capability_status_equality() {
    assert_eq!(CapabilityStatus::Available, CapabilityStatus::Available);
    assert_eq!(CapabilityStatus::Unavailable, CapabilityStatus::Unavailable);
    assert_eq!(
        CapabilityStatus::Degraded {
            reason: "test".into()
        },
        CapabilityStatus::Degraded {
            reason: "test".into()
        }
    );
    assert_ne!(CapabilityStatus::Available, CapabilityStatus::Unavailable);
}

#[tokio::test]
async fn dyn_signer_sign_boxed() {
    use crate::traits::signing::testing::MockSigner;
    use crate::types::Did;

    let did = Did::new("did:key:test");
    let signer = MockSigner::new(did.clone());

    let boxed: BoxedSigner = Arc::new(signer);

    let data = crate::types::ByteBuffer::from_static(b"test data");
    let sig = boxed.sign_boxed(data).await;
    assert!(sig.is_ok());

    assert_eq!(boxed.did(), &did);
}

#[tokio::test]
async fn dyn_verifier_verify_boxed() {
    use crate::traits::signing::testing::MockVerifier;
    use crate::types::{Did, Signature};

    let verifier = MockVerifier::permissive();
    let boxed: BoxedVerifier = Arc::new(verifier);

    let data = crate::types::ByteBuffer::from_static(b"test data");
    let sig = Signature::from_vec(vec![1, 2, 3]);
    let did = Did::new("did:key:test");

    let result = boxed.verify_boxed(data, sig, did).await;
    assert!(result.is_ok());
    assert!(result.unwrap_or_else(|_| unreachable!()).valid);
}

#[tokio::test]
async fn dyn_verifier_verify_entry_boxed() {
    use crate::entry::{Entry, EntryType};
    use crate::traits::signing::testing::MockVerifier;
    use crate::types::Did;

    let verifier = MockVerifier::permissive();
    let boxed: BoxedVerifier = Arc::new(verifier);

    let entry = Entry::new(
        0,
        None,
        Did::new("did:test"),
        EntryType::SpineSealed { reason: None },
    );

    let result = boxed.verify_entry_boxed(entry).await;
    assert!(result.is_ok());
    assert!(result.unwrap_or_else(|_| unreachable!()).valid);
}

#[tokio::test]
async fn dyn_verifier_strict_fails() {
    use crate::traits::signing::testing::MockVerifier;
    use crate::types::{Did, Signature};

    let verifier = MockVerifier::strict();
    let boxed: BoxedVerifier = Arc::new(verifier);

    let data = crate::types::ByteBuffer::from_static(b"test data");
    let sig = Signature::from_vec(vec![1, 2, 3]);
    let did = Did::new("did:key:test");

    let result = boxed.verify_boxed(data, sig, did).await;
    assert!(result.is_ok());
    assert!(!result.unwrap_or_else(|_| unreachable!()).valid);
}

#[test]
fn capability_status_debug_clone() {
    let status = CapabilityStatus::Degraded {
        reason: "test".into(),
    };
    let debug_str = format!("{status:?}");
    assert!(debug_str.contains("Degraded"));

    let cloned = status.clone();
    assert_eq!(status, cloned);
}

#[tokio::test]
async fn registry_clone() {
    use crate::traits::signing::testing::MockSigner;
    use crate::types::Did;

    let registry = CapabilityRegistry::new();
    let signer = Arc::new(MockSigner::new(Did::new("did:key:test")));
    registry.register_signer(signer).await;

    let cloned = registry.clone();

    assert!(registry.get_signer().await.is_some());
    assert!(cloned.get_signer().await.is_some());
}

#[tokio::test]
async fn discover_from_registry_fails_when_not_configured() {
    let registry = CapabilityRegistry::new();

    let result = registry.discover_from_registry().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("registry") || err.to_string().contains("unavailable"));
}

#[tokio::test]
async fn advertise_to_registry_fails_when_not_configured() {
    let registry = CapabilityRegistry::new();

    let result = registry
        .advertise_to_registry("http://localhost:9001", "http://localhost:8080")
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("registry") || err.to_string().contains("unavailable"));
}

#[tokio::test]
async fn heartbeat_registry_fails_when_not_configured() {
    let registry = CapabilityRegistry::new();

    let result = registry.heartbeat_registry().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("registry") || err.to_string().contains("unavailable"));
}

#[tokio::test]
async fn with_service_registry_fails_for_unreachable_endpoint() {
    let result = CapabilityRegistry::with_service_registry("http://127.0.0.1:1").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("unavailable")
            || err.to_string().contains("registry")
            || err.to_string().contains("transport"),
        "Expected connection/transport error: {err}",
    );
}

#[tokio::test]
#[allow(deprecated)]
async fn deprecated_songbird_aliases_work() {
    let registry = CapabilityRegistry::new();
    assert!(registry.discover_from_songbird().await.is_err());
    assert!(registry.heartbeat_songbird().await.is_err());
    assert!(
        registry
            .advertise_to_songbird("http://localhost:9001", "http://localhost:8080")
            .await
            .is_err()
    );
    assert!(
        CapabilityRegistry::with_service_registry("http://127.0.0.1:1")
            .await
            .is_err()
    );
}

#[test]
fn capability_status_degraded_variant() {
    let degraded = CapabilityStatus::Degraded {
        reason: "heartbeat failed".to_string(),
    };
    assert!(matches!(degraded, CapabilityStatus::Degraded { .. }));
    assert_eq!(
        degraded,
        CapabilityStatus::Degraded {
            reason: "heartbeat failed".to_string(),
        }
    );
}

// =========================================================================
// Attestation provider coverage
// =========================================================================

#[tokio::test]
async fn register_attestation_provider() {
    use crate::types::{Did, Timestamp};
    use crate::waypoint::{AttestationContext, AttestationResult};

    struct TestProvider;
    impl DynAttestationProvider for TestProvider {
        fn request_attestation(
            &self,
            _context: AttestationContext,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = crate::error::LoamSpineResult<AttestationResult>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async {
                Ok(AttestationResult {
                    attested: true,
                    attester: Did::new("did:attestation:test"),
                    timestamp: Timestamp::now(),
                    token: vec![1, 2, 3],
                    denial_reason: None,
                })
            })
        }
    }

    let registry = CapabilityRegistry::new();
    assert_eq!(
        registry.attestation_provider_status().await,
        CapabilityStatus::Unavailable
    );
    assert!(registry.get_attestation_provider().await.is_none());

    registry
        .register_attestation_provider(Arc::new(TestProvider))
        .await;
    assert_eq!(
        registry.attestation_provider_status().await,
        CapabilityStatus::Available
    );
    assert!(registry.get_attestation_provider().await.is_some());
}

#[tokio::test]
async fn unregister_attestation_provider() {
    use crate::types::{Did, Timestamp};
    use crate::waypoint::{AttestationContext, AttestationResult};

    struct TestProvider;
    impl DynAttestationProvider for TestProvider {
        fn request_attestation(
            &self,
            _context: AttestationContext,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = crate::error::LoamSpineResult<AttestationResult>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async {
                Ok(AttestationResult {
                    attested: true,
                    attester: Did::new("did:attestation:test"),
                    timestamp: Timestamp::now(),
                    token: vec![],
                    denial_reason: None,
                })
            })
        }
    }

    let registry = CapabilityRegistry::new();
    registry
        .register_attestation_provider(Arc::new(TestProvider))
        .await;
    assert_eq!(
        registry.attestation_provider_status().await,
        CapabilityStatus::Available
    );

    registry.unregister_attestation_provider().await;
    assert_eq!(
        registry.attestation_provider_status().await,
        CapabilityStatus::Unavailable
    );
    assert!(registry.get_attestation_provider().await.is_none());
}

#[tokio::test]
async fn request_attestation_no_provider() {
    let registry = CapabilityRegistry::new();

    let context = crate::waypoint::AttestationContext {
        operation: "anchor".to_string(),
        waypoint_spine_id: crate::types::SpineId::now_v7(),
        slice_id: crate::types::SliceId::now_v7(),
        caller: None,
    };
    let result = registry.request_attestation(context).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Attestation")
            || err.to_string().contains("attestation")
            || err.to_string().contains("unavailable"),
        "Expected attestation unavailable error: {err}",
    );
}

#[tokio::test]
async fn request_attestation_success() {
    use crate::types::{Did, Timestamp};
    use crate::waypoint::{AttestationContext, AttestationResult};

    struct ApproveProvider;
    impl DynAttestationProvider for ApproveProvider {
        fn request_attestation(
            &self,
            _context: AttestationContext,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = crate::error::LoamSpineResult<AttestationResult>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async {
                Ok(AttestationResult {
                    attested: true,
                    attester: Did::new("did:attestation:approved"),
                    timestamp: Timestamp::now(),
                    token: vec![42],
                    denial_reason: None,
                })
            })
        }
    }

    let registry = CapabilityRegistry::new();
    registry
        .register_attestation_provider(Arc::new(ApproveProvider))
        .await;

    let context = AttestationContext {
        operation: "anchor".to_string(),
        waypoint_spine_id: crate::types::SpineId::now_v7(),
        slice_id: crate::types::SliceId::now_v7(),
        caller: Some(Did::new("did:key:caller")),
    };
    let result = registry.request_attestation(context).await;
    assert!(result.is_ok());
    let att = result.unwrap();
    assert!(att.attested);
    assert_eq!(att.token, vec![42]);
}

#[tokio::test]
async fn request_attestation_denied_with_reason() {
    use crate::types::{Did, Timestamp};
    use crate::waypoint::{AttestationContext, AttestationResult};

    struct DenyProvider;
    impl DynAttestationProvider for DenyProvider {
        fn request_attestation(
            &self,
            _context: AttestationContext,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = crate::error::LoamSpineResult<AttestationResult>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async {
                Ok(AttestationResult {
                    attested: false,
                    attester: Did::new("did:attestation:denier"),
                    timestamp: Timestamp::now(),
                    token: vec![],
                    denial_reason: Some("policy violation".to_string()),
                })
            })
        }
    }

    let registry = CapabilityRegistry::new();
    registry
        .register_attestation_provider(Arc::new(DenyProvider))
        .await;

    let context = AttestationContext {
        operation: "depart".to_string(),
        waypoint_spine_id: crate::types::SpineId::now_v7(),
        slice_id: crate::types::SliceId::now_v7(),
        caller: None,
    };
    let result = registry.request_attestation(context).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("policy violation"),
        "Expected denial reason in error: {err}",
    );
}

#[tokio::test]
async fn request_attestation_denied_no_reason() {
    use crate::types::{Did, Timestamp};
    use crate::waypoint::{AttestationContext, AttestationResult};

    struct DenyNoReasonProvider;
    impl DynAttestationProvider for DenyNoReasonProvider {
        fn request_attestation(
            &self,
            _context: AttestationContext,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = crate::error::LoamSpineResult<AttestationResult>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async {
                Ok(AttestationResult {
                    attested: false,
                    attester: Did::new("did:attestation:denier"),
                    timestamp: Timestamp::now(),
                    token: vec![],
                    denial_reason: None,
                })
            })
        }
    }

    let registry = CapabilityRegistry::new();
    registry
        .register_attestation_provider(Arc::new(DenyNoReasonProvider))
        .await;

    let context = AttestationContext {
        operation: "use".to_string(),
        waypoint_spine_id: crate::types::SpineId::now_v7(),
        slice_id: crate::types::SliceId::now_v7(),
        caller: None,
    };
    let result = registry.request_attestation(context).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("denied") || err.to_string().contains("Denied"),
        "Expected denial fallback in error: {err}",
    );
}

#[tokio::test]
async fn request_attestation_provider_returns_error() {
    use crate::waypoint::{AttestationContext, AttestationResult};

    struct ErrorProvider;
    impl DynAttestationProvider for ErrorProvider {
        fn request_attestation(
            &self,
            _context: AttestationContext,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = crate::error::LoamSpineResult<AttestationResult>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async {
                Err(crate::error::LoamSpineError::Internal(
                    "attestation service unavailable".into(),
                ))
            })
        }
    }

    let registry = CapabilityRegistry::new();
    registry
        .register_attestation_provider(Arc::new(ErrorProvider))
        .await;

    let context = AttestationContext {
        operation: "anchor".to_string(),
        waypoint_spine_id: crate::types::SpineId::now_v7(),
        slice_id: crate::types::SliceId::now_v7(),
        caller: None,
    };
    let result = registry.request_attestation(context).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("unavailable"));
}

#[tokio::test]
async fn all_statuses_includes_attestation() {
    let registry = CapabilityRegistry::new();
    let statuses = registry.all_statuses().await;

    let attestation_status = statuses.iter().find(|(name, _)| *name == "Attestation");
    assert!(attestation_status.is_some());
    assert_eq!(
        attestation_status.map(|(_, s)| s),
        Some(&CapabilityStatus::Unavailable)
    );
}
