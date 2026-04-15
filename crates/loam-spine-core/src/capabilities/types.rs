// SPDX-License-Identifier: AGPL-3.0-or-later

//! Capability type definitions for LoamSpine and external services.

use super::identifiers::{external, loamspine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    /// Public chain anchoring for external provenance verification
    PublicAnchoring {
        /// Supported anchor target types
        anchor_targets: Vec<String>,
        /// Whether verification of anchors is supported
        supports_verification: bool,
    },

    /// Bond ledger persistence for cross-primal ionic bond contracts.
    BondLedger {
        /// Whether the ledger is append-only (immutable history).
        append_only: bool,
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ServiceHealth {
    /// Service is healthy and responding
    Healthy,
    /// Service is degraded but functional
    Degraded,
    /// Service is unreachable
    Unreachable,
    /// Health status unknown
    #[default]
    Unknown,
}

impl LoamSpineCapability {
    /// Get the capability identifier for this capability
    #[must_use]
    pub const fn identifier(&self) -> &'static str {
        match self {
            Self::PermanentLedger { .. } => loamspine::PERMANENT_LEDGER,
            Self::CertificateAuthority { .. } => loamspine::CERTIFICATE_AUTHORITY,
            Self::ProofGeneration { .. } => loamspine::PROOF_GENERATION,
            Self::TemporalTracking { .. } => loamspine::TEMPORAL_TRACKING,
            Self::WaypointAnchoring { .. } => loamspine::WAYPOINT_ANCHORING,
            Self::PublicAnchoring { .. } => loamspine::PUBLIC_ANCHORING,
            Self::BondLedger { .. } => loamspine::BOND_LEDGER,
        }
    }

    /// Introspect our own capabilities — the only hardcoded self-knowledge.
    #[must_use]
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
                max_spine_size: None,
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
            Self::PublicAnchoring {
                anchor_targets: vec![
                    "bitcoin".into(),
                    "ethereum".into(),
                    "federated-spine".into(),
                    "data-commons".into(),
                ],
                supports_verification: true,
            },
            Self::BondLedger { append_only: true },
        ]
    }
}

impl ExternalCapability {
    /// Get the capability identifier for this capability
    #[must_use]
    pub const fn identifier(&self) -> &'static str {
        match self {
            Self::Signing { .. } => external::SIGNING,
            Self::Storage { .. } => external::STORAGE,
            Self::Discovery { .. } => external::DISCOVERY,
            Self::SessionManagement { .. } => external::SESSION_MANAGEMENT,
            Self::Compute { .. } => external::COMPUTE,
        }
    }
}
