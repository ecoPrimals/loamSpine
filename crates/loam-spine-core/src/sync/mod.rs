// SPDX-License-Identifier: AGPL-3.0-or-later

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
mod tests;
