// SPDX-License-Identifier: AGPL-3.0-or-later

//! Shared SQLite helpers for connection management, WAL flushing, and row counting.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;

use crate::error::{LoamSpineError, LoamSpineResult, StorageResultExt};

/// Acquire the connection mutex, mapping a poisoned lock to a storage error.
pub fn lock_conn(
    conn: &std::sync::Arc<Mutex<Connection>>,
) -> LoamSpineResult<MutexGuard<'_, Connection>> {
    conn.lock()
        .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))
}

/// Force a WAL checkpoint so the database file is self-contained.
pub fn flush_wal(conn: &Connection) -> LoamSpineResult<()> {
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .storage_err()?;
    Ok(())
}

/// Count rows returned by a single-column integer query, returning 0 on error.
pub fn count_rows(conn: &Connection, query: &str) -> usize {
    conn.query_row(query, [], |row| row.get::<_, i64>(0))
        .map_or(0, |n| usize::try_from(n).unwrap_or(0))
}

/// Open a persistent SQLite connection at the given path.
///
/// Enables WAL journal mode and sets a busy timeout for concurrent access
/// resilience (parallel tests, multi-process deployments).
pub fn open_connection<P: AsRef<Path>>(path: P) -> LoamSpineResult<Connection> {
    let conn = Connection::open(path).storage_err()?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")
        .storage_err()?;
    Ok(conn)
}

/// Open an ephemeral in-memory SQLite connection (useful for tests).
pub fn open_in_memory() -> LoamSpineResult<Connection> {
    Connection::open_in_memory().storage_err()
}
