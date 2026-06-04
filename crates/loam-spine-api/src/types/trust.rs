// SPDX-License-Identifier: AGPL-3.0-or-later

//! Request/response types for the `trust.*` wire contract.
//!
//! Wire shapes for cross-gate trust event anchoring. When a signing
//! primal registers a trusted issuer or verifies a cross-gate token,
//! it calls `trust.anchor` to create a permanent ledger record.

use loam_spine_core::entry::EntryType;
use loam_spine_core::types::{Did, EntryHash};
use serde::{Deserialize, Serialize};

/// Request for `trust.anchor`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAnchorRequest {
    /// The trust event to anchor. Must be a trust-domain `EntryType` variant:
    /// `KeyExchange`, `TrustIssuerRegistration`, or `TokenVerificationCrossGate`.
    pub entry_type: EntryType,
}

/// Response from `trust.anchor`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAnchorResponse {
    /// Blake3 hash of the anchored entry.
    pub entry_hash: EntryHash,
    /// Index in the trust ledger spine.
    pub index: u64,
}

/// Request for `trust.query`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustQueryRequest {
    /// Gate DID to query trust events for.
    pub gate_did: Did,
}

/// Response from `trust.query`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustQueryResponse {
    /// Trust events involving the queried gate.
    pub events: Vec<EntryType>,
}

/// Request for `trust.event_count`.
///
/// Takes no parameters; an empty object `{}` is valid.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrustEventCountRequest {}

/// Response from `trust.event_count`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEventCountResponse {
    /// Number of trust events in the ledger.
    pub count: u64,
}
