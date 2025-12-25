//! Sled-backed persistent storage for production use.
//!
//! Uses [Sled](https://github.com/spacejam/sled), a pure Rust embedded database,
//! for persistent storage. No external dependencies or C tooling required.
//!
//! # Features
//!
//! - **Pure Rust**: No C dependencies, works everywhere Rust compiles
//! - **ACID**: Atomic, consistent, isolated, durable operations
//! - **Embedded**: No separate database server needed
//! - **Fast**: Zero-copy reads, batched writes
//!
//! # Production Use
//!
//! For production deployments, use [`SledStorage::open`] with a persistent path:
//!
//! ```no_run
//! use loam_spine_core::storage::SledStorage;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = SledStorage::open("./loamspine-data")?;
//! # Ok(())
//! # }
//! ```

use std::path::Path;

use crate::entry::Entry;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::spine::Spine;
use crate::types::{EntryHash, SpineId};

use super::{EntryStorage, SpineStorage};

/// Sled-backed spine storage for production use.
///
/// Uses Sled, a pure Rust embedded database, for persistent storage.
/// No external dependencies or C tooling required.
#[derive(Clone)]
pub struct SledSpineStorage {
    db: sled::Db,
    tree: sled::Tree,
}

impl SledSpineStorage {
    /// Open spine storage at the given path.
    ///
    /// Creates the directory if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be opened or directory cannot be created.
    pub fn open<P: AsRef<Path>>(path: P) -> LoamSpineResult<Self> {
        let db = sled::open(path).map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let tree = db
            .open_tree("spines")
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(Self { db, tree })
    }

    /// Create storage with a temporary database (for testing).
    ///
    /// The database is automatically deleted when dropped.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be created.
    pub fn temporary() -> LoamSpineResult<Self> {
        let config = sled::Config::new().temporary(true);
        let db = config
            .open()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let tree = db
            .open_tree("spines")
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(Self { db, tree })
    }

    /// Get the number of stored spines.
    ///
    /// This is an O(1) operation.
    #[must_use]
    pub fn spine_count(&self) -> usize {
        self.tree.len()
    }

    /// Flush all pending writes to disk.
    ///
    /// Sled batches writes for performance. Call this to force a flush.
    ///
    /// # Errors
    ///
    /// Returns error if flush fails (e.g., disk full).
    pub fn flush(&self) -> LoamSpineResult<()> {
        self.db
            .flush()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }
}

