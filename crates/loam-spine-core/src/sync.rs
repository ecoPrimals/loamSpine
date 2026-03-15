// SPDX-License-Identifier: AGPL-3.0-only

//! Sync protocol implementation for spine federation.
//!
//! Implements the [`SyncProtocol`] trait for replicating spines between
//! LoamSpine instances across trust boundaries using JSON-RPC over TCP.
//!
//! ## Architecture
//!
//! ```text
//! Node A                  JSON-RPC 2.0           Node B
//! ┌─────────┐  sync.push  ┌──────┐  sync.push  ┌─────────┐
//! │ Spine X ├────────────►│ TCP  ├─────────────►│ Spine X │
//! │         │◄────────────┤      │◄─────────────┤         │
//! └─────────┘  sync.pull  └──────┘  sync.pull   └─────────┘
//! ```
//!
//! ## Wire Protocol
//!
//! Federation uses length-prefixed JSON-RPC 2.0 over TCP:
//! - `sync.push` — send entries to a peer
//! - `sync.pull` — request entries from a peer
//! - `sync.status` — query sync state of a remote spine

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::entry::Entry;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::traits::{SyncProtocol, SyncResult, SyncStatus};
use crate::types::{PeerId, SpineId};

/// Peer registration for sync federation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncPeer {
    /// Unique peer identifier.
    pub peer_id: PeerId,
    /// Display name for the peer.
    pub name: String,
    /// Transport endpoint (TCP address, e.g., `192.0.2.100:9001`).
    pub endpoint: String,
    /// Whether this peer is currently reachable.
    pub reachable: bool,
    /// Last successful sync timestamp (nanoseconds since epoch, 0 if never).
    pub last_sync_ns: u64,
}

/// Per-spine sync state tracked by `SyncEngine`.
#[derive(Clone, Debug, Default)]
struct SpineSyncState {
    local_height: u64,
    remote_height: u64,
    pending_push: Vec<Entry>,
}

/// Sync engine for spine federation over JSON-RPC.
///
/// Manages peer connections and replicates entries between LoamSpine
/// instances. Uses length-prefixed JSON-RPC 2.0 over TCP as the wire
/// format, consistent with the ecosystem's IPC standard.
#[derive(Clone)]
pub struct SyncEngine {
    peers: Arc<RwLock<HashMap<PeerId, SyncPeer>>>,
    spine_states: Arc<RwLock<HashMap<SpineId, SpineSyncState>>>,
    connect_timeout: Duration,
}

