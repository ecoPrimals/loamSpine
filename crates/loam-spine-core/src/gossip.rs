// SPDX-License-Identifier: AGPL-3.0-or-later

//! Gossip injection for swarmVine mesh integration.
//!
//! Defines the events loamSpine emits to the ecosystem gossip mesh
//! via swarmVine's `gossip.inject` JSON-RPC method. Events are
//! fire-and-forget — gossip failures never block spine operations.
//!
//! ## Topics
//!
//! loamSpine emits on the **`data`** topic:
//!
//! | Key prefix | Emitted when | Purpose |
//! |------------|--------------|---------|
//! | `cas.have` | Entry appended | Data availability for CAS federation |
//! | `braid.head` | Braid committed | Provenance chain head for verification |
//! | `spine.sealed` | Spine sealed | Finality signal for federation |
//! | `anchor.published` | Chain anchor recorded | Public chain proof available |
//!
//! ## Wire Contract
//!
//! ```json
//! {
//!   "method": "gossip.inject",
//!   "params": {
//!     "topic": "data",
//!     "key": "cas.have:<gate>:<spine_id>",
//!     "value": { "entry_hash": "...", "height": 42 },
//!     "ttl": 10
//!   }
//! }
//! ```

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use tracing::{debug, warn};

use crate::types::{EntryHash, SpineId};

/// Default TTL for gossip entries (minutes).
const DEFAULT_GOSSIP_TTL: u64 = 10;

/// Events that loamSpine injects into the gossip mesh.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum GossipEvent {
    /// A new entry was appended to a spine — CAS availability signal.
    CasHave {
        /// Spine the entry belongs to.
        spine_id: SpineId,
        /// Hash of the appended entry.
        entry_hash: EntryHash,
        /// New spine height after the append.
        height: u64,
    },
    /// A braid was committed — provenance chain head update.
    BraidHead {
        /// Spine the braid was committed on.
        spine_id: SpineId,
        /// Hash of the braid commit entry.
        entry_hash: EntryHash,
        /// Content hash the braid covers.
        #[serde(skip_serializing_if = "Option::is_none")]
        subject_hash: Option<[u8; 32]>,
    },
    /// A spine was sealed — finality signal.
    SpineSealed {
        /// Sealed spine identifier.
        spine_id: SpineId,
        /// Final height of the sealed spine.
        height: u64,
    },
    /// An anchor was published to a public chain.
    AnchorPublished {
        /// Spine that was anchored.
        spine_id: SpineId,
        /// Hash of the anchor entry.
        entry_hash: EntryHash,
        /// Target chain or system name.
        #[serde(skip_serializing_if = "Option::is_none")]
        chain: Option<String>,
    },
}

impl GossipEvent {
    /// The gossip topic for this event — all loamSpine events are `"data"`.
    const TOPIC: &'static str = "data";

    /// The gossip key for routing and dedup.
    fn key(&self, gate_id: &str) -> String {
        match self {
            Self::CasHave { spine_id, .. } => {
                format!("cas.have:{gate_id}:{spine_id}")
            }
            Self::BraidHead { spine_id, .. } => {
                format!("braid.head:{gate_id}:{spine_id}")
            }
            Self::SpineSealed { spine_id, .. } => {
                format!("spine.sealed:{gate_id}:{spine_id}")
            }
            Self::AnchorPublished { spine_id, .. } => {
                format!("anchor.published:{gate_id}:{spine_id}")
            }
        }
    }
}

/// Gossip emitter that sends events to swarmVine via `gossip.inject`.
///
/// Fire-and-forget: errors are logged but never propagated. Spine
/// operations must never fail because gossip is unavailable.
pub struct GossipEmitter {
    socket_path: PathBuf,
    gate_id: String,
    request_counter: AtomicU64,
}

