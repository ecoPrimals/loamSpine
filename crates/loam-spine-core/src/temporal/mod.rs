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
//! - **rhizoCrypt (DAG)**: Lives in the Present/Future (branching possibilities)
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

/// Ephemeral provenance from rhizoCrypt dehydration.
///
/// Links a permanent moment back to the ephemeral session that created it.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EphemeralProvenance {
    /// rhizoCrypt session ID
    pub session_id: String,

    /// Merkle root from rhizoCrypt DAG
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