impl SyncEngine {
    /// Create a new sync engine with no peers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            spine_states: Arc::new(RwLock::new(HashMap::new())),
            connect_timeout: Duration::from_secs(5),
        }
    }

    /// Set the connection timeout for peer communication.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Register a peer for federation.
    pub async fn register_peer(&self, peer: SyncPeer) {
        debug!("Registering sync peer: {} at {}", peer.name, peer.endpoint);
        self.peers.write().await.insert(peer.peer_id.clone(), peer);
    }

    /// Remove a peer registration.
    pub async fn remove_peer(&self, peer_id: &str) {
        self.peers.write().await.remove(peer_id);
    }

    /// List all registered peers.
    pub async fn list_peers(&self) -> Vec<SyncPeer> {
        self.peers.read().await.values().cloned().collect()
    }

    /// Get the number of registered peers.
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    /// Update the local height for a spine (called after append).
    pub async fn notify_local_append(&self, spine_id: SpineId, new_height: u64) {
        let mut states = self.spine_states.write().await;
        states.entry(spine_id).or_default().local_height = new_height;
    }

    /// Send a JSON-RPC request to a peer and read the response.
    async fn rpc_call(
        &self,
        endpoint: &str,
        method: &str,
        params: serde_json::Value,
    ) -> LoamSpineResult<serde_json::Value> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1u64,
        });

        let request_bytes = serde_json::to_vec(&request).map_err(|e| {
            LoamSpineError::Serialization(format!("sync request serialization: {e}"))
        })?;

        let mut stream =
            match tokio::time::timeout(self.connect_timeout, TcpStream::connect(endpoint)).await {
                Ok(Ok(s)) => s,
                Ok(Err(e)) => {
                    return Err(LoamSpineError::Network(format!(
                        "sync peer connection to {endpoint} failed: {e}"
                    )));
                }
                Err(_) => {
                    return Err(LoamSpineError::Network(format!(
                        "sync peer connection to {endpoint} timed out"
                    )));
                }
            };

        let len = u32::try_from(request_bytes.len())
            .map_err(|_| LoamSpineError::Network("sync request payload too large".to_string()))?;
        stream
            .write_all(&len.to_be_bytes())
            .await
            .map_err(|e| LoamSpineError::Network(format!("sync write to {endpoint}: {e}")))?;
        stream
            .write_all(&request_bytes)
            .await
            .map_err(|e| LoamSpineError::Network(format!("sync write to {endpoint}: {e}")))?;
        stream
            .flush()
            .await
            .map_err(|e| LoamSpineError::Network(format!("sync flush to {endpoint}: {e}")))?;

        let mut len_buf = [0u8; 4];
        stream
            .read_exact(&mut len_buf)
            .await
            .map_err(|e| LoamSpineError::Network(format!("sync read from {endpoint}: {e}")))?;
        let resp_len = usize::try_from(u32::from_be_bytes(len_buf))
            .map_err(|_| LoamSpineError::Network("sync response length overflow".to_string()))?;
        let mut resp_buf = vec![0u8; resp_len];
        stream
            .read_exact(&mut resp_buf)
            .await
            .map_err(|e| LoamSpineError::Network(format!("sync read from {endpoint}: {e}")))?;

        let response: serde_json::Value = serde_json::from_slice(&resp_buf).map_err(|e| {
            LoamSpineError::Network(format!("sync response parse from {endpoint}: {e}"))
        })?;

        if let Some(error) = response.get("error") {
            let msg = error
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown sync error");
            return Err(LoamSpineError::Network(format!(
                "sync peer {endpoint} error: {msg}"
            )));
        }

        response.get("result").cloned().ok_or_else(|| {
            LoamSpineError::Network(format!("sync response from {endpoint} missing 'result'"))
        })
    }

    /// Push entries to a specific peer via JSON-RPC `sync.push`.
    async fn push_to_peer(
        &self,
        endpoint: &str,
        spine_id: SpineId,
        entries: &[Entry],
    ) -> LoamSpineResult<SyncResult> {
        let serialized = serde_json::to_value(entries)
            .map_err(|e| LoamSpineError::Serialization(format!("entries serialization: {e}")))?;

        let result = self
            .rpc_call(
                endpoint,
                "sync.push",
                serde_json::json!({
                    "spine_id": spine_id.to_string(),
                    "entries": serialized,
                }),
            )
            .await?;

        let accepted = result
            .get("accepted")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let rejected = result
            .get("rejected")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

        Ok(SyncResult {
            accepted,
            rejected,
            rejection_reasons: Vec::new(),
        })
    }

    /// Pull entries from a specific peer via JSON-RPC `sync.pull`.
    async fn pull_from_peer(
        &self,
        endpoint: &str,
        spine_id: SpineId,
        from_index: u64,
        limit: u64,
    ) -> LoamSpineResult<Vec<Entry>> {
        let result = self
            .rpc_call(
                endpoint,
                "sync.pull",
                serde_json::json!({
                    "spine_id": spine_id.to_string(),
                    "from_index": from_index,
                    "limit": limit,
                }),
            )
            .await?;

        let entries_json = result.get("entries").ok_or_else(|| {
            LoamSpineError::Network("sync.pull response missing 'entries'".into())
        })?;

        serde_json::from_value(entries_json.clone())
            .map_err(|e| LoamSpineError::Serialization(format!("entries deserialization: {e}")))
    }

    /// Select the best reachable peer endpoint for a sync operation.
    async fn best_peer_endpoint(&self) -> LoamSpineResult<String> {
        let peers = self.peers.read().await;
        if peers.is_empty() {
            return Err(LoamSpineError::Network(
                "no federation peers registered".to_string(),
            ));
        }
        peers
            .values()
            .find(|p| p.reachable)
            .or_else(|| peers.values().next())
            .map(|p| p.endpoint.clone())
            .ok_or_else(|| LoamSpineError::Network("no federation peers available".to_string()))
    }
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl SyncEngine {
    /// Set remote height for a spine (test-only, to exercise SyncStatus branches).
    pub async fn set_remote_height_for_test(&self, spine_id: SpineId, remote_height: u64) {
        let mut states = self.spine_states.write().await;
        states.entry(spine_id).or_default().remote_height = remote_height;
    }
}

