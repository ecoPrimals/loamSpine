// SPDX-License-Identifier: AGPL-3.0-only

//! Waypoint types and policies.
//!
//! Waypoints provide local permanence for borrowed state.  When a slice is
//! loaned, the borrower anchors it on a waypoint spine that records local
//! operations without propagating upward to the origin.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::types::{EntryHash, SliceId, SpineId, Timestamp};

// ============================================================================
// Configuration
// ============================================================================

/// Waypoint spine configuration.
///
/// Controls anchor acceptance, depth limits, origin filtering, and
/// propagation on return.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaypointConfig {
    /// Accept anchors from external spines.
    pub accept_anchors: bool,

    /// Maximum concurrent anchored slices.
    pub max_anchored_slices: Option<usize>,

    /// Maximum anchor depth for relending.
    /// `0` = cannot relend, `1` = can relend once.
    pub max_anchor_depth: Option<u32>,

    /// Allowed origin spines (`None` = any).
    pub allowed_origins: Option<Vec<SpineId>>,

    /// Forbidden origin spines.
    pub forbidden_origins: Vec<SpineId>,

    /// What data is propagated back to the origin spine on return.
    pub propagation_policy: PropagationPolicy,

    /// Auto-return on expiry.
    pub auto_return_expired: bool,

    /// Grace period (seconds) before forced return after expiry.
    pub expiry_grace_secs: u64,
}

impl Default for WaypointConfig {
    fn default() -> Self {
        Self {
            accept_anchors: true,
            max_anchored_slices: Some(100),
            max_anchor_depth: Some(2),
            allowed_origins: None,
            forbidden_origins: Vec::new(),
            propagation_policy: PropagationPolicy::default(),
            auto_return_expired: true,
            expiry_grace_secs: 3600,
        }
    }
}

// ============================================================================
// Propagation
// ============================================================================

/// What data is propagated back to the origin spine when a slice returns.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum PropagationPolicy {
    /// Never propagate anything.
    Never,

    /// Propagate only a summary (operation count, duration, hash).
    #[default]
    SummaryOnly,

    /// Propagate only specific operation types.
    Selective {
        /// Operation type names allowed to propagate.
        allowed_types: Vec<String>,
    },

    /// Full propagation (requires owner consent).
    Full {
        /// Whether the owner must sign the full propagation.
        require_owner_signature: bool,
    },
}

// ============================================================================
// Departure / Return
// ============================================================================

/// Reasons for slice departure from a waypoint.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DepartureReason {
    /// Loan term expired.
    Expired,
    /// Borrower manually returned.
    ManualReturn,
    /// Owner recalled the slice.
    OwnerRecall,
    /// Relent to another waypoint.
    Relend {
        /// Target waypoint spine.
        target_waypoint: SpineId,
    },
    /// Administrative action.
    Administrative {
        /// Explanation.
        reason: String,
    },
}

impl std::fmt::Display for DepartureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expired => write!(f, "expired"),
            Self::ManualReturn => write!(f, "manual_return"),
            Self::OwnerRecall => write!(f, "owner_recall"),
            Self::Relend { target_waypoint } => {
                write!(f, "relend:{target_waypoint}")
            }
            Self::Administrative { reason } => write!(f, "admin:{reason}"),
        }
    }
}

/// Summary of waypoint usage returned to the origin spine on departure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaypointSummary {
    /// Slice that was anchored.
    pub slice_id: SliceId,

    /// Total time anchored (nanoseconds).
    pub duration_nanos: u64,

    /// Number of operations performed.
    pub operation_count: u64,

    /// Distinct operation type names recorded.
    pub operation_types: Vec<String>,

    /// First operation timestamp.
    pub first_operation: Option<Timestamp>,

    /// Last operation timestamp.
    pub last_operation: Option<Timestamp>,

    /// BLAKE3 hash of the full operation log for verification.
    pub operations_hash: EntryHash,

    /// Whether the slice was relent to another waypoint.
    pub was_relent: bool,

    /// Maximum relend depth that was reached.
    pub max_relend_depth: u32,
}

// ============================================================================
// Operation types
// ============================================================================

/// Types of operations that can be recorded on an anchored slice.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SliceOperationType {
    /// Generic "use" action.
    Use {
        /// What the borrower did.
        action: String,
        /// Duration in seconds.
        duration_secs: Option<u64>,
    },
    /// Passive viewing.
    View {
        /// Optional viewport label.
        viewport: Option<String>,
    },
    /// Read operation (e.g., pages of a document).
    Read {
        /// Number of pages read.
        pages: Option<usize>,
    },
    /// Edit operation.
    Edit {
        /// Type of edit (insert, delete, replace …).
        operation_type: String,
    },
    /// Export to another format.
    Export {
        /// Target format (pdf, epub, …).
        format: String,
    },
    /// Arbitrary domain-specific operation.
    Custom {
        /// Operation name.
        operation_name: String,
    },
}

