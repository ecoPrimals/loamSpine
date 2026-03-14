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
        let conn = self
            .conn
            .lock()
            .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }
}

impl SpineStorage for SqliteSpineStorage {
    async fn get_spine(&self, id: SpineId) -> LoamSpineResult<Option<Spine>> {
        let id_str = id.to_string();
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
            Ok(Some(spine))
        } else {
            Ok(None)
        }
    }

    async fn save_spine(&self, spine: &Spine) -> LoamSpineResult<()> {
        let id_str = spine.id.to_string();
        let data = serde_json::to_vec(spine)
            .map_err(|e| LoamSpineError::Storage(format!("serialize: {e}")))?;
        let conn = self
            .conn
            .lock()
            .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
        conn.execute(
            "INSERT OR REPLACE INTO spines (id, data) VALUES (?1, ?2)",
            rusqlite::params![&id_str, &data],
        )
        .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn delete_spine(&self, id: SpineId) -> LoamSpineResult<()> {
        let id_str = id.to_string();
        let conn = self
            .conn
            .lock()
            .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
        conn.execute("DELETE FROM spines WHERE id = ?1", [&id_str])
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn list_spines(&self) -> LoamSpineResult<Vec<SpineId>> {
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
        let conn = self
            .conn
            .lock()
            .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }
}

impl EntryStorage for SqliteEntryStorage {
    async fn get_entry(&self, hash: EntryHash) -> LoamSpineResult<Option<Entry>> {
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
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    async fn save_entry(&self, entry: &Entry) -> LoamSpineResult<EntryHash> {
        let hash = entry.compute_hash()?;
        let spine_id_str = entry.spine_id.to_string();
        let data = serde_json::to_vec(entry)
            .map_err(|e| LoamSpineError::Storage(format!("serialize: {e}")))?;

        let conn = self
            .conn
            .lock()
            .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
        conn.execute(
            "INSERT OR REPLACE INTO entries (hash, spine_id, entry_index, data) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![hash.as_slice(), &spine_id_str, i64::try_from(entry.index).unwrap_or(i64::MAX), &data],
        )
        .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(hash)
    }

    async fn entry_exists(&self, hash: EntryHash) -> LoamSpineResult<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM entries WHERE hash = ?1",
                [hash.as_slice()],
                |row| row.get(0),
            )
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
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
        let conn = self
            .conn
            .lock()
            .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }
}

impl CertificateStorage for SqliteCertificateStorage {
    async fn get_certificate(
        &self,
        id: CertificateId,
    ) -> LoamSpineResult<Option<(Certificate, SpineId)>> {
        let id_str = id.to_string();
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
            Ok(Some((cert, spine_id)))
        } else {
            Ok(None)
        }
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
        let conn = self
            .conn
            .lock()
            .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
        conn.execute(
            "INSERT OR REPLACE INTO certificates (id, spine_id, data) VALUES (?1, ?2, ?3)",
            rusqlite::params![&id_str, &spine_id_str, &data],
        )
        .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn delete_certificate(&self, id: CertificateId) -> LoamSpineResult<()> {
        let id_str = id.to_string();
        let conn = self
            .conn
            .lock()
            .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))?;
        conn.execute("DELETE FROM certificates WHERE id = ?1", [&id_str])
            .map_err(|e| LoamSpineError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn list_certificates(&self) -> LoamSpineResult<Vec<CertificateId>> {
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
