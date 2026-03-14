// SPDX-License-Identifier: AGPL-3.0-only

//! In-memory storage implementations for testing and development.
//!
//! These implementations are fast but transient — data is lost when the process exits.
//! Ideal for unit tests, integration tests, and development workflows.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::certificate::Certificate;
use crate::entry::Entry;
use crate::error::LoamSpineResult;
use crate::spine::Spine;
use crate::types::{CertificateId, EntryHash, SpineId};

use super::{CertificateStorage, EntryStorage, SpineStorage};

/// In-memory spine storage implementation.
///
/// Fast, thread-safe storage using `Arc<RwLock<HashMap>>`.
/// All data is lost when dropped.
#[derive(Debug, Clone, Default)]
pub struct InMemorySpineStorage {
    spines: Arc<RwLock<HashMap<SpineId, Spine>>>,
}

impl InMemorySpineStorage {
    /// Create a new in-memory storage.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of stored spines.
    ///
    /// Useful for testing and debugging.
    pub async fn spine_count(&self) -> usize {
        self.spines.read().await.len()
    }
}

impl SpineStorage for InMemorySpineStorage {
    async fn get_spine(&self, id: SpineId) -> LoamSpineResult<Option<Spine>> {
        let spines = self.spines.read().await;
        Ok(spines.get(&id).cloned())
    }

    async fn save_spine(&self, spine: &Spine) -> LoamSpineResult<()> {
        self.spines.write().await.insert(spine.id, spine.clone());
        Ok(())
    }

    async fn delete_spine(&self, id: SpineId) -> LoamSpineResult<()> {
        self.spines.write().await.remove(&id);
        Ok(())
    }

    async fn list_spines(&self) -> LoamSpineResult<Vec<SpineId>> {
        let spines = self.spines.read().await;
        Ok(spines.keys().copied().collect())
    }
}

/// In-memory entry storage implementation.
///
/// Fast, thread-safe storage with spine indexing for efficient queries.
/// All data is lost when dropped.
#[derive(Debug, Clone, Default)]
pub struct InMemoryEntryStorage {
    entries: Arc<RwLock<HashMap<EntryHash, Entry>>>,
    /// Index: spine_id → entry hashes (in order)
    spine_index: Arc<RwLock<HashMap<SpineId, Vec<EntryHash>>>>,
}

impl InMemoryEntryStorage {
    /// Create a new in-memory storage.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of stored entries.
    ///
    /// Useful for testing and debugging.
    pub async fn entry_count(&self) -> usize {
        self.entries.read().await.len()
    }
}

impl EntryStorage for InMemoryEntryStorage {
    async fn get_entry(&self, hash: EntryHash) -> LoamSpineResult<Option<Entry>> {
        let entries = self.entries.read().await;
        Ok(entries.get(&hash).cloned())
    }

    async fn save_entry(&self, entry: &Entry) -> LoamSpineResult<EntryHash> {
        let hash = entry.compute_hash()?;
        let spine_id = entry.spine_id;

        // Save entry
        {
            let mut entries = self.entries.write().await;
            entries.insert(hash, entry.clone());
        }

        // Update spine index
        let mut index = self.spine_index.write().await;
        let hashes = index.entry(spine_id).or_default();
        if !hashes.contains(&hash) {
            hashes.push(hash);
        }
        drop(index);

        Ok(hash)
    }

    async fn entry_exists(&self, hash: EntryHash) -> LoamSpineResult<bool> {
        let entries = self.entries.read().await;
        Ok(entries.contains_key(&hash))
    }

    async fn get_entries_for_spine(
        &self,
        spine_id: SpineId,
        start_index: u64,
        limit: u64,
    ) -> LoamSpineResult<Vec<Entry>> {
        let hashes: Vec<EntryHash> = {
            let index = self.spine_index.read().await;
            index.get(&spine_id).cloned().unwrap_or_default()
        };

        if hashes.is_empty() {
            return Ok(Vec::new());
        }

        let start = usize::try_from(start_index).unwrap_or(usize::MAX);
        let limit = usize::try_from(limit).unwrap_or(usize::MAX);

        let result: Vec<Entry> = {
            let entries = self.entries.read().await;
            hashes
                .iter()
                .skip(start)
                .take(limit)
                .filter_map(|hash| entries.get(hash).cloned())
                .collect()
        };

        Ok(result)
    }
}

/// In-memory certificate storage implementation.
///
/// Thread-safe storage using `Arc<RwLock<HashMap>>` that pairs each
/// `Certificate` with the `SpineId` of the spine containing its lifecycle
/// entries.  All data is lost when dropped.
#[derive(Debug, Clone, Default)]
pub struct InMemoryCertificateStorage {
    certs: Arc<RwLock<HashMap<CertificateId, (Certificate, SpineId)>>>,
}

impl InMemoryCertificateStorage {
    /// Create a new in-memory certificate storage.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of stored certificates.
    pub async fn certificate_count(&self) -> usize {
        self.certs.read().await.len()
    }
}

impl CertificateStorage for InMemoryCertificateStorage {
    async fn get_certificate(
        &self,
        id: CertificateId,
    ) -> LoamSpineResult<Option<(Certificate, SpineId)>> {
        let certs = self.certs.read().await;
        Ok(certs.get(&id).cloned())
    }

    async fn save_certificate(
        &self,
        certificate: &Certificate,
        spine_id: SpineId,
    ) -> LoamSpineResult<()> {
        self.certs
            .write()
            .await
            .insert(certificate.id, (certificate.clone(), spine_id));
        Ok(())
    }

    async fn delete_certificate(&self, id: CertificateId) -> LoamSpineResult<()> {
        self.certs.write().await.remove(&id);
        Ok(())
    }

    async fn list_certificates(&self) -> LoamSpineResult<Vec<CertificateId>> {
        let certs = self.certs.read().await;
        Ok(certs.keys().copied().collect())
    }
}

/// Combined in-memory storage for spines, entries, and certificates.
///
/// Convenience wrapper that implements `SpineStorage`, `EntryStorage`,
/// and `CertificateStorage`.
///
/// # Example
///
/// ```no_run
/// use loam_spine_core::storage::{InMemoryStorage, SpineStorage, EntryStorage};
///
/// # async fn example() {
/// let storage = InMemoryStorage::new();
///
/// // Use as both SpineStorage and EntryStorage
/// // storage.save_spine(&spine).await.unwrap();
/// // storage.save_entry(&entry).await.unwrap();
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct InMemoryStorage {
    /// Spine storage component.
    pub spines: InMemorySpineStorage,
    /// Entry storage component.
    pub entries: InMemoryEntryStorage,
    /// Certificate storage component.
    pub certificates: InMemoryCertificateStorage,
}

impl InMemoryStorage {
    /// Create a new combined in-memory storage.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
