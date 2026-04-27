// SPDX-License-Identifier: AGPL-3.0-or-later

//! Commit acceptor traits for DAG session commits.
//!
//! These traits define the interface for accepting commits from ephemeral DAG primals
//! (ephemeral storage primals). The actual primal is discovered at runtime.

use std::collections::HashMap;

use crate::entry::Entry;
use crate::error::LoamSpineResult;
use crate::spine::Spine;
use crate::types::{ContentHash, Did, EntryHash, SessionId, SpineId, Timestamp};

/// Reference to a commit in LoamSpine.
///
/// Serves as a provenance receipt: contains enough information
/// for downstream consumers (guideStone, composition scripts)
/// to verify and trace the commit chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoamCommitRef {
    /// Spine where the commit resides.
    pub spine_id: SpineId,
    /// Entry hash.
    pub entry_hash: EntryHash,
    /// Entry index in the spine.
    pub index: u64,
    /// Timestamp of the committed entry.
    pub committed_at: Timestamp,
}

/// Summary of a dehydration session from an ephemeral DAG primal.
#[derive(Clone, Debug)]
pub struct DehydrationSummary {
    /// Session ID.
    pub session_id: SessionId,
    /// Session type (game, transaction, etc.).
    pub session_type: String,
    /// Merkle root of the session DAG.
    pub merkle_root: ContentHash,
    /// Number of vertices in the session.
    pub vertex_count: u64,
    /// Session start time.
    pub started_at: Timestamp,
    /// Session end time.
    pub ended_at: Timestamp,
    /// Result entries (outcomes of the session).
    pub result_entries: Vec<ResultEntry>,
    /// Session metadata.
    pub metadata: HashMap<String, String>,
}

impl DehydrationSummary {
    /// Create a new dehydration summary.
    #[must_use]
    pub fn new(
        session_id: SessionId,
        session_type: impl Into<String>,
        merkle_root: ContentHash,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            session_id,
            session_type: session_type.into(),
            merkle_root,
            vertex_count: 0,
            started_at: now,
            ended_at: now,
            result_entries: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the vertex count.
    #[must_use]
    pub const fn with_vertex_count(mut self, count: u64) -> Self {
        self.vertex_count = count;
        self
    }

    /// Add a result entry.
    #[must_use]
    pub fn with_result(mut self, entry: ResultEntry) -> Self {
        self.result_entries.push(entry);
        self
    }

    /// Add metadata.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Compute a hash of this summary.
    #[must_use]
    pub fn compute_hash(&self) -> ContentHash {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.session_id.as_bytes());
        hasher.update(&self.merkle_root);
        *hasher.finalize().as_bytes()
    }
}

/// A result entry from a session.
#[derive(Clone, Debug)]
pub struct ResultEntry {
    /// Entry type/category.
    pub entry_type: String,
    /// Content hash of the result.
    pub content_hash: ContentHash,
    /// Result metadata.
    pub metadata: HashMap<String, String>,
}

/// Commit acceptor trait - for accepting commits from ephemeral DAG primals.
///
/// This trait is implemented by LoamSpine to accept commits from any primal
/// that provides the `CommitAcceptor` capability.
pub trait CommitAcceptor: Send + Sync {
    /// Commit a dehydration session.
    fn commit_session(
        &self,
        spine_id: SpineId,
        committer: Did,
        summary: DehydrationSummary,
    ) -> impl std::future::Future<Output = LoamSpineResult<LoamCommitRef>> + Send;

    /// Verify a commit exists.
    fn verify_commit(
        &self,
        commit_ref: &LoamCommitRef,
    ) -> impl std::future::Future<Output = LoamSpineResult<bool>> + Send;

    /// Get a commit's entry.
    fn get_commit(
        &self,
        commit_ref: &LoamCommitRef,
    ) -> impl std::future::Future<Output = LoamSpineResult<Option<Entry>>> + Send;
}

