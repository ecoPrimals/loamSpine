// SPDX-License-Identifier: AGPL-3.0-or-later

//! redb-backed persistent storage for production use.
//!
//! Uses [redb](https://github.com/cberner/redb), a Pure Rust embedded database,
//! for persistent storage. No external dependencies or C tooling required.
//!
//! # Features
//!
//! - **Pure Rust**: No C dependencies, ecoBin compliant
//! - **ACID**: Atomic, consistent, isolated, durable operations
//! - **Embedded**: No separate database server needed
//! - **MVCC**: Concurrent readers without blocking writers
//!
//! # Production Use
//!
//! For production deployments, use [`RedbStorage::open`] with a persistent path:
//!
//! ```no_run
//! use loam_spine_core::storage::RedbStorage;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = RedbStorage::open("./loamspine-data")?;
//! # Ok(())
//! # }
//! ```

use std::path::Path;
use std::sync::Arc;

use redb::{Database, ReadableTable, ReadableTableMetadata, TableDefinition};

use crate::certificate::Certificate;
use crate::entry::Entry;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::spine::Spine;
use crate::types::{CertificateId, EntryHash, SpineId};

use super::{CertificateStorage, EntryStorage, SpineStorage};

/// Table definitions. Keys and values stored as bytes for flexibility.
const SPINES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("spines");
const ENTRIES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("entries");
const ENTRY_INDEX: TableDefinition<&[u8], &[u8]> = TableDefinition::new("entry_index");
const CERTIFICATES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("certificates");

/// redb-backed spine storage for production use.
///
/// Uses redb, a Pure Rust embedded database, for persistent storage.
#[derive(Clone)]
pub struct RedbSpineStorage {
    db: Arc<Database>,
}

impl RedbSpineStorage {
    /// Open spine storage at the given path.
    ///
    /// Creates the directory if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be opened or directory cannot be created.
    pub fn open<P: AsRef<Path>>(path: P) -> LoamSpineResult<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| LoamSpineError::Storage(format!("create dir: {e}")))?;
        }
        let db = Database::create(path).map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        ensure_table(&db, SPINES)?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Create storage with a temporary database (for testing).
    ///
    /// The database file is created in the system temp directory.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be created.
    pub fn temporary() -> LoamSpineResult<Self> {
        let path =
            std::env::temp_dir().join(format!("loamspine-redb-{}.redb", uuid::Uuid::now_v7()));
        Self::open(path)
    }

    /// Get the number of stored spines.
    #[must_use]
    pub fn spine_count(&self) -> usize {
        let Ok(read_txn) = self.db.begin_read() else {
            return 0;
        };
        let Ok(table) = read_txn.open_table(SPINES) else {
            return 0;
        };
        table.len().unwrap_or(0).try_into().unwrap_or(0)
    }

    /// Flush all pending writes to disk.
    ///
    /// redb commits on each write transaction; this is a no-op for compatibility.
    ///
    /// # Errors
    ///
    /// This method is infallible for redb (commits are synchronous) but returns
    /// Result for trait compatibility.
    pub const fn flush(&self) -> LoamSpineResult<()> {
        // redb commits synchronously; no explicit flush needed
        Ok(())
    }
}

impl SpineStorage for RedbSpineStorage {
    async fn get_spine(&self, id: SpineId) -> LoamSpineResult<Option<Spine>> {
        let key = id.as_bytes();
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let table = read_txn
            .open_table(SPINES)
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let value = table
            .get(key.as_ref())
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        match value {
            Some(guard) => {
                let bytes = guard.value();
                let spine: Spine = bincode::deserialize(bytes)
                    .map_err(|e| LoamSpineError::Storage(format!("deserialize: {e}")))?;
                Ok(Some(spine))
            }
            None => Ok(None),
        }
    }

    async fn save_spine(&self, spine: &Spine) -> LoamSpineResult<()> {
        let key = spine.id.as_bytes();
        let bytes = bincode::serialize(spine)
            .map_err(|e| LoamSpineError::Storage(format!("serialize: {e}")))?;
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(SPINES)
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            table
                .insert(key.as_ref(), bytes.as_slice())
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn delete_spine(&self, id: SpineId) -> LoamSpineResult<()> {
        let key = id.as_bytes();
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(SPINES)
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            table
                .remove(key.as_ref())
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn list_spines(&self) -> LoamSpineResult<Vec<SpineId>> {
        let mut ids = Vec::new();
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let table = read_txn
            .open_table(SPINES)
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let range = table
            .iter()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        for item in range {
            let (key_guard, _) = item.map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            let key = key_guard.value();
            if key.len() == 16 {
                let mut bytes = [0u8; 16];
                bytes.copy_from_slice(key);
                ids.push(SpineId::from_bytes(bytes));
            }
        }
        Ok(ids)
    }
}

/// redb-backed entry storage for production use.
///
/// Maintains two tables:
/// - `entries`: Content-addressable entry storage (hash → entry)
/// - `entry_index`: Spine index for efficient queries (spine_id + index → hash)
#[derive(Clone)]
pub struct RedbEntryStorage {
    db: Arc<Database>,
}

impl RedbEntryStorage {
    /// Open entry storage at the given path.
    ///
    /// # Errors
    ///
    /// Returns error if the database file cannot be created or opened.
    pub fn open<P: AsRef<Path>>(path: P) -> LoamSpineResult<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| LoamSpineError::Storage(format!("create dir: {e}")))?;
        }
        let db = Database::create(path).map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        ensure_table(&db, ENTRIES)?;
        ensure_table(&db, ENTRY_INDEX)?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Create storage with a temporary database (for testing).
    ///
    /// # Errors
    ///
    /// Returns error if the temporary database cannot be created.
    pub fn temporary() -> LoamSpineResult<Self> {
        let path = std::env::temp_dir().join(format!(
            "loamspine-redb-entries-{}.redb",
            uuid::Uuid::now_v7()
        ));
        Self::open(path)
    }

