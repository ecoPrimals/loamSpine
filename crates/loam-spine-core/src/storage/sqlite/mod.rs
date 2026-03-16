// SPDX-License-Identifier: AGPL-3.0-or-later

//! SQLite storage backend. Feature-gated behind `sqlite`.
//!
//! Note: bundles C SQLite library — not ecoBin compliant. Use for development or
//! deployments where SQLite compatibility is required.

mod certificate;
mod common;
mod entry;
mod spine;

#[cfg(test)]
mod tests;

pub use certificate::SqliteCertificateStorage;
pub use entry::SqliteEntryStorage;
pub use spine::SqliteSpineStorage;

use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::error::LoamSpineResult;

use common::open_in_memory;

/// Combined SQLite storage for both spines and entries.
///
/// Uses a single database file with separate tables for spines and entries.
/// Convenience wrapper that provides persistent storage for both types.
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
    /// Returns an error if the database cannot be opened or initialized.
    pub fn open<P: AsRef<Path>>(path: P) -> LoamSpineResult<Self> {
        let conn = common::open_connection(path)?;
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
    /// Returns an error if database cannot be created.
    pub fn temporary() -> LoamSpineResult<Self> {
        let conn = open_in_memory()?;
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
    /// Returns an error if the WAL checkpoint fails.
    pub fn flush(&self) -> LoamSpineResult<()> {
        self.spines.flush()?;
        self.entries.flush()?;
        self.certificates.flush()?;
        Ok(())
    }
}
