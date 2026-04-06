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
        }
    }

    /// Introspect our own capabilities
    ///
    /// This is the ONLY hardcoded knowledge - what WE can do.
    /// Everything else is discovered at runtime.
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
            Self::PublicAnchoring {
                anchor_targets: vec![
                    "bitcoin".into(),
                    "ethereum".into(),
                    "federated-spine".into(),
                    "data-commons".into(),
                ],
                supports_verification: true,
            },
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

// ──────────────────────────────────────────────────────────────────────────────
// extract_capabilities — parse partner capability.list responses
// ──────────────────────────────────────────────────────────────────────────────

/// Information about a single capability method from a partner primal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityMethod {
    /// Method name (e.g. `"spine.create"`).
    pub method: String,
    /// Domain (e.g. `"spine"`, `"certificate"`).
    pub domain: Option<String>,
    /// Cost tier (e.g. `"low"`, `"medium"`, `"high"`).
    pub cost: Option<String>,
    /// Dependencies (other methods that must be called first).
    pub deps: Vec<String>,
}

/// Parsed capability list from a partner primal's `capability.list` response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCapabilities {
    /// Primal name.
    pub primal: Option<String>,
    /// Primal version.
    pub version: Option<String>,
    /// Flat capability strings (e.g. `["permanence", "commit.session"]`).
    pub capabilities: Vec<String>,
    /// Structured method descriptors (if `methods` array is present).
    pub methods: Vec<CapabilityMethod>,
}

/// Parse a `capability.list` JSON-RPC response from any primal.
///
/// Supports 4 formats used across the ecosystem:
/// 1. Flat array: `{ "capabilities": ["a", "b"] }`
/// 2. Object array: `{ "methods": [{ "method": "a", "domain": "x" }] }`
/// 3. Nested domains: `{ "domains": { "spine": ["create", "get"] } }`
/// 4. Combined: flat `capabilities` + structured `methods`
///
/// Aligns with wetSpring V125 / airSpring v0.8.7 `parse_capabilities()`.
#[must_use]
pub fn extract_capabilities(response: &serde_json::Value) -> ParsedCapabilities {
    let primal = response
        .get("primal")
        .and_then(serde_json::Value::as_str)
        .map(String::from);
    let version = response
        .get("version")
        .and_then(serde_json::Value::as_str)
        .map(String::from);

    let mut capabilities = Vec::new();
    let mut methods = Vec::new();

    if let Some(caps) = response
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
    {
        for cap in caps {
            if let Some(s) = cap.as_str() {
                capabilities.push(s.to_string());
            }
        }
    }

    if let Some(meths) = response
        .get("methods")
        .and_then(serde_json::Value::as_array)
    {
        for m in meths {
            let Some(method_name) = m.get("method").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let domain = m
                .get("domain")
                .and_then(serde_json::Value::as_str)
                .map(String::from);
            let cost = m
                .get("cost")
                .and_then(serde_json::Value::as_str)
                .map(String::from);
            let deps = m
                .get("deps")
                .and_then(serde_json::Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            methods.push(CapabilityMethod {
                method: method_name.to_string(),
                domain,
                cost,
                deps,
            });
        }
    }

    if let Some(domains) = response
        .get("domains")
        .and_then(serde_json::Value::as_object)
    {
        for (domain_name, methods_val) in domains {
            if let Some(arr) = methods_val.as_array() {
                for m in arr {
                    if let Some(method_str) = m.as_str() {
                        let full_method = format!("{domain_name}.{method_str}");
                        if !capabilities.contains(&full_method) {
                            capabilities.push(full_method);
                        }
                    }
                }
            }
        }
    }

    ParsedCapabilities {
        primal,
        version,
        capabilities,
        methods,
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
        assert!(capabilities.len() >= 6);

        // Verify we have core capabilities
        assert!(
            capabilities
                .iter()
                .any(|c| matches!(c, LoamSpineCapability::PermanentLedger { .. }))
        );
        assert!(
            capabilities
                .iter()
                .any(|c| matches!(c, LoamSpineCapability::TemporalTracking { .. }))
        );
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

    #[test]
    fn extract_flat_capabilities() {
        let response = serde_json::json!({
            "primal": "rhizoCrypt",
            "version": "0.13.0",
            "capabilities": ["signing", "verification", "key-management"],
        });
        let parsed = extract_capabilities(&response);
        assert_eq!(parsed.primal.as_deref(), Some("rhizoCrypt"));
        assert_eq!(parsed.version.as_deref(), Some("0.13.0"));
        assert_eq!(
            parsed.capabilities,
            vec!["signing", "verification", "key-management"]
        );
        assert!(parsed.methods.is_empty());
    }

    #[test]
    fn extract_methods_with_domain_cost_deps() {
        let response = serde_json::json!({
            "primal": "loamSpine",
            "methods": [
                { "method": "spine.create", "domain": "spine", "cost": "low", "deps": [] },
                { "method": "entry.append", "domain": "entry", "cost": "low", "deps": ["spine.create"] },
            ],
        });
        let parsed = extract_capabilities(&response);
        assert_eq!(parsed.methods.len(), 2);
        assert_eq!(parsed.methods[1].method, "entry.append");
        assert_eq!(parsed.methods[1].domain.as_deref(), Some("entry"));
        assert_eq!(parsed.methods[1].deps, vec!["spine.create"]);
    }

    #[test]
    fn extract_nested_domains() {
        let response = serde_json::json!({
            "domains": {
                "spine": ["create", "get", "seal"],
                "entry": ["append", "get"],
            },
        });
        let parsed = extract_capabilities(&response);
        assert!(parsed.capabilities.contains(&"spine.create".to_string()));
        assert!(parsed.capabilities.contains(&"entry.append".to_string()));
        assert_eq!(parsed.capabilities.len(), 5);
    }

    #[test]
    fn extract_combined_format() {
        let response = serde_json::json!({
            "primal": "sweetGrass",
            "version": "0.7.19",
            "capabilities": ["attribution", "braid.create"],
            "methods": [
                { "method": "braid.create", "domain": "braid", "cost": "medium", "deps": [] },
            ],
        });
        let parsed = extract_capabilities(&response);
        assert_eq!(parsed.capabilities.len(), 2);
        assert_eq!(parsed.methods.len(), 1);
        assert_eq!(parsed.primal.as_deref(), Some("sweetGrass"));
    }

    #[test]
    fn extract_empty_response() {
        let response = serde_json::json!({});
        let parsed = extract_capabilities(&response);
        assert!(parsed.primal.is_none());
        assert!(parsed.capabilities.is_empty());
        assert!(parsed.methods.is_empty());
    }
}
