// SPDX-License-Identifier: AGPL-3.0-only

//! Time markers - named references to moments (like Git branches/tags).

use serde::{Deserialize, Serialize};

use super::MomentId;

/// A named reference to a moment in time.
///
/// Can be:
/// - Mutable (like Git branches) - can move to point to different moments
/// - Immutable (like Git tags) - fixed to a specific moment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeMarker {
    /// Human-readable name
    pub name: String,

    /// Which moment does this marker point to?
    pub moment: MomentId,

    /// Can this marker move?
    pub marker_type: MarkerType,

    /// Optional description
    pub description: Option<String>,

    /// When was this marker created?
    pub created_at: std::time::SystemTime,

    /// Who created this marker?
    pub created_by: String, // DID
}

/// Whether a marker can move or is fixed.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MarkerType {
    /// Can be updated to point to different moments (like Git branches)
    Mutable,

    /// Fixed to a specific moment (like Git tags)
    Immutable,
}

impl TimeMarker {
    /// Create a new mutable marker (branch-like).
    #[must_use]
    pub fn branch(name: String, moment: MomentId, created_by: String) -> Self {
        Self {
            name,
            moment,
            marker_type: MarkerType::Mutable,
            description: None,
            created_at: std::time::SystemTime::now(),
            created_by,
        }
    }

    /// Create a new immutable marker (tag-like).
    #[must_use]
    pub fn tag(
        name: String,
        moment: MomentId,
        created_by: String,
        description: Option<String>,
    ) -> Self {
        Self {
            name,
            moment,
            marker_type: MarkerType::Immutable,
            description,
            created_at: std::time::SystemTime::now(),
            created_by,
        }
    }

    /// Can this marker be updated?
    #[must_use]
    pub fn is_mutable(&self) -> bool {
        self.marker_type == MarkerType::Mutable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ContentHash;

    #[test]
    fn create_branch_marker() {
        let marker = TimeMarker::branch(
            "main".to_string(),
            ContentHash::default(),
            "did:example:alice".to_string(),
        );

        assert_eq!(marker.name, "main");
        assert!(marker.is_mutable());
    }

    #[test]
    fn create_tag_marker() {
        let marker = TimeMarker::tag(
            "v1.0.0".to_string(),
            ContentHash::default(),
            "did:example:alice".to_string(),
            Some("First release".to_string()),
        );

        assert_eq!(marker.name, "v1.0.0");
        assert!(!marker.is_mutable());
        assert_eq!(marker.description, Some("First release".to_string()));
    }
}
