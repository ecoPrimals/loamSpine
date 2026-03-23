// SPDX-License-Identifier: AGPL-3.0-or-later

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
pub use slice::{ActiveSlice, SliceManager, SliceOrigin, SliceResolution, SliceStatus};

// ============================================================================
// Provenance Source — for attribution primals (e.g. sweetGrass)
// ============================================================================

/// Provenance source trait — for querying provenance data from this spine.
///
/// This trait is implemented by LoamSpine to provide provenance data to any primal
/// that discovers the `provenance-source` capability. Attribution primals
/// query this to build provenance chains and attribution graphs.
pub trait ProvenanceSource: Send + Sync {
    /// Get entries related to a given content hash.
    fn get_entries_for_data(
        &self,
        content_hash: crate::types::ContentHash,
    ) -> impl std::future::Future<Output = crate::error::LoamSpineResult<Vec<crate::entry::Entry>>> + Send;

    /// Get the certificate history for a given certificate ID.
    fn get_certificate_history(
        &self,
        certificate_id: crate::types::CertificateId,
    ) -> impl std::future::Future<Output = crate::error::LoamSpineResult<Vec<crate::entry::Entry>>> + Send;

    /// Get attribution data for a content hash.
    fn get_attribution(
        &self,
        content_hash: crate::types::ContentHash,
    ) -> impl std::future::Future<Output = crate::error::LoamSpineResult<Option<AttributionRecord>>> + Send;

    /// Get the full provenance chain for a content hash.
    fn get_provenance_chain(
        &self,
        content_hash: crate::types::ContentHash,
    ) -> impl std::future::Future<Output = crate::error::LoamSpineResult<Vec<ProvenanceLink>>> + Send;
}

/// Attribution record linking content to its creator and contributors.
#[derive(Clone, Debug)]
pub struct AttributionRecord {
    /// Content hash being attributed.
    pub content_hash: crate::types::ContentHash,
    /// Primary creator DID.
    pub creator: crate::types::Did,
    /// Contributing agents.
    pub contributors: Vec<crate::types::Did>,
    /// Associated certificate (if any).
    pub certificate_id: Option<crate::types::CertificateId>,
    /// When the attribution was recorded.
    pub recorded_at: crate::types::Timestamp,
}

/// A link in a provenance chain.
#[derive(Clone, Debug)]
pub struct ProvenanceLink {
    /// Entry hash that forms this link.
    pub entry_hash: crate::types::EntryHash,
    /// Spine where this link resides.
    pub spine_id: crate::types::SpineId,
    /// Index within the spine.
    pub index: u64,
    /// Agent who created this link.
    pub agent: crate::types::Did,
    /// Timestamp of the link.
    pub timestamp: crate::types::Timestamp,
    /// Relationship type (e.g., "derived-from", "attributed-to", "certified-by").
    pub relationship: String,
}

// ============================================================================
// Content Addressable Storage — for content storage primals (e.g. NestGate)
// ============================================================================

/// Content-addressable storage trait — for storing and retrieving payloads.
///
/// This trait defines the interface for interaction with any primal providing
/// the `content-storage` capability. LoamSpine stores `PayloadRef` entries
/// that point to content stored in a CAS primal.
pub trait ContentAddressableStore: Send + Sync {
    /// Store content and return its hash.
    fn put(
        &self,
        data: crate::types::ByteBuffer,
    ) -> impl std::future::Future<Output = crate::error::LoamSpineResult<crate::types::ContentHash>> + Send;

    /// Retrieve content by hash.
    fn get(
        &self,
        hash: crate::types::ContentHash,
    ) -> impl std::future::Future<
        Output = crate::error::LoamSpineResult<Option<crate::types::ByteBuffer>>,
    > + Send;

    /// Check if content exists.
    fn exists(
        &self,
        hash: crate::types::ContentHash,
    ) -> impl std::future::Future<Output = crate::error::LoamSpineResult<bool>> + Send;
}

// ============================================================================
// Sync Protocol — for federation and replication
// ============================================================================

/// Sync protocol trait — for spine replication and federation.
///
/// This trait defines the interface for pushing/pulling entries between
/// LoamSpine instances, enabling federation across trust boundaries.
pub trait SyncProtocol: Send + Sync {
    /// Push entries to a remote spine.
    fn push_entries(
        &self,
        spine_id: crate::types::SpineId,
        entries: Vec<crate::entry::Entry>,
    ) -> impl std::future::Future<Output = crate::error::LoamSpineResult<SyncResult>> + Send;

