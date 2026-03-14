# LoamSpine — Storage Backends Specification

**Version**: 1.0.0  
**Status**: Active  
**Last Updated**: December 22, 2025

---

## 1. Overview

LoamSpine uses a pluggable storage architecture. **redb** is the default embedded backend for local storage. **sled** is optional and available via the `sled-storage` feature. Additional backends:

| Backend | Use Case | Scalability | Query Power |
|---------|----------|-------------|-------------|
| **redb** | Default embedded, local | Single node | Key-value |
| **sled** | Optional embedded (feature-gated) | Single node | Key-value |
| **SQLite** | Personal spines, portable | Single node | Full SQL |
| **PostgreSQL** | Community spines, scalable | Multi-node | Full SQL |
| **RocksDB** | High-performance local | Single node | Key-value |

All backends implement the same trait interface.

---

## 2. Storage Traits

### 2.1 Entry Store Trait

```rust
/// Primary storage trait for entries
#[async_trait]
pub trait EntryStore: Send + Sync + Clone {
    /// Append an entry to a spine
    async fn append(
        &self,
        spine_id: SpineId,
        entry: Entry,
    ) -> Result<EntryHash, StorageError>;
    
    /// Get entry by hash
    async fn get_by_hash(
        &self,
        entry_hash: &EntryHash,
    ) -> Result<Option<Entry>, StorageError>;
    
    /// Get entry by index
    async fn get_by_index(
        &self,
        spine_id: SpineId,
        index: u64,
    ) -> Result<Option<Entry>, StorageError>;
    
    /// Get entries in range
    async fn get_range(
        &self,
        spine_id: SpineId,
        start: u64,
        end: u64,
    ) -> Result<Vec<Entry>, StorageError>;
    
    /// Get tip entry
    async fn get_tip(&self, spine_id: SpineId) -> Result<Option<Entry>, StorageError>;
    
    /// Get next entry after hash
    async fn get_next(&self, entry_hash: &EntryHash) -> Result<Option<EntryHash>, StorageError>;
    
    /// Count entries in spine
    async fn count(&self, spine_id: SpineId) -> Result<u64, StorageError>;
    
    /// Stream entries
    fn stream(
        &self,
        spine_id: SpineId,
    ) -> impl Stream<Item = Result<Entry, StorageError>> + Send;
    
    /// Query entries by type
    async fn query_by_type(
        &self,
        spine_id: SpineId,
        entry_type: &str,
        limit: usize,
    ) -> Result<Vec<Entry>, StorageError>;
    
    /// Query entries by committer
    async fn query_by_committer(
        &self,
        spine_id: SpineId,
        committer: &Did,
        limit: usize,
    ) -> Result<Vec<Entry>, StorageError>;
    
    /// Get entries for a certificate
    async fn get_certificate_entries(
        &self,
        cert_id: CertificateId,
    ) -> Result<Vec<Entry>, StorageError>;
    
    /// Health check
    async fn health(&self) -> HealthStatus;
    
    /// Get statistics
    async fn stats(&self) -> StorageStats;
}
```

### 2.2 Spine Store Trait

```rust
/// Storage trait for spine metadata
#[async_trait]
pub trait SpineStore: Send + Sync + Clone {
    /// Create a new spine
    async fn create(&self, spine: Spine, genesis: Entry) -> Result<(), StorageError>;
    
    /// Get spine by ID
    async fn get(&self, id: SpineId) -> Result<Option<Spine>, StorageError>;
    
    /// Update spine metadata
    async fn update(&self, spine: Spine) -> Result<(), StorageError>;
    
    /// List spines by filter
    async fn list(&self, filter: SpineFilter, limit: usize) -> Result<Vec<Spine>, StorageError>;
    
    /// Count spines by filter
    async fn count(&self, filter: SpineFilter) -> Result<u64, StorageError>;
    
    /// Update tip
    async fn update_tip(&self, id: SpineId, tip: EntryHash, height: u64) -> Result<(), StorageError>;
    
    /// Update state
    async fn update_state(&self, id: SpineId, state: SpineState) -> Result<(), StorageError>;
}

/// Spine filter
#[derive(Clone, Debug, Default)]
pub struct SpineFilter {
    pub owner: Option<Did>,
    pub spine_type: Option<SpineType>,
    pub state: Option<SpineState>,
    pub created_after: Option<u64>,
    pub created_before: Option<u64>,
}
```

