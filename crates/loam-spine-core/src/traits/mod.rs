// SPDX-License-Identifier: AGPL-3.0-only

//! Integration traits for `LoamSpine`.
//!
//! This module defines capability-based interfaces for integrating with other primals.
//! Primals discover each other at runtime through the discovery system - no hardcoded
//! dependencies or compile-time coupling.
//!
//! ## Design Philosophy
//!
//! - **Capability-based**: Traits define what a primal *can do*, not what it *is*
//! - **Runtime discovery**: Primals are discovered at runtime, not compile time
//! - **Self-knowledge only**: `LoamSpine` knows only its own capabilities
//! - **Zero vendor lock-in**: No hardcoded primal names or external service references
//! - **Zero unsafe**: All operations are safe Rust
//!
//! ## Capability Categories
//!
//! - **Signing**: Cryptographic signing/verification (any Ed25519 provider)
//! - **Commit**: Session/braid acceptance from ephemeral storage primals
//! - **Slice**: Borrowed state management with provenance
//! - **Braid**: Semantic attribution from attribution primals

/// CLI-based signer integration for external signing services.
pub mod cli_signer;
mod commit;
/// Signing traits and test utilities.
pub mod signing;
mod slice;

pub use cli_signer::{CliSigner, CliVerifier};
pub use commit::{CommitAcceptor, DehydrationSummary, LoamCommitRef, ResultEntry, SpineQuery};
pub use signing::{SignatureVerification, Signer, Verifier};
pub use slice::{SliceManager, SliceOrigin, SliceResolution};

/// Braid acceptor trait - for semantic attribution primal integration.
///
/// This trait is implemented by LoamSpine to accept Braids from any primal
/// that provides the `BraidAcceptor` capability.
pub trait BraidAcceptor: Send + Sync {
    /// Commit a Braid to the spine.
    fn commit_braid(
        &self,
        spine_id: crate::types::SpineId,
        committer: crate::types::Did,
        braid: BraidSummary,
    ) -> impl std::future::Future<Output = crate::error::LoamSpineResult<crate::types::EntryHash>> + Send;

    /// Verify a Braid exists.
    fn verify_braid(
        &self,
        braid_id: crate::types::BraidId,
    ) -> impl std::future::Future<Output = crate::error::LoamSpineResult<bool>> + Send;

    /// Get Braids for a subject.
    fn get_braids_for_subject(
        &self,
        subject_hash: crate::types::ContentHash,
    ) -> impl std::future::Future<Output = crate::error::LoamSpineResult<Vec<crate::types::EntryHash>>>
           + Send;
}

/// A Braid summary from a semantic attribution primal.
#[derive(Clone, Debug)]
pub struct BraidSummary {
    /// Braid ID.
    pub braid_id: crate::types::BraidId,
    /// Braid type (attribution, derivation, etc.).
    pub braid_type: String,
    /// Subject content hash.
    pub subject_hash: crate::types::ContentHash,
    /// Braid content hash.
    pub braid_hash: crate::types::ContentHash,
    /// Contributing agents.
    pub agents: Vec<crate::types::Did>,
    /// Timestamp.
    pub created_at: crate::types::Timestamp,
    /// Optional signature.
    pub signature: Option<crate::types::Signature>,
}

impl BraidSummary {
    /// Create a new braid summary.
    #[must_use]
    pub fn new(
        braid_id: crate::types::BraidId,
        braid_type: impl Into<String>,
        subject_hash: crate::types::ContentHash,
        braid_hash: crate::types::ContentHash,
    ) -> Self {
        Self {
            braid_id,
            braid_type: braid_type.into(),
            subject_hash,
            braid_hash,
            agents: Vec::new(),
            created_at: crate::types::Timestamp::now(),
            signature: None,
        }
    }

    /// Add an agent.
    #[must_use]
    pub fn with_agent(mut self, agent: crate::types::Did) -> Self {
        self.agents.push(agent);
        self
    }

    /// Add a signature.
    #[must_use]
    pub fn with_signature(mut self, signature: crate::types::Signature) -> Self {
        self.signature = Some(signature);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BraidId, Did, Signature};

    #[test]
    fn braid_summary_creation() {
        let braid_id = BraidId::now_v7();
        let summary = BraidSummary::new(braid_id, "attribution", [1u8; 32], [2u8; 32]);

        assert_eq!(summary.braid_type, "attribution");
        assert_eq!(summary.subject_hash, [1u8; 32]);
        assert_eq!(summary.braid_hash, [2u8; 32]);
        assert!(summary.agents.is_empty());
        assert!(summary.signature.is_none());
    }

    #[test]
    fn braid_summary_with_agent() {
        let braid_id = BraidId::now_v7();
        let agent = Did::new("did:key:z6MkAgent");
        let summary =
            BraidSummary::new(braid_id, "derivation", [0u8; 32], [0u8; 32]).with_agent(agent);

        assert_eq!(summary.agents.len(), 1);
    }

    #[test]
    fn braid_summary_with_multiple_agents() {
        let braid_id = BraidId::now_v7();
        let agent1 = Did::new("did:key:z6MkAgent1");
        let agent2 = Did::new("did:key:z6MkAgent2");

        let summary = BraidSummary::new(braid_id, "collab", [0u8; 32], [0u8; 32])
            .with_agent(agent1)
            .with_agent(agent2);

        assert_eq!(summary.agents.len(), 2);
    }

    #[test]
    fn braid_summary_with_signature() {
        let braid_id = BraidId::now_v7();
        let sig = Signature::from_vec(vec![1, 2, 3, 4]);
        let summary =
            BraidSummary::new(braid_id, "signed", [0u8; 32], [0u8; 32]).with_signature(sig);

        assert!(summary.signature.is_some());
    }

    #[test]
    fn braid_summary_chained_builders() {
        let braid_id = BraidId::now_v7();
        let agent = Did::new("did:key:z6MkAgent");
        let sig = Signature::from_vec(vec![5, 6, 7]);

        let summary = BraidSummary::new(braid_id, "full", [3u8; 32], [4u8; 32])
            .with_agent(agent)
            .with_signature(sig);

        assert_eq!(summary.agents.len(), 1);
        assert!(summary.signature.is_some());
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn braid_summary_debug_and_clone() {
        let braid_id = BraidId::now_v7();
        let summary = BraidSummary::new(braid_id, "test", [0u8; 32], [0u8; 32]);

        // Test clone - verifies Clone trait implementation
        let cloned = summary.clone();
        assert_eq!(summary.braid_type, cloned.braid_type);

        let debug_str = format!("{summary:?}");
        assert!(debug_str.contains("BraidSummary"));
    }
}
