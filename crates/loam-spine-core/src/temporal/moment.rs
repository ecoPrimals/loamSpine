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
    pub agent: String,  // DID
    
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
        message: String,
        tree_hash: ContentHash,  // From NestGate
    },
    
    /// Art creation
    ArtCreation {
        title: String,
        medium: String,
        content_hash: ContentHash,  // From NestGate
    },
    
    /// Life event
    LifeEvent {
        event_type: String,
        participants: Vec<String>,  // DIDs
        description: String,
    },
    
    /// Performance (concert, play, etc.)
    Performance {
        venue: String,
        duration_seconds: u64,
        recording_hash: Option<ContentHash>,
    },
    
    /// Scientific experiment
    Experiment {
        hypothesis: String,
        result: String,
        data_hash: ContentHash,
    },
    
    /// Business milestone
    Milestone {
        achievement: String,
        metrics: HashMap<String, f64>,
    },
    
    /// Generic moment (for future use cases)
    Generic {
        category: String,
        metadata: HashMap<String, String>,
        content_hash: Option<ContentHash>,
    },
}

impl MomentContext {
    /// Get a human-readable category name.
    pub fn category(&self) -> &str {
        match self {
            MomentContext::CodeChange { .. } => "code",
            MomentContext::ArtCreation { .. } => "art",
            MomentContext::LifeEvent { .. } => "life",
            MomentContext::Performance { .. } => "performance",
            MomentContext::Experiment { .. } => "experiment",
            MomentContext::Milestone { .. } => "milestone",
            MomentContext::Generic { category, .. } => category,
        }
    }
}

#[cfg(test)]
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
}

