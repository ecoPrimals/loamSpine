// SPDX-License-Identifier: AGPL-3.0-only

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;

use crate::error::{LoamSpineError, LoamSpineResult};

pub fn to_storage_err<E: std::fmt::Display>(e: E) -> LoamSpineError {
    LoamSpineError::Storage(e.to_string())
}

pub fn lock_conn(
    conn: &std::sync::Arc<Mutex<Connection>>,
) -> LoamSpineResult<MutexGuard<'_, Connection>> {
    conn.lock()
        .map_err(|e| LoamSpineError::Storage(format!("sqlite mutex poisoned: {e}")))
}

pub fn flush_wal(conn: &Connection) -> LoamSpineResult<()> {
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(to_storage_err)?;
    Ok(())
}

pub fn count_rows(conn: &Connection, query: &str) -> usize {
    conn.query_row(query, [], |row| row.get::<_, i64>(0))
        .map_or(0, |n| usize::try_from(n).unwrap_or(0))
}

pub fn open_connection<P: AsRef<Path>>(path: P) -> LoamSpineResult<Connection> {
    Connection::open(path).map_err(to_storage_err)
}

pub fn open_in_memory() -> LoamSpineResult<Connection> {
    Connection::open_in_memory().map_err(to_storage_err)
}