### 2.3 Certificate Store Trait

```rust
/// Storage trait for certificates
#[async_trait]
pub trait CertificateStore: Send + Sync + Clone {
    /// Store a certificate
    async fn put(&self, cert: Certificate) -> Result<(), StorageError>;
    
    /// Get certificate by ID
    async fn get(&self, id: CertificateId) -> Result<Option<Certificate>, StorageError>;
    
    /// Update certificate
    async fn update(&self, cert: Certificate) -> Result<(), StorageError>;
    
    /// List certificates by owner
    async fn list_by_owner(
        &self,
        owner: &Did,
        limit: usize,
    ) -> Result<Vec<Certificate>, StorageError>;
    
    /// List certificates by holder (including loans)
    async fn list_by_holder(
        &self,
        holder: &Did,
        limit: usize,
    ) -> Result<Vec<Certificate>, StorageError>;
    
    /// Query certificates
    async fn query(
        &self,
        filter: CertificateFilter,
        limit: usize,
    ) -> Result<Vec<Certificate>, StorageError>;
}

/// Certificate filter
#[derive(Clone, Debug, Default)]
pub struct CertificateFilter {
    pub cert_type: Option<String>,
    pub owner: Option<Did>,
    pub holder: Option<Did>,
    pub state: Option<CertificateState>,
    pub mint_spine: Option<SpineId>,
}
```

---

## 3. SQLite Backend

Portable, single-file storage for personal spines.

### 3.1 Schema

