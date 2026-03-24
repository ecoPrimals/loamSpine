// SPDX-License-Identifier: AGPL-3.0-or-later

//! Waypoint types and policies.
//!
//! Waypoints provide local permanence for borrowed state.  When a slice is
//! loaned, the borrower anchors it on a waypoint spine that records local
//! operations without propagating upward to the origin.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::error::{LoamSpineError, LoamSpineResult};
use crate::types::{Did, EntryHash, SliceId, SpineId, Timestamp};

// ============================================================================
// Configuration
// ============================================================================

/// Waypoint spine configuration.
///
/// Controls anchor acceptance, depth limits, origin filtering,
/// propagation on return, and attestation requirements.
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

    /// Attestation requirement for operations at this waypoint.
    ///
    /// When set to anything other than [`AttestationRequirement::None`],
    /// operations must be attested by a capability-discovered attestation
    /// provider (e.g. a primal offering `"attestation"` capability).
    #[serde(default)]
    pub operation_attestation: AttestationRequirement,
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
            operation_attestation: AttestationRequirement::default(),
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
// Attestation
// ============================================================================

/// Whether waypoint operations require external attestation.
///
/// Attestation is provided by a capability-discovered primal offering the
/// `"attestation"` capability (e.g. a Beardog-like primal). LoamSpine never
/// hard-codes the attesting primal's name — it discovers the provider at
/// runtime through the service registry.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttestationRequirement {
    /// No attestation required.
    #[default]
    None,

    /// Attestation required for anchor and depart operations only.
    BoundaryOnly,

    /// Attestation required for every operation at the waypoint.
    AllOperations,

    /// Attestation required for specific operation types.
    Selective {
        /// Operation type names that require attestation.
        operation_types: Vec<String>,
    },
}

impl AttestationRequirement {
    /// Whether any attestation is required.
    #[must_use]
    pub fn is_required(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Whether a specific operation type requires attestation.
    #[must_use]
    pub fn requires_for_operation(&self, operation: &str) -> bool {
        match self {
            Self::None => false,
            Self::BoundaryOnly => operation == "anchor" || operation == "depart",
            Self::AllOperations => true,
            Self::Selective { operation_types } => operation_types.iter().any(|t| t == operation),
        }
    }
}

/// Context passed to the attestation provider when requesting attestation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttestationContext {
    /// Operation being attested (e.g. "anchor", "depart", "use").
    pub operation: String,

    /// Waypoint spine where the operation occurs.
    pub waypoint_spine_id: SpineId,

    /// Slice being operated on.
    pub slice_id: SliceId,

    /// Optional caller DID (if known).
    pub caller: Option<Did>,
}

/// Attestation result from a capability-discovered attestation provider.
///
/// LoamSpine does not implement attestation itself — it consumes attestation
/// results from external primals discovered at runtime via their
/// `"attestation"` capability.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttestationResult {
    /// Whether the attestation was granted.
    pub attested: bool,

    /// DID of the attesting entity (discovered at runtime).
    pub attester: Did,

    /// Attestation timestamp.
    pub timestamp: Timestamp,

    /// Opaque attestation token for verification.
    pub token: Vec<u8>,

    /// Reason for denial, if applicable.
    pub denial_reason: Option<String>,
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

// ============================================================================
// Relending chain
// ============================================================================

/// A single link in a relending chain.
///
/// Each link represents one borrower in the chain from owner to current holder.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelendingLink {
    /// Borrower DID at this depth.
    pub borrower: Did,
    /// Loan entry hash that created this link.
    pub loan_entry: EntryHash,
}

/// Tracks the chain of borrowers when a certificate is sub-lent.
///
/// When owner loans to A, and A sub-lends to B, the chain records
/// [A, B]. Depth 0 = first borrower, depth 1 = second, etc.
/// Supports validation against `LoanTerms`, depth limits, and unwinding.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RelendingChain {
    /// Links in order from first borrower to current holder.
    pub links: Vec<RelendingLink>,
}

impl RelendingChain {
    /// Create an empty chain (no relending yet).
    #[must_use]
    pub fn new() -> Self {
        Self { links: Vec::new() }
    }

    /// Create a chain with the initial borrower (depth 0).
    #[must_use]
    pub fn with_initial(borrower: Did, loan_entry: EntryHash) -> Self {
        Self {
            links: vec![RelendingLink {
                borrower,
                loan_entry,
            }],
        }
    }

    /// Current depth (0 = first borrower only, 1 = one sublend, etc.).
    #[must_use]
    pub fn depth(&self) -> u32 {
        u32::try_from(self.links.len().saturating_sub(1)).unwrap_or(u32::MAX)
    }

    /// Current holder (last borrower in chain).
    #[must_use]
    pub fn current_holder(&self) -> Option<&Did> {
        self.links.last().map(|l| &l.borrower)
    }