impl SpineStorage for SledSpineStorage {
    async fn get_spine(&self, id: SpineId) -> LoamSpineResult<Option<Spine>> {
        let key = id.as_bytes();
        match self.tree.get(key) {
            Ok(Some(bytes)) => {
                let spine: Spine = bincode::deserialize(&bytes)
                    .map_err(|e| LoamSpineError::Storage(format!("deserialize: {e}")))?;
                Ok(Some(spine))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(LoamSpineError::Storage(e.to_string())),
        }
    }

    async fn save_spine(&self, spine: &Spine) -> LoamSpineResult<()> {
        let key = spine.id.as_bytes();
        let bytes = bincode::serialize(spine)
            .map_err(|e| LoamSpineError::Storage(format!("serialize: {e}")))?;
        self.tree
            .insert(key, bytes)
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn delete_spine(&self, id: SpineId) -> LoamSpineResult<()> {
        let key = id.as_bytes();
        self.tree
            .remove(key)
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn list_spines(&self) -> LoamSpineResult<Vec<SpineId>> {
        let mut ids = Vec::new();
        for item in &self.tree {
            let (key, _) = item.map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            if key.len() == 16 {
                let mut bytes = [0u8; 16];
                bytes.copy_from_slice(&key);
                ids.push(SpineId::from_bytes(bytes));
            }
        }
        Ok(ids)
    }
}

/// Sled-backed entry storage for production use.
///
/// Maintains two trees:
/// - `entries`: Content-addressable entry storage (hash → entry)
/// - `entry_index`: Spine index for efficient queries (spine_id + index → hash)
#[derive(Clone)]
pub struct SledEntryStorage {
    db: sled::Db,
    entries_tree: sled::Tree,
    index_tree: sled::Tree,
}

impl SledEntryStorage {
    /// Open entry storage at the given path.
    ///
    /// Creates the directory if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be opened or directory cannot be created.
    pub fn open<P: AsRef<Path>>(path: P) -> LoamSpineResult<Self> {
        let db = sled::open(path).map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let entries_tree = db
            .open_tree("entries")
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let index_tree = db
            .open_tree("entry_index")
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(Self {
            db,
            entries_tree,
            index_tree,
        })
    }

    /// Create storage with a temporary database (for testing).
    ///
    /// The database is automatically deleted when dropped.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be created.
    pub fn temporary() -> LoamSpineResult<Self> {
        let config = sled::Config::new().temporary(true);
        let db = config
            .open()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let entries_tree = db
            .open_tree("entries")
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let index_tree = db
            .open_tree("entry_index")
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(Self {
            db,
            entries_tree,
            index_tree,
        })
    }

    /// Get the number of stored entries.
    ///
    /// This is an O(1) operation.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.entries_tree.len()
    }

    /// Flush all pending writes to disk.
    ///
    /// Sled batches writes for performance. Call this to force a flush.
    ///
    /// # Errors
    ///
    /// Returns error if flush fails (e.g., disk full).
    pub fn flush(&self) -> LoamSpineResult<()> {
        self.db
            .flush()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Create an index key for spine_id + entry_index.
    ///
    /// Format: `[spine_id (16 bytes)][entry_index (8 bytes)]`
    ///
    /// This allows efficient range queries for entries in a spine.
    fn make_index_key(spine_id: SpineId, entry_index: u64) -> Vec<u8> {
        let mut key = Vec::with_capacity(24);
        key.extend_from_slice(spine_id.as_bytes());
        key.extend_from_slice(&entry_index.to_be_bytes());
        key
    }
}

impl EntryStorage for SledEntryStorage {
    async fn get_entry(&self, hash: EntryHash) -> LoamSpineResult<Option<Entry>> {
        match self.entries_tree.get(hash) {
            Ok(Some(bytes)) => {
                let entry: Entry = bincode::deserialize(&bytes)
                    .map_err(|e| LoamSpineError::Storage(format!("deserialize: {e}")))?;
                Ok(Some(entry))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(LoamSpineError::Storage(e.to_string())),
        }
    }

    async fn save_entry(&self, entry: &Entry) -> LoamSpineResult<EntryHash> {
        let hash = entry.compute_hash();
        let bytes = bincode::serialize(entry)
            .map_err(|e| LoamSpineError::Storage(format!("serialize: {e}")))?;

        // Save the entry
        self.entries_tree
            .insert(hash, bytes)
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;

        // Update index: spine_id + entry_index → entry_hash
        let index_key = Self::make_index_key(entry.spine_id, entry.index);
        self.index_tree
            .insert(index_key, hash.as_slice())
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;

        Ok(hash)
    }

    async fn entry_exists(&self, hash: EntryHash) -> LoamSpineResult<bool> {
        self.entries_tree
            .contains_key(hash)
            .map_err(|e| LoamSpineError::Storage(e.to_string()))
    }

    async fn get_entries_for_spine(
        &self,
        spine_id: SpineId,
        start_index: u64,
        limit: u64,
    ) -> LoamSpineResult<Vec<Entry>> {
        let mut entries = Vec::new();
        let start_key = Self::make_index_key(spine_id, start_index);

        // Scan from start_key
        for item in self.index_tree.range(start_key..) {
            if entries.len() >= usize::try_from(limit).unwrap_or(usize::MAX) {
                break;
            }

            let (key, hash_bytes) = item.map_err(|e| LoamSpineError::Storage(e.to_string()))?;

            // Check if still same spine_id (first 16 bytes)
            if key.len() < 16 || &key[..16] != spine_id.as_bytes() {
                break;
            }

            // Get the entry
            if let Some(entry_bytes) = self
                .entries_tree
                .get(&hash_bytes)
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?
            {
                let entry: Entry = bincode::deserialize(&entry_bytes)
                    .map_err(|e| LoamSpineError::Storage(format!("deserialize: {e}")))?;
                entries.push(entry);
            }
        }

        Ok(entries)
    }
}

/// Combined Sled storage for both spines and entries.
///
/// Convenience wrapper that provides persistent storage for both types.
/// Uses separate subdirectories for spines and entries.
///
/// # Example
///
/// ```no_run
/// use loam_spine_core::storage::SledStorage;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Open persistent storage
/// let storage = SledStorage::open("./loamspine-data")?;
///
/// // Use for both spines and entries
/// // storage.spines.save_spine(&spine).await?;
/// // storage.entries.save_entry(&entry).await?;
///
/// // Flush to ensure durability
/// storage.flush()?;
/// # Ok(())
/// # }
/// ```
pub struct SledStorage {
    /// Spine storage component.
    pub spines: SledSpineStorage,
    /// Entry storage component.
    pub entries: SledEntryStorage,
}

impl SledStorage {
    /// Open storage at the given base path.
    ///
    /// Creates subdirectories:
    /// - `{base_path}/spines` — Spine storage
    /// - `{base_path}/entries` — Entry storage
    ///
    /// # Errors
    ///
    /// Returns error if databases cannot be opened or directories cannot be created.
    pub fn open<P: AsRef<Path>>(base_path: P) -> LoamSpineResult<Self> {
        let base = base_path.as_ref();
        let spines = SledSpineStorage::open(base.join("spines"))?;
        let entries = SledEntryStorage::open(base.join("entries"))?;
        Ok(Self { spines, entries })
    }

    /// Create storage with temporary databases (for testing).
    ///
    /// The databases are automatically deleted when dropped.
    ///
    /// # Errors
    ///
    /// Returns error if databases cannot be created.
    pub fn temporary() -> LoamSpineResult<Self> {
        let spines = SledSpineStorage::temporary()?;
        let entries = SledEntryStorage::temporary()?;
        Ok(Self { spines, entries })
    }

    /// Flush all pending writes to disk.
    ///
    /// Ensures all data is persisted. Call this before critical operations
    /// or when you need to guarantee durability.
    ///
    /// # Errors
    ///
    /// Returns error if flush fails (e.g., disk full).
    pub fn flush(&self) -> LoamSpineResult<()> {
        self.spines.flush()?;
        self.entries.flush()?;
        Ok(())
    }
}