```sql
-- Spines table
CREATE TABLE spines (
    id TEXT PRIMARY KEY,
    name TEXT,
    owner TEXT NOT NULL,
    spine_type TEXT NOT NULL,
    config TEXT NOT NULL,  -- JSON
    genesis_hash BLOB NOT NULL,
    tip_hash BLOB NOT NULL,
    height INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    state TEXT NOT NULL DEFAULT 'active',
    state_data TEXT,  -- JSON for state-specific data
    metadata TEXT  -- JSON
);

CREATE INDEX idx_spines_owner ON spines(owner);
CREATE INDEX idx_spines_type ON spines(spine_type);
CREATE INDEX idx_spines_state ON spines(state);

-- Entries table
CREATE TABLE entries (
    hash BLOB PRIMARY KEY,
    spine_id TEXT NOT NULL REFERENCES spines(id),
    idx INTEGER NOT NULL,
    previous_hash BLOB,
    timestamp INTEGER NOT NULL,
    committer TEXT NOT NULL,
    entry_type TEXT NOT NULL,
    entry_data TEXT NOT NULL,  -- JSON
    payload_hash BLOB,
    payload_size INTEGER,
    payload_mime TEXT,
    metadata TEXT,  -- JSON
    signature BLOB NOT NULL,
    attestations TEXT,  -- JSON array
    
    UNIQUE(spine_id, idx)
);

CREATE INDEX idx_entries_spine ON entries(spine_id);
CREATE INDEX idx_entries_spine_idx ON entries(spine_id, idx);
CREATE INDEX idx_entries_committer ON entries(committer);
CREATE INDEX idx_entries_type ON entries(entry_type);
CREATE INDEX idx_entries_timestamp ON entries(timestamp);

-- Certificates table
CREATE TABLE certificates (
    id TEXT PRIMARY KEY,
    cert_type TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    owner TEXT NOT NULL,
    holder TEXT,
    mint_minter TEXT NOT NULL,
    mint_spine TEXT NOT NULL,
    mint_entry BLOB NOT NULL,
    mint_timestamp INTEGER NOT NULL,
    current_spine TEXT NOT NULL,
    current_entry BLOB NOT NULL,
    current_index INTEGER NOT NULL,
    state TEXT NOT NULL DEFAULT 'active',
    state_data TEXT,  -- JSON
    transfer_count INTEGER NOT NULL DEFAULT 0,
    active_loan TEXT,  -- JSON
    metadata TEXT NOT NULL,  -- JSON
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_certificates_owner ON certificates(owner);
CREATE INDEX idx_certificates_holder ON certificates(holder);
CREATE INDEX idx_certificates_type ON certificates(cert_type);
CREATE INDEX idx_certificates_state ON certificates(state);

-- Certificate entries index (for history queries)
CREATE TABLE certificate_entries (
    cert_id TEXT NOT NULL REFERENCES certificates(id),
    entry_hash BLOB NOT NULL REFERENCES entries(hash),
    entry_type TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    
    PRIMARY KEY(cert_id, entry_hash)
);

CREATE INDEX idx_cert_entries_cert ON certificate_entries(cert_id);

-- Waypoint anchors table
CREATE TABLE waypoint_anchors (
    slice_id TEXT PRIMARY KEY,
    anchor_entry BLOB NOT NULL REFERENCES entries(hash),
    origin_spine TEXT NOT NULL,
    origin_entry BLOB NOT NULL,
    owner TEXT NOT NULL,
    depth INTEGER NOT NULL,
    terms TEXT NOT NULL,  -- JSON
    anchored_at INTEGER NOT NULL,
    expires_at INTEGER,
    state TEXT NOT NULL DEFAULT 'active',
    waypoint_spine TEXT NOT NULL REFERENCES spines(id)
);

CREATE INDEX idx_anchors_waypoint ON waypoint_anchors(waypoint_spine);
CREATE INDEX idx_anchors_state ON waypoint_anchors(state);
CREATE INDEX idx_anchors_expires ON waypoint_anchors(expires_at);
```

### 3.2 Implementation

```rust
use rusqlite::{Connection, params};
use tokio::sync::Mutex;

/// SQLite entry store
#[derive(Clone)]
pub struct SqliteEntryStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteEntryStore {
    /// Open or create a database
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;
        
        // Enable WAL mode for better concurrency
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
        
        // Create schema
        conn.execute_batch(SCHEMA)?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
    
    /// In-memory database (for testing)
    pub fn in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

#[async_trait]
impl EntryStore for SqliteEntryStore {
    async fn append(
        &self,
        spine_id: SpineId,
        entry: Entry,
    ) -> Result<EntryHash, StorageError> {
        let conn = self.conn.lock().await;
        let hash = entry.compute_hash();
        
        conn.execute(
            "INSERT INTO entries (
                hash, spine_id, idx, previous_hash, timestamp, committer,
                entry_type, entry_data, payload_hash, payload_size, payload_mime,
                metadata, signature, attestations
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                hash.as_slice(),
                spine_id.to_string(),
                entry.index as i64,
                entry.previous.as_ref().map(|h| h.as_slice()),
                entry.timestamp as i64,
                entry.committer,
                entry.entry_type.domain(),
                serde_json::to_string(&entry.entry_type)?,
                entry.payload.as_ref().map(|p| p.hash.as_slice()),
                entry.payload.as_ref().map(|p| p.size as i64),
                entry.payload.as_ref().and_then(|p| p.mime_type.as_deref()),
                serde_json::to_string(&entry.metadata)?,
                entry.signature.as_slice(),
                serde_json::to_string(&entry.attestations)?,
            ],
        )?;
        
        // Update spine tip
        conn.execute(
            "UPDATE spines SET tip_hash = ?1, height = ?2, updated_at = ?3 WHERE id = ?4",
            params![
                hash.as_slice(),
                entry.index as i64 + 1,
                current_timestamp_nanos() as i64,
                spine_id.to_string(),
            ],
        )?;
        
        Ok(hash)
    }
    
    async fn get_by_hash(
        &self,
        entry_hash: &EntryHash,
    ) -> Result<Option<Entry>, StorageError> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM entries WHERE hash = ?1"
        )?;
        
        let entry = stmt.query_row(params![entry_hash.as_slice()], |row| {
            Ok(entry_from_row(row)?)
        }).optional()?;
        
        Ok(entry)
    }
    
    async fn get_by_index(
        &self,
        spine_id: SpineId,
        index: u64,
    ) -> Result<Option<Entry>, StorageError> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM entries WHERE spine_id = ?1 AND idx = ?2"
        )?;
        
        let entry = stmt.query_row(
            params![spine_id.to_string(), index as i64],
            |row| Ok(entry_from_row(row)?),
        ).optional()?;
        
        Ok(entry)
    }
    
    async fn get_range(
        &self,
        spine_id: SpineId,
        start: u64,
        end: u64,
    ) -> Result<Vec<Entry>, StorageError> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM entries WHERE spine_id = ?1 AND idx >= ?2 AND idx < ?3 ORDER BY idx"
        )?;
        
        let entries = stmt.query_map(
            params![spine_id.to_string(), start as i64, end as i64],
            |row| Ok(entry_from_row(row)?),
        )?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(entries)
    }
    
    // ... other implementations
}
```

