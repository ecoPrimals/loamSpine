// SPDX-License-Identifier: AGPL-3.0-only

//! Sync protocol implementation for spine federation.
//!
//! Implements the [`SyncProtocol`] trait defined in `traits/mod.rs` for
//! replicating spines between LoamSpine instances across trust boundaries.
//!
//! ## Status
//!
//! This is a **stub** implementation that tracks sync state locally but
//! does not perform actual network replication. It serves as the foundation
//! for the RootPulse `federate` workflow and will be evolved into a full
//! peer-to-peer sync engine.
//!
//! ## Architecture
//!
//! ```text
//! Node A                          Node B
//! ┌─────────┐  push_entries  ┌─────────┐
//! │ Spine X ├───────────────►│ Spine X │
//! │         │◄───────────────┤         │
//! └─────────┘  pull_entries  └─────────┘
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

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
    /// Transport endpoint (URL, socket path, or address).
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
    last_sync_ns: u64,
    pending_push: Vec<Entry>,
}

/// Local-only sync engine stub.
///
/// Tracks sync state per spine and queues entries for push, but does not
/// perform actual network replication. This is the stub that RootPulse's
/// `federate` workflow calls into.
#[derive(Clone)]
pub struct SyncEngine {
    peers: Arc<RwLock<HashMap<PeerId, SyncPeer>>>,
    spine_states: Arc<RwLock<HashMap<SpineId, SpineSyncState>>>,
}

impl SyncEngine {
    /// Create a new sync engine with no peers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            spine_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a peer for federation.
    pub async fn register_peer(&self, peer: SyncPeer) {
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
        let state = states.entry(spine_id).or_default();
        state.local_height = new_height;
    }
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncProtocol for SyncEngine {
    async fn push_entries(
        &self,
        spine_id: SpineId,
        entries: Vec<Entry>,
    ) -> LoamSpineResult<SyncResult> {
        let peers = self.peers.read().await;
        if peers.is_empty() {
            return Err(LoamSpineError::Network(
                "no federation peers registered".to_string(),
            ));
        }

        let entry_count = entries.len() as u64;

        let mut states = self.spine_states.write().await;
        let state = states.entry(spine_id).or_default();
        state.pending_push.extend(entries);

        // Stub: accept all entries locally, actual replication is future work
        Ok(SyncResult {
            accepted: entry_count,
            rejected: 0,
            rejection_reasons: Vec::new(),
        })
    }

    async fn pull_entries(
        &self,
        spine_id: SpineId,
        from_index: u64,
        limit: u64,
    ) -> LoamSpineResult<Vec<Entry>> {
        let peers = self.peers.read().await;
        if peers.is_empty() {
            return Err(LoamSpineError::Network(
                "no federation peers registered".to_string(),
            ));
        }

        let states = self.spine_states.read().await;
        if let Some(state) = states.get(&spine_id) {
            let entries: Vec<Entry> = state
                .pending_push
                .iter()
                .filter(|e| e.index >= from_index)
                .take(limit as usize)
                .cloned()
                .collect();
            Ok(entries)
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_sync_status(&self, spine_id: SpineId) -> LoamSpineResult<SyncStatus> {
        let states = self.spine_states.read().await;
        match states.get(&spine_id) {
            Some(state) => {
                if state.local_height == state.remote_height {
                    Ok(SyncStatus::InSync)
                } else if state.local_height > state.remote_height {
                    Ok(SyncStatus::LocalAhead {
                        entries_ahead: state.local_height - state.remote_height,
                    })
                } else {
                    Ok(SyncStatus::RemoteAhead {
                        entries_behind: state.remote_height - state.local_height,
                    })
                }
            }
            None => Ok(SyncStatus::Unknown),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::entry::{Entry, EntryType, SpineConfig};
    use crate::types::Did;

    fn test_peer() -> SyncPeer {
        SyncPeer {
            peer_id: "peer-1".to_string(),
            name: "Test Peer".to_string(),
            endpoint: "tcp://192.0.2.100:9090".to_string(),
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
        assert_eq!(
            status,
            SyncStatus::LocalAhead { entries_ahead: 5 }
        );
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
        engine
            .push_entries(spine_id, entries)
            .await
            .unwrap();

        let pulled = engine.pull_entries(spine_id, 0, 10).await.unwrap();
        assert_eq!(pulled.len(), 2);
    }

    #[tokio::test]
    async fn default_engine_is_empty() {
        let engine = SyncEngine::default();
        assert_eq!(engine.peer_count().await, 0);
    }
}
