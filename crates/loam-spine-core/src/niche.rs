// SPDX-License-Identifier: AGPL-3.0-or-later

//! Primal self-knowledge for LoamSpine.
//!
//! LoamSpine is a **primal** — a first-class service in the ecoPrimals
//! ecosystem. It provides permanence, certificates, and proofs as its
//! core capability domains.
//!
//! This module holds the primal's self-knowledge:
//! - Identity (who am I?)
//! - Capabilities (what do I expose via biomeOS?)
//! - Semantic mappings (capability domain → JSON-RPC methods)
//! - Dependencies (what capabilities do I consume?)
//! - Operation costs (scheduling hints for biomeOS)
//!
//! Other modules (`neural_api`, `capabilities`, `service`) reference
//! these constants rather than duplicating string literals. LoamSpine
//! only knows itself — it discovers other primals at runtime.

/// Primal identity — used in all JSON-RPC, provenance, and IPC interactions.
pub const PRIMAL_ID: &str = crate::primal_names::SELF_ID;

/// Human-readable primal description for biomeOS registration.
pub const PRIMAL_DESCRIPTION: &str =
    "Permanence layer providing selective memory, certificates, and inclusion proofs";

/// Primal category for biomeOS deployment.
pub const PRIMAL_CATEGORY: &str = "infrastructure";

/// All capability domains this primal exposes.
pub const DOMAINS: &[&str] = &[
    "spine",
    "entry",
    "certificate",
    "proof",
    "waypoint",
    "anchor",
    "health",
    "meta",
    "integration",
];

/// All JSON-RPC methods this primal exposes to biomeOS.
///
/// Each string is a fully qualified method name (`{domain}.{operation}`)
/// that biomeOS can route via `capability.call`.
pub const METHODS: &[&str] = &[
    "spine.create",
    "spine.get",
    "spine.seal",
    "entry.append",
    "entry.get",
    "entry.get_tip",
    "certificate.mint",
    "certificate.transfer",
    "certificate.loan",
    "certificate.return",
    "certificate.get",
    "certificate.verify",
    "certificate.lifecycle",
    "slice.anchor",
    "slice.checkout",
    "slice.record_operation",
    "slice.depart",
    "proof.generate_inclusion",
    "proof.verify_inclusion",
    "session.commit",
    "braid.commit",
    "anchor.publish",
    "anchor.verify",
    "health.check",
    "health.liveness",
    "health.readiness",
    "capability.list",
    "tools.list",
    "tools.call",
];

/// Semantic mappings: short operation name → fully qualified method.
///
/// biomeOS uses these during domain registration so
/// `capability.call { domain: "spine", operation: "create" }`
/// routes to `spine.create` on our socket.
pub const SEMANTIC_MAPPINGS: &[(&str, &str)] = &[
    ("create_spine", "spine.create"),
    ("get_spine", "spine.get"),
    ("seal_spine", "spine.seal"),
    ("append_entry", "entry.append"),
    ("get_entry", "entry.get"),
    ("get_tip", "entry.get_tip"),
    ("mint_certificate", "certificate.mint"),
    ("transfer_certificate", "certificate.transfer"),
    ("loan_certificate", "certificate.loan"),
    ("return_certificate", "certificate.return"),
    ("get_certificate", "certificate.get"),
    ("verify_certificate", "certificate.verify"),
    ("certificate_lifecycle", "certificate.lifecycle"),
    ("anchor_slice", "slice.anchor"),
    ("checkout_slice", "slice.checkout"),
    ("record_operation", "slice.record_operation"),
    ("depart_slice", "slice.depart"),
    ("generate_inclusion_proof", "proof.generate_inclusion"),
    ("verify_inclusion_proof", "proof.verify_inclusion"),
    ("commit_session", "session.commit"),
    ("commit_braid", "braid.commit"),
    ("publish_anchor", "anchor.publish"),
    ("verify_anchor", "anchor.verify"),
    ("health_check", "health.check"),
    ("capability_list", "capability.list"),
    ("tools_list", "tools.list"),
    ("tools_call", "tools.call"),
];

/// Consumed capabilities — what LoamSpine calls on other primals.
///
/// LoamSpine discovers these at runtime via capability-based discovery;
/// it never hardcodes which primal provides them.
pub const CONSUMED_CAPABILITIES: &[&str] = &[
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
        crate::capabilities::identifiers::external::SIGNING,
        false,
        "external signature verification (graceful degradation to CLI signer)",
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

/// Cost estimates for biomeOS scheduling.
///
/// Each entry: `(method, estimated_ms, gpu_beneficial)`.
pub const COST_ESTIMATES: &[(&str, u32, bool)] = &[
    ("spine.create", 1, false),
    ("spine.get", 1, false),
    ("spine.seal", 1, false),
    ("entry.append", 2, false),
    ("entry.get", 1, false),
    ("entry.get_tip", 1, false),
    ("certificate.mint", 3, false),
    ("certificate.transfer", 2, false),
    ("certificate.loan", 2, false),
    ("certificate.return", 2, false),
    ("certificate.get", 1, false),
    ("certificate.verify", 5, false),
    ("certificate.lifecycle", 3, false),
    ("slice.anchor", 2, false),
    ("slice.checkout", 1, false),
    ("proof.generate_inclusion", 10, false),
    ("proof.verify_inclusion", 5, false),
    ("session.commit", 5, false),
    ("braid.commit", 5, false),
    ("anchor.publish", 2, false),
    ("anchor.verify", 2, false),
    ("health.check", 1, false),
    ("capability.list", 1, false),
    ("tools.list", 1, false),
    ("tools.call", 5, false),
];

/// Protocols supported by this primal.
pub const PROTOCOLS: &[&str] = &["jsonrpc", "tarpc"];

/// Storage backends available.
pub const STORAGE_BACKENDS: &[(&str, bool)] = &[
    ("redb", true),
    ("memory", true),
    ("sled", true),
    ("sqlite", false),
];

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
