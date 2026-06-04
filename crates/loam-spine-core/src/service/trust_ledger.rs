// SPDX-License-Identifier: AGPL-3.0-or-later

//! Trust event ledger for cross-gate trust establishment.
//!
//! When a signing primal (e.g. bearDog) registers a trusted issuer,
//! exchanges keys with a remote gate, or verifies a cross-gate token,
//! it calls loamSpine to anchor the event as a permanent ledger entry.
//!
//! Each `trust.anchor` appends a `KeyExchange`, `TrustIssuerRegistration`,
//! or `TokenVerificationCrossGate` entry to a dedicated trust spine,
//! maintaining an immutable audit trail of cross-gate trust relationships.

use crate::entry::EntryType;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::storage::{EntryStorage, SpineStorage};
use crate::types::{Did, EntryHash, SpineId};

use super::LoamSpineService;

const TRUST_LEDGER_OWNER: &str = "did:primal:loamspine:trust-ledger";

impl LoamSpineService {
    /// Ensure the trust ledger spine exists, creating it on first use.
    async fn ensure_trust_ledger_spine(&self) -> LoamSpineResult<SpineId> {
        let existing = *self.trust_ledger_spine.read().await;
        if let Some(id) = existing {
            return Ok(id);
        }

        let owner = Did::new(TRUST_LEDGER_OWNER);
        let spine = crate::spine::Spine::new(
            owner,
            Some("trust-ledger".into()),
            crate::entry::SpineConfig::default(),
        )?;
        let id = spine.id;

        if let Some(genesis) = spine.genesis_entry() {
            self.entry_storage.save_entry(genesis).await?;
        }
        self.spine_storage.save_spine(&spine).await?;
        *self.trust_ledger_spine.write().await = Some(id);

        Ok(id)
    }

    /// Anchor a trust event as a permanent ledger entry.
    ///
    /// Validates that `entry_type` is a trust-domain variant, then appends
    /// it to the dedicated trust spine. Returns the entry hash and index.
    ///
    /// # Errors
    ///
    /// Returns an error if the entry type is not in the `"trust"` domain,
    /// or if spine creation/entry append fails.
    pub async fn trust_anchor_event(
        &self,
        entry_type: EntryType,
    ) -> LoamSpineResult<(EntryHash, u64)> {
        if entry_type.domain() != "trust" {
            return Err(LoamSpineError::Internal(
                "trust.anchor requires a trust-domain entry type".into(),
            ));
        }

        let spine_id = self.ensure_trust_ledger_spine().await?;

        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or_else(|| LoamSpineError::Internal("trust ledger spine vanished".into()))?;

        let entry = spine.create_entry(entry_type);
        let hash = spine.append(entry)?;
        let index = spine.height - 1;

        if let Some(tip) = spine.tip_entry() {
            self.entry_storage.save_entry(tip).await?;
        }
        self.spine_storage.save_spine(&spine).await?;

        Ok((hash, index))
    }

    /// Query trust events by gate DID.
    ///
    /// Scans the trust spine for entries involving the given gate DID
    /// in any role (local, remote, issuer, registering, verifier).
    pub async fn trust_query_by_gate(&self, gate_did: &Did) -> Vec<EntryType> {
        let Some(spine_id) = *self.trust_ledger_spine.read().await else {
            return Vec::new();
        };

        let Ok(Some(spine)) = self.spine_storage.get_spine(spine_id).await else {
            return Vec::new();
        };

        spine
            .entries()
            .iter()
            .filter(|e| Self::trust_entry_involves_gate(&e.entry_type, gate_did))
            .map(|e| e.entry_type.clone())
            .collect()
    }

    fn trust_entry_involves_gate(entry_type: &EntryType, gate: &Did) -> bool {
        match entry_type {
            EntryType::KeyExchange {
                local_gate,
                remote_gate,
                ..
            } => local_gate == gate || remote_gate == gate,
            EntryType::TrustIssuerRegistration {
                issuer_did,
                registering_gate,
                ..
            } => issuer_did == gate || registering_gate == gate,
            EntryType::TokenVerificationCrossGate {
                issuer_gate,
                verifier_gate,
                ..
            } => issuer_gate == gate || verifier_gate == gate,
            _ => false,
        }
    }

    /// Return the trust spine height (0 if no trust events recorded).
    pub async fn trust_event_count(&self) -> u64 {
        let Some(spine_id) = *self.trust_ledger_spine.read().await else {
            return 0;
        };

        match self.spine_storage.get_spine(spine_id).await {
            Ok(Some(s)) if s.height > 0 => s.height - 1,
            _ => 0,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests use unwrap for concise assertions"
)]
mod tests {
    use super::*;
    use crate::types::Timestamp;

    fn test_gate_a() -> Did {
        Did::new("did:key:z6MkStrandGate")
    }

    fn test_gate_b() -> Did {
        Did::new("did:key:z6MkBiomeGate")
    }

