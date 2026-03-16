// SPDX-License-Identifier: AGPL-3.0-or-later

//! Primal name constants — single source of truth for IPC identifiers.
//!
//! All primal names used in IPC discovery, registration, and capability
//! routing are defined here. No hardcoded primal name strings elsewhere
//! in library code. LoamSpine discovers other primals at runtime via
//! capability-based discovery; these constants are identifiers, not
//! assumptions about what is running.

/// This primal's canonical identifier.
pub const SELF_ID: &str = "loamspine";

/// biomeOS orchestrator.
pub const BIOMEOS: &str = "biomeos";

/// `Songbird` discovery mesh.
pub const SONGBIRD: &str = "songbird";

/// `NestGate` content-addressed storage.
pub const NESTGATE: &str = "nestgate";

/// `BearDog` security foundation.
pub const BEARDOG: &str = "beardog";

/// `ToadStool` compute orchestrator.
pub const TOADSTOOL: &str = "toadstool";

/// `coralReef` sovereign shader compiler.
pub const CORALREEF: &str = "coralreef";

/// `rhizoCrypt` ephemeral DAG.
pub const RHIZOCRYPT: &str = "rhizocrypt";

/// `sweetGrass` semantic attribution.
pub const SWEETGRASS: &str = "sweetgrass";

/// `Squirrel` AI assistant.
pub const SQUIRREL: &str = "squirrel";

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
    fn all_names_are_lowercase_ascii() {
        for name in [
            SELF_ID, BIOMEOS, SONGBIRD, NESTGATE, BEARDOG, TOADSTOOL, CORALREEF, RHIZOCRYPT,
            SWEETGRASS, SQUIRREL,
        ] {
            assert!(
                name.chars().all(|c| c.is_ascii_lowercase()),
                "{name} must be lowercase ASCII"
            );
        }
    }

    #[test]
    fn socket_dir_matches_convention() {
        assert_eq!(BIOMEOS_SOCKET_DIR, "biomeos");
    }
}
