// SPDX-License-Identifier: AGPL-3.0-or-later

//! Public chain anchor RPC types.

use serde::{Deserialize, Serialize};

pub use loam_spine_core::entry::AnchorTarget;

use super::{ContentHash, EntryHash, SpineId, Timestamp};

/// Request to record a public chain anchor on a spine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorPublishRequest {
    /// Spine to anchor.
    pub spine_id: SpineId,
    /// Target system (bitcoin, ethereum, federated spine, data commons, etc.).
    pub anchor_target: AnchorTarget,
    /// Transaction hash or proof reference on the external system.
    pub tx_ref: String,
    /// Block height or sequence number (0 if not applicable).
    #[serde(default)]
    pub block_height: u64,
    /// Timestamp when the anchor was confirmed externally.
    pub anchor_timestamp: Timestamp,
}

/// Response after recording a public chain anchor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorPublishResponse {
    /// Hash of the `PublicChainAnchor` entry on the spine.
    pub entry_hash: EntryHash,
    /// The spine state hash that was anchored.
    pub state_hash: ContentHash,
}

/// Request to verify a spine's state against a recorded public anchor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorVerifyRequest {
    /// Spine to verify.
    pub spine_id: SpineId,
    /// Specific anchor entry hash to verify (defaults to latest if absent).
    #[serde(default)]
    pub anchor_entry_hash: Option<EntryHash>,
}

/// Verification result for a recorded public chain anchor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorVerifyResponse {
    /// Whether the recorded state hash matches the spine's actual state.
    pub verified: bool,
    /// The anchor target system.
    pub anchor_target: AnchorTarget,
    /// The recorded state hash.
    pub state_hash: ContentHash,
    /// Transaction reference on the external system.
    pub tx_ref: String,
    /// Block height or sequence number.
    pub block_height: u64,
    /// When the anchor was confirmed externally.
    pub anchor_timestamp: Timestamp,
}