    #[tokio::test]
    async fn anchor_key_exchange() {
        let svc = LoamSpineService::new();
        let et = EntryType::KeyExchange {
            local_gate: test_gate_a(),
            remote_gate: test_gate_b(),
            public_key_hash: [0xab; 32],
            direction: "initiated".into(),
            family_id: Some("alpha".into()),
        };

        let (hash, index) = svc.trust_anchor_event(et).await.unwrap();
        assert_ne!(hash, [0u8; 32]);
        assert_eq!(index, 1);
    }

    #[tokio::test]
    async fn anchor_trust_issuer_registration() {
        let svc = LoamSpineService::new();
        let et = EntryType::TrustIssuerRegistration {
            issuer_did: test_gate_b(),
            registering_gate: test_gate_a(),
            trust_scope: "cross-gate".into(),
            capabilities: vec!["signing".into(), "verification".into()],
            expires_at: Some(Timestamp::now()),
        };

        let (_, index) = svc.trust_anchor_event(et).await.unwrap();
        assert_eq!(index, 1);
    }

    #[tokio::test]
    async fn anchor_token_verification() {
        let svc = LoamSpineService::new();
        let et = EntryType::TokenVerificationCrossGate {
            issuer_gate: test_gate_b(),
            verifier_gate: test_gate_a(),
            token_hash: [0xcd; 32],
            verified: true,
            failure_reason: None,
        };

        let (_, index) = svc.trust_anchor_event(et).await.unwrap();
        assert_eq!(index, 1);
    }

    #[tokio::test]
    async fn anchor_rejects_non_trust_entry() {
        let svc = LoamSpineService::new();
        let et = EntryType::MetadataUpdate {
            field: "name".into(),
            value: "test".into(),
        };

        let err = svc.trust_anchor_event(et).await.unwrap_err();
        assert!(err.to_string().contains("trust-domain"));
    }

    #[tokio::test]
    async fn trust_spine_lazily_created() {
        let svc = LoamSpineService::new();
        assert!(svc.trust_ledger_spine.read().await.is_none());

        let et = EntryType::KeyExchange {
            local_gate: test_gate_a(),
            remote_gate: test_gate_b(),
            public_key_hash: [1; 32],
            direction: "accepted".into(),
            family_id: None,
        };
        svc.trust_anchor_event(et).await.unwrap();

        assert!(svc.trust_ledger_spine.read().await.is_some());
    }

    #[tokio::test]
    async fn trust_spine_reused() {
        let svc = LoamSpineService::new();

        let et1 = EntryType::KeyExchange {
            local_gate: test_gate_a(),
            remote_gate: test_gate_b(),
            public_key_hash: [1; 32],
            direction: "initiated".into(),
            family_id: None,
        };
        svc.trust_anchor_event(et1).await.unwrap();
        let id1 = svc.trust_ledger_spine.read().await.unwrap();

        let et2 = EntryType::TrustIssuerRegistration {
            issuer_did: test_gate_b(),
            registering_gate: test_gate_a(),
            trust_scope: "family".into(),
            capabilities: vec![],
            expires_at: None,
        };
        svc.trust_anchor_event(et2).await.unwrap();
        let id2 = svc.trust_ledger_spine.read().await.unwrap();

        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn trust_event_count() {
        let svc = LoamSpineService::new();
        assert_eq!(svc.trust_event_count().await, 0);

        let et = EntryType::KeyExchange {
            local_gate: test_gate_a(),
            remote_gate: test_gate_b(),
            public_key_hash: [2; 32],
            direction: "initiated".into(),
            family_id: None,
        };
        svc.trust_anchor_event(et).await.unwrap();
        assert_eq!(svc.trust_event_count().await, 1);
    }

    #[tokio::test]
    async fn query_by_gate() {
        let svc = LoamSpineService::new();

        let et1 = EntryType::KeyExchange {
            local_gate: test_gate_a(),
            remote_gate: test_gate_b(),
            public_key_hash: [3; 32],
            direction: "initiated".into(),
            family_id: None,
        };
        svc.trust_anchor_event(et1).await.unwrap();

        let et2 = EntryType::TrustIssuerRegistration {
            issuer_did: Did::new("did:key:z6MkOtherGate"),
            registering_gate: Did::new("did:key:z6MkOtherGate2"),
            trust_scope: "global".into(),
            capabilities: vec!["signing".into()],
            expires_at: None,
        };
        svc.trust_anchor_event(et2).await.unwrap();

        let results_a = svc.trust_query_by_gate(&test_gate_a()).await;
        assert_eq!(results_a.len(), 1);

        let results_b = svc.trust_query_by_gate(&test_gate_b()).await;
        assert_eq!(results_b.len(), 1);

        let results_other = svc
            .trust_query_by_gate(&Did::new("did:key:z6MkNobody"))
            .await;
        assert!(results_other.is_empty());
    }
}
