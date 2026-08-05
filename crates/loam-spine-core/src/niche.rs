// SPDX-License-Identifier: AGPL-3.0-or-later

//! Primal self-knowledge for LoamSpine.
//!
//! LoamSpine is a **primal** — a first-class service in the ecoPrimals
//! ecosystem. It provides permanence, certificates, and proofs as its
//! core capability domains.
//!
//! This module holds the primal's self-knowledge:
//! - Identity (who am I?)
//! - Capabilities (what do I expose via the orchestration layer?)
//! - Semantic mappings (capability domain → JSON-RPC methods)
//! - Dependencies (what capabilities do I consume?)
//! - Operation costs (scheduling hints for the ecosystem orchestrator)
//!
//! Other modules (`neural_api`, `capabilities`, `service`) reference
//! these constants rather than duplicating string literals. LoamSpine
//! only knows itself — it discovers other primals at runtime.

/// Primal identity — used in all JSON-RPC, provenance, and IPC interactions.
pub const PRIMAL_ID: &str = crate::primal_names::SELF_ID;

/// Human-readable primal description for orchestrator registration.
pub const PRIMAL_DESCRIPTION: &str =
    "Permanence layer providing selective memory, certificates, and inclusion proofs";

/// Primal category for ecosystem deployment.
pub const PRIMAL_CATEGORY: &str = "infrastructure";

/// All capability domains this primal exposes.
pub const DOMAINS: &[&str] = &[
    "spine",
    "entry",
    "certificate",
    "proof",
    "waypoint",
    "anchor",
    "bonding",
    "trust",
    "health",
    "meta",
    "integration",
];

/// All JSON-RPC methods this primal exposes to the orchestration layer.
///
/// Each string is a fully qualified method name (`{domain}.{operation}`)
/// that the ecosystem orchestrator can route via `capability.call`.
pub const METHODS: &[&str] = &[
    // Spine lifecycle
    "spine.create",
    "spine.get",
    "spine.list",
    "spine.status",
    "spine.seal",
    // Entry CRUD
    "entry.append",
    "entry.append_batch",
    "entry.get",
    "entry.get_tip",
    "entry.list",
    // Certificate lifecycle
    "certificate.mint",
    "certificate.mint_batch",
    "certificate.transfer",
    "certificate.loan",
    "certificate.return",
    "certificate.get",
    "certificate.verify",
    "certificate.lifecycle",
    "certificate.history",
    // Waypoint slices
    "slice.anchor",
    "slice.checkout",
    // Proofs
    "proof.generate_inclusion",
    "proof.verify_inclusion",
    // Provenance trio integration
    "session.dehydrate",
    "session.commit",
    "braid.commit",
    // Public chain anchoring
    "anchor.publish",
    "anchor.publish_batch",
    "anchor.verify",
    // Ionic bond ledger
    "bonding.ledger.store",
    "bonding.ledger.retrieve",
    "bonding.ledger.list",
    // Cross-gate trust ledger
    "trust.anchor",
    "trust.query",
    "trust.event_count",
    // BTSP Phase 3
    "btsp.negotiate",
    "btsp.capabilities",
    // Infrastructure (public)
    "primal.announce",
    "health.check",
    "health.liveness",
    "health.readiness",
    "lifecycle.status",
    "capabilities.list",
    "tools.list",
    "tools.call",
    "identity.get",
    "auth.check",
    "auth.mode",
    "auth.peer_info",
    // Permanence compat layer
    "permanence.commit_session",
    "permanence.verify_commit",
    "permanence.get_commit",
    "permanence.health_check",
];