---

## 4. PostgreSQL Backend

Scalable storage for community spines with replication support.

### 4.1 Schema

```sql
-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Spines table
CREATE TABLE spines (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    name TEXT,
    owner TEXT NOT NULL,
    spine_type TEXT NOT NULL,
    config JSONB NOT NULL,
    genesis_hash BYTEA NOT NULL,
    tip_hash BYTEA NOT NULL,
    height BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    state TEXT NOT NULL DEFAULT 'active',
    state_data JSONB,
    metadata JSONB
);

CREATE INDEX idx_spines_owner ON spines(owner);
CREATE INDEX idx_spines_type ON spines(spine_type);
CREATE INDEX idx_spines_state ON spines(state);

-- Entries table (partitioned by spine for scalability)
CREATE TABLE entries (
    hash BYTEA NOT NULL,
    spine_id UUID NOT NULL REFERENCES spines(id),
    idx BIGINT NOT NULL,
    previous_hash BYTEA,
    timestamp BIGINT NOT NULL,
    committer TEXT NOT NULL,
    entry_type TEXT NOT NULL,
    entry_data JSONB NOT NULL,
    payload_hash BYTEA,
    payload_size BIGINT,
    payload_mime TEXT,
    metadata JSONB,
    signature BYTEA NOT NULL,
    attestations JSONB,
    
    PRIMARY KEY (spine_id, hash)
) PARTITION BY HASH (spine_id);

-- Create partitions
CREATE TABLE entries_p0 PARTITION OF entries FOR VALUES WITH (MODULUS 8, REMAINDER 0);
CREATE TABLE entries_p1 PARTITION OF entries FOR VALUES WITH (MODULUS 8, REMAINDER 1);
CREATE TABLE entries_p2 PARTITION OF entries FOR VALUES WITH (MODULUS 8, REMAINDER 2);
CREATE TABLE entries_p3 PARTITION OF entries FOR VALUES WITH (MODULUS 8, REMAINDER 3);
CREATE TABLE entries_p4 PARTITION OF entries FOR VALUES WITH (MODULUS 8, REMAINDER 4);
CREATE TABLE entries_p5 PARTITION OF entries FOR VALUES WITH (MODULUS 8, REMAINDER 5);
CREATE TABLE entries_p6 PARTITION OF entries FOR VALUES WITH (MODULUS 8, REMAINDER 6);
CREATE TABLE entries_p7 PARTITION OF entries FOR VALUES WITH (MODULUS 8, REMAINDER 7);

-- Indexes
CREATE INDEX idx_entries_spine_idx ON entries(spine_id, idx);
CREATE INDEX idx_entries_committer ON entries(committer);
CREATE INDEX idx_entries_type ON entries(entry_type);
CREATE INDEX idx_entries_timestamp ON entries(timestamp);

-- GIN index for JSON queries
CREATE INDEX idx_entries_data ON entries USING GIN (entry_data);

-- Certificates table
CREATE TABLE certificates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    cert_type TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    owner TEXT NOT NULL,
    holder TEXT,
    mint_info JSONB NOT NULL,
    current_location JSONB NOT NULL,
    state TEXT NOT NULL DEFAULT 'active',
    state_data JSONB,
    transfer_count INTEGER NOT NULL DEFAULT 0,
    active_loan JSONB,
    metadata JSONB NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

CREATE INDEX idx_certificates_owner ON certificates(owner);
CREATE INDEX idx_certificates_holder ON certificates(holder);
CREATE INDEX idx_certificates_type ON certificates(cert_type);
CREATE INDEX idx_certificates_state ON certificates(state);
CREATE INDEX idx_certificates_metadata ON certificates USING GIN (metadata);
```