    /// Pull entries from a remote spine.
    fn pull_entries(
        &self,
        spine_id: crate::types::SpineId,
        from_index: u64,
        limit: u64,
    ) -> impl std::future::Future<Output = crate::error::LoamSpineResult<Vec<crate::entry::Entry>>> + Send;

    /// Get the sync status for a spine.
    fn get_sync_status(
        &self,
        spine_id: crate::types::SpineId,
    ) -> impl std::future::Future<Output = crate::error::LoamSpineResult<SyncStatus>> + Send;
}

/// Result of a sync push operation.
#[derive(Clone, Debug)]
pub struct SyncResult {
    /// Number of entries accepted.
    pub accepted: u64,
    /// Number of entries rejected.
    pub rejected: u64,
    /// Rejection reasons (if any).
    pub rejection_reasons: Vec<String>,
}

/// Status of a sync relationship.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncStatus {
    /// Local and remote are in sync.
    InSync,
    /// Local is ahead of remote.
    LocalAhead {
        /// Number of entries ahead.
        entries_ahead: u64,
    },
    /// Remote is ahead of local.
    RemoteAhead {
        /// Number of entries behind.
        entries_behind: u64,
    },
    /// Unknown status (never synced).
    Unknown,
}

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
    ) -> impl std::future::Future<
        Output = crate::error::LoamSpineResult<Vec<crate::types::EntryHash>>,
    > + Send;
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
    fn braid_summary_debug_and_clone() {
        let braid_id = BraidId::now_v7();
        let summary = BraidSummary::new(braid_id, "test", [0u8; 32], [0u8; 32]);

        let cloned = summary.clone();
        assert_eq!(summary.braid_type, cloned.braid_type);

        let debug_str = format!("{summary:?}");
        assert!(debug_str.contains("BraidSummary"));
    }

    #[test]
    fn attribution_record_creation() {
        let record = AttributionRecord {
            content_hash: [1u8; 32],
            creator: Did::new("did:key:z6MkCreator"),
            contributors: vec![Did::new("did:key:z6MkContrib1")],
            certificate_id: None,
            recorded_at: crate::types::Timestamp::now(),
        };

        assert_eq!(record.content_hash, [1u8; 32]);
        assert_eq!(record.contributors.len(), 1);
        assert!(record.certificate_id.is_none());
    }

    #[test]
    fn provenance_link_creation() {
        let link = ProvenanceLink {
            entry_hash: [2u8; 32],
            spine_id: crate::types::SpineId::now_v7(),
            index: 42,
            agent: Did::new("did:key:z6MkAgent"),
            timestamp: crate::types::Timestamp::now(),
            relationship: "derived-from".to_string(),
        };

        assert_eq!(link.index, 42);
        assert_eq!(link.relationship, "derived-from");
    }

    #[test]
    fn sync_result_creation() {
        let result = SyncResult {
            accepted: 10,
            rejected: 2,
            rejection_reasons: vec!["duplicate".to_string()],
        };

        assert_eq!(result.accepted, 10);
        assert_eq!(result.rejected, 2);
        assert_eq!(result.rejection_reasons.len(), 1);
    }

    #[test]
    fn sync_status_variants() {
        assert_eq!(SyncStatus::InSync, SyncStatus::InSync);
        assert_ne!(SyncStatus::InSync, SyncStatus::Unknown);

        let ahead = SyncStatus::LocalAhead { entries_ahead: 5 };
        assert!(matches!(ahead, SyncStatus::LocalAhead { entries_ahead: 5 }));

        let behind = SyncStatus::RemoteAhead { entries_behind: 3 };
        assert!(matches!(
            behind,
            SyncStatus::RemoteAhead { entries_behind: 3 }
        ));
    }

    #[test]
    fn integration_types_debug_clone() {
        let record = AttributionRecord {
            content_hash: [0u8; 32],
            creator: Did::new("did:key:test"),
            contributors: vec![],
            certificate_id: None,
            recorded_at: crate::types::Timestamp::now(),
        };
        let cloned = record.clone();
        assert_eq!(record.content_hash, cloned.content_hash);
        let _ = format!("{record:?}");

        let link = ProvenanceLink {
            entry_hash: [0u8; 32],
            spine_id: crate::types::SpineId::now_v7(),
            index: 0,
            agent: Did::new("did:key:test"),
            timestamp: crate::types::Timestamp::now(),
            relationship: "test".to_string(),
        };
        let cloned = link.clone();
        assert_eq!(link.relationship, cloned.relationship);
        let _ = format!("{link:?}");

        let result = SyncResult {
            accepted: 0,
            rejected: 0,
            rejection_reasons: vec![],
        };
        let cloned = result.clone();
        assert_eq!(result.accepted, cloned.accepted);
        let _ = format!("{result:?}");
    }
}