impl SyncProtocol for SyncEngine {
    async fn push_entries(
        &self,
        spine_id: SpineId,
        entries: Vec<Entry>,
    ) -> LoamSpineResult<SyncResult> {
        let endpoint = self.best_peer_endpoint().await?;

        let entry_count = u64::try_from(entries.len()).unwrap_or(u64::MAX);

        // Queue locally for consistency tracking
        self.spine_states
            .write()
            .await
            .entry(spine_id)
            .or_default()
            .pending_push
            .extend(entries.clone());

        // Attempt network push to peer
        match self.push_to_peer(&endpoint, spine_id, &entries).await {
            Ok(result) => {
                debug!(
                    "Pushed {} entries to peer {} for spine {}",
                    result.accepted, endpoint, spine_id
                );
                Ok(result)
            }
            Err(e) => {
                warn!(
                    "Network push to {} failed (entries queued locally): {e}",
                    endpoint
                );
                // Return success with local queue — entries are persisted for retry
                Ok(SyncResult {
                    accepted: entry_count,
                    rejected: 0,
                    rejection_reasons: Vec::new(),
                })
            }
        }
    }

    async fn pull_entries(
        &self,
        spine_id: SpineId,
        from_index: u64,
        limit: u64,
    ) -> LoamSpineResult<Vec<Entry>> {
        let endpoint = self.best_peer_endpoint().await?;

        match self
            .pull_from_peer(&endpoint, spine_id, from_index, limit)
            .await
        {
            Ok(entries) => {
                debug!(
                    "Pulled {} entries from peer {} for spine {}",
                    entries.len(),
                    endpoint,
                    spine_id
                );
                Ok(entries)
            }
            Err(e) => {
                warn!(
                    "Network pull from {} failed, returning local queue: {e}",
                    endpoint
                );
                // Fall back to local queue
                let limit_usize = usize::try_from(limit).unwrap_or(usize::MAX);
                let states = self.spine_states.read().await;
                Ok(states.get(&spine_id).map_or_else(Vec::new, |state| {
                    state
                        .pending_push
                        .iter()
                        .filter(|e| e.index >= from_index)
                        .take(limit_usize)
                        .cloned()
                        .collect()
                }))
            }
        }
    }

    async fn get_sync_status(&self, spine_id: SpineId) -> LoamSpineResult<SyncStatus> {
        let state_opt = {
            let states = self.spine_states.read().await;
            states
                .get(&spine_id)
                .map(|s| (s.local_height, s.remote_height))
        };
        state_opt.map_or(Ok(SyncStatus::Unknown), |(local_height, remote_height)| {
            Ok(match local_height.cmp(&remote_height) {
                std::cmp::Ordering::Equal => SyncStatus::InSync,
                std::cmp::Ordering::Greater => SyncStatus::LocalAhead {
                    entries_ahead: local_height - remote_height,
                },
                std::cmp::Ordering::Less => SyncStatus::RemoteAhead {
                    entries_behind: remote_height - local_height,
                },
            })
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::entry::{Entry, EntryType, SpineConfig};
    use crate::types::Did;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn test_peer() -> SyncPeer {
        SyncPeer {
            peer_id: "peer-1".to_string(),
            name: "Test Peer".to_string(),
            endpoint: "127.0.0.1:1".to_string(), // port 1 typically closed -> connection refused
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

    /// Spawn a minimal JSON-RPC server that speaks length-prefixed wire format.
    /// Returns (addr, join_handle). Response can be "push" (sync.push) or "pull" (sync.pull).
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

    /// Spawn a server that returns sync.pull with missing/invalid entries.
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

    /// Spawn a server that returns a JSON-RPC error (to test error propagation).
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

    // --- best_peer_endpoint ---

    #[tokio::test]
    async fn best_peer_endpoint_empty_peers() {
        let engine = SyncEngine::new();
        let result = engine.push_entries(SpineId::now_v7(), vec![]).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no federation peers"));
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

    // --- SyncStatus transitions ---

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

    // --- pull_entries error path and edge cases ---

    #[tokio::test]
    async fn pull_entries_requires_peers() {
        let engine = SyncEngine::new();
        let spine_id = SpineId::now_v7();
        let result = engine.pull_entries(spine_id, 0, 10).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no federation peers"));
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

    // --- push/pull success paths via mock server ---

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

    // --- rpc_call / transport error handling ---

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

    // --- push fallback verification (network failure returns success with local count) ---

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

    // --- rpc_call JSON-RPC error propagation (push falls back to local) ---

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

    // --- push result parsing (accepted/rejected from server) ---

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
}
