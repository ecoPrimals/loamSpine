// SPDX-License-Identifier: AGPL-3.0-only

//! Storage traits and implementations for `LoamSpine`.
//!
//! This module defines the storage interfaces for persisting spines and entries.
//! Includes both in-memory (for testing) and Sled-backed (for production) implementations.
//!
//! # Architecture
//!
//! - **Traits**: `SpineStorage`, `EntryStorage` — Define storage interfaces
//! - **InMemory**: Fast, transient storage for testing and development
//! - **Sled**: Persistent, embedded database for production
//!
//! # Example
//!
//! ```no_run
//! use loam_spine_core::storage::{SledStorage, SpineStorage, EntryStorage};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create persistent storage
//! let storage = SledStorage::open("./data")?;
//!
//! // Storage implements both traits
//! // storage.save_spine(&spine).await?;
//! // storage.save_entry(&entry).await?;
//! # Ok(())
//! # }
//! ```

use crate::entry::Entry;
use crate::error::LoamSpineResult;
use crate::spine::Spine;
use crate::types::{EntryHash, SpineId};

// Submodules
mod memory;
mod sled;

// Tests
#[cfg(test)]
mod tests;

// Re-exports
pub use memory::{InMemoryEntryStorage, InMemorySpineStorage, InMemoryStorage};
pub use sled::{SledEntryStorage, SledSpineStorage, SledStorage};

/// Storage backend for spines.
///
/// Implementations must be thread-safe (`Send + Sync`) for use in async contexts.
pub trait SpineStorage: Send + Sync {
    /// Get a spine by ID.
    ///
    /// Returns `None` if the spine doesn't exist.
    fn get_spine(
        &self,
        id: SpineId,
    ) -> impl std::future::Future<Output = LoamSpineResult<Option<Spine>>> + Send;

    /// Save a spine.
    ///
    /// Overwrites existing spine with the same ID.
    fn save_spine(
        &self,
        spine: &Spine,
    ) -> impl std::future::Future<Output = LoamSpineResult<()>> + Send;

    /// Delete a spine by ID.
    ///
    /// Returns `Ok(())` even if the spine doesn't exist (idempotent).
    fn delete_spine(
        &self,
        id: SpineId,
    ) -> impl std::future::Future<Output = LoamSpineResult<()>> + Send;

    /// List all spine IDs.
    ///
    /// Returns an empty vector if no spines exist.
    fn list_spines(
        &self,
    ) -> impl std::future::Future<Output = LoamSpineResult<Vec<SpineId>>> + Send;
}

/// Storage backend for entries (indexed separately for fast lookup).
///
/// Implementations must be thread-safe (`Send + Sync`) for use in async contexts.
pub trait EntryStorage: Send + Sync {
    /// Get an entry by its content hash.
    ///
    /// Returns `None` if the entry doesn't exist.
    fn get_entry(
        &self,
        hash: EntryHash,
    ) -> impl std::future::Future<Output = LoamSpineResult<Option<Entry>>> + Send;

    /// Save an entry and return its hash.
    ///
    /// The entry hash is computed from the entry's content for content-addressable storage.
    fn save_entry(
        &self,
        entry: &Entry,
    ) -> impl std::future::Future<Output = LoamSpineResult<EntryHash>> + Send;

    /// Check if an entry exists by hash.
    ///
    /// More efficient than `get_entry()` when you only need existence.
    fn entry_exists(
        &self,
        hash: EntryHash,
    ) -> impl std::future::Future<Output = LoamSpineResult<bool>> + Send;

    /// Get entries for a spine (by spine ID and index range).
    ///
    /// Returns entries in the range `[start_index, start_index + limit)`.
    /// Useful for paginated queries.
    fn get_entries_for_spine(
        &self,
        spine_id: SpineId,
        start_index: u64,
        limit: u64,
    ) -> impl std::future::Future<Output = LoamSpineResult<Vec<Entry>>> + Send;
}

/// Storage backend type.
///
/// Used to select between available storage backends. All backends implement
/// the same `SpineStorage` + `EntryStorage` traits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StorageBackend {
    /// In-memory storage (for testing and development).
    ///
    /// Fast but transient — data is lost when the process exits.
    #[default]
    InMemory,

    /// Sled-backed persistent storage (for production).
    ///
    /// Embedded, pure-Rust database. Good default for single-node deployments.
    Sled,

    /// SQLite-backed persistent storage (planned).
    ///
    /// Widely supported, file-based relational storage.
    /// Note: Requires the `sqlite` feature to be enabled.
    Sqlite,

    /// PostgreSQL-backed persistent storage (planned).
    ///
    /// Production-grade relational storage for multi-node deployments.
    /// Note: Requires the `postgres` feature to be enabled.
    Postgres,

    /// RocksDB-backed persistent storage (planned).
    ///
    /// High-performance LSM-tree storage.
    /// Note: Requires the `rocksdb` feature to be enabled.
    Rocksdb,
}

impl StorageBackend {
    /// Check if this backend is currently implemented and available.
    #[must_use]
    pub const fn is_available(&self) -> bool {
        matches!(self, Self::InMemory | Self::Sled)
    }

    /// Get a human-readable name for this backend.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::InMemory => "in-memory",
            Self::Sled => "sled",
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
            Self::Rocksdb => "rocksdb",
        }
    }
}

impl std::fmt::Display for StorageBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
