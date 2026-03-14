// SPDX-License-Identifier: AGPL-3.0-only

//! SQLite storage backend. Feature-gated behind `sqlite`.
//!
//! Note: bundles C SQLite library — not ecoBin compliant. Use for development or
//! deployments where SQLite compatibility is required.

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::certificate::Certificate;
use crate::entry::Entry;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::spine::Spine;
use crate::types::{CertificateId, EntryHash, SpineId};

use super::{CertificateStorage, EntryStorage, SpineStorage};

/// SQLite-backed spine storage.
///
/// Uses JSON serialization for queryability. Wraps `rusqlite::Connection` in
/// `Arc<Mutex<>>` for thread safety.
#[derive(Clone)]
pub struct SqliteSpineStorage {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteSpineStorage {
    /// Open spine storage at the given path.
    ///
    /// Creates the database file and tables if they don't exist.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be opened or schema creation fails.
    pub fn open<P: AsRef<Path>>(path: P) -> LoamSpineResult<Self> {
        let conn = Connection::open(path).map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Self::init_conn(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create storage with an in-memory database (for testing).
    ///
    /// The database is automatically deleted when dropped.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be created.
    pub fn temporary() -> LoamSpineResult<Self> {
        let conn =
            Connection::open_in_memory().map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Self::init_conn(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn init_conn(conn: &Connection) -> LoamSpineResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS spines (id TEXT PRIMARY KEY, data BLOB)",
            [],
        )
        .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Get the number of stored spines.
    #[must_use]
    pub fn spine_count(&self) -> usize {
        let Ok(conn) = self.conn.lock() else {
            return 0;
        };
        conn.query_row("SELECT COUNT(*) FROM spines", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_or(0, |n| usize::try_from(n).unwrap_or(0))
    }

    /// Flush all pending writes to disk.
    ///
    /// SQLite uses WAL mode when available. This ensures durability.
    ///
    /// # Errors
    ///
    /// Returns error if checkpoint/flush fails.
    pub fn flush(&self) -> LoamSpineResult<()> {
        {
            let conn = self
                .conn
                .lock()
                .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        }
        Ok(())
    }
}

#[allow(clippy::significant_drop_tightening)] // MutexGuard<Connection> must outlive Rows/Statement borrows
impl SpineStorage for SqliteSpineStorage {
    async fn get_spine(&self, id: SpineId) -> LoamSpineResult<Option<Spine>> {
        let id_str = id.to_string();
        let result = {
            let conn = self
                .conn
                .lock()
                .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
            let mut stmt = conn
                .prepare("SELECT data FROM spines WHERE id = ?1")
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            let mut rows = stmt
                .query([&id_str])
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;

            if let Some(row) = rows
                .next()
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?
            {
                let data: Vec<u8> = row
                    .get(0)
                    .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
                let spine: Spine = serde_json::from_slice(&data)
                    .map_err(|e| LoamSpineError::Storage(format!("deserialize: {e}")))?;
                Some(spine)
            } else {
                None
            }
        };
        Ok(result)
    }

    async fn save_spine(&self, spine: &Spine) -> LoamSpineResult<()> {
        let id_str = spine.id.to_string();
        let data = serde_json::to_vec(spine)
            .map_err(|e| LoamSpineError::Storage(format!("serialize: {e}")))?;
        {
            let conn = self
                .conn
                .lock()
                .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
            conn.execute(
                "INSERT OR REPLACE INTO spines (id, data) VALUES (?1, ?2)",
                rusqlite::params![&id_str, &data],
            )
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        }
        Ok(())
    }

    async fn delete_spine(&self, id: SpineId) -> LoamSpineResult<()> {
        let id_str = id.to_string();
        {
            let conn = self
                .conn
                .lock()
                .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
            conn.execute("DELETE FROM spines WHERE id = ?1", [&id_str])
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        }
        Ok(())
    }

    async fn list_spines(&self) -> LoamSpineResult<Vec<SpineId>> {
        let ids = {
            let conn = self
                .conn
                .lock()
                .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
            let mut stmt = conn
                .prepare("SELECT id FROM spines")
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            let mut rows = stmt
                .query([])
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;

            let mut ids = Vec::new();
            while let Some(row) = rows
                .next()
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?
            {
                let id_str: String = row
                    .get(0)
                    .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
                if let Ok(uuid) = SpineId::parse_str(&id_str) {
                    ids.push(uuid);
                }
            }
            ids
        };
        Ok(ids)
    }
}

/// SQLite-backed entry storage.
///
/// Uses JSON serialization for queryability. Maintains entries table with
/// spine_id and entry_index for efficient range queries.
#[derive(Clone)]
pub struct SqliteEntryStorage {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteEntryStorage {
    /// Open entry storage at the given path.
    ///
    /// Creates the database file and tables if they don't exist.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be opened or schema creation fails.
    pub fn open<P: AsRef<Path>>(path: P) -> LoamSpineResult<Self> {
        let conn = Connection::open(path).map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Self::init_conn(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create storage with an in-memory database (for testing).
    ///
    /// The database is automatically deleted when dropped.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be created.
    pub fn temporary() -> LoamSpineResult<Self> {
        let conn =
            Connection::open_in_memory().map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Self::init_conn(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn init_conn(conn: &Connection) -> LoamSpineResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS entries (
                hash BLOB PRIMARY KEY,
                spine_id TEXT NOT NULL,
                entry_index INTEGER NOT NULL,
                data BLOB NOT NULL
            )",
            [],
        )
        .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_entries_spine ON entries(spine_id, entry_index)",
            [],
        )
        .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Get the number of stored entries.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        let Ok(conn) = self.conn.lock() else {
            return 0;
        };
        conn.query_row("SELECT COUNT(*) FROM entries", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_or(0, |n| usize::try_from(n).unwrap_or(0))
    }

    /// Flush all pending writes to disk.
    ///
    /// # Errors
    ///
    /// Returns error if checkpoint/flush fails.
    pub fn flush(&self) -> LoamSpineResult<()> {
        {
            let conn = self
                .conn
                .lock()
                .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        }
        Ok(())
    }
}

#[allow(clippy::significant_drop_tightening)] // MutexGuard<Connection> must outlive Rows/Statement borrows
impl EntryStorage for SqliteEntryStorage {
    async fn get_entry(&self, hash: EntryHash) -> LoamSpineResult<Option<Entry>> {
        let result = {
            let conn = self
                .conn
                .lock()
                .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
            let mut stmt = conn
                .prepare("SELECT data FROM entries WHERE hash = ?1")
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            let mut rows = stmt
                .query([hash.as_slice()])
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;

            if let Some(row) = rows
                .next()
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?
            {
                let data: Vec<u8> = row
                    .get(0)
                    .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
                let entry: Entry = serde_json::from_slice(&data)
                    .map_err(|e| LoamSpineError::Storage(format!("deserialize: {e}")))?;
                Some(entry)
            } else {
                None
            }
        };
        Ok(result)
    }

    async fn save_entry(&self, entry: &Entry) -> LoamSpineResult<EntryHash> {
        let hash = entry.compute_hash()?;
        let spine_id_str = entry.spine_id.to_string();
        let data = serde_json::to_vec(entry)
            .map_err(|e| LoamSpineError::Storage(format!("serialize: {e}")))?;

        {
            let conn = self
                .conn
                .lock()
                .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
            conn.execute(
                "INSERT OR REPLACE INTO entries (hash, spine_id, entry_index, data) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![hash.as_slice(), &spine_id_str, i64::try_from(entry.index).unwrap_or(i64::MAX), &data],
            )
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        }
        Ok(hash)
    }

    async fn entry_exists(&self, hash: EntryHash) -> LoamSpineResult<bool> {
        let count: i32 = {
            let conn = self
                .conn
                .lock()
                .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
            conn.query_row(
                "SELECT COUNT(*) FROM entries WHERE hash = ?1",
                [hash.as_slice()],
                |row| row.get(0),
            )
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?
        };
        Ok(count > 0)
    }

    async fn get_entries_for_spine(
        &self,
        spine_id: SpineId,
        start_index: u64,
        limit: u64,
    ) -> LoamSpineResult<Vec<Entry>> {
        let spine_id_str = spine_id.to_string();
        let start_i64 = i64::try_from(start_index).unwrap_or(i64::MAX);
        let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);

        let entries = {
            let conn = self
                .conn
                .lock()
                .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
            let mut stmt = conn
                .prepare(
                    "SELECT data FROM entries WHERE spine_id = ?1 AND entry_index >= ?2 ORDER BY entry_index ASC LIMIT ?3",
                )
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            let mut rows = stmt
                .query(rusqlite::params![&spine_id_str, start_i64, limit_i64])
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;

            let mut entries = Vec::new();
            while let Some(row) = rows
                .next()
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?
            {
                let data: Vec<u8> = row
                    .get(0)
                    .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
                let entry: Entry = serde_json::from_slice(&data)
                    .map_err(|e| LoamSpineError::Storage(format!("deserialize: {e}")))?;
                entries.push(entry);
            }
            entries
        };
        Ok(entries)
    }
}

/// SQLite-backed certificate storage.
///
/// Stores `(Certificate, SpineId)` pairs in a `certificates` table,
/// keyed by `CertificateId` (UUID text). Uses JSON serialization for queryability.
#[derive(Clone)]
pub struct SqliteCertificateStorage {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteCertificateStorage {
    /// Open certificate storage at the given path.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be opened or schema creation fails.
    pub fn open<P: AsRef<Path>>(path: P) -> LoamSpineResult<Self> {
        let conn = Connection::open(path).map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Self::init_conn(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create storage with an in-memory database (for testing).
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be created.
    pub fn temporary() -> LoamSpineResult<Self> {
        let conn =
            Connection::open_in_memory().map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Self::init_conn(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn init_conn(conn: &Connection) -> LoamSpineResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS certificates (
                id TEXT PRIMARY KEY,
                spine_id TEXT NOT NULL,
                data BLOB NOT NULL
            )",
            [],
        )
        .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Get the number of stored certificates.
    #[must_use]
    pub fn certificate_count(&self) -> usize {
        let Ok(conn) = self.conn.lock() else {
            return 0;
        };
        conn.query_row("SELECT COUNT(*) FROM certificates", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_or(0, |n| usize::try_from(n).unwrap_or(0))
    }

    /// Flush all pending writes to disk.
    ///
    /// # Errors
    ///
    /// Returns error if checkpoint/flush fails.
    pub fn flush(&self) -> LoamSpineResult<()> {
        {
            let conn = self
                .conn
                .lock()
                .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        }
        Ok(())
    }
}

#[allow(clippy::significant_drop_tightening)] // MutexGuard<Connection> must outlive Rows/Statement borrows
impl CertificateStorage for SqliteCertificateStorage {
    async fn get_certificate(
        &self,
        id: CertificateId,
    ) -> LoamSpineResult<Option<(Certificate, SpineId)>> {
        let id_str = id.to_string();
        let result = {
            let conn = self
                .conn
                .lock()
                .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
            let mut stmt = conn
                .prepare("SELECT spine_id, data FROM certificates WHERE id = ?1")
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            let mut rows = stmt
                .query([&id_str])
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;

            if let Some(row) = rows
                .next()
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?
            {
                let spine_id_str: String = row
                    .get(0)
                    .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
                let data: Vec<u8> = row
                    .get(1)
                    .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
                let spine_id = SpineId::parse_str(&spine_id_str)
                    .map_err(|e| LoamSpineError::Storage(format!("invalid spine_id: {e}")))?;
                let cert: Certificate = serde_json::from_slice(&data)
                    .map_err(|e| LoamSpineError::Storage(format!("deserialize: {e}")))?;
                Some((cert, spine_id))
            } else {
                None
            }
        };
        Ok(result)
    }

    async fn save_certificate(
        &self,
        certificate: &Certificate,
        spine_id: SpineId,
    ) -> LoamSpineResult<()> {
        let id_str = certificate.id.to_string();
        let spine_id_str = spine_id.to_string();
        let data = serde_json::to_vec(certificate)
            .map_err(|e| LoamSpineError::Storage(format!("serialize: {e}")))?;
        {
            let conn = self
                .conn
                .lock()
                .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
            conn.execute(
                "INSERT OR REPLACE INTO certificates (id, spine_id, data) VALUES (?1, ?2, ?3)",
                rusqlite::params![&id_str, &spine_id_str, &data],
            )
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        }
        Ok(())
    }

    async fn delete_certificate(&self, id: CertificateId) -> LoamSpineResult<()> {
        let id_str = id.to_string();
        {
            let conn = self
                .conn
                .lock()
                .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
            conn.execute("DELETE FROM certificates WHERE id = ?1", [&id_str])
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        }
        Ok(())
    }

    async fn list_certificates(&self) -> LoamSpineResult<Vec<CertificateId>> {
        let ids = {
            let conn = self
                .conn
                .lock()
                .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
            let mut stmt = conn
                .prepare("SELECT id FROM certificates")
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
            let mut rows = stmt
                .query([])
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?;

            let mut ids = Vec::new();
            while let Some(row) = rows
                .next()
                .map_err(|e| LoamSpineError::Storage(e.to_string()))?
            {
                let id_str: String = row
                    .get(0)
                    .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
                if let Ok(uuid) = CertificateId::parse_str(&id_str) {
                    ids.push(uuid);
                }
            }
            ids
        };
        Ok(ids)
    }
}

/// Combined SQLite storage for both spines and entries.
///
/// Uses a single database file with separate tables for spines and entries.
/// Convenience wrapper that provides persistent storage for both types.
///
/// # Example
///
/// ```no_run
/// use loam_spine_core::storage::SqliteStorage;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let storage = SqliteStorage::open("./loamspine-data.db")?;
/// // storage.spines.save_spine(&spine);
/// // storage.entries.save_entry(&entry);
/// storage.flush()?;
/// # Ok(())
/// # }
/// ```
pub struct SqliteStorage {
    /// Spine storage component.
    pub spines: SqliteSpineStorage,
    /// Entry storage component.
    pub entries: SqliteEntryStorage,
    /// Certificate storage component.
    pub certificates: SqliteCertificateStorage,
}

impl SqliteStorage {
    /// Open storage at the given path.
    ///
    /// Uses a single database file with `spines`, `entries`, and `certificates` tables.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be opened or schema creation fails.
    pub fn open<P: AsRef<Path>>(path: P) -> LoamSpineResult<Self> {
        let path_ref = path.as_ref();
        let conn =
            Connection::open(path_ref).map_err(|e| LoamSpineError::Storage(e.to_string()))?;

        SqliteSpineStorage::init_conn(&conn)?;
        SqliteEntryStorage::init_conn(&conn)?;
        SqliteCertificateStorage::init_conn(&conn)?;

        let conn = Arc::new(Mutex::new(conn));
        Ok(Self {
            spines: SqliteSpineStorage {
                conn: Arc::clone(&conn),
            },
            entries: SqliteEntryStorage {
                conn: Arc::clone(&conn),
            },
            certificates: SqliteCertificateStorage { conn },
        })
    }

    /// Create storage with an in-memory database (for testing).
    ///
    /// The database is automatically deleted when dropped.
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be created.
    pub fn temporary() -> LoamSpineResult<Self> {
        let conn =
            Connection::open_in_memory().map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        SqliteSpineStorage::init_conn(&conn)?;
        SqliteEntryStorage::init_conn(&conn)?;
        SqliteCertificateStorage::init_conn(&conn)?;

        let conn = Arc::new(Mutex::new(conn));
        Ok(Self {
            spines: SqliteSpineStorage {
                conn: Arc::clone(&conn),
            },
            entries: SqliteEntryStorage {
                conn: Arc::clone(&conn),
            },
            certificates: SqliteCertificateStorage { conn },
        })
    }

    /// Flush all pending writes to disk.
    ///
    /// Ensures all data is persisted.
    ///
    /// # Errors
    ///
    /// Returns error if flush fails.
    pub fn flush(&self) -> LoamSpineResult<()> {
        self.spines.flush()?;
        self.entries.flush()?;
        self.certificates.flush()?;
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[cfg(feature = "sqlite")]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::certificate::{Certificate, CertificateType, MintInfo};
    use crate::entry::{Entry, EntryType, SpineConfig};
    use crate::spine::Spine;
    use crate::types::{CertificateId, Did, SpineId, Timestamp};
    use tempfile::TempDir;

    fn create_test_spine() -> Spine {
        let owner = Did::new("did:key:z6MkOwner");
        Spine::new(owner, Some("Test".into()), SpineConfig::default())
            .unwrap_or_else(|_| unreachable!())
    }

    fn create_test_entry(owner: &Did, spine_id: SpineId) -> Entry {
        Entry::genesis(owner.clone(), spine_id, SpineConfig::default())
    }

    fn create_test_certificate(owner: &Did, spine_id: SpineId) -> Certificate {
        let cert_id = CertificateId::now_v7();
        let mint_info = MintInfo {
            minter: owner.clone(),
            spine: spine_id,
            entry: [0u8; 32],
            timestamp: Timestamp::now(),
            authority: None,
        };
        Certificate::new(
            cert_id,
            CertificateType::DigitalGame {
                platform: "steam".into(),
                game_id: "hl3".into(),
                edition: None,
            },
            owner,
            &mint_info,
        )
    }

    fn spine_storage_from_tempdir() -> (TempDir, SqliteSpineStorage) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("spines.db");
        let storage = SqliteSpineStorage::open(&db_path).unwrap();
        (temp_dir, storage)
    }

    fn entry_storage_from_tempdir() -> (TempDir, SqliteEntryStorage) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("entries.db");
        let storage = SqliteEntryStorage::open(&db_path).unwrap();
        (temp_dir, storage)
    }

    fn certificate_storage_from_tempdir() -> (TempDir, SqliteCertificateStorage) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("certs.db");
        let storage = SqliteCertificateStorage::open(&db_path).unwrap();
        (temp_dir, storage)
    }

    // ========================================================================
    // SqliteSpineStorage tests
    // ========================================================================

    #[tokio::test]
    async fn sqlite_spine_storage_crud() {
        let (_temp_dir, storage) = spine_storage_from_tempdir();
        let spine = create_test_spine();
        let id = spine.id;

        // Create
        storage.save_spine(&spine).await.unwrap();
        assert_eq!(storage.spine_count(), 1);

        // Read
        let retrieved = storage.get_spine(id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, id);

        // Update
        let mut updated_spine = spine;
        updated_spine.height = 42;
        storage.save_spine(&updated_spine).await.unwrap();
        let retrieved = storage.get_spine(id).await.unwrap().unwrap();
        assert_eq!(retrieved.height, 42);

        // Delete
        storage.delete_spine(id).await.unwrap();
        assert_eq!(storage.spine_count(), 0);
        assert!(storage.get_spine(id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn sqlite_spine_storage_list_empty() {
        let (_temp_dir, storage) = spine_storage_from_tempdir();
        let ids = storage.list_spines().await.unwrap();
        assert!(ids.is_empty());
    }

    #[tokio::test]
    async fn sqlite_spine_storage_list_populated() {
        let (_temp_dir, storage) = spine_storage_from_tempdir();
        let spine1 = create_test_spine();
        let spine2 = Spine::new(
            Did::new("did:key:z6MkOther"),
            Some("Other".into()),
            SpineConfig::default(),
        )
        .unwrap();
        storage.save_spine(&spine1).await.unwrap();
        storage.save_spine(&spine2).await.unwrap();

        let ids = storage.list_spines().await.unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&spine1.id));
        assert!(ids.contains(&spine2.id));
    }

    #[tokio::test]
    async fn sqlite_spine_storage_get_nonexistent() {
        let (_temp_dir, storage) = spine_storage_from_tempdir();
        let result = storage.get_spine(SpineId::now_v7()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn sqlite_spine_storage_delete_nonexistent_idempotent() {
        let (_temp_dir, storage) = spine_storage_from_tempdir();
        let result = storage.delete_spine(SpineId::now_v7()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sqlite_spine_storage_flush() {
        let (_temp_dir, storage) = spine_storage_from_tempdir();
        storage.save_spine(&create_test_spine()).await.unwrap();
        let result = storage.flush();
        assert!(result.is_ok());
    }

    // ========================================================================
    // SqliteEntryStorage tests
    // ========================================================================

    #[tokio::test]
    async fn sqlite_entry_storage_save_and_retrieve() {
        let (_temp_dir, storage) = entry_storage_from_tempdir();
        let owner = Did::new("did:key:z6MkOwner");
        let spine_id = SpineId::now_v7();
        let entry = create_test_entry(&owner, spine_id);

        let hash = storage.save_entry(&entry).await.unwrap();
        assert_eq!(storage.entry_count(), 1);

        let retrieved = storage.get_entry(hash).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().spine_id, spine_id);
    }

    #[tokio::test]
    async fn sqlite_entry_storage_entry_exists() {
        let (_temp_dir, storage) = entry_storage_from_tempdir();
        let owner = Did::new("did:key:z6MkOwner");
        let spine_id = SpineId::now_v7();
        let entry = create_test_entry(&owner, spine_id);

        let hash = storage.save_entry(&entry).await.unwrap();
        assert!(storage.entry_exists(hash).await.unwrap());

        assert!(!storage.entry_exists([0u8; 32]).await.unwrap());
    }

    #[tokio::test]
    async fn sqlite_entry_storage_get_entries_for_spine() {
        let (_temp_dir, storage) = entry_storage_from_tempdir();
        let owner = Did::new("did:key:z6MkOwner");
        let spine_id = SpineId::now_v7();

        let mut prev_hash = None;
        for i in 0..5 {
            let entry = if i == 0 {
                Entry::genesis(owner.clone(), spine_id, SpineConfig::default())
            } else {
                Entry::new(
                    i,
                    prev_hash,
                    owner.clone(),
                    EntryType::SpineSealed { reason: None },
                )
                .with_spine_id(spine_id)
            };
            prev_hash = Some(storage.save_entry(&entry).await.unwrap());
        }

        let entries = storage
            .get_entries_for_spine(spine_id, 0, 10)
            .await
            .unwrap();
        assert_eq!(entries.len(), 5);

        let entries = storage.get_entries_for_spine(spine_id, 1, 2).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].index, 1);
        assert_eq!(entries[1].index, 2);
    }

    #[tokio::test]
    async fn sqlite_entry_storage_empty_spine_returns_empty() {
        let (_temp_dir, storage) = entry_storage_from_tempdir();
        let entries = storage
            .get_entries_for_spine(SpineId::now_v7(), 0, 10)
            .await
            .unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn sqlite_entry_storage_flush() {
        let (_temp_dir, storage) = entry_storage_from_tempdir();
        let owner = Did::new("did:key:z6MkOwner");
        let entry = create_test_entry(&owner, SpineId::now_v7());
        storage.save_entry(&entry).await.unwrap();
        let result = storage.flush();
        assert!(result.is_ok());
    }

    // ========================================================================
    // SqliteCertificateStorage tests
    // ========================================================================

    #[tokio::test]
    async fn sqlite_certificate_storage_crud() {
        let (_temp_dir, storage) = certificate_storage_from_tempdir();
        let owner = Did::new("did:key:z6MkOwner");
        let spine_id = SpineId::now_v7();
        let cert = create_test_certificate(&owner, spine_id);
        let cert_id = cert.id;

        storage.save_certificate(&cert, spine_id).await.unwrap();
        assert_eq!(storage.certificate_count(), 1);

        let retrieved = storage.get_certificate(cert_id).await.unwrap();
        assert!(retrieved.is_some());
        let (retrieved_cert, retrieved_spine) = retrieved.unwrap();
        assert_eq!(retrieved_cert.id, cert_id);
        assert_eq!(retrieved_spine, spine_id);

        let ids = storage.list_certificates().await.unwrap();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&cert_id));

        storage.delete_certificate(cert_id).await.unwrap();
        assert_eq!(storage.certificate_count(), 0);
        assert!(storage.get_certificate(cert_id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn sqlite_certificate_storage_list() {
        let (_temp_dir, storage) = certificate_storage_from_tempdir();
        let owner = Did::new("did:key:z6MkOwner");

        for _ in 0..3 {
            let spine_id = SpineId::now_v7();
            let cert = create_test_certificate(&owner, spine_id);
            storage.save_certificate(&cert, spine_id).await.unwrap();
        }

        let ids = storage.list_certificates().await.unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[tokio::test]
    async fn sqlite_certificate_storage_get_nonexistent() {
        let (_temp_dir, storage) = certificate_storage_from_tempdir();
        let result = storage
            .get_certificate(CertificateId::now_v7())
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn sqlite_certificate_storage_delete_nonexistent_idempotent() {
        let (_temp_dir, storage) = certificate_storage_from_tempdir();
        let result = storage.delete_certificate(CertificateId::now_v7()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sqlite_certificate_storage_flush() {
        let (_temp_dir, storage) = certificate_storage_from_tempdir();
        let owner = Did::new("did:key:z6MkOwner");
        let cert = create_test_certificate(&owner, SpineId::now_v7());
        storage
            .save_certificate(&cert, cert.mint_info.spine)
            .await
            .unwrap();
        let result = storage.flush();
        assert!(result.is_ok());
    }
}