impl SliceOperationType {
    /// Canonical short name for the operation.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Use { .. } => "use",
            Self::View { .. } => "view",
            Self::Read { .. } => "read",
            Self::Edit { .. } => "edit",
            Self::Export { .. } => "export",
            Self::Custom { operation_name, .. } => operation_name,
        }
    }
}

// ============================================================================
// Slice terms
// ============================================================================

/// Terms governing how a slice can be used at a waypoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SliceTerms {
    /// Loan duration in seconds (`None` = until manual return).
    pub duration_secs: Option<u64>,

    /// Allowed operations (`None` = any that are not forbidden).
    pub allowed_operations: Option<HashSet<String>>,

    /// Explicitly forbidden operations (checked before allowed list).
    pub forbidden_operations: HashSet<String>,

    /// Whether this slice may be relent.
    pub allow_relend: bool,

    /// Maximum relend depth from this anchor point.
    pub max_relend_depth: Option<u32>,

    /// What gets propagated back on return.
    pub propagation: PropagationPolicy,
}

impl Default for SliceTerms {
    fn default() -> Self {
        Self {
            duration_secs: None,
            allowed_operations: None,
            forbidden_operations: HashSet::new(),
            allow_relend: false,
            max_relend_depth: Some(0),
            propagation: PropagationPolicy::SummaryOnly,
        }
    }
}

impl SliceTerms {
    /// Check if an operation name is allowed by these terms.
    #[must_use]
    pub fn is_operation_allowed(&self, op_name: &str) -> bool {
        if self.forbidden_operations.contains(op_name) {
            return false;
        }
        if let Some(allowed) = &self.allowed_operations {
            return allowed.contains(op_name);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_waypoint_config() {
        let config = WaypointConfig::default();
        assert!(config.accept_anchors);
        assert_eq!(config.max_anchored_slices, Some(100));
        assert_eq!(config.max_anchor_depth, Some(2));
        assert!(config.auto_return_expired);
        assert_eq!(config.propagation_policy, PropagationPolicy::SummaryOnly);
    }

    #[test]
    fn propagation_policy_default() {
        let policy = PropagationPolicy::default();
        assert_eq!(policy, PropagationPolicy::SummaryOnly);
    }

    #[test]
    fn departure_reason_display() {
        assert_eq!(DepartureReason::Expired.to_string(), "expired");
        assert_eq!(DepartureReason::ManualReturn.to_string(), "manual_return");
        assert_eq!(DepartureReason::OwnerRecall.to_string(), "owner_recall");
        assert_eq!(
            DepartureReason::Administrative {
                reason: "cleanup".into()
            }
            .to_string(),
            "admin:cleanup"
        );
    }

    #[test]
    fn slice_operation_type_name() {
        assert_eq!(
            SliceOperationType::Use {
                action: "play".into(),
                duration_secs: None,
            }
            .name(),
            "use"
        );
        assert_eq!(
            SliceOperationType::Custom {
                operation_name: "forge".into(),
            }
            .name(),
            "forge"
        );
    }

    #[test]
    fn slice_terms_operation_allowed() {
        let terms = SliceTerms {
            forbidden_operations: ["export"].iter().map(|s| (*s).to_string()).collect(),
            ..SliceTerms::default()
        };
        assert!(terms.is_operation_allowed("use"));
        assert!(!terms.is_operation_allowed("export"));
    }

    #[test]
    fn slice_terms_allowed_list() {
        let terms = SliceTerms {
            allowed_operations: Some(["read", "view"].iter().map(|s| (*s).to_string()).collect()),
            ..SliceTerms::default()
        };
        assert!(terms.is_operation_allowed("read"));
        assert!(!terms.is_operation_allowed("edit"));
    }

    #[test]
    fn waypoint_summary_serde_roundtrip() {
        let summary = WaypointSummary {
            slice_id: crate::types::SliceId::now_v7(),
            duration_nanos: 1_000_000_000,
            operation_count: 5,
            operation_types: vec!["use".into(), "view".into()],
            first_operation: Some(Timestamp::now()),
            last_operation: Some(Timestamp::now()),
            operations_hash: [0u8; 32],
            was_relent: false,
            max_relend_depth: 0,
        };
        let json = serde_json::to_string(&summary).expect("serialize");
        let restored: WaypointSummary = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.operation_count, 5);
        assert_eq!(restored.operation_types.len(), 2);
    }

    #[test]
    fn waypoint_config_serde_roundtrip() {
        let config = WaypointConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let restored: WaypointConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.max_anchor_depth, Some(2));
    }

    #[test]
    fn slice_terms_serde_roundtrip() {
        let terms = SliceTerms::default();
        let json = serde_json::to_string(&terms).expect("serialize");
        let restored: SliceTerms = serde_json::from_str(&json).expect("deserialize");
        assert!(!restored.allow_relend);
    }
}
