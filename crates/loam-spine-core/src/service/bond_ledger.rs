// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bond ledger persistence for cross-primal ionic bond contracts.
//!
//! Implements the `bonding.ledger.*` wire contract from primalSpring's
//! `STORAGE_WIRE_CONTRACT.md`. The crypto capability primal keeps ionic
//! bonds in-memory and delegates persistence to loamSpine via JSON-RPC.
//!
//! Each `bonding.ledger.store` appends a `BondLedgerRecord` entry to a
//! dedicated spine, maintaining an immutable audit trail. An in-memory
//! index provides O(1) retrieval by `bond_id`.

use crate::entry::EntryType;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::storage::{EntryStorage, SpineStorage};
use crate::types::{Did, SpineId};

use super::LoamSpineService;

/// Well-known DID for the bond ledger spine owner.
///
/// This is internal bookkeeping — the DID is not a real identity, it's a
/// namespace marker so the bond ledger spine is distinguishable from
/// user-created spines.
const BOND_LEDGER_OWNER: &str = "did:primal:loamspine:bond-ledger";

impl LoamSpineService {
    /// Ensure the bond ledger spine exists, creating it on first use.
    async fn ensure_bond_ledger_spine(&self) -> LoamSpineResult<SpineId> {
        let existing = *self.bond_ledger_spine.read().await;
        if let Some(id) = existing {
            return Ok(id);
        }

        let owner = Did::new(BOND_LEDGER_OWNER);
        let spine = crate::spine::Spine::new(
            owner,
            Some("bond-ledger".into()),
            crate::entry::SpineConfig::default(),
        )?;
        let id = spine.id;

        if let Some(genesis) = spine.genesis_entry() {
            self.entry_storage.save_entry(genesis).await?;
        }
        self.spine_storage.save_spine(&spine).await?;
        *self.bond_ledger_spine.write().await = Some(id);

        Ok(id)
    }

    /// Store a bond record in the ledger.
    ///
    /// Appends a `BondLedgerRecord` entry to the dedicated bond spine and
    /// updates the in-memory index. If a bond with the same `bond_id`
    /// already exists, the new data overwrites the index entry (the old
    /// entry remains in the spine for audit purposes).
    ///
    /// # Errors
    ///
    /// Returns an error if spine creation or entry append fails.
    pub async fn bond_ledger_store(
        &self,
        bond_id: impl Into<String>,
        data: serde_json::Value,
    ) -> LoamSpineResult<()> {
        let bond_id = bond_id.into();
        let spine_id = self.ensure_bond_ledger_spine().await?;

        let entry_type = EntryType::BondLedgerRecord {
            bond_id: bond_id.clone(),
            data: data.clone(),
        };

        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or_else(|| LoamSpineError::Internal("bond ledger spine vanished".into()))?;

        let entry = spine.create_entry(entry_type);
        spine.append(entry)?;

        if let Some(tip) = spine.tip_entry() {
            self.entry_storage.save_entry(tip).await?;
        }
        self.spine_storage.save_spine(&spine).await?;

        self.bond_ledger.write().await.insert(bond_id, data);

        Ok(())
    }

    /// Retrieve a bond record by ID.
    ///
    /// Returns `None` if no bond with the given ID has been stored.
    pub async fn bond_ledger_retrieve(&self, bond_id: &str) -> Option<serde_json::Value> {
        self.bond_ledger.read().await.get(bond_id).cloned()
    }

    /// List all stored bond IDs.
    pub async fn bond_ledger_list(&self) -> Vec<String> {
        self.bond_ledger.read().await.keys().cloned().collect()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests use unwrap for concise assertions"
)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn store_and_retrieve_bond() {
        let svc = LoamSpineService::new();
        let data = serde_json::json!({
            "bond_id": "bond-001",
            "proposer": "tower_a",
            "acceptor": "tower_b",
            "state": "sealed",
            "terms_hash": "abc123",
        });

        svc.bond_ledger_store("bond-001", data.clone())
            .await
            .unwrap();

        let retrieved = svc.bond_ledger_retrieve("bond-001").await;
        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test]
    async fn retrieve_nonexistent_bond_returns_none() {
        let svc = LoamSpineService::new();
        assert!(svc.bond_ledger_retrieve("no-such-bond").await.is_none());
    }

    #[tokio::test]
    async fn list_bonds_empty() {
        let svc = LoamSpineService::new();
        assert!(svc.bond_ledger_list().await.is_empty());
    }

    #[tokio::test]
    async fn list_bonds_after_stores() {
        let svc = LoamSpineService::new();
        svc.bond_ledger_store("b1", serde_json::json!({"x": 1}))
            .await
            .unwrap();
        svc.bond_ledger_store("b2", serde_json::json!({"x": 2}))
            .await
            .unwrap();

        let mut bonds = svc.bond_ledger_list().await;
        bonds.sort();
        assert_eq!(bonds, vec!["b1", "b2"]);
    }

    #[tokio::test]
    async fn store_overwrites_index_preserves_spine_history() {
        let svc = LoamSpineService::new();
        let v1 = serde_json::json!({"state": "active"});
        let v2 = serde_json::json!({"state": "sealed"});

        svc.bond_ledger_store("bond-x", v1).await.unwrap();
        svc.bond_ledger_store("bond-x", v2.clone()).await.unwrap();

        let retrieved = svc.bond_ledger_retrieve("bond-x").await;
        assert_eq!(retrieved, Some(v2));

        let spine_id = svc.bond_ledger_spine.read().await.unwrap();
        let spine = svc
            .spine_storage
            .get_spine(spine_id)
            .await
            .unwrap()
            .unwrap();
        // Genesis + 2 bond entries = height 3
        assert_eq!(spine.height, 3);
    }

    #[tokio::test]
    async fn bond_ledger_spine_is_lazily_created() {
        let svc = LoamSpineService::new();
        assert!(svc.bond_ledger_spine.read().await.is_none());

        svc.bond_ledger_store("b", serde_json::json!({}))
            .await
            .unwrap();

        assert!(svc.bond_ledger_spine.read().await.is_some());
    }

    #[tokio::test]
    async fn bond_ledger_spine_reused_across_stores() {
        let svc = LoamSpineService::new();

        svc.bond_ledger_store("a", serde_json::json!(1))
            .await
            .unwrap();
        let first_id = svc.bond_ledger_spine.read().await.unwrap();

        svc.bond_ledger_store("b", serde_json::json!(2))
            .await
            .unwrap();
        let second_id = svc.bond_ledger_spine.read().await.unwrap();

        assert_eq!(first_id, second_id);
    }
}
