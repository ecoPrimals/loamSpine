// SPDX-License-Identifier: AGPL-3.0-or-later

//! Moments - the fundamental unit of time tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Anchor, EphemeralProvenance};
use crate::types::{ContentHash, Signature};

/// A moment in time - the fundamental temporal unit.
///
/// Can represent:
/// - Code commits
/// - Art creations
/// - Life events
/// - Performances
/// - Experiments
/// - Any point in time!
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Moment {
    /// Unique identifier for this moment
    pub id: MomentId,

    /// When this moment occurred
    pub timestamp: std::time::SystemTime,

    /// Who created/witnessed this moment
    pub agent: String, // DID

    /// State hash at this moment (from NestGate or rhizoCrypt)
    pub state_hash: ContentHash,

    /// Cryptographic signature
    pub signature: Signature,

    /// What kind of moment is this?
    pub context: MomentContext,

    /// Parent moments (0 = genesis, 1+ = history)
    pub parents: Vec<MomentId>,

    /// How is this moment anchored? (optional, defaults to atomic time)
    pub anchor: Option<Anchor>,

    /// Link back to ephemeral provenance (if from rhizoCrypt)
    pub ephemeral_provenance: Option<EphemeralProvenance>,
}

/// Unique identifier for a moment.
pub type MomentId = ContentHash;

/// Context: What kind of moment is this?
///
/// Extensible enum - can add new contexts without breaking existing code.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MomentContext {
    /// Code change (version control pattern)
    CodeChange {
        /// Commit message describing the change
        message: String,
        /// Tree hash from NestGate representing the code state
        tree_hash: ContentHash,
    },

    /// Art creation
    ArtCreation {
        /// Title of the artwork
        title: String,
        /// Medium used (oil, digital, sculpture, etc.)
        medium: String,
        /// Content hash from NestGate
        content_hash: ContentHash,
    },

    /// Life event
    LifeEvent {
        /// Type of event (birth, marriage, graduation, etc.)
        event_type: String,
        /// DIDs of participants in the event
        participants: Vec<String>,
        /// Human-readable description of the event
        description: String,
    },

    /// Performance (concert, play, etc.)
    Performance {
        /// Venue where the performance occurred
        venue: String,
        /// Duration of the performance in seconds
        duration_seconds: u64,
        /// Optional hash of recording/video
        recording_hash: Option<ContentHash>,
    },

    /// Scientific experiment
    Experiment {
        /// Hypothesis being tested
        hypothesis: String,
        /// Result of the experiment
        result: String,
        /// Hash of experimental data
        data_hash: ContentHash,
    },

    /// Business milestone
    Milestone {
        /// Description of the achievement
        achievement: String,
        /// Key metrics associated with this milestone
        metrics: HashMap<String, f64>,
    },

    /// Generic moment (for future use cases)
    Generic {
        /// Category of this moment
        category: String,
        /// Arbitrary metadata key-value pairs
        metadata: HashMap<String, String>,
        /// Optional content hash
        content_hash: Option<ContentHash>,
    },
}

impl MomentContext {
    /// Get a human-readable category name.
    #[must_use]
    pub fn category(&self) -> &str {
        match self {
            Self::CodeChange { .. } => "code",
            Self::ArtCreation { .. } => "art",
            Self::LifeEvent { .. } => "life",
            Self::Performance { .. } => "performance",
            Self::Experiment { .. } => "experiment",
            Self::Milestone { .. } => "milestone",
            Self::Generic { category, .. } => category,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn moment_context_category() {
        let code = MomentContext::CodeChange {
            message: "test".to_string(),
            tree_hash: ContentHash::default(),
        };
        assert_eq!(code.category(), "code");

        let art = MomentContext::ArtCreation {
            title: "Starry Night".to_string(),
            medium: "Oil on canvas".to_string(),
            content_hash: ContentHash::default(),
        };
        assert_eq!(art.category(), "art");
    }

    #[test]
    fn moment_context_all_categories() {
        let life = MomentContext::LifeEvent {
            event_type: "graduation".into(),
            participants: vec!["did:eco:alice".into()],
            description: "Finished school".into(),
        };
        assert_eq!(life.category(), "life");

        let perf = MomentContext::Performance {
            venue: "Hall A".into(),
            duration_seconds: 3600,
            recording_hash: None,
        };
        assert_eq!(perf.category(), "performance");

        let exp = MomentContext::Experiment {
            hypothesis: "H0".into(),
            result: "confirmed".into(),
            data_hash: ContentHash::default(),
        };
        assert_eq!(exp.category(), "experiment");

        let ms = MomentContext::Milestone {
            achievement: "1M users".into(),
            metrics: HashMap::from([("users".into(), 1_000_000.0)]),
        };
        assert_eq!(ms.category(), "milestone");

        let generic = MomentContext::Generic {
            category: "custom".into(),
            metadata: HashMap::new(),
            content_hash: None,
        };
        assert_eq!(generic.category(), "custom");
    }

    #[test]
    fn moment_context_serde_roundtrip() {
        let ctx = MomentContext::CodeChange {
            message: "fix bug".into(),
            tree_hash: ContentHash::default(),
        };
        let json = serde_json::to_string(&ctx).unwrap();
        let parsed: MomentContext = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.category(), "code");
    }
}
