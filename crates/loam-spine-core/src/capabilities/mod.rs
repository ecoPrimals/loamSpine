// SPDX-License-Identifier: AGPL-3.0-or-later

//! Capability definitions for LoamSpine
//!
//! This module defines capabilities in the ecoPrimals ecosystem using
//! **capability-based discovery** instead of hardcoded primal names.
//!
//! ## Philosophy: Infant Discovery
//!
//! Each primal starts with ZERO knowledge of other primals and discovers
//! services at runtime based on **what they do** (capabilities), not **who they are** (names).
//!
//! ## Examples
//!
//! ```rust,no_run
//! use loam_spine_core::capabilities::*;
//! use loam_spine_core::infant_discovery::InfantDiscovery;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // LoamSpine advertises its capabilities
//! let my_capabilities = vec![
//!     LoamSpineCapability::PermanentLedger {
//!         entry_types: vec!["generic".into(), "certificate".into()],
//!         max_spine_size: None,
//!         supports_sealing: true,
//!     },
//!     LoamSpineCapability::CertificateAuthority {
//!         cert_types: vec!["loam".into()],
//!         supports_revocation: true,
//!         supports_lending: true,
//!     },
//! ];
//!
//! // LoamSpine discovers services by capability (not by name!)
//! // "Who can sign?" not "Where is BearDog?"
//! let discovery = InfantDiscovery::new()?;
//! let signing_services = discovery
//!     .find_capability(identifiers::external::SIGNING)
//!     .await?;
//! # Ok(())
//! # }
//! ```

mod parser;
mod types;

pub use parser::{extract_capabilities, CapabilityMethod, ParsedCapabilities};
pub use types::{
    DiscoveredService, ExternalCapability, LoamSpineCapability, ServiceHealth,
};

/// Capability identifier constants
pub mod identifiers {
    /// LoamSpine's own capabilities
    pub mod loamspine {
        /// Provides permanent, immutable ledger storage
        pub const PERMANENT_LEDGER: &str = "permanent-ledger";
        /// Issues and manages certificates
        pub const CERTIFICATE_AUTHORITY: &str = "certificate-authority";
        /// Generates cryptographic proofs
        pub const PROOF_GENERATION: &str = "proof-generation";
        /// Tracks universal temporal moments
        pub const TEMPORAL_TRACKING: &str = "temporal-tracking";
        /// Provides waypoint anchoring for data journeys
        pub const WAYPOINT_ANCHORING: &str = "waypoint-anchoring";
        /// Provides public chain anchoring for external provenance verification
        pub const PUBLIC_ANCHORING: &str = "public-anchoring";

        /// Canonical capability set for service advertisement and discovery.
        ///
        /// Used by lifecycle manager, health checks, and infant discovery
        /// to advertise our high-level capabilities to the ecosystem.
        pub const ADVERTISED: &[&str] = &[
            PERMANENT_LEDGER,
            WAYPOINT_ANCHORING,
            CERTIFICATE_AUTHORITY,
            PROOF_GENERATION,
            TEMPORAL_TRACKING,
            PUBLIC_ANCHORING,
        ];
    }

    /// External capabilities we discover and consume
    pub mod external {
        /// Cryptographic signing service
        pub const SIGNING: &str = "cryptographic-signing";
        /// Content-addressable storage
        pub const STORAGE: &str = "content-storage";
        /// Service discovery and registration (universal adapter)
        pub const DISCOVERY: &str = "service-discovery";
        /// Session and workflow management
        pub const SESSION_MANAGEMENT: &str = "session-management";
        /// Compute orchestration and execution
        pub const COMPUTE: &str = "compute-orchestration";
        /// Operation attestation for waypoint semantics
        pub const ATTESTATION: &str = "attestation";
        /// Chain anchor submission (blockchain/data-commons write access)
        pub const CHAIN_ANCHOR: &str = "chain-anchor";
    }
}

// Re-export for convenience
pub use identifiers::*;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