/// Semantic mappings: short operation name → fully qualified method.
///
/// The orchestrator uses these during domain registration so
/// `capability.call { domain: "spine", operation: "create" }`
/// routes to `spine.create` on our socket.
pub const SEMANTIC_MAPPINGS: &[(&str, &str)] = &[
    // Spine lifecycle
    ("create_spine", "spine.create"),
    ("get_spine", "spine.get"),
    ("list_spines", "spine.list"),
    ("spine_status", "spine.status"),
    ("seal_spine", "spine.seal"),
    // Entry CRUD
    ("append_entry", "entry.append"),
    ("append_entry_batch", "entry.append_batch"),
    ("get_entry", "entry.get"),
    ("get_tip", "entry.get_tip"),
    ("list_entries", "entry.list"),
    // Certificate lifecycle
    ("mint_certificate", "certificate.mint"),
    ("mint_certificate_batch", "certificate.mint_batch"),
    ("transfer_certificate", "certificate.transfer"),
    ("loan_certificate", "certificate.loan"),
    ("return_certificate", "certificate.return"),
    ("get_certificate", "certificate.get"),
    ("verify_certificate", "certificate.verify"),
    ("certificate_lifecycle", "certificate.lifecycle"),
    ("certificate_history", "certificate.history"),
    // Waypoint slices
    ("anchor_slice", "slice.anchor"),
    ("checkout_slice", "slice.checkout"),
    // Proofs
    ("generate_inclusion_proof", "proof.generate_inclusion"),
    ("verify_inclusion_proof", "proof.verify_inclusion"),
    // Provenance trio
    ("dehydrate_session", "session.dehydrate"),
    ("commit_session", "session.commit"),
    ("commit_braid", "braid.commit"),
    // Public chain anchoring
    ("publish_anchor", "anchor.publish"),
    ("publish_anchor_batch", "anchor.publish_batch"),
    ("verify_anchor", "anchor.verify"),
    // Ionic bond ledger
    ("bond_ledger_store", "bonding.ledger.store"),
    ("bond_ledger_retrieve", "bonding.ledger.retrieve"),
    ("bond_ledger_list", "bonding.ledger.list"),
    // Cross-gate trust ledger
    ("trust_anchor", "trust.anchor"),
    ("trust_query", "trust.query"),
    ("trust_event_count", "trust.event_count"),
    // BTSP Phase 3
    ("btsp_negotiate", "btsp.negotiate"),
    ("btsp_capabilities", "btsp.capabilities"),
    // Infrastructure
    ("primal_announce", "primal.announce"),
    ("health_check", "health.check"),
    ("health_liveness", "health.liveness"),
    ("health_readiness", "health.readiness"),
    ("lifecycle_status", "lifecycle.status"),
    ("capability_list", "capabilities.list"),
    ("identity_get", "identity.get"),
    ("tools_list", "tools.list"),
    ("tools_call", "tools.call"),
    ("auth_check", "auth.check"),
    ("auth_mode", "auth.mode"),
    ("auth_peer_info", "auth.peer_info"),
    // Permanence compat layer
    ("permanence_commit_session", "permanence.commit_session"),
    ("permanence_verify_commit", "permanence.verify_commit"),
    ("permanence_get_commit", "permanence.get_commit"),
    ("permanence_health_check", "permanence.health_check"),
];

/// Consumed capabilities — what LoamSpine calls on other primals.
///
/// LoamSpine discovers these at runtime via capability-based discovery;
/// it never hardcodes which primal provides them.
pub const CONSUMED_CAPABILITIES: &[&str] = &[
    crate::capabilities::identifiers::external::BTSP,
    crate::capabilities::identifiers::external::SIGNING,
    crate::capabilities::identifiers::external::STORAGE,
    crate::capabilities::identifiers::external::DISCOVERY,
    crate::capabilities::identifiers::external::SESSION_MANAGEMENT,
    crate::capabilities::identifiers::external::COMPUTE,
    crate::capabilities::identifiers::external::ATTESTATION,
    crate::capabilities::identifiers::external::CHAIN_ANCHOR,
];

/// Dependencies for primal deployment.
///
/// Each entry: `(capability_id, required, description)`.
/// `required = true` means LoamSpine cannot function without it.
/// `required = false` means graceful degradation is supported.
pub const DEPENDENCIES: &[(&str, bool, &str)] = &[
    (
        crate::capabilities::identifiers::external::BTSP,
        false,
        "BTSP handshake-as-a-service (required when FAMILY_ID is set; development mode skips)",
    ),
    (
        crate::capabilities::identifiers::external::SIGNING,
        false,
        "crypto.sign_ed25519 / crypto.verify_ed25519 via JSON-RPC (graceful degradation to CLI signer)",
    ),
    (
        crate::capabilities::identifiers::external::STORAGE,
        false,
        "external content-addressable storage (local redb is self-sufficient)",
    ),
    (
        crate::capabilities::identifiers::external::DISCOVERY,
        false,
        "service registry (mDNS / DNS-SRV / etcd) for primal discovery (env vars as fallback)",
    ),
    (
        crate::capabilities::identifiers::external::ATTESTATION,
        false,
        "operation attestation for waypoint semantics (enforcement deferred)",
    ),
    (
        crate::capabilities::identifiers::external::CHAIN_ANCHOR,
        false,
        "external chain anchor submission (loamSpine only records receipts)",
    ),
];