### 4.2 Implementation

```rust
use sqlx::{PgPool, postgres::PgPoolOptions};

/// PostgreSQL entry store
#[derive(Clone)]
pub struct PostgresEntryStore {
    pool: PgPool,
}

impl PostgresEntryStore {
    /// Connect to PostgreSQL
    pub async fn connect(url: &str) -> Result<Self, StorageError> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(url)
            .await?;
        
        Ok(Self { pool })
    }
    
    /// Run migrations
    pub async fn migrate(&self) -> Result<(), StorageError> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}

#[async_trait]
impl EntryStore for PostgresEntryStore {
    async fn append(
        &self,
        spine_id: SpineId,
        entry: Entry,
    ) -> Result<EntryHash, StorageError> {
        let hash = entry.compute_hash();
        
        // Use transaction for atomicity
        let mut tx = self.pool.begin().await?;
        
        sqlx::query(
            r#"
            INSERT INTO entries (
                hash, spine_id, idx, previous_hash, timestamp, committer,
                entry_type, entry_data, payload_hash, payload_size, payload_mime,
                metadata, signature, attestations
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#
        )
        .bind(hash.as_slice())
        .bind(spine_id)
        .bind(entry.index as i64)
        .bind(entry.previous.as_ref().map(|h| h.as_slice()))
        .bind(entry.timestamp as i64)
        .bind(&entry.committer)
        .bind(entry.entry_type.domain())
        .bind(serde_json::to_value(&entry.entry_type)?)
        .bind(entry.payload.as_ref().map(|p| p.hash.as_slice()))
        .bind(entry.payload.as_ref().map(|p| p.size as i64))
        .bind(entry.payload.as_ref().and_then(|p| p.mime_type.as_deref()))
        .bind(serde_json::to_value(&entry.metadata)?)
        .bind(entry.signature.as_slice())
        .bind(serde_json::to_value(&entry.attestations)?)
        .execute(&mut *tx)
        .await?;
        
        sqlx::query(
            r#"
            UPDATE spines 
            SET tip_hash = $1, height = $2, updated_at = $3 
            WHERE id = $4
            "#
        )
        .bind(hash.as_slice())
        .bind(entry.index as i64 + 1)
        .bind(current_timestamp_nanos() as i64)
        .bind(spine_id)
        .execute(&mut *tx)
        .await?;
        
        tx.commit().await?;
        
        Ok(hash)
    }
    
    async fn get_by_hash(
        &self,
        entry_hash: &EntryHash,
    ) -> Result<Option<Entry>, StorageError> {
        let row = sqlx::query(
            "SELECT * FROM entries WHERE hash = $1"
        )
        .bind(entry_hash.as_slice())
        .fetch_optional(&self.pool)
        .await?;
        
        row.map(|r| entry_from_pg_row(&r)).transpose()
    }
    
    // ... other implementations
}
```

