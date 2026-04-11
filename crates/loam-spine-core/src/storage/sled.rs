// SPDX-License-Identifier: AGPL-3.0-or-later

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

use crate::certificate::Certificate;
use crate::entry::Entry;
use crate::error::{LoamSpineResult, StorageResultExt};
use crate::spine::Spine;
use crate::types::{CertificateId, EntryHash, SpineId};

use super::{CertificateStorage, EntryStorage, SpineStorage};

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
        let db = sled::open(path).storage_err()?;
        let tree = db.open_tree("spines").storage_err()?;
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
        let db = config.open().storage_err()?;
        let tree = db.open_tree("spines").storage_err()?;
        Ok(Self { db, tree })
    }

    /// Wrap a pre-opened sled `Db` handle.
    ///
    /// Avoids the close-reopen lock contention that can occur in parallel tests
    /// when the same database path is opened by raw `sled::open` and then by
    /// this constructor.
    ///
    /// # Errors
    ///
    /// Returns error if the tree cannot be opened.
    pub fn from_db(db: sled::Db) -> LoamSpineResult<Self> {
        let tree = db.open_tree("spines").storage_err()?;
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
        self.db.flush().storage_err()?;
        Ok(())
    }
}

impl SpineStorage for SledSpineStorage {
    async fn get_spine(&self, id: SpineId) -> LoamSpineResult<Option<Spine>> {
        let key = id.as_bytes();
        let value = self.tree.get(key).storage_err()?;
        match value {
            Some(bytes) => {
                let spine: Spine = bincode::deserialize(&bytes).storage_ctx("deserialize")?;
                Ok(Some(spine))
            }
            None => Ok(None),
        }
    }

    async fn save_spine(&self, spine: &Spine) -> LoamSpineResult<()> {
        let key = spine.id.as_bytes();
        let bytes = bincode::serialize(spine).storage_ctx("serialize")?;
        self.tree.insert(key, bytes).storage_err()?;
        Ok(())
    }

    async fn delete_spine(&self, id: SpineId) -> LoamSpineResult<()> {
        let key = id.as_bytes();
        self.tree.remove(key).storage_err()?;
        Ok(())
    }

    async fn list_spines(&self) -> LoamSpineResult<Vec<SpineId>> {
        let mut ids = Vec::new();
        for item in &self.tree {
            let (key, _) = item.storage_err()?;
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
        let db = sled::open(path).storage_err()?;
        let entries_tree = db.open_tree("entries").storage_err()?;
        let index_tree = db.open_tree("entry_index").storage_err()?;
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
        let db = config.open().storage_err()?;
        let entries_tree = db.open_tree("entries").storage_err()?;
        let index_tree = db.open_tree("entry_index").storage_err()?;
        Ok(Self {
            db,
            entries_tree,
            index_tree,
        })
    }

    /// Wrap a pre-opened sled `Db` handle.
    ///
    /// # Errors
    ///
    /// Returns error if the trees cannot be opened.
    pub fn from_db(db: sled::Db) -> LoamSpineResult<Self> {
        let entries_tree = db.open_tree("entries").storage_err()?;
        let index_tree = db.open_tree("entry_index").storage_err()?;
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
        self.db.flush().storage_err()?;
        Ok(())
    }

    /// Create an index key for spine_id + entry_index.
    ///
    /// Format: `[spine_id (16 bytes)][entry_index (8 bytes)]`
    ///
    /// This allows efficient range queries for entries in a spine.
    fn make_index_key(spine_id: SpineId, entry_index: u64) -> [u8; 24] {
        let mut key = [0u8; 24];
        key[..16].copy_from_slice(spine_id.as_bytes());
        key[16..].copy_from_slice(&entry_index.to_be_bytes());
        key
    }
}

impl EntryStorage for SledEntryStorage {
    async fn get_entry(&self, hash: EntryHash) -> LoamSpineResult<Option<Entry>> {
        let value = self.entries_tree.get(hash).storage_err()?;
        match value {
            Some(bytes) => {
                let entry: Entry = bincode::deserialize(&bytes).storage_ctx("deserialize")?;
                Ok(Some(entry))
            }
            None => Ok(None),
        }
    }

    async fn save_entry(&self, entry: &Entry) -> LoamSpineResult<EntryHash> {
        let hash = entry.compute_hash()?;
        let bytes = bincode::serialize(entry).storage_ctx("serialize")?;
        self.entries_tree.insert(hash, bytes).storage_err()?;
        let index_key = Self::make_index_key(entry.spine_id, entry.index);
        self.index_tree
            .insert(index_key, hash.as_slice())
            .storage_err()?;
        Ok(hash)
    }

    async fn entry_exists(&self, hash: EntryHash) -> LoamSpineResult<bool> {
        self.entries_tree.contains_key(hash).storage_err()
    }