    /// Root borrower (first in chain).
    #[must_use]
    pub fn root_borrower(&self) -> Option<&Did> {
        self.links.first().map(|l| &l.borrower)
    }

    /// Check if sublending is allowed given terms.
    ///
    /// Returns `Ok(())` if sublending is permitted, `Err` otherwise.
    ///
    /// # Errors
    ///
    /// Returns error if sublending is not allowed or depth limit exceeded.
    pub fn can_sublend(
        &self,
        allow_sublend: bool,
        max_sublend_depth: Option<u32>,
    ) -> LoamSpineResult<()> {
        if !allow_sublend {
            return Err(LoamSpineError::LoanTermsViolation(
                "sublending not allowed".into(),
            ));
        }
        let next_depth = u32::try_from(self.links.len()).unwrap_or(u32::MAX);
        if let Some(max) = max_sublend_depth
            && next_depth > max
        {
            return Err(LoamSpineError::LoanTermsViolation(format!(
                "sublend depth {next_depth} exceeds max {max}"
            )));
        }
        Ok(())
    }

    /// Add a sublend link. Validates via `can_sublend` first.
    ///
    /// # Errors
    ///
    /// Returns error if sublending is not allowed or depth limit exceeded.
    pub fn sublend(
        &mut self,
        borrower: Did,
        loan_entry: EntryHash,
        allow_sublend: bool,
        max_sublend_depth: Option<u32>,
    ) -> LoamSpineResult<()> {
        self.can_sublend(allow_sublend, max_sublend_depth)?;
        self.links.push(RelendingLink {
            borrower,
            loan_entry,
        });
        Ok(())
    }

    /// Unwind the chain by returning at the given borrower.
    ///
    /// Removes the borrower and all subsequent links. Returns the loan entries
    /// that were unwound (for recording returns).
    ///
    /// # Errors
    ///
    /// Returns error if borrower not found in chain.
    pub fn return_at(&mut self, borrower: &Did) -> LoamSpineResult<Vec<EntryHash>> {
        let pos = self
            .links
            .iter()
            .position(|l| l.borrower == *borrower)
            .ok_or_else(|| {
                LoamSpineError::LoanTermsViolation(format!(
                    "borrower {borrower} not in relending chain"
                ))
            })?;
        let unwound: Vec<EntryHash> = self.links[pos..].iter().map(|l| l.loan_entry).collect();
        self.links.truncate(pos);
        Ok(unwound)
    }

