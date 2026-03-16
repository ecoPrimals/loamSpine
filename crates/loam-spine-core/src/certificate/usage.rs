// SPDX-License-Identifier: AGPL-3.0-or-later

//! Usage tracking for certificate loans.
//!
//! When a certificate is returned from a loan period, a [`UsageSummary`]
//! captures what happened while it was out. If the loan was anchored at a
//! waypoint spine, the existing [`WaypointSummary`](crate::waypoint::WaypointSummary)
//! provides the permanence-layer view.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::waypoint::WaypointSummary;

/// Summary of certificate usage during a loan period.
///
/// Attached to `CertificateReturn` entries and stored in [`LoanRecord`]s
/// to provide a complete provenance trail of what happened while a
/// certificate was borrowed.
///
/// Per `CERTIFICATE_LAYER.md` §8: every return MAY include a usage summary;
/// the summary is optional because some loans have no observable operations.
///
/// [`LoanRecord`]: super::LoanRecord
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UsageSummary {
    /// Total usage duration in nanoseconds.
    pub duration_nanos: u64,

    /// Number of discrete operations performed.
    pub operation_count: u64,

    /// Kinds of operations performed (e.g. `"read"`, `"transform"`).
    pub operation_types: Vec<String>,

    /// Waypoint summary, present when the loan was anchored at a waypoint
    /// spine.
    pub waypoint_summary: Option<WaypointSummary>,

    /// Arbitrary caller-supplied usage data (e.g. metrics, provenance tags).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, serde_json::Value>,
}

impl UsageSummary {
    /// Create a minimal summary from a duration and operation count.
    #[must_use]
    pub fn new(duration_nanos: u64, operation_count: u64) -> Self {
        Self {
            duration_nanos,
            operation_count,
            ..Self::default()
        }
    }

    /// Attach operation types.
    #[must_use]
    pub fn with_operation_types(mut self, types: Vec<String>) -> Self {
        self.operation_types = types;
        self
    }

    /// Attach a waypoint summary.
    #[must_use]
    pub fn with_waypoint_summary(mut self, summary: WaypointSummary) -> Self {
        self.waypoint_summary = Some(summary);
        self
    }

    /// Insert a custom key-value pair.
    #[must_use]
    pub fn with_custom(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom.insert(key.into(), value);
        self
    }

    /// Whether this summary represents zero usage.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.operation_count == 0 && self.duration_nanos == 0
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::types::{ContentHash, SliceId, Timestamp};

    fn test_waypoint_summary() -> WaypointSummary {
        WaypointSummary {
            slice_id: SliceId::now_v7(),
            duration_nanos: 500,
            operation_count: 3,
            operation_types: vec!["anchor".into()],
            first_operation: Some(Timestamp::now()),
            last_operation: Some(Timestamp::now()),
            operations_hash: ContentHash::default(),
            was_relent: false,
            max_relend_depth: 0,
        }
    }

    #[test]
    fn usage_summary_default_is_empty() {
        let summary = UsageSummary::default();
        assert!(summary.is_empty());
        assert_eq!(summary.operation_count, 0);
        assert_eq!(summary.duration_nanos, 0);
        assert!(summary.waypoint_summary.is_none());
        assert!(summary.custom.is_empty());
    }

    #[test]
    fn usage_summary_builder() {
        let summary = UsageSummary::new(5_000_000_000, 42)
            .with_operation_types(vec!["read".into(), "transform".into()])
            .with_custom("source", serde_json::Value::String("test".into()));

        assert_eq!(summary.duration_nanos, 5_000_000_000);
        assert_eq!(summary.operation_count, 42);
        assert_eq!(summary.operation_types.len(), 2);
        assert!(!summary.is_empty());
        assert!(summary.custom.contains_key("source"));
    }

    #[test]
    fn usage_summary_roundtrip_serde() {
        let summary = UsageSummary::new(100, 5).with_operation_types(vec!["read".into()]);

        let json = serde_json::to_string(&summary).unwrap();
        let deser: UsageSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(deser.duration_nanos, 100);
        assert_eq!(deser.operation_count, 5);
        assert_eq!(deser.operation_types, vec!["read"]);
    }

    #[test]
    fn usage_summary_with_waypoint_roundtrip() {
        let ws = test_waypoint_summary();
        let summary = UsageSummary::new(1000, 3).with_waypoint_summary(ws);

        let json = serde_json::to_string(&summary).unwrap();
        let deser: UsageSummary = serde_json::from_str(&json).unwrap();

        assert!(deser.waypoint_summary.is_some());
        let ws = deser.waypoint_summary.unwrap();
        assert_eq!(ws.duration_nanos, 500);
        assert_eq!(ws.operation_count, 3);
    }
}