    async fn get_entries_for_spine(
        &self,
        spine_id: SpineId,
        start_index: u64,
        limit: u64,
    ) -> LoamSpineResult<Vec<Entry>> {
        let mut entries = Vec::new();
        let start_key = Self::make_index_key(spine_id, start_index);

        for item in self.index_tree.range(start_key..) {
            if entries.len() >= usize::try_from(limit).unwrap_or(usize::MAX) {
                break;
            }

            let (key, hash_bytes) = item.storage_err()?;

            if key.len() < 16 || &key[..16] != spine_id.as_bytes() {
                break;
            }

            if let Some(entry_bytes) = self.entries_tree.get(&hash_bytes).storage_err()? {
                let entry: Entry = bincode::deserialize(&entry_bytes).storage_ctx("deserialize")?;
                entries.push(entry);
            }
        }

        Ok(entries)
    }
}

/// Sled-backed certificate storage for production use.
///
/// Stores `(Certificate, SpineId)` pairs in a dedicated `certificates` tree,
/// keyed by `CertificateId` (UUID, 16 bytes). Uses bincode for serialization.
#[derive(Clone)]
pub struct SledCertificateStorage {
    db: sled::Db,
    tree: sled::Tree,
}

impl SledCertificateStorage {
    /// Open certificate storage at the given path.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be opened.
    pub fn open<P: AsRef<Path>>(path: P) -> LoamSpineResult<Self> {
        let db = sled::open(path).storage_err()?;
        let tree = db.open_tree("certificates").storage_err()?;
        Ok(Self { db, tree })
    }

    /// Create storage with a temporary database (for testing).
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be created.
    pub fn temporary() -> LoamSpineResult<Self> {
        let config = sled::Config::new().temporary(true);
        let db = config.open().storage_err()?;
        let tree = db.open_tree("certificates").storage_err()?;
        Ok(Self { db, tree })
    }

    /// Wrap a pre-opened sled `Db` handle.
    ///
    /// # Errors
    ///
    /// Returns error if the tree cannot be opened.
    pub fn from_db(db: sled::Db) -> LoamSpineResult<Self> {
        let tree = db.open_tree("certificates").storage_err()?;
        Ok(Self { db, tree })
    }

    /// Get the number of stored certificates.
    #[must_use]
    pub fn certificate_count(&self) -> usize {
        self.tree.len()
    }

    /// Flush all pending writes to disk.
    ///
    /// # Errors
    ///
    /// Returns error if flush fails.
    pub fn flush(&self) -> LoamSpineResult<()> {
        self.db.flush().storage_err()?;
        Ok(())
    }
}

impl CertificateStorage for SledCertificateStorage {
    async fn get_certificate(
        &self,
        id: CertificateId,
    ) -> LoamSpineResult<Option<(Certificate, SpineId)>> {
        let value = self.tree.get(id.as_bytes()).storage_err()?;
        match value {
            Some(bytes) => {
                let pair: (Certificate, SpineId) =
                    bincode::deserialize(&bytes).storage_ctx("deserialize")?;
                Ok(Some(pair))
            }
            None => Ok(None),
        }
    }

    async fn save_certificate(
        &self,
        certificate: &Certificate,
        spine_id: SpineId,
    ) -> LoamSpineResult<()> {
        let key = certificate.id.as_bytes();
        let bytes = bincode::serialize(&(certificate, spine_id)).storage_ctx("serialize")?;
        self.tree.insert(key, bytes).storage_err()?;
        Ok(())
    }

    async fn delete_certificate(&self, id: CertificateId) -> LoamSpineResult<()> {
        self.tree.remove(id.as_bytes()).storage_err()?;
        Ok(())
    }

    async fn list_certificates(&self) -> LoamSpineResult<Vec<CertificateId>> {
        let mut ids = Vec::new();
        for item in &self.tree {
            let (key, _) = item.storage_err()?;
            if key.len() == 16 {
                let mut bytes = [0u8; 16];
                bytes.copy_from_slice(&key);
                ids.push(CertificateId::from_bytes(bytes));
            }
        }
        Ok(ids)
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
    /// Certificate storage component.
    pub certificates: SledCertificateStorage,
}

impl SledStorage {
    /// Open storage at the given base path.
    ///
    /// Creates subdirectories:
    /// - `{base_path}/spines` — Spine storage
    /// - `{base_path}/entries` — Entry storage
    /// - `{base_path}/certificates` — Certificate storage
    ///
    /// # Errors
    ///
    /// Returns error if databases cannot be opened or directories cannot be created.
    pub fn open<P: AsRef<Path>>(base_path: P) -> LoamSpineResult<Self> {
        let base = base_path.as_ref();
        let spines = SledSpineStorage::open(base.join("spines"))?;
        let entries = SledEntryStorage::open(base.join("entries"))?;
        let certificates = SledCertificateStorage::open(base.join("certificates"))?;
        Ok(Self {
            spines,
            entries,
            certificates,
        })
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
        let certificates = SledCertificateStorage::temporary()?;
        Ok(Self {
            spines,
            entries,
            certificates,
        })
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
        self.certificates.flush()?;
        Ok(())
    }
}