    /// Get the number of stored entries.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        let Ok(read_txn) = self.db.begin_read() else {
            return 0;
        };
        let Ok(table) = read_txn.open_table(ENTRIES) else {
            return 0;
        };
        table.len().unwrap_or(0).try_into().unwrap_or(0)
    }

    /// Flush all pending writes to disk.
    ///
    /// # Errors
    ///
    /// This method is infallible for redb (commits are synchronous) but returns
    /// Result for trait compatibility.
    pub const fn flush(&self) -> LoamSpineResult<()> {
        Ok(())
    }

    fn make_index_key(spine_id: SpineId, entry_index: u64) -> [u8; 24] {
        let mut key = [0u8; 24];
        key[..16].copy_from_slice(spine_id.as_bytes());
        key[16..].copy_from_slice(&entry_index.to_be_bytes());
        key
    }
}

impl EntryStorage for RedbEntryStorage {
    async fn get_entry(&self, hash: EntryHash) -> LoamSpineResult<Option<Entry>> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let table = read_txn
            .open_table(ENTRIES)
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let value = table
            .get(&hash[..])
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        match value {
            Some(guard) => {
                let bytes = guard.value();
                let entry: Entry = bincode::deserialize(bytes)
                    .map_err(|e| LoamSpineError::Storage(format!("deserialize: {e}")))?;
                Ok(Some(entry))
            }
            None => Ok(None),
        }
    }

    async fn save_entry(&self, entry: &Entry) -> LoamSpineResult<EntryHash> {
        let hash = entry.compute_hash()?;
        let bytes = bincode::serialize(entry)
            .map_err(|e| LoamSpineError::Storage(format!("serialize: {e}")))?;
        let index_key = Self::make_index_key(entry.spine_id, entry.index);

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        {
            let mut entries_table = write_txn
                .open_table(ENTRIES)
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            let mut index_table = write_txn
                .open_table(ENTRY_INDEX)
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            entries_table
                .insert(&hash[..], bytes.as_slice())
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            index_table
                .insert(index_key.as_slice(), &hash[..])
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(hash)
    }

    async fn entry_exists(&self, hash: EntryHash) -> LoamSpineResult<bool> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let table = read_txn
            .open_table(ENTRIES)
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let exists = table
            .get(&hash[..])
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?
            .is_some();
        Ok(exists)
    }

    async fn get_entries_for_spine(
        &self,
        spine_id: SpineId,
        start_index: u64,
        limit: u64,
    ) -> LoamSpineResult<Vec<Entry>> {
        let start_key = Self::make_index_key(spine_id, start_index);
        let limit_usize = usize::try_from(limit).unwrap_or(usize::MAX);
        let spine_prefix = spine_id.as_bytes();

        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let index_table = read_txn
            .open_table(ENTRY_INDEX)
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let entries_table = read_txn
            .open_table(ENTRIES)
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;

        let mut entries = Vec::new();
        let range = index_table
            .range(start_key.as_slice()..)
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        for item in range {
            if entries.len() >= limit_usize {
                break;
            }
            let (key_guard, hash_guard) =
                item.map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            let key = key_guard.value();
            if key.len() < 16 || &key[..16] != spine_prefix {
                break;
            }
            let hash_bytes = hash_guard.value();
            if let Some(entry_guard) = entries_table
                .get(hash_bytes)
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?
            {
                let entry: Entry = bincode::deserialize(entry_guard.value())
                    .map_err(|e| LoamSpineError::Storage(format!("deserialize: {e}")))?;
                entries.push(entry);
            }
        }
        Ok(entries)
    }
}

/// redb-backed certificate storage for production use.
#[derive(Clone)]
pub struct RedbCertificateStorage {
    db: Arc<Database>,
}