---

## 5. RocksDB Backend

High-performance local storage for maximum throughput.

### 5.1 Key Schema

```
// Key prefixes
const PREFIX_ENTRY: u8 = 0x01;
const PREFIX_ENTRY_IDX: u8 = 0x02;
const PREFIX_SPINE: u8 = 0x03;
const PREFIX_CERT: u8 = 0x04;
const PREFIX_CERT_ENTRY: u8 = 0x05;
const PREFIX_ANCHOR: u8 = 0x06;

// Entry key: PREFIX_ENTRY | entry_hash (32 bytes)
// Entry by index: PREFIX_ENTRY_IDX | spine_id (16 bytes) | index (8 bytes, big-endian)
// Spine key: PREFIX_SPINE | spine_id (16 bytes)
// Certificate key: PREFIX_CERT | cert_id (16 bytes)
// Certificate entries: PREFIX_CERT_ENTRY | cert_id (16 bytes) | entry_hash (32 bytes)
// Anchor key: PREFIX_ANCHOR | slice_id (16 bytes)
```

### 5.2 Implementation

```rust
use rocksdb::{DB, Options, WriteBatch};

/// RocksDB entry store
#[derive(Clone)]
pub struct RocksDbEntryStore {
    db: Arc<DB>,
}

impl RocksDbEntryStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        opts.set_max_background_jobs(4);
        opts.increase_parallelism(num_cpus::get() as i32);
        
        let db = DB::open(&opts, path)?;
        
        Ok(Self { db: Arc::new(db) })
    }
    
    fn entry_key(hash: &EntryHash) -> Vec<u8> {
        let mut key = Vec::with_capacity(33);
        key.push(PREFIX_ENTRY);
        key.extend_from_slice(hash);
        key
    }
    
    fn entry_idx_key(spine_id: SpineId, index: u64) -> Vec<u8> {
        let mut key = Vec::with_capacity(25);
        key.push(PREFIX_ENTRY_IDX);
        key.extend_from_slice(spine_id.as_bytes());
        key.extend_from_slice(&index.to_be_bytes());
        key
    }
}

#[async_trait]
impl EntryStore for RocksDbEntryStore {
    async fn append(
        &self,
        spine_id: SpineId,
        entry: Entry,
    ) -> Result<EntryHash, StorageError> {
        let hash = entry.compute_hash();
        let entry_bytes = entry.to_cbor();
        
        let mut batch = WriteBatch::default();
        
        // Store entry by hash
        batch.put(Self::entry_key(&hash), &entry_bytes);
        
        // Store entry by index
        batch.put(Self::entry_idx_key(spine_id, entry.index), &hash);
        
        // Update spine tip
        let spine_key = Self::spine_key(spine_id);
        if let Some(spine_bytes) = self.db.get(&spine_key)? {
            let mut spine: Spine = Spine::from_cbor(&spine_bytes)?;
            spine.tip = hash;
            spine.height = entry.index + 1;
            spine.updated_at = current_timestamp_nanos();
            batch.put(spine_key, spine.to_cbor());
        }
        
        self.db.write(batch)?;
        
        Ok(hash)
    }
    
    async fn get_by_hash(
        &self,
        entry_hash: &EntryHash,
    ) -> Result<Option<Entry>, StorageError> {
        let key = Self::entry_key(entry_hash);
        
        match self.db.get(&key)? {
            Some(bytes) => Ok(Some(Entry::from_cbor(&bytes)?)),
            None => Ok(None),
        }
    }
    
    async fn get_by_index(
        &self,
        spine_id: SpineId,
        index: u64,
    ) -> Result<Option<Entry>, StorageError> {
        let idx_key = Self::entry_idx_key(spine_id, index);
        
        match self.db.get(&idx_key)? {
            Some(hash_bytes) => {
                let hash: EntryHash = hash_bytes.try_into()
                    .map_err(|_| StorageError::Corruption("Invalid hash".into()))?;
                self.get_by_hash(&hash).await
            }
            None => Ok(None),
        }
    }
    
    async fn get_range(
        &self,
        spine_id: SpineId,
        start: u64,
        end: u64,
    ) -> Result<Vec<Entry>, StorageError> {
        let start_key = Self::entry_idx_key(spine_id, start);
        let end_key = Self::entry_idx_key(spine_id, end);
        
        let mut entries = Vec::new();
        let iter = self.db.iterator(rocksdb::IteratorMode::From(
            &start_key,
            rocksdb::Direction::Forward,
        ));
        
        for item in iter {
            let (key, hash_bytes) = item?;
            if key.as_ref() >= end_key.as_slice() {
                break;
            }
            
            let hash: EntryHash = hash_bytes.as_ref().try_into()
                .map_err(|_| StorageError::Corruption("Invalid hash".into()))?;
            
            if let Some(entry) = self.get_by_hash(&hash).await? {
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }
    
    // ... other implementations
}
```

