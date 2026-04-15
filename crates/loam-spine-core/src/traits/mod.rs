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
// Provenance Source — for attribution capability primals
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
// Content Addressable Storage — for content storage capability primals
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
#[path = "mod_tests.rs"]
mod tests;