    /// Check if the given DID is in the chain.
    #[must_use]
    pub fn contains(&self, did: &Did) -> bool {
        self.links.iter().any(|l| l.borrower == *did)
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "tests use expect for conciseness")]
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
        assert_eq!(config.operation_attestation, AttestationRequirement::None);
    }

    #[test]
    fn attestation_requirement_default_is_none() {
        let req = AttestationRequirement::default();
        assert!(!req.is_required());
        assert!(!req.requires_for_operation("anchor"));
    }

    #[test]
    fn attestation_requirement_boundary_only() {
        let req = AttestationRequirement::BoundaryOnly;
        assert!(req.is_required());
        assert!(req.requires_for_operation("anchor"));
        assert!(req.requires_for_operation("depart"));
        assert!(!req.requires_for_operation("use"));
    }

    #[test]
    fn attestation_requirement_all_operations() {
        let req = AttestationRequirement::AllOperations;
        assert!(req.is_required());
        assert!(req.requires_for_operation("anchor"));
        assert!(req.requires_for_operation("use"));
        assert!(req.requires_for_operation("anything"));
    }

    #[test]
    fn attestation_requirement_selective() {
        let req = AttestationRequirement::Selective {
            operation_types: vec!["transfer".into(), "export".into()],
        };
        assert!(req.is_required());
        assert!(req.requires_for_operation("transfer"));
        assert!(req.requires_for_operation("export"));
        assert!(!req.requires_for_operation("view"));
    }

    #[test]
    fn attestation_requirement_serde_roundtrip() {
        let req = AttestationRequirement::Selective {
            operation_types: vec!["anchor".into()],
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let restored: AttestationRequirement = serde_json::from_str(&json).expect("deserialize");
        assert!(restored.requires_for_operation("anchor"));
        assert!(!restored.requires_for_operation("view"));
    }

    #[test]
    fn attestation_result_serde_roundtrip() {
        let result = AttestationResult {
            attested: true,
            attester: crate::types::Did::new("did:key:z6MkAttest"),
            timestamp: Timestamp::now(),
            token: vec![1, 2, 3, 4],
            denial_reason: None,
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let restored: AttestationResult = serde_json::from_str(&json).expect("deserialize");
        assert!(restored.attested);
        assert_eq!(restored.token, vec![1, 2, 3, 4]);
    }

    #[test]
    fn waypoint_config_with_attestation_serde() {
        let config = WaypointConfig {
            operation_attestation: AttestationRequirement::BoundaryOnly,
            ..WaypointConfig::default()
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let restored: WaypointConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(
            restored.operation_attestation,
            AttestationRequirement::BoundaryOnly
        );
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
            forbidden_operations: HashSet::from(["export".to_string()]),
            ..SliceTerms::default()
        };
        assert!(terms.is_operation_allowed("use"));
        assert!(!terms.is_operation_allowed("export"));
    }

    #[test]
    fn slice_terms_allowed_list() {
        let terms = SliceTerms {
            allowed_operations: Some(HashSet::from(["read".to_string(), "view".to_string()])),
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

    #[test]
    fn relending_chain_initial() {
        let did_a = crate::types::Did::new("did:key:z6MkA");
        let chain = RelendingChain::with_initial(did_a.clone(), [1u8; 32]);
        assert_eq!(chain.depth(), 0);
        assert_eq!(chain.current_holder(), Some(&did_a));
        assert_eq!(chain.root_borrower(), Some(&did_a));
        assert!(chain.contains(&did_a));
    }

    #[test]
    fn relending_chain_sublend_validation() {
        let did_a = crate::types::Did::new("did:key:z6MkA");
        let mut chain = RelendingChain::with_initial(did_a, [1u8; 32]);

        // allow_sublend=false -> cannot sublend
        assert!(chain.can_sublend(false, Some(2)).is_err());

        // allow_sublend=true, max_depth=1 -> can add one more (depth 0 -> 1)
        assert!(chain.can_sublend(true, Some(1)).is_ok());

        chain
            .sublend(
                crate::types::Did::new("did:key:z6MkB"),
                [2u8; 32],
                true,
                Some(1),
            )
            .expect("sublend");

        assert_eq!(chain.depth(), 1);

        // Now at max depth, cannot sublend further
        assert!(chain.can_sublend(true, Some(1)).is_err());
    }

    #[test]
    fn relending_chain_return_at() {
        let did_a = crate::types::Did::new("did:key:z6MkA");
        let did_b = crate::types::Did::new("did:key:z6MkB");
        let did_c = crate::types::Did::new("did:key:z6MkC");

        let mut chain = RelendingChain::with_initial(did_a.clone(), [1u8; 32]);
        chain
            .sublend(did_b.clone(), [2u8; 32], true, Some(2))
            .expect("sublend");
        chain
            .sublend(did_c.clone(), [3u8; 32], true, Some(2))
            .expect("sublend");

        assert_eq!(chain.depth(), 2);
        assert_eq!(chain.current_holder(), Some(&did_c));

        // Return at B - unwinds B and C
        let unwound = chain.return_at(&did_b).expect("return_at");
        assert_eq!(unwound.len(), 2); // B and C entries
        assert_eq!(chain.depth(), 0);
        assert_eq!(chain.current_holder(), Some(&did_a));
    }

    #[test]
    fn relending_chain_return_at_not_found() {
        let did_a = crate::types::Did::new("did:key:z6MkA");
        let mut chain = RelendingChain::with_initial(did_a, [1u8; 32]);
        let did_x = crate::types::Did::new("did:key:z6MkX");
        assert!(chain.return_at(&did_x).is_err());
    }

    #[test]
    fn relending_chain_serde_roundtrip() {
        let did_a = crate::types::Did::new("did:key:z6MkA");
        let chain = RelendingChain::with_initial(did_a, [1u8; 32]);
        let json = serde_json::to_string(&chain).expect("serialize");
        let restored: RelendingChain = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.depth(), chain.depth());
    }

    #[test]
    fn relending_chain_new_empty() {
        let chain = RelendingChain::new();
        assert_eq!(chain.depth(), 0);
        assert!(chain.current_holder().is_none());
    }

    #[test]
    fn departure_reason_display_relend() {
        let wp_id = crate::types::SpineId::now_v7();
        let reason = DepartureReason::Relend {
            target_waypoint: wp_id,
        };
        let display = reason.to_string();
        assert!(display.starts_with("relend:"));
        assert!(display.contains(&wp_id.to_string()));
    }

    #[test]
    fn slice_operation_type_names() {
        assert_eq!(SliceOperationType::View { viewport: None }.name(), "view");
        assert_eq!(SliceOperationType::Read { pages: None }.name(), "read");
        assert_eq!(
            SliceOperationType::Edit {
                operation_type: "insert".into()
            }
            .name(),
            "edit"
        );
        assert_eq!(
            SliceOperationType::Export {
                format: "json".into()
            }
            .name(),
            "export"
        );
        assert_eq!(
            SliceOperationType::Custom {
                operation_name: "special".into(),
            }
            .name(),
            "special"
        );
    }
}
