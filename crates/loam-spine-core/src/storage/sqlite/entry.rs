// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::entry::Entry;
use crate::error::LoamSpineResult;
use crate::types::{EntryHash, SpineId};

use super::common::{
    count_rows, flush_wal, lock_conn, open_connection, open_in_memory, to_storage_err,
};
use crate::storage::EntryStorage;

/// SQLite-backed entry storage.
///
/// Uses JSON serialization for queryability. Maintains entries table with
/// spine_id and entry_index for efficient range queries.
#[derive(Clone)]
pub struct SqliteEntryStorage {
    pub(super) conn: Arc<Mutex<Connection>>,
}

impl SqliteEntryStorage {
    /// Open entry storage at the given path.
    ///
    /// Creates the database file and tables if they don't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened or initialized.
    pub fn open<P: AsRef<Path>>(path: P) -> LoamSpineResult<Self> {
        let conn = open_connection(path)?;
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
    /// Returns an error if database cannot be created.
    pub fn temporary() -> LoamSpineResult<Self> {
        let conn = open_in_memory()?;
        Self::init_conn(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub(super) fn init_conn(conn: &Connection) -> LoamSpineResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS entries (
                hash BLOB PRIMARY KEY,
                spine_id TEXT NOT NULL,
                entry_index INTEGER NOT NULL,
                data BLOB NOT NULL
            )",
            [],
        )
        .map_err(to_storage_err)?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_entries_spine ON entries(spine_id, entry_index)",
            [],
        )
        .map_err(to_storage_err)?;
        Ok(())
    }

    /// Get the number of stored entries.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        let Ok(conn) = self.conn.lock() else {
            return 0;
        };
        count_rows(&conn, "SELECT COUNT(*) FROM entries")
    }

    /// Flush all pending writes to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the WAL checkpoint fails.
    pub fn flush(&self) -> LoamSpineResult<()> {
        let conn = lock_conn(&self.conn)?;
        flush_wal(&conn)
    }
}

#[expect(
    clippy::significant_drop_tightening,
    reason = "MutexGuard must span full SQL transaction"
)]
impl EntryStorage for SqliteEntryStorage {
    async fn get_entry(&self, hash: EntryHash) -> LoamSpineResult<Option<Entry>> {
        let result = {
            let conn = lock_conn(&self.conn)?;
            let mut stmt = conn
                .prepare("SELECT data FROM entries WHERE hash = ?1")
                .map_err(to_storage_err)?;
            let mut rows = stmt.query([hash.as_slice()]).map_err(to_storage_err)?;

            if let Some(row) = rows.next().map_err(to_storage_err)? {
                let data: Vec<u8> = row.get(0).map_err(to_storage_err)?;
                let entry: Entry = serde_json::from_slice(&data)
                    .map_err(|e| to_storage_err(format!("deserialize: {e}")))?;
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
        let data =
            serde_json::to_vec(entry).map_err(|e| to_storage_err(format!("serialize: {e}")))?;

        {
            let conn = lock_conn(&self.conn)?;
            conn.execute(
                "INSERT OR REPLACE INTO entries (hash, spine_id, entry_index, data) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    hash.as_slice(),
                    &spine_id_str,
                    i64::try_from(entry.index).unwrap_or(i64::MAX),
                    &data
                ],
            )
            .map_err(to_storage_err)?;
        }
        Ok(hash)
    }

    async fn entry_exists(&self, hash: EntryHash) -> LoamSpineResult<bool> {
        let count: i32 = {
            let conn = lock_conn(&self.conn)?;
            conn.query_row(
                "SELECT COUNT(*) FROM entries WHERE hash = ?1",
                [hash.as_slice()],
                |row| row.get(0),
            )
            .map_err(to_storage_err)?
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
            let conn = lock_conn(&self.conn)?;
            let mut stmt = conn
                .prepare(
                    "SELECT data FROM entries WHERE spine_id = ?1 AND entry_index >= ?2 ORDER BY entry_index ASC LIMIT ?3",
                )
                .map_err(to_storage_err)?;
            let mut rows = stmt
                .query(rusqlite::params![&spine_id_str, start_i64, limit_i64])
                .map_err(to_storage_err)?;

            let mut entries = Vec::new();
            while let Some(row) = rows.next().map_err(to_storage_err)? {
                let data: Vec<u8> = row.get(0).map_err(to_storage_err)?;
                let entry: Entry = serde_json::from_slice(&data)
                    .map_err(|e| to_storage_err(format!("deserialize: {e}")))?;
                entries.push(entry);
            }
            entries
        };
        Ok(entries)
    }
}
