// SPDX-License-Identifier: AGPL-3.0-only

use std::time::Duration;

use super::*;
use crate::entry::{Entry, EntryType, SpineConfig};
use crate::types::Did;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn test_peer() -> SyncPeer {
    SyncPeer {
        peer_id: "peer-1".to_string(),
        name: "Test Peer".to_string(),
        endpoint: "127.0.0.1:1".to_string(),
        reachable: true,
        last_sync_ns: 0,
    }
}

fn test_peer_with_endpoint(endpoint: &str) -> SyncPeer {
    SyncPeer {
        peer_id: "peer-1".to_string(),
        name: "Test Peer".to_string(),
        endpoint: endpoint.to_string(),
        reachable: true,
        last_sync_ns: 0,
    }
}

fn test_peer_unreachable(endpoint: &str) -> SyncPeer {
    SyncPeer {
        peer_id: "peer-2".to_string(),
        name: "Unreachable Peer".to_string(),
        endpoint: endpoint.to_string(),
        reachable: false,
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

async fn spawn_mock_server_pull_missing_entries() -> (String, tokio::task::JoinHandle<()>) {
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
        let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {},
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

async fn spawn_mock_server_jsonrpc_error() -> (String, tokio::task::JoinHandle<()>) {
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
        let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32000, "message": "sync peer rejected"},
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

#[tokio::test]
async fn sync_engine_peer_management() {
    let engine = SyncEngine::new();
    assert_eq!(engine.peer_count().await, 0);

    engine.register_peer(test_peer()).await;
    assert_eq!(engine.peer_count().await, 1);

    let peers = engine.list_peers().await;
    assert_eq!(peers[0].peer_id, "peer-1");

    engine.remove_peer("peer-1").await;
    assert_eq!(engine.peer_count().await, 0);
}

#[tokio::test]
async fn sync_status_unknown_for_new_spine() {
    let engine = SyncEngine::new();
    let spine_id = SpineId::now_v7();
    let status = engine.get_sync_status(spine_id).await.unwrap();
    assert_eq!(status, SyncStatus::Unknown);
}

#[tokio::test]
async fn sync_status_tracks_local_height() {
    let engine = SyncEngine::new();
    let spine_id = SpineId::now_v7();

    engine.notify_local_append(spine_id, 5).await;
    let status = engine.get_sync_status(spine_id).await.unwrap();
    assert_eq!(status, SyncStatus::LocalAhead { entries_ahead: 5 });
}

#[tokio::test]
async fn push_entries_requires_peers() {
    let engine = SyncEngine::new();
    let spine_id = SpineId::now_v7();
    let result = engine.push_entries(spine_id, vec![]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn push_entries_with_peer_succeeds() {
    let engine = SyncEngine::new();
    engine.register_peer(test_peer()).await;

    let spine_id = SpineId::now_v7();
    let entries = vec![test_entry(spine_id, 0), test_entry(spine_id, 1)];
    let result = engine.push_entries(spine_id, entries).await.unwrap();
    assert_eq!(result.accepted, 2);
    assert_eq!(result.rejected, 0);
}

#[tokio::test]
async fn pull_entries_returns_queued() {
    let engine = SyncEngine::new();
    engine.register_peer(test_peer()).await;

    let spine_id = SpineId::now_v7();
    let entries = vec![test_entry(spine_id, 0), test_entry(spine_id, 1)];
    engine.push_entries(spine_id, entries).await.unwrap();

    let pulled = engine.pull_entries(spine_id, 0, 10).await.unwrap();
    assert_eq!(pulled.len(), 2);
}

#[tokio::test]
async fn default_engine_is_empty() {
    let engine = SyncEngine::default();
    assert_eq!(engine.peer_count().await, 0);
}

#[tokio::test]
async fn best_peer_endpoint_empty_peers() {
    let engine = SyncEngine::new();
    let result = engine.push_entries(SpineId::now_v7(), vec![]).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("no federation peers")
    );
}

#[tokio::test]
async fn best_peer_endpoint_prefers_reachable() {
    let engine = SyncEngine::new();
    engine
        .register_peer(test_peer_unreachable("127.0.0.1:2"))
        .await;
    engine
        .register_peer(test_peer_with_endpoint("127.0.0.1:3"))
        .await;
    let spine_id = SpineId::now_v7();
    let entries = vec![test_entry(spine_id, 0)];
    let result = engine.push_entries(spine_id, entries).await.unwrap();
    assert_eq!(result.accepted, 1);
}

#[tokio::test]
async fn best_peer_endpoint_falls_back_to_unreachable() {
    let engine = SyncEngine::new();
    engine
        .register_peer(test_peer_unreachable("127.0.0.1:1"))
        .await;
    let spine_id = SpineId::now_v7();
    let entries = vec![test_entry(spine_id, 0)];
    let result = engine.push_entries(spine_id, entries).await.unwrap();
    assert_eq!(result.accepted, 1);
}

#[tokio::test]
async fn sync_status_in_sync() {
    let engine = SyncEngine::new();
    let spine_id = SpineId::now_v7();
    engine.notify_local_append(spine_id, 3).await;
    engine.set_remote_height_for_test(spine_id, 3).await;
    let status = engine.get_sync_status(spine_id).await.unwrap();
    assert_eq!(status, SyncStatus::InSync);
}

#[tokio::test]
async fn sync_status_remote_ahead() {
    let engine = SyncEngine::new();
    let spine_id = SpineId::now_v7();
    engine.notify_local_append(spine_id, 2).await;
    engine.set_remote_height_for_test(spine_id, 5).await;
    let status = engine.get_sync_status(spine_id).await.unwrap();
    assert_eq!(status, SyncStatus::RemoteAhead { entries_behind: 3 });
}

#[tokio::test]
async fn pull_entries_requires_peers() {
    let engine = SyncEngine::new();
    let spine_id = SpineId::now_v7();
    let result = engine.pull_entries(spine_id, 0, 10).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("no federation peers")
    );
}

#[tokio::test]
async fn pull_entries_fallback_filters_by_from_index() {
    let engine = SyncEngine::new();
    engine.register_peer(test_peer()).await;
    let spine_id = SpineId::now_v7();
    let entries = vec![
        test_entry(spine_id, 0),
        test_entry(spine_id, 1),
        test_entry(spine_id, 2),
    ];
    engine.push_entries(spine_id, entries).await.unwrap();
    let pulled = engine.pull_entries(spine_id, 1, 10).await.unwrap();
    assert_eq!(pulled.len(), 2);
    assert_eq!(pulled[0].index, 1);
    assert_eq!(pulled[1].index, 2);
}

#[tokio::test]
async fn pull_entries_fallback_respects_limit() {
    let engine = SyncEngine::new();
    engine.register_peer(test_peer()).await;
    let spine_id = SpineId::now_v7();
    let entries = vec![
        test_entry(spine_id, 0),
        test_entry(spine_id, 1),
        test_entry(spine_id, 2),
    ];
    engine.push_entries(spine_id, entries).await.unwrap();
    let pulled = engine.pull_entries(spine_id, 0, 2).await.unwrap();
    assert_eq!(pulled.len(), 2);
}

#[tokio::test]
async fn pull_entries_fallback_empty_when_no_state() {
    let engine = SyncEngine::new();
    engine.register_peer(test_peer()).await;
    let spine_id = SpineId::now_v7();
    let pulled = engine.pull_entries(spine_id, 0, 10).await.unwrap();
    assert!(pulled.is_empty());
}

#[tokio::test]
async fn pull_entries_fallback_with_large_limit() {
    let engine = SyncEngine::new();
    engine.register_peer(test_peer()).await;
    let spine_id = SpineId::now_v7();
    let entries = vec![test_entry(spine_id, 0), test_entry(spine_id, 1)];
    engine.push_entries(spine_id, entries).await.unwrap();
    let pulled = engine.pull_entries(spine_id, 0, u64::MAX).await.unwrap();
    assert_eq!(pulled.len(), 2);
}

#[tokio::test]
async fn push_to_peer_success_via_mock_server() {
    let (endpoint, _handle) = spawn_mock_sync_server(
        serde_json::json!({"accepted": 3, "rejected": 0}),
        serde_json::json!({"entries": []}),
    )
    .await;

    let engine = SyncEngine::new();
    engine
        .register_peer(test_peer_with_endpoint(&endpoint))
        .await;

    let spine_id = SpineId::now_v7();
    let entries = vec![
        test_entry(spine_id, 0),
        test_entry(spine_id, 1),
        test_entry(spine_id, 2),
    ];
    let result = engine.push_entries(spine_id, entries).await.unwrap();
    assert_eq!(result.accepted, 3);
    assert_eq!(result.rejected, 0);
}

#[tokio::test]
async fn pull_from_peer_success_via_mock_server() {
    let spine_id = SpineId::now_v7();
    let entries = vec![test_entry(spine_id, 0), test_entry(spine_id, 1)];
    let entries_json = serde_json::to_value(&entries).unwrap();

    let (endpoint, _handle) = spawn_mock_sync_server(
        serde_json::json!({"accepted": 0, "rejected": 0}),
        serde_json::json!({"entries": entries_json}),
    )
    .await;

    let engine = SyncEngine::new();
    engine
        .register_peer(test_peer_with_endpoint(&endpoint))
        .await;

    let pulled = engine.pull_entries(spine_id, 0, 10).await.unwrap();
    assert_eq!(pulled.len(), 2);
    assert_eq!(pulled[0].index, 0);
    assert_eq!(pulled[1].index, 1);
}

#[tokio::test]
async fn rpc_call_connection_timeout() {
    let engine = SyncEngine::new().with_timeout(Duration::from_millis(1));
    engine
        .register_peer(test_peer_with_endpoint("192.0.2.1:9"))
        .await;
    let spine_id = SpineId::now_v7();
    let result = engine
        .push_entries(spine_id, vec![test_entry(spine_id, 0)])
        .await;
    assert!(result.is_ok());
    let r = result.unwrap();
    assert_eq!(r.accepted, 1);
}

#[tokio::test]
async fn with_timeout_builder() {
    let engine = SyncEngine::new().with_timeout(Duration::from_secs(2));
    engine.register_peer(test_peer()).await;
    let spine_id = SpineId::now_v7();
    let result = engine
        .push_entries(spine_id, vec![test_entry(spine_id, 0)])
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn push_entries_fallback_on_network_failure() {
    let engine = SyncEngine::new();
    engine.register_peer(test_peer()).await;
    let spine_id = SpineId::now_v7();
    let entries = vec![
        test_entry(spine_id, 0),
        test_entry(spine_id, 1),
        test_entry(spine_id, 2),
    ];
    let result = engine.push_entries(spine_id, entries).await.unwrap();
    assert_eq!(result.accepted, 3);
    assert_eq!(result.rejected, 0);
}

#[tokio::test]
async fn push_fallback_when_peer_returns_jsonrpc_error() {
    let (endpoint, _handle) = spawn_mock_server_jsonrpc_error().await;

    let engine = SyncEngine::new();
    engine
        .register_peer(test_peer_with_endpoint(&endpoint))
        .await;

    let spine_id = SpineId::now_v7();
    let entries = vec![test_entry(spine_id, 0), test_entry(spine_id, 1)];
    let result = engine.push_entries(spine_id, entries).await.unwrap();
    assert_eq!(result.accepted, 2);
}

#[tokio::test]
async fn pull_fallback_when_peer_returns_jsonrpc_error() {
    let (endpoint, _handle) = spawn_mock_server_jsonrpc_error().await;

    let engine = SyncEngine::new();
    engine
        .register_peer(test_peer_with_endpoint(&endpoint))
        .await;

    let spine_id = SpineId::now_v7();
    engine
        .push_entries(spine_id, vec![test_entry(spine_id, 0)])
        .await
        .unwrap();

    let pulled = engine.pull_entries(spine_id, 0, 10).await.unwrap();
    assert_eq!(pulled.len(), 1);
    assert_eq!(pulled[0].index, 0);
}

#[tokio::test]
async fn pull_fallback_when_peer_returns_missing_entries() {
    let (endpoint, _handle) = spawn_mock_server_pull_missing_entries().await;

    let engine = SyncEngine::new();
    engine
        .register_peer(test_peer_with_endpoint(&endpoint))
        .await;

    let spine_id = SpineId::now_v7();
    engine
        .push_entries(spine_id, vec![test_entry(spine_id, 0)])
        .await
        .unwrap();

    let pulled = engine.pull_entries(spine_id, 0, 10).await.unwrap();
    assert_eq!(pulled.len(), 1);
}

#[tokio::test]
async fn push_to_peer_parses_rejected_from_server() {
    let (endpoint, _handle) = spawn_mock_sync_server(
        serde_json::json!({"accepted": 1, "rejected": 2}),
        serde_json::json!({"entries": []}),
    )
    .await;

    let engine = SyncEngine::new();
    engine
        .register_peer(test_peer_with_endpoint(&endpoint))
        .await;

    let spine_id = SpineId::now_v7();
    let entries = vec![
        test_entry(spine_id, 0),
        test_entry(spine_id, 1),
        test_entry(spine_id, 2),
    ];
    let result = engine.push_entries(spine_id, entries).await.unwrap();
    assert_eq!(result.accepted, 1);
    assert_eq!(result.rejected, 2);
}