/// Cost estimates for ecosystem orchestrator scheduling.
///
/// Each entry: `(method, estimated_ms, gpu_beneficial)`.
pub const COST_ESTIMATES: &[(&str, u32, bool)] = &[
    // Spine lifecycle
    ("spine.create", 1, false),
    ("spine.get", 1, false),
    ("spine.list", 1, false),
    ("spine.status", 2, false),
    ("spine.seal", 1, false),
    // Entry CRUD
    ("entry.append", 2, false),
    ("entry.append_batch", 10, false),
    ("entry.get", 1, false),
    ("entry.get_tip", 1, false),
    ("entry.list", 2, false),
    // Certificate lifecycle
    ("certificate.mint", 3, false),
    ("certificate.mint_batch", 15, false),
    ("certificate.transfer", 2, false),
    ("certificate.loan", 2, false),
    ("certificate.return", 2, false),
    ("certificate.get", 1, false),
    ("certificate.verify", 2, false),
    ("certificate.lifecycle", 2, false),
    ("certificate.history", 3, false),
    // Waypoint slices
    ("slice.anchor", 2, false),
    ("slice.checkout", 1, false),
    // Proofs
    ("proof.generate_inclusion", 10, false),
    ("proof.verify_inclusion", 5, false),
    // Provenance trio
    ("session.dehydrate", 3, false),
    ("session.commit", 5, false),
    ("braid.commit", 5, false),
    // Public chain anchoring
    ("anchor.publish", 2, false),
    ("anchor.publish_batch", 10, false),
    ("anchor.verify", 2, false),
    // Ionic bond ledger
    ("bonding.ledger.store", 2, false),
    ("bonding.ledger.retrieve", 1, false),
    ("bonding.ledger.list", 1, false),
    // Cross-gate trust ledger
    ("trust.anchor", 2, false),
    ("trust.query", 5, false),
    ("trust.event_count", 1, false),
    // BTSP Phase 3
    ("btsp.negotiate", 2, false),
    ("btsp.capabilities", 1, false),
    // Infrastructure
    ("primal.announce", 1, false),
    ("health.check", 1, false),
    ("health.liveness", 1, false),
    ("health.readiness", 1, false),
    ("lifecycle.status", 1, false),
    ("capabilities.list", 1, false),
    ("identity.get", 1, false),
    ("tools.list", 1, false),
    ("tools.call", 5, false),
    ("auth.check", 1, false),
    ("auth.mode", 1, false),
    ("auth.peer_info", 1, false),
    // Permanence compat layer
    ("permanence.commit_session", 5, false),
    ("permanence.verify_commit", 2, false),
    ("permanence.get_commit", 1, false),
    ("permanence.health_check", 1, false),
];

/// Protocols supported by this primal.
pub const PROTOCOLS: &[&str] = &["jsonrpc", "tarpc"];

/// Storage backends available.
pub const STORAGE_BACKENDS: &[(&str, bool)] = &[("redb", true), ("memory", true)];

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests {
    use super::*;

    #[test]
    fn primal_id_matches_convention() {
        assert!(!PRIMAL_ID.is_empty());
        assert!(PRIMAL_ID.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn methods_all_contain_dot() {
        for method in METHODS {
            assert!(
                method.contains('.'),
                "method {method} must be domain.operation format"
            );
        }
    }

    #[test]
    fn semantic_mappings_target_valid_methods() {
        for (_, method) in SEMANTIC_MAPPINGS {
            assert!(
                METHODS.contains(method),
                "mapping target {method} not in METHODS"
            );
        }
    }

    #[test]
    fn cost_estimates_cover_key_methods() {
        for (method, _, _) in COST_ESTIMATES {
            assert!(
                METHODS.contains(method),
                "cost estimate for {method} not in METHODS"
            );
        }
    }

    #[test]
    fn all_dependencies_are_optional() {
        for (_, required, _) in DEPENDENCIES {
            assert!(
                !required,
                "LoamSpine is self-contained; all deps should be optional"
            );
        }
    }

    #[test]
    fn redb_is_default_storage() {
        let default = STORAGE_BACKENDS.iter().find(|(name, _)| *name == "redb");
        assert!(default.is_some());
        assert!(default.unwrap().1, "redb should be default-enabled");
    }
}
