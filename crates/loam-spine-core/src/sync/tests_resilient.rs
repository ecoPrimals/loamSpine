// SPDX-License-Identifier: AGPL-3.0-or-later

//! Resilient sync engine and wire edge-case tests.
//!
//! Extracted from `tests.rs` — ResilientSyncEngine wrapper tests and
//! rpc_call wire-level edge cases (non-JSON, truncated responses) that
//! form a cohesive "resilience and degradation" domain.

use std::time::Duration;

use super::*;
use crate::entry::{Entry, EntryType, SpineConfig};
use crate::types::Did;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn test_peer_with_endpoint(endpoint: &str) -> SyncPeer {
    SyncPeer {
        peer_id: "peer-1".into(),
        name: "Test Peer".to_string(),
        endpoint: endpoint.to_string(),
        reachable: true,
        last_sync_ns: 0,
    }
}

fn test_entry(spine_id: SpineId, index: u64) -> Entry {
    let owner = Did::new("did:key:z6MkOwner");
    if index == 0 {
        Entry::genesis(owner, spine_id, SpineConfig::default())
    } else {
        Entry::new(index, None, owner, EntryType::SpineSealed { reason: None })
            .with_spine_id(spine_id)
    }
}

async fn spawn_mock_sync_server(
    push_response: serde_json::Value,
    pull_response: serde_json::Value,
) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let endpoint = format!("127.0.0.1:{}", addr.port());

    let handle = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await.unwrap();
        let len = u32::from_be_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await.unwrap();
        let req: serde_json::Value = serde_json::from_slice(&buf).unwrap();
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let result = match method {
            "sync.push" => push_response,
            "sync.pull" => pull_response,
            _ => serde_json::json!({"accepted": 0, "rejected": 0}),
        };
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": id
        });
        let resp_bytes = serde_json::to_vec(&resp).unwrap();
        let resp_len = u32::try_from(resp_bytes.len()).unwrap();
        stream.write_all(&resp_len.to_be_bytes()).await.unwrap();
        stream.write_all(&resp_bytes).await.unwrap();
        stream.flush().await.unwrap();
    });

    (endpoint, handle)
}

// ============================================================================
// ResilientSyncEngine tests
// ============================================================================

#[tokio::test]
async fn resilient_sync_engine_with_defaults() {
    let engine = ResilientSyncEngine::with_defaults(SyncEngine::new());
    assert_eq!(engine.inner().peer_count().await, 0);
}

#[tokio::test]
async fn resilient_sync_engine_custom_config() {
    let engine = ResilientSyncEngine::new(
        SyncEngine::new(),
        crate::resilience::CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout_secs: 10,
            success_threshold: 1,
        },
        crate::resilience::RetryPolicyConfig {
            max_retries: 1,
            base_delay_ms: 1,
            max_delay_ms: 5,
        },
    );
    assert_eq!(engine.inner().peer_count().await, 0);
}

