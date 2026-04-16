// SPDX-License-Identifier: AGPL-3.0-or-later

//! Primal name constants — single source of truth for IPC identifiers.
//!
//! LoamSpine follows the **self-knowledge only** philosophy: it knows its own
//! identity and discovers other primals at runtime via capability-based
//! discovery. No other primal names are hardcoded here.

/// This primal's canonical identifier.
pub const SELF_ID: &str = "loamspine";

/// Primary capability domain — used for socket naming per
/// `PRIMAL_SELF_KNOWLEDGE_STANDARD.md` §3 Socket Naming Convention.
pub const DOMAIN: &str = "permanence";

/// Capability-domain stem for ecosystem socket routing.
///
/// Other primals discover us via `ledger.sock` under the runtime IPC path segment [`BIOMEOS_SOCKET_DIR`],
/// matching `by_capability = "ledger"` in ecosystem deploy graphs.
pub const CAPABILITY_DOMAIN: &str = "ledger";

/// Ecosystem orchestrator identifier for socket/IPC paths.
pub const BIOMEOS: &str = "biomeos";

/// Socket directory name for the ecosystem IPC mesh.
pub const BIOMEOS_SOCKET_DIR: &str = "biomeos";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_id_is_lowercase() {
        assert!(SELF_ID.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn biomeos_is_lowercase() {
        assert!(BIOMEOS.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn domain_is_lowercase() {
        assert!(DOMAIN.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn domain_is_permanence() {
        assert_eq!(DOMAIN, "permanence");
    }

    #[test]
    fn capability_domain_is_lowercase() {
        assert!(CAPABILITY_DOMAIN.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn capability_domain_is_ledger() {
        assert_eq!(CAPABILITY_DOMAIN, "ledger");
    }

    #[test]
    fn socket_dir_matches_convention() {
        assert_eq!(BIOMEOS_SOCKET_DIR, "biomeos");
    }
}