/// Compute a hash of a result entry.
impl ResultEntry {
    /// Create a new result entry.
    #[must_use]
    pub fn new(entry_type: impl Into<String>, content_hash: ContentHash) -> Self {
        Self {
            entry_type: entry_type.into(),
            content_hash,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Query interface for LoamSpine entries.
pub trait SpineQuery: Send + Sync {
    /// Get an entry by hash.
    fn get_entry(
        &self,
        hash: EntryHash,
    ) -> impl std::future::Future<Output = LoamSpineResult<Option<Entry>>> + Send;

    /// Get entries in a range.
    fn get_entries(
        &self,
        spine_id: SpineId,
        start: u64,
        limit: u64,
    ) -> impl std::future::Future<Output = LoamSpineResult<Vec<Entry>>> + Send;

    /// Get the latest entry (tip) for a spine.
    fn get_tip(
        &self,
        spine_id: SpineId,
    ) -> impl std::future::Future<Output = LoamSpineResult<Option<Entry>>> + Send;

    /// Get spine metadata.
    fn get_spine(
        &self,
        id: SpineId,
    ) -> impl std::future::Future<Output = LoamSpineResult<Option<Spine>>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dehydration_summary_creation() {
        let session_id = SessionId::now_v7();
        let summary = DehydrationSummary::new(session_id, "game", [1u8; 32]);

        assert_eq!(summary.session_type, "game");
        assert_eq!(summary.merkle_root, [1u8; 32]);
        assert_eq!(summary.vertex_count, 0);
        assert!(summary.result_entries.is_empty());
        assert!(summary.metadata.is_empty());
    }

    #[test]
    fn dehydration_summary_with_vertex_count() {
        let session_id = SessionId::now_v7();
        let summary = DehydrationSummary::new(session_id, "test", [0u8; 32]).with_vertex_count(42);

        assert_eq!(summary.vertex_count, 42);
    }

    #[test]
    fn dehydration_summary_with_result() {
        let session_id = SessionId::now_v7();
        let result = ResultEntry::new("outcome", [2u8; 32]);
        let summary = DehydrationSummary::new(session_id, "test", [0u8; 32]).with_result(result);

        assert_eq!(summary.result_entries.len(), 1);
        assert_eq!(summary.result_entries[0].entry_type, "outcome");
    }

    #[test]
    fn dehydration_summary_with_metadata() {
        let session_id = SessionId::now_v7();
        let summary = DehydrationSummary::new(session_id, "test", [0u8; 32])
            .with_metadata("key1", "value1")
            .with_metadata("key2", "value2");

        assert_eq!(summary.metadata.len(), 2);
        assert_eq!(summary.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(summary.metadata.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn dehydration_summary_compute_hash() {
        let session_id = SessionId::now_v7();
        let summary1 = DehydrationSummary::new(session_id, "test", [0u8; 32]);
        let summary2 = DehydrationSummary::new(session_id, "test", [0u8; 32]);

        // Same input should produce same hash
        assert_eq!(summary1.compute_hash(), summary2.compute_hash());

        // Different merkle root should produce different hash
        let summary3 = DehydrationSummary::new(session_id, "test", [1u8; 32]);
        assert_ne!(summary1.compute_hash(), summary3.compute_hash());
    }

    #[test]
    fn dehydration_summary_chained_builders() {
        let session_id = SessionId::now_v7();
        let result = ResultEntry::new("final", [3u8; 32]).with_metadata("score", "100");

        let summary = DehydrationSummary::new(session_id, "game", [4u8; 32])
            .with_vertex_count(1000)
            .with_result(result)
            .with_metadata("player", "alice");

        assert_eq!(summary.vertex_count, 1000);
        assert_eq!(summary.result_entries.len(), 1);
        assert_eq!(summary.metadata.get("player"), Some(&"alice".to_string()));
    }

    #[test]
    fn result_entry_creation() {
        let entry = ResultEntry::new("outcome", [5u8; 32]);

        assert_eq!(entry.entry_type, "outcome");
        assert_eq!(entry.content_hash, [5u8; 32]);
        assert!(entry.metadata.is_empty());
    }

    #[test]
    fn result_entry_with_metadata() {
        let entry = ResultEntry::new("outcome", [6u8; 32])
            .with_metadata("key1", "value1")
            .with_metadata("key2", "value2");

        assert_eq!(entry.metadata.len(), 2);
        assert_eq!(entry.metadata.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn loam_commit_ref_equality() {
        let spine_id = SpineId::now_v7();
        let ts = Timestamp::now();
        let ref1 = LoamCommitRef {
            spine_id,
            entry_hash: [7u8; 32],
            index: 42,
            committed_at: ts,
        };
        let ref2 = LoamCommitRef {
            spine_id,
            entry_hash: [7u8; 32],
            index: 42,
            committed_at: ts,
        };

        assert_eq!(ref1, ref2);

        let ref3 = LoamCommitRef {
            spine_id,
            entry_hash: [8u8; 32],
            index: 42,
            committed_at: ts,
        };
        assert_ne!(ref1, ref3);
    }

    #[test]
    fn dehydration_summary_debug() {
        let session_id = SessionId::now_v7();
        let summary = DehydrationSummary::new(session_id, "test", [0u8; 32]);

        // Should not panic
        let debug_str = format!("{summary:?}");
        assert!(debug_str.contains("DehydrationSummary"));
    }

    #[test]
    fn result_entry_debug_and_clone() {
        let entry = ResultEntry::new("test", [0u8; 32]);

        // Test clone - verifies Clone trait implementation
        let cloned = entry.clone();
        assert_eq!(entry.entry_type, cloned.entry_type);
        assert_eq!(entry.content_hash, cloned.content_hash);

        let debug_str = format!("{entry:?}");
        assert!(debug_str.contains("ResultEntry"));
    }
}