impl GossipEmitter {
    /// Create a gossip emitter targeting a swarmVine UDS socket.
    ///
    /// `gate_id` identifies this gate in gossip keys (e.g., `"sporeGate"`).
    #[must_use]
    pub const fn new(socket_path: PathBuf, gate_id: String) -> Self {
        Self {
            socket_path,
            gate_id,
            request_counter: AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> u64 {
        self.request_counter.fetch_add(1, Ordering::Relaxed)
    }

    /// Emit a gossip event (fire-and-forget).
    ///
    /// Spawns a background task so the caller is never blocked.
    pub fn emit(&self, event: &GossipEvent) {
        let topic = GossipEvent::TOPIC.to_owned();
        let key = event.key(&self.gate_id);
        let value = serde_json::to_value(event).unwrap_or_default();
        let socket = self.socket_path.clone();
        let id = self.next_id();

        tokio::spawn(async move {
            if let Err(e) = inject(socket, topic, key, value, id).await {
                warn!(error = %e, "gossip.inject failed (non-fatal)");
            }
        });
    }
}

/// Send a `gossip.inject` JSON-RPC call to swarmVine.
async fn inject(
    socket: PathBuf,
    topic: String,
    key: String,
    value: serde_json::Value,
    request_id: u64,
) -> Result<(), crate::error::LoamSpineError> {
    use crate::transport::{
        DEFAULT_IPC_TIMEOUT, connect_transport, endpoint_from_path, ndjson_rpc_call,
    };

    let params = serde_json::json!({
        "topic": topic,
        "key": key,
        "value": value,
        "ttl": DEFAULT_GOSSIP_TTL,
    });

    let endpoint = endpoint_from_path(&socket);
    let stream = connect_transport(&endpoint).await?;

    let _: serde_json::Value = ndjson_rpc_call(
        stream,
        "gossip.inject",
        params,
        request_id,
        DEFAULT_IPC_TIMEOUT,
        "swarmVine gossip",
    )
    .await?;

    debug!(key, "gossip event injected");
    Ok(())
}

/// Shared handle for optional gossip emission.
///
/// `None` when swarmVine is not available (standalone mode).
pub type GossipHandle = Option<Arc<GossipEmitter>>;

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests {
    use super::*;

    fn test_spine_id() -> SpineId {
        SpineId::from_bytes([0u8; 16])
    }

    fn test_entry_hash() -> EntryHash {
        [1u8; 32]
    }

    #[test]
    fn gossip_event_topic_is_data() {
        assert_eq!(GossipEvent::TOPIC, "data");
    }

    #[test]
    fn gossip_event_key_includes_gate_id() {
        let event = GossipEvent::CasHave {
            spine_id: test_spine_id(),
            entry_hash: test_entry_hash(),
            height: 10,
        };
        let key = event.key("sporeGate");
        assert!(key.starts_with("cas.have:sporeGate:"));
    }

    #[test]
    fn braid_head_key_format() {
        let event = GossipEvent::BraidHead {
            spine_id: test_spine_id(),
            entry_hash: [0u8; 32],
            subject_hash: None,
        };
        let key = event.key("westGate");
        assert!(key.starts_with("braid.head:westGate:"));
    }

    #[test]
    fn spine_sealed_key_format() {
        let event = GossipEvent::SpineSealed {
            spine_id: test_spine_id(),
            height: 42,
        };
        let key = event.key("ironGate");
        assert!(key.starts_with("spine.sealed:ironGate:"));
    }

    #[test]
    fn anchor_published_key_format() {
        let event = GossipEvent::AnchorPublished {
            spine_id: test_spine_id(),
            entry_hash: [0u8; 32],
            chain: Some("bitcoin".into()),
        };
        let key = event.key("strandGate");
        assert!(key.starts_with("anchor.published:strandGate:"));
    }

    #[test]
    fn gossip_event_serializes_to_json() {
        let event = GossipEvent::CasHave {
            spine_id: test_spine_id(),
            entry_hash: test_entry_hash(),
            height: 7,
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["event"], "cas_have");
        assert_eq!(json["height"], 7);
    }

    #[test]
    fn gossip_emitter_increments_request_id() {
        let emitter = GossipEmitter::new(PathBuf::from("/tmp/test.sock"), "test".into());
        let id1 = emitter.next_id();
        let id2 = emitter.next_id();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn all_events_use_data_topic() {
        assert_eq!(GossipEvent::TOPIC, "data");
    }

    #[test]
    fn anchor_published_omits_none_chain() {
        let event = GossipEvent::AnchorPublished {
            spine_id: test_spine_id(),
            entry_hash: [0u8; 32],
            chain: None,
        };
        let json = serde_json::to_value(&event).unwrap();
        assert!(!json.as_object().unwrap().contains_key("chain"));
    }

    #[test]
    fn chain_name_roundtrips_in_json() {
        let event = GossipEvent::AnchorPublished {
            spine_id: test_spine_id(),
            entry_hash: [0u8; 32],
            chain: Some("bitcoin".into()),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["chain"], "bitcoin");
    }
}
