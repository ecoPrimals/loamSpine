// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::certificate::Certificate;
use crate::error::{LoamSpineResult, StorageResultExt};
use crate::types::{CertificateId, SpineId};

use super::common::{count_rows, flush_wal, lock_conn, open_connection, open_in_memory};
use crate::storage::CertificateStorage;

/// SQLite-backed certificate storage.
///
/// Stores `(Certificate, SpineId)` pairs in a `certificates` table,
/// keyed by `CertificateId` (UUID text). Uses JSON serialization for queryability.
#[derive(Clone)]
pub struct SqliteCertificateStorage {
    pub(super) conn: Arc<Mutex<Connection>>,
}

impl SqliteCertificateStorage {
    /// Open certificate storage at the given path.
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
            "CREATE TABLE IF NOT EXISTS certificates (
                id TEXT PRIMARY KEY,
                spine_id TEXT NOT NULL,
                data BLOB NOT NULL
            )",
            [],
        )
        .storage_err()?;
        Ok(())
    }

    /// Get the number of stored certificates.
    #[must_use]
    pub fn certificate_count(&self) -> usize {
        let Ok(conn) = self.conn.lock() else {
            return 0;
        };
        count_rows(&conn, "SELECT COUNT(*) FROM certificates")
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
impl CertificateStorage for SqliteCertificateStorage {
    async fn get_certificate(
        &self,
        id: CertificateId,
    ) -> LoamSpineResult<Option<(Certificate, SpineId)>> {
        let id_str = id.to_string();
        let result = {
            let conn = lock_conn(&self.conn)?;
            let mut stmt = conn
                .prepare("SELECT spine_id, data FROM certificates WHERE id = ?1")
                .storage_err()?;
            let mut rows = stmt.query([&id_str]).storage_err()?;

            if let Some(row) = rows.next().storage_err()? {
                let spine_id_str: String = row.get(0).storage_err()?;
                let data: Vec<u8> = row.get(1).storage_err()?;
                let spine_id = SpineId::parse_str(&spine_id_str).storage_ctx("invalid spine_id")?;
                let cert: Certificate = serde_json::from_slice(&data).storage_ctx("deserialize")?;
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
        let data = serde_json::to_vec(certificate).storage_ctx("serialize")?;
        {
            let conn = lock_conn(&self.conn)?;
            conn.execute(
                "INSERT OR REPLACE INTO certificates (id, spine_id, data) VALUES (?1, ?2, ?3)",
                rusqlite::params![&id_str, &spine_id_str, &data],
            )
            .storage_err()?;
        }
        Ok(())
    }

    async fn delete_certificate(&self, id: CertificateId) -> LoamSpineResult<()> {
        let id_str = id.to_string();
        {
            let conn = lock_conn(&self.conn)?;
            conn.execute("DELETE FROM certificates WHERE id = ?1", [&id_str])
                .storage_err()?;
        }
        Ok(())
    }

    async fn list_certificates(&self) -> LoamSpineResult<Vec<CertificateId>> {
        let ids = {
            let conn = lock_conn(&self.conn)?;
            let mut stmt = conn.prepare("SELECT id FROM certificates").storage_err()?;
            let mut rows = stmt.query([]).storage_err()?;

            let mut ids = Vec::new();
            while let Some(row) = rows.next().storage_err()? {
                let id_str: String = row.get(0).storage_err()?;
                if let Ok(uuid) = CertificateId::parse_str(&id_str) {
                    ids.push(uuid);
                }
            }
            ids
        };
        Ok(ids)
    }
}
