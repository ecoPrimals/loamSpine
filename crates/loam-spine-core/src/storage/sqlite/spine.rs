// SPDX-License-Identifier: AGPL-3.0-only

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::error::LoamSpineResult;
use crate::spine::Spine;
use crate::types::SpineId;

use super::common::{
    count_rows, flush_wal, lock_conn, open_connection, open_in_memory, to_storage_err,
};
use crate::storage::SpineStorage;

/// SQLite-backed spine storage.
///
/// Uses JSON serialization for queryability. Wraps `rusqlite::Connection` in
/// `Arc<Mutex<>>` for thread safety.
#[derive(Clone)]
pub struct SqliteSpineStorage {
    pub(super) conn: Arc<Mutex<Connection>>,
}

impl SqliteSpineStorage {
    /// Open spine storage at the given path.
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
            "CREATE TABLE IF NOT EXISTS spines (id TEXT PRIMARY KEY, data BLOB)",
            [],
        )
        .map_err(to_storage_err)?;
        Ok(())
    }

    /// Get the number of stored spines.
    #[must_use]
    pub fn spine_count(&self) -> usize {
        let Ok(conn) = self.conn.lock() else {
            return 0;
        };
        count_rows(&conn, "SELECT COUNT(*) FROM spines")
    }

    /// Flush all pending writes to disk.
    ///
    /// SQLite uses WAL mode when available. This ensures durability.
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
impl SpineStorage for SqliteSpineStorage {
    async fn get_spine(&self, id: SpineId) -> LoamSpineResult<Option<Spine>> {
        let id_str = id.to_string();
        let result = {
            let conn = lock_conn(&self.conn)?;
            let mut stmt = conn
                .prepare("SELECT data FROM spines WHERE id = ?1")
                .map_err(to_storage_err)?;
            let mut rows = stmt.query([&id_str]).map_err(to_storage_err)?;

            if let Some(row) = rows.next().map_err(to_storage_err)? {
                let data: Vec<u8> = row.get(0).map_err(to_storage_err)?;
                let spine: Spine = serde_json::from_slice(&data)
                    .map_err(|e| to_storage_err(format!("deserialize: {e}")))?;
                Some(spine)
            } else {
                None
            }
        };
        Ok(result)
    }

    async fn save_spine(&self, spine: &Spine) -> LoamSpineResult<()> {
        let id_str = spine.id.to_string();
        let data =
            serde_json::to_vec(spine).map_err(|e| to_storage_err(format!("serialize: {e}")))?;
        {
            let conn = lock_conn(&self.conn)?;
            conn.execute(
                "INSERT OR REPLACE INTO spines (id, data) VALUES (?1, ?2)",
                rusqlite::params![&id_str, &data],
            )
            .map_err(to_storage_err)?;
        }
        Ok(())
    }

    async fn delete_spine(&self, id: SpineId) -> LoamSpineResult<()> {
        let id_str = id.to_string();
        {
            let conn = lock_conn(&self.conn)?;
            conn.execute("DELETE FROM spines WHERE id = ?1", [&id_str])
                .map_err(to_storage_err)?;
        }
        Ok(())
    }

    async fn list_spines(&self) -> LoamSpineResult<Vec<SpineId>> {
        let ids = {
            let conn = lock_conn(&self.conn)?;
            let mut stmt = conn
                .prepare("SELECT id FROM spines")
                .map_err(to_storage_err)?;
            let mut rows = stmt.query([]).map_err(to_storage_err)?;

            let mut ids = Vec::new();
            while let Some(row) = rows.next().map_err(to_storage_err)? {
                let id_str: String = row.get(0).map_err(to_storage_err)?;
                if let Ok(uuid) = SpineId::parse_str(&id_str) {
                    ids.push(uuid);
                }
            }
            ids
        };
        Ok(ids)
    }
}
