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
//! let discovery = InfantDiscovery::new().await?;
//! let signing_services = discovery
//!     .find_capability(identifiers::external::SIGNING)
//!     .await?;
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    }
}

// Re-export for convenience
pub use identifiers::*;

/// LoamSpine's capabilities that we provide to the ecosystem
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum LoamSpineCapability {
    /// Permanent ledger storage
    PermanentLedger {
        /// Supported entry types
        entry_types: Vec<String>,
        /// Maximum spine size (None = unlimited)
        max_spine_size: Option<usize>,
        /// Supports spine sealing
        supports_sealing: bool,
    },

    /// Certificate authority
    CertificateAuthority {
        /// Types of certificates we can issue
        cert_types: Vec<String>,
        /// Supports certificate revocation
        supports_revocation: bool,
        /// Supports certificate lending (waypoints)
        supports_lending: bool,
    },

    /// Cryptographic proof generation
    ProofGeneration {
        /// Types of proofs (inclusion, exclusion, etc.)
        proof_types: Vec<String>,
        /// Cryptographic algorithms used
        algorithms: Vec<String>,
    },

    /// Temporal moment tracking
    TemporalTracking {
        /// Types of moments we track
        moment_types: Vec<String>,
        /// Types of anchors we support
        anchor_types: Vec<String>,
    },

    /// Waypoint anchoring for data journeys
    WaypointAnchoring {
        /// Tracks slice arrivals, operations, departures
        supports_journey_tracking: bool,
        /// Permanent audit trail
        immutable_history: bool,
    },
}

/// External capabilities we discover and consume
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExternalCapability {
    /// Cryptographic signing (e.g., Ed25519, RSA)
    Signing {
        /// Supported algorithms
        algorithms: Vec<String>,
        /// Supported key types
        key_types: Vec<String>,
        /// Hardware security module support
        hsm_support: bool,
    },

    /// Content-addressable storage
    Storage {
        /// Content-addressable (hash-based)
        content_addressable: bool,
        /// Maximum content size
        max_size: Option<usize>,
        /// Supports replication
        supports_replication: bool,
    },

    /// Service discovery (universal adapter)
    Discovery {
        /// Discovery protocols (mDNS, DNS-SD, etc.)
        protocols: Vec<String>,
        /// Supports multi-site federation
        supports_federation: bool,
        /// Capability-based routing
        capability_routing: bool,
    },

    /// Session and workflow management
    SessionManagement {
        /// Persistent sessions
        supports_persistence: bool,
        /// Multi-agent coordination
        multi_agent: bool,
    },

    /// Compute orchestration
    Compute {
        /// Resource types (CPU, GPU, etc.)
        resource_types: Vec<String>,
        /// Task scheduling
        supports_scheduling: bool,
    },
}

/// Discovered service with capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService {
    /// Unique service ID (not a primal name!)
    pub id: String,

    /// Capability this service provides
    pub capability: String,

    /// Service endpoint
    pub endpoint: String,

    /// How we discovered this service
    pub discovered_via: String,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// Service health status
    pub health: ServiceHealth,

    /// Discovery timestamp
    pub discovered_at: std::time::SystemTime,

    /// Time-to-live for cache
    pub ttl_secs: u64,
}

/// Service health status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceHealth {
    /// Service is healthy and responding
    Healthy,
    /// Service is degraded but functional
    Degraded,
    /// Service is unreachable
    Unreachable,
    /// Health status unknown
    Unknown,
}

impl LoamSpineCapability {
    /// Get the capability identifier for this capability
    pub fn identifier(&self) -> &'static str {
        match self {
            Self::PermanentLedger { .. } => loamspine::PERMANENT_LEDGER,
            Self::CertificateAuthority { .. } => loamspine::CERTIFICATE_AUTHORITY,
            Self::ProofGeneration { .. } => loamspine::PROOF_GENERATION,
            Self::TemporalTracking { .. } => loamspine::TEMPORAL_TRACKING,
            Self::WaypointAnchoring { .. } => loamspine::WAYPOINT_ANCHORING,
        }
    }

    /// Introspect our own capabilities
    ///
    /// This is the ONLY hardcoded knowledge - what WE can do.
    /// Everything else is discovered at runtime.
    pub fn introspect() -> Vec<Self> {
        vec![
            Self::PermanentLedger {
                entry_types: vec![
                    "generic".into(),
                    "certificate".into(),
                    "proof".into(),
                    "temporal_moment".into(),
                    "waypoint".into(),
                    "spine_reference".into(),
                ],
                max_spine_size: None, // Unlimited
                supports_sealing: true,
            },
            Self::CertificateAuthority {
                cert_types: vec!["loam".into()],
                supports_revocation: true,
                supports_lending: true,
            },
            Self::ProofGeneration {
                proof_types: vec!["inclusion".into(), "exclusion".into()],
                algorithms: vec!["sha256".into(), "blake3".into()],
            },
            Self::TemporalTracking {
                moment_types: vec![
                    "code_change".into(),
                    "art_creation".into(),
                    "life_event".into(),
                    "experiment".into(),
                    "milestone".into(),
                ],
                anchor_types: vec![
                    "atomic".into(),
                    "crypto".into(),
                    "causal".into(),
                    "consensus".into(),
                ],
            },
            Self::WaypointAnchoring {
                supports_journey_tracking: true,
                immutable_history: true,
            },
        ]
    }
}

impl ExternalCapability {
    /// Get the capability identifier for this capability
    pub fn identifier(&self) -> &'static str {
        match self {
            Self::Signing { .. } => external::SIGNING,
            Self::Storage { .. } => external::STORAGE,
            Self::Discovery { .. } => external::DISCOVERY,
            Self::SessionManagement { .. } => external::SESSION_MANAGEMENT,
            Self::Compute { .. } => external::COMPUTE,
        }
    }
}

impl Default for ServiceHealth {
    fn default() -> Self {
        Self::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_identifiers() {
        // Verify identifiers are lowercase-kebab-case
        assert_eq!(loamspine::PERMANENT_LEDGER, "permanent-ledger");
        assert_eq!(external::SIGNING, "cryptographic-signing");

        // Verify no spaces or underscores
        assert!(!loamspine::PERMANENT_LEDGER.contains(' '));
        assert!(!loamspine::PERMANENT_LEDGER.contains('_'));
    }

    #[test]
    fn test_introspection() {
        let capabilities = LoamSpineCapability::introspect();

        // We should know our own capabilities
        assert!(!capabilities.is_empty());
        assert!(capabilities.len() >= 5);

        // Verify we have core capabilities
        assert!(capabilities
            .iter()
            .any(|c| matches!(c, LoamSpineCapability::PermanentLedger { .. })));
        assert!(capabilities
            .iter()
            .any(|c| matches!(c, LoamSpineCapability::TemporalTracking { .. })));
    }

    #[test]
    fn test_capability_identifier_extraction() {
        let cap = LoamSpineCapability::PermanentLedger {
            entry_types: vec![],
            max_spine_size: None,
            supports_sealing: true,
        };

        assert_eq!(cap.identifier(), "permanent-ledger");
    }

    #[test]
    fn test_service_health_default() {
        let health = ServiceHealth::default();
        assert_eq!(health, ServiceHealth::Unknown);
    }
}
