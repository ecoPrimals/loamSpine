// SPDX-License-Identifier: AGPL-3.0-or-later

//! Primal name constants — single source of truth for IPC identifiers.
//!
//! LoamSpine follows the **self-knowledge only** philosophy: it knows its own
//! identity and discovers other primals at runtime via capability-based
//! discovery. No other primal names are hardcoded here.

/// This primal's canonical identifier.
pub const SELF_ID: &str = "loamspine";

/// biomeOS orchestrator identifier (used for socket/IPC paths).
pub const BIOMEOS: &str = "biomeos";

/// Socket directory name for biomeOS IPC mesh.
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
    fn socket_dir_matches_convention() {
        assert_eq!(BIOMEOS_SOCKET_DIR, "biomeos");
    }
}
