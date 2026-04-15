// SPDX-License-Identifier: AGPL-3.0-or-later

//! Temporal primitives for universal time tracking.
//!
//! This module provides the core types for tracking time across any domain:
//! code commits, art creation, life events, scientific experiments, and more.
//!
//! ## Philosophy
//!
//! **Time is the primitive, not version control.**
//!
//! - **Moments**: Points in time (instantaneous)
//! - **Epochs**: Periods with coherence
//! - **Eras**: Long spans (multiple epochs)
//! - **Convergences**: Where timelines meet
//! - **Branches**: Diverging timelines
//!
//! ## Architecture
//!
//! - **Ephemeral DAG primal**: Lives in the Present/Future (branching possibilities)
//! - **LoamSpine (Linear)**: Lives in the Past (what has happened)
//! - **Dehydration**: Temporal collapse (flexible timescales)
//! - **Anchors**: Define ordering (crypto, atomic, causal, consensus)

mod anchor;
mod moment;
mod time_marker;

pub use anchor::{
    Anchor, AnchorType, AtomicAnchor, CausalAnchor, ConsensusAnchor, CryptoAnchor, TimePrecision,
};
pub use moment::{Moment, MomentContext, MomentId};
pub use time_marker::{MarkerType, TimeMarker};

/// Ephemeral provenance from DAG primal dehydration.
///
/// Links a permanent moment back to the ephemeral session that created it.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EphemeralProvenance {
    /// Ephemeral session ID from the DAG primal
    pub session_id: String,

    /// Merkle root from the ephemeral DAG
    pub merkle_root: crate::types::ContentHash,

    /// Attestations from all agents involved
    pub attestations: Vec<Attestation>,

    /// When dehydration occurred
    pub dehydration_timestamp: std::time::SystemTime,
}

/// An attestation from an agent.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Attestation {
    /// Agent DID
    pub agent: String,

    /// Signature over the session
    pub signature: crate::types::Signature,

    /// When this agent signed
    pub timestamp: std::time::SystemTime,
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "temporal mod tests use unwrap for assertion clarity"
)]
mod tests {
    use super::*;
    use crate::types::Signature;
    use std::time::SystemTime;

    fn sample_attestation() -> Attestation {
        Attestation {
            agent: "did:example:agent123".into(),
            signature: Signature::from_vec(vec![0xAB; 64]),
            timestamp: SystemTime::UNIX_EPOCH,
        }
    }

    fn sample_provenance() -> EphemeralProvenance {
        EphemeralProvenance {
            session_id: "ses_abc".into(),
            merkle_root: [0x42; 32],
            attestations: vec![sample_attestation()],
            dehydration_timestamp: SystemTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn attestation_serde_roundtrip() {
        let att = sample_attestation();
        let json = serde_json::to_string(&att).unwrap();
        let back: Attestation = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent, att.agent);
        assert_eq!(back.timestamp, att.timestamp);
    }

    #[test]
    fn ephemeral_provenance_serde_roundtrip() {
        let prov = sample_provenance();
        let json = serde_json::to_string(&prov).unwrap();
        let back: EphemeralProvenance = serde_json::from_str(&json).unwrap();
        assert_eq!(back.session_id, prov.session_id);
        assert_eq!(back.merkle_root, prov.merkle_root);
        assert_eq!(back.attestations.len(), 1);
    }

    #[test]
    fn ephemeral_provenance_clone() {
        let prov = sample_provenance();
        let clone = prov.clone();
        assert_eq!(clone.session_id, prov.session_id);
        assert_eq!(clone.merkle_root, prov.merkle_root);
    }

    #[test]
    fn ephemeral_provenance_debug() {
        let prov = sample_provenance();
        let debug = format!("{prov:?}");
        assert!(debug.contains("ses_abc"));
    }

    #[test]
    fn attestation_empty_signature() {
        let att = Attestation {
            agent: "did:example:none".into(),
            signature: Signature::empty(),
            timestamp: SystemTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&att).unwrap();
        let back: Attestation = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent, "did:example:none");
    }

    #[test]
    fn provenance_multiple_attestations() {
        let prov = EphemeralProvenance {
            session_id: "multi".into(),
            merkle_root: [0xFF; 32],
            attestations: vec![sample_attestation(), sample_attestation()],
            dehydration_timestamp: SystemTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&prov).unwrap();
        let back: EphemeralProvenance = serde_json::from_str(&json).unwrap();
        assert_eq!(back.attestations.len(), 2);
    }
}