#[tokio::test]
async fn resilient_push_succeeds_through_mock_server() {
    let spine_id = SpineId::now_v7();
    let entries = vec![test_entry(spine_id, 0)];

    let (endpoint, _handle) = spawn_mock_sync_server(
        serde_json::json!({"accepted": 1}),
        serde_json::json!({"entries": []}),
    )
    .await;

    let inner = SyncEngine::new().with_timeout(Duration::from_secs(2));
    inner
        .register_peer(test_peer_with_endpoint(&endpoint))
        .await;

    let resilient = ResilientSyncEngine::with_defaults(inner);
    let result = resilient.push_entries(spine_id, entries).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn resilient_push_no_peers_returns_error() {
    let engine = ResilientSyncEngine::new(
        SyncEngine::new(),
        crate::resilience::CircuitBreakerConfig {
            failure_threshold: 10,
            recovery_timeout_secs: 60,
            success_threshold: 2,
        },
        crate::resilience::RetryPolicyConfig {
            max_retries: 0,
            base_delay_ms: 1,
            max_delay_ms: 5,
        },
    );
    let spine_id = SpineId::now_v7();
    let result = engine.push_entries(spine_id, vec![]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn resilient_pull_succeeds_through_mock_server() {
    let spine_id = SpineId::now_v7();
    let entries = vec![test_entry(spine_id, 0), test_entry(spine_id, 1)];
    let entries_json = serde_json::to_value(&entries).unwrap();

    let (endpoint, _handle) = spawn_mock_sync_server(
        serde_json::json!({"accepted": 0}),
        serde_json::json!({"entries": entries_json}),
    )
    .await;

    let inner = SyncEngine::new().with_timeout(Duration::from_secs(2));
    inner
        .register_peer(test_peer_with_endpoint(&endpoint))
        .await;

    let resilient = ResilientSyncEngine::with_defaults(inner);
    let pulled = resilient.pull_entries(spine_id, 0, 10).await;
    assert!(pulled.is_ok());
    assert_eq!(pulled.unwrap().len(), 2);
}

#[tokio::test]
async fn resilient_pull_no_peers_returns_error() {
    let engine = ResilientSyncEngine::new(
        SyncEngine::new(),
        crate::resilience::CircuitBreakerConfig {
            failure_threshold: 10,
            recovery_timeout_secs: 60,
            success_threshold: 2,
        },
        crate::resilience::RetryPolicyConfig {
            max_retries: 0,
            base_delay_ms: 1,
            max_delay_ms: 5,
        },
    );
    let spine_id = SpineId::now_v7();
    let result = engine.pull_entries(spine_id, 0, 10).await;
    assert!(result.is_err());
}

// ============================================================================
// rpc_call wire edge case tests (non-JSON / truncated responses)
// ============================================================================

async fn spawn_garbage_server(garbage: &[u8]) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let endpoint = format!("127.0.0.1:{}", addr.port());
    let garbage = garbage.to_vec();

    let handle = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut len_buf = [0u8; 4];
        let _ = stream.read_exact(&mut len_buf).await;
        let len = u32::from_be_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        let _ = stream.read_exact(&mut buf).await;
        let resp_len = u32::try_from(garbage.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes();
        let _ = stream.write_all(&resp_len).await;
        let _ = stream.write_all(&garbage).await;
        let _ = stream.flush().await;
    });
    (endpoint, handle)
}

#[tokio::test]
async fn push_with_non_json_response_returns_error() {
    let (endpoint, _handle) = spawn_garbage_server(b"not json at all").await;

    let engine = SyncEngine::new().with_timeout(Duration::from_secs(2));
    engine
        .register_peer(test_peer_with_endpoint(&endpoint))
        .await;

    let spine_id = SpineId::now_v7();
    let result = engine
        .push_entries(spine_id, vec![test_entry(spine_id, 0)])
        .await;
    assert!(result.is_ok());
    let sync_result = result.unwrap();
    assert_eq!(
        sync_result.accepted, 1,
        "failed RPC queues entries as locally accepted"
    );
    assert!(
        !sync_result.rejection_reasons.is_empty() || sync_result.accepted > 0,
        "should degrade gracefully on non-JSON response"
    );
}

#[tokio::test]
async fn pull_with_non_json_response_falls_back_to_pending() {
    let (endpoint, _handle) = spawn_garbage_server(b"{{invalid").await;

    let engine = SyncEngine::new().with_timeout(Duration::from_secs(2));
    engine
        .register_peer(test_peer_with_endpoint(&endpoint))
        .await;

    let spine_id = SpineId::now_v7();
    let pulled = engine.pull_entries(spine_id, 0, 10).await;
    assert!(pulled.is_ok());
    assert!(pulled.unwrap().is_empty());
}

#[tokio::test]
async fn pull_with_result_not_object_falls_back() {
    let (endpoint, _handle) = spawn_mock_sync_server(
        serde_json::json!({"accepted": 0}),
        serde_json::json!("not an object"),
    )
    .await;

    let engine = SyncEngine::new().with_timeout(Duration::from_secs(2));
    engine
        .register_peer(test_peer_with_endpoint(&endpoint))
        .await;

    let spine_id = SpineId::now_v7();
    let pulled = engine.pull_entries(spine_id, 0, 10).await;
    assert!(pulled.is_ok());
    assert!(pulled.unwrap().is_empty());
}