---

## 6. Backend Selection

### 6.1 Configuration

```rust
/// Storage configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend: StorageBackendType,
    pub path: Option<PathBuf>,
    pub connection_string: Option<String>,
    pub sqlite: Option<SqliteOptions>,
    pub postgres: Option<PostgresOptions>,
    pub rocksdb: Option<RocksDbOptions>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StorageBackendType {
    Sqlite,
    Postgres,
    RocksDb,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SqliteOptions {
    pub journal_mode: String,
    pub synchronous: String,
    pub cache_size: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostgresOptions {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RocksDbOptions {
    pub compression: CompressionType,
    pub cache_size: usize,
    pub max_open_files: i32,
}
```

### 6.2 Factory

```rust
/// Create storage backends from configuration
pub async fn create_entry_store(
    config: &StorageConfig,
) -> Result<Box<dyn EntryStore>, StorageError> {
    match config.backend {
        StorageBackendType::Sqlite => {
            let path = config.path.as_ref()
                .ok_or_else(|| StorageError::Config("Path required for SQLite".into()))?;
            Ok(Box::new(SqliteEntryStore::open(path)?))
        }
        StorageBackendType::Postgres => {
            let url = config.connection_string.as_ref()
                .ok_or_else(|| StorageError::Config("Connection string required for Postgres".into()))?;
            Ok(Box::new(PostgresEntryStore::connect(url).await?))
        }
        StorageBackendType::RocksDb => {
            let path = config.path.as_ref()
                .ok_or_else(|| StorageError::Config("Path required for RocksDB".into()))?;
            Ok(Box::new(RocksDbEntryStore::open(path)?))
        }
    }
}
```

---

## 7. Performance Characteristics

| Operation | SQLite | PostgreSQL | RocksDB |
|-----------|--------|------------|---------|
| Append entry | ~100µs | ~1ms | ~50µs |
| Get by hash | ~50µs | ~500µs | ~10µs |
| Get range (100) | ~1ms | ~5ms | ~500µs |
| Query by type | ~10ms | ~50ms | ~100ms* |
| Concurrent writes | Low | High | Medium |
| Concurrent reads | High | High | Very High |
| Query flexibility | Full SQL | Full SQL | Key-value only |
| Replication | None | Built-in | External |

*RocksDB requires full scan for non-key queries

---

## 8. References

- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [DATA_MODEL.md](./DATA_MODEL.md) — Data structures
- [RhizoCrypt Storage](../../rhizoCrypt/specs/STORAGE_BACKENDS.md)

---

*LoamSpine: The permanent record that gives memory its meaning.*

