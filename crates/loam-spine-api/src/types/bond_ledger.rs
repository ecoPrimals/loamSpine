// SPDX-License-Identifier: AGPL-3.0-or-later

//! Request/response types for the `bonding.ledger.*` wire contract.
//!
//! Wire shapes follow primalSpring's `STORAGE_WIRE_CONTRACT.md` §Bonding
//! Ledger Persistence so that any bond-producing primal can
//! call loamSpine via JSON-RPC with the canonical parameter names.

use serde::{Deserialize, Serialize};

/// Request for `bonding.ledger.store`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondLedgerStoreRequest {
    /// Unique bond identifier (matches `IonicBond.bond_id` from the signing primal).
    pub bond_id: String,
    /// Opaque bond data to persist.
    pub data: serde_json::Value,
}

/// Response from `bonding.ledger.store`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondLedgerStoreResponse {
    /// Status indicator (`"stored"`).
    pub status: String,
}

/// Request for `bonding.ledger.retrieve`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondLedgerRetrieveRequest {
    /// Bond identifier to look up.
    pub bond_id: String,
}

/// Response from `bonding.ledger.retrieve`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondLedgerRetrieveResponse {
    /// Bond data (null/absent if not found).
    pub data: Option<serde_json::Value>,
}

/// Request for `bonding.ledger.list`.
///
/// Takes no parameters; an empty object `{}` is valid.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BondLedgerListRequest {}

/// Response from `bonding.ledger.list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondLedgerListResponse {
    /// All stored bond identifiers.
    pub bonds: Vec<String>,
}