impl RedbCertificateStorage {
    /// Open certificate storage at the given path.
    ///
    /// # Errors
    ///
    /// Returns error if the database file cannot be created or opened.
    pub fn open<P: AsRef<Path>>(path: P) -> LoamSpineResult<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| LoamSpineError::Storage(format!("create dir: {e}")))?;
        }
        let db = Database::create(path).map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        ensure_table(&db, CERTIFICATES)?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Create storage with a temporary database (for testing).
    ///
    /// # Errors
    ///
    /// Returns error if the temporary database cannot be created.
    pub fn temporary() -> LoamSpineResult<Self> {
        let path = std::env::temp_dir().join(format!(
            "loamspine-redb-certs-{}.redb",
            uuid::Uuid::now_v7()
        ));
        Self::open(path)
    }

    /// Get the number of stored certificates.
    #[must_use]
    pub fn certificate_count(&self) -> usize {
        let Ok(read_txn) = self.db.begin_read() else {
            return 0;
        };
        let Ok(table) = read_txn.open_table(CERTIFICATES) else {
            return 0;
        };
        table.len().unwrap_or(0).try_into().unwrap_or(0)
    }

    /// Flush all pending writes to disk.
    ///
    /// # Errors
    ///
    /// This method is infallible for redb (commits are synchronous) but returns
    /// Result for trait compatibility.
    pub const fn flush(&self) -> LoamSpineResult<()> {
        Ok(())
    }
}

impl CertificateStorage for RedbCertificateStorage {
    async fn get_certificate(
        &self,
        id: CertificateId,
    ) -> LoamSpineResult<Option<(Certificate, SpineId)>> {
        let key = id.as_bytes();
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let table = read_txn
            .open_table(CERTIFICATES)
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let value = table
            .get(key.as_ref())
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        match value {
            Some(guard) => {
                let bytes = guard.value();
                let pair: (Certificate, SpineId) = bincode::deserialize(bytes)
                    .map_err(|e| LoamSpineError::Storage(format!("deserialize: {e}")))?;
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
        let bytes = bincode::serialize(&(certificate, spine_id))
            .map_err(|e| LoamSpineError::Storage(format!("serialize: {e}")))?;
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(CERTIFICATES)
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            table
                .insert(key.as_ref(), bytes.as_slice())
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn delete_certificate(&self, id: CertificateId) -> LoamSpineResult<()> {
        let key = id.as_bytes();
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(CERTIFICATES)
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            table
                .remove(key.as_ref())
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn list_certificates(&self) -> LoamSpineResult<Vec<CertificateId>> {
        let mut ids = Vec::new();
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let table = read_txn
            .open_table(CERTIFICATES)
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        let range = table
            .iter()
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        for item in range {
            let (key_guard, _) = item.map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            let key = key_guard.value();
            if key.len() == 16 {
                let mut bytes = [0u8; 16];
                bytes.copy_from_slice(key);
                ids.push(CertificateId::from_bytes(bytes));
            }
        }
        Ok(ids)
    }
}

/// Combined redb storage for spines, entries, and certificates.
///
/// Uses a single database file with separate tables.
pub struct RedbStorage {
    /// Spine storage component.
    pub spines: RedbSpineStorage,
    /// Entry storage component.
    pub entries: RedbEntryStorage,
    /// Certificate storage component.
    pub certificates: RedbCertificateStorage,
}

impl RedbStorage {
    /// Open storage at the given base path.
    ///
    /// Creates subdirectories for each component:
    /// - `{base_path}/spines.redb` — Spine storage
    /// - `{base_path}/entries.redb` — Entry storage
    /// - `{base_path}/certificates.redb` — Certificate storage
    ///
    /// # Errors
    ///
    /// Returns error if the database file cannot be created or opened.
    pub fn open<P: AsRef<Path>>(base_path: P) -> LoamSpineResult<Self> {
        let base = base_path.as_ref();
        let spines = RedbSpineStorage::open(base.join("spines.redb"))?;
        let entries = RedbEntryStorage::open(base.join("entries.redb"))?;
        let certificates = RedbCertificateStorage::open(base.join("certificates.redb"))?;
        Ok(Self {
            spines,
            entries,
            certificates,
        })
    }

    /// Create storage with temporary databases (for testing).
    ///
    /// # Errors
    ///
    /// Returns error if the temporary database cannot be created.
    pub fn temporary() -> LoamSpineResult<Self> {
        let spines = RedbSpineStorage::temporary()?;
        let entries = RedbEntryStorage::temporary()?;
        let certificates = RedbCertificateStorage::temporary()?;
        Ok(Self {
            spines,
            entries,
            certificates,
        })
    }

    /// Flush all pending writes to disk.
    ///
    /// # Errors
    ///
    /// This method is infallible for redb (commits are synchronous) but returns
    /// Result for trait compatibility.
    pub fn flush(&self) -> LoamSpineResult<()> {
        self.spines.flush()?;
        self.entries.flush()?;
        self.certificates.flush()?;
        Ok(())
    }
}

/// Ensure a table exists (redb creates on first write; we need for reads).
fn ensure_table(db: &Database, table: TableDefinition<&[u8], &[u8]>) -> LoamSpineResult<()> {
    let write_txn = db
        .begin_write()
        .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
    {
        let _ = write_txn
            .open_table(table)
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
    }
    write_txn
        .commit()
        .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
    Ok(())
}
