// SPDX-License-Identifier: AGPL-3.0-or-later

//! Attestation provider tests: local DynAttestationProvider implementations,
//! registration/unregistration, attestation request success/denial paths,
//! and DiscoveredAttestationProvider TCP JSON-RPC integration tests.

use std::sync::Arc;

use super::*;

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

// =========================================================================
// DiscoveredAttestationProvider — TCP JSON-RPC integration tests
// =========================================================================

/// Spawn a mock attestation TCP server returning a fixed JSON-RPC response.
async fn spawn_mock_attestation_server(
    response: serde_json::Value,
) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            let _ = reader.read_line(&mut line).await;
            let resp = serde_json::to_string(&response).unwrap() + "\n";
            let _ = stream.write_all(resp.as_bytes()).await;
            let _ = stream.flush().await;
        }
    });
    (addr, handle)
}

#[tokio::test]
async fn discovered_attestation_provider_jsonrpc_call_success() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": { "attested": true, "token": "dGVzdA==" },
        "id": 1
    });
    let (addr, handle) = spawn_mock_attestation_server(response).await;
    let endpoint = format!("{addr}");

    let result = super::DiscoveredAttestationProvider::jsonrpc_call(
        &endpoint,
        "test.method",
        serde_json::json!({}),
    )
    .await;
    handle.abort();

    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val["attested"], true);
}

#[tokio::test]
async fn discovered_attestation_provider_jsonrpc_call_rpc_error() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": -32601, "message": "Method not found" },
        "id": 1
    });
    let (addr, handle) = spawn_mock_attestation_server(response).await;
    let endpoint = format!("{addr}");

    let result = super::DiscoveredAttestationProvider::jsonrpc_call(
        &endpoint,
        "bad.method",
        serde_json::json!({}),
    )
    .await;
    handle.abort();

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Method not found"));
}

#[tokio::test]
async fn discovered_attestation_provider_jsonrpc_call_missing_result() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1
    });
    let (addr, handle) = spawn_mock_attestation_server(response).await;
    let endpoint = format!("{addr}");

    let result = super::DiscoveredAttestationProvider::jsonrpc_call(
        &endpoint,
        "test.method",
        serde_json::json!({}),
    )
    .await;
    handle.abort();

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("missing result"));
}

#[tokio::test]
async fn discovered_attestation_provider_jsonrpc_call_connect_failure() {
    let result = super::DiscoveredAttestationProvider::jsonrpc_call(
        "127.0.0.1:1",
        "test.method",
        serde_json::json!({}),
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn discovered_attestation_provider_jsonrpc_call_invalid_json_response() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            let _ = reader.read_line(&mut line).await;
            let _ = stream.write_all(b"not json at all\n").await;
            let _ = stream.flush().await;
        }
    });
    let endpoint = format!("{addr}");

    let result = super::DiscoveredAttestationProvider::jsonrpc_call(
        &endpoint,
        "test.method",
        serde_json::json!({}),
    )
    .await;
    handle.abort();

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("parse"));
}

#[tokio::test]
async fn discovered_attestation_provider_request_success_via_tcp() {
    use crate::types::Did;
    use crate::waypoint::AttestationContext;

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": { "attested": true, "token": "dG9rZW4=", "denial_reason": null },
        "id": 1
    });
    let (addr, handle) = spawn_mock_attestation_server(response).await;

    let provider = super::DiscoveredAttestationProvider {
        attester_did: Did::new("did:attestation:mock"),
        endpoint: format!("{addr}"),
    };

    let context = AttestationContext {
        operation: "anchor".to_string(),
        waypoint_spine_id: crate::types::SpineId::now_v7(),
        slice_id: crate::types::SliceId::now_v7(),
        caller: Some(Did::new("did:key:test")),
    };

    let result = DynAttestationProvider::request_attestation(&provider, context).await;
    handle.abort();

    assert!(result.is_ok());
    let att = result.unwrap();
    assert!(att.attested);
}

#[tokio::test]
async fn discovered_attestation_provider_fallback_on_unreachable() {
    use crate::types::Did;
    use crate::waypoint::AttestationContext;

    let provider = super::DiscoveredAttestationProvider {
        attester_did: Did::new("did:attestation:unreachable"),
        endpoint: "127.0.0.1:1".to_string(),
    };

    let context = AttestationContext {
        operation: "anchor".to_string(),
        waypoint_spine_id: crate::types::SpineId::now_v7(),
        slice_id: crate::types::SliceId::now_v7(),
        caller: None,
    };

    let result = DynAttestationProvider::request_attestation(&provider, context).await;
    assert!(result.is_ok());
    let att = result.unwrap();
    assert!(
        att.attested,
        "should fallback to local approval on unreachable"
    );
}

#[tokio::test]
async fn discovered_attestation_provider_request_denied() {
    use crate::types::Did;
    use crate::waypoint::AttestationContext;

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": { "attested": false, "denial_reason": "unauthorized", "token": "" },
        "id": 1
    });
    let (addr, handle) = spawn_mock_attestation_server(response).await;

    let provider = super::DiscoveredAttestationProvider {
        attester_did: Did::new("did:attestation:strict"),
        endpoint: format!("{addr}"),
    };

    let context = AttestationContext {
        operation: "depart".to_string(),
        waypoint_spine_id: crate::types::SpineId::now_v7(),
        slice_id: crate::types::SliceId::now_v7(),
        caller: None,
    };

    let result = DynAttestationProvider::request_attestation(&provider, context).await;
    handle.abort();

    assert!(result.is_ok());
    let att = result.unwrap();
    assert!(!att.attested);
    assert_eq!(att.denial_reason.as_deref(), Some("unauthorized"));
}

#[tokio::test]
async fn capability_registry_with_discovered_attestation_via_tcp() {
    use crate::types::Did;
    use crate::waypoint::AttestationContext;

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": { "attested": true, "token": "abc123" },
        "id": 1
    });
    let (addr, handle) = spawn_mock_attestation_server(response).await;

    let provider = Arc::new(super::DiscoveredAttestationProvider {
        attester_did: Did::new("did:attestation:tcp-test"),
        endpoint: format!("{addr}"),
    });

    let registry = CapabilityRegistry::new();
    registry.register_attestation_provider(provider).await;
    assert_eq!(
        registry.attestation_provider_status().await,
        CapabilityStatus::Available
    );

    let context = AttestationContext {
        operation: "anchor".to_string(),
        waypoint_spine_id: crate::types::SpineId::now_v7(),
        slice_id: crate::types::SliceId::now_v7(),
        caller: Some(Did::new("did:key:caller")),
    };
    let result = registry.request_attestation(context).await;
    handle.abort();

    assert!(result.is_ok());
    assert!(result.unwrap().attested);
}
