<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# LoamSpine — Integration Specification

**Version**: 1.1.0  
**Status**: Active  
**Last Updated**: December 24, 2025

---

## 1. Overview

LoamSpine integrates with the ecoPrimals ecosystem as the permanent ledger layer. This document specifies how LoamSpine interacts with external services.

### 1.1 Capability-Based Discovery

**Key Principle**: LoamSpine code never hardcodes specific primal names. All external services are discovered at runtime via:

- **Environment Variables**: `LOAMSPINE_SIGNER_PATH`, `LOAMSPINE_STORAGE_URL`, etc.
- **Capability Registry**: Runtime registration of `Signer`, `Verifier`, and other capabilities
- **Trait Abstractions**: Code depends on traits, not specific implementations

The primal names in this document describe the *ecosystem architecture* for documentation purposes. The actual implementations use agnostic capability names like `CliSigner`, `CliVerifier`, `CommitAcceptor`, etc.

```
┌─────────────────────────────────────────────────────────────────┐
│                         LoamSpine                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│                    ┌─────────────────┐                          │
│                    │   LoamSpine     │                          │
│                    │     Core        │                          │
│                    └────────┬────────┘                          │
│                             │                                    │
│     ┌───────────────────────┼───────────────────────┐           │
│     │           │           │           │           │           │
│ ┌───▼───┐  ┌───▼───┐  ┌───▼───┐  ┌───▼───┐  ┌───▼───┐        │
│ │BearDog│  │Rhizo  │  │Sweet  │  │Nest   │  │Songbird│        │
│ │Adapter│  │Crypt  │  │Grass  │  │Gate   │  │Adapter │        │
│ │       │  │Adapter│  │Adapter│  │Adapter│  │        │        │
│ └───┬───┘  └───┬───┘  └───┬───┘  └───┬───┘  └───┬───┘        │
│     │           │           │           │           │           │
└─────┼───────────┼───────────┼───────────┼───────────┼───────────┘
      │           │           │           │           │
      ▼           ▼           ▼           ▼           ▼
 ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
 │ BearDog │ │RhizoCrypt│ │SweetGrass│ │ NestGate │ │ Songbird│
 │   🐻    │ │   🔐    │ │   🌾    │ │   🏠    │ │   🐦   │
 └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘
```

---

## 2. BearDog Integration

BearDog provides identity, signing, and policy enforcement for LoamSpine.

### 2.1 Client Interface

```rust
/// BearDog client for LoamSpine
#[async_trait]
pub trait BearDogClient: Send + Sync {
    // ==================== Identity ====================
    
    /// Resolve a DID to its document
    async fn resolve_did(&self, did: &Did) -> Result<DidDocument, BearDogError>;
    
    /// Verify that a DID is valid and active
    async fn verify_did(&self, did: &Did) -> Result<bool, BearDogError>;
    
    // ==================== Signing ====================
    
    /// Sign data with a specific key
    async fn sign(&self, data: &[u8], key_id: &KeyId) -> Result<Signature, BearDogError>;
    
    /// Sign an entry
    async fn sign_entry(&self, entry: &Entry, key_id: &KeyId) -> Result<Signature, BearDogError>;
    
    /// Verify a signature
    async fn verify_signature(
        &self,
        data: &[u8],
        signature: &Signature,
        did: &Did,
    ) -> Result<bool, BearDogError>;
    
    /// Verify an entry signature
    async fn verify_entry_signature(&self, entry: &Entry) -> Result<bool, BearDogError>;
    
    // ==================== Attestations ====================
    
    /// Request an attestation
    async fn request_attestation(
        &self,
        attester: &Did,
        subject_hash: ContentHash,
    ) -> Result<Attestation, BearDogError>;
    
    /// Verify an attestation
    async fn verify_attestation(&self, attestation: &Attestation) -> Result<bool, BearDogError>;
    
    // ==================== Permissions ====================
    
    /// Check permission for an action
    async fn check_permission(
        &self,
        did: &Did,
        resource: &str,
        action: &str,
    ) -> Result<PermissionResult, BearDogError>;
}
```

### 2.2 Permission Model

```rust
pub mod permissions {
    // Spine permissions
    pub const SPINE_CREATE: &str = "loamspine:spine:create";
    pub const SPINE_READ: &str = "loamspine:spine:{id}:read";
    pub const SPINE_WRITE: &str = "loamspine:spine:{id}:write";
    pub const SPINE_ADMIN: &str = "loamspine:spine:{id}:admin";
    
    // Certificate permissions
    pub const CERT_MINT: &str = "loamspine:certificate:mint";
    pub const CERT_TRANSFER: &str = "loamspine:certificate:{id}:transfer";
    pub const CERT_LOAN: &str = "loamspine:certificate:{id}:loan";
    
    // Waypoint permissions
    pub const WAYPOINT_ANCHOR: &str = "loamspine:waypoint:{id}:anchor";
    pub const WAYPOINT_OPERATE: &str = "loamspine:waypoint:{id}:operate";
}
```

### 2.3 Signature Requirements

| Entry Type | Signature Required | Additional Attestations |
|------------|-------------------|------------------------|
| Genesis | Yes (owner) | None |
| SessionCommit | Yes (committer) | Optional (witnesses) |
| CertificateMint | Yes (minter) | Optional (authority) |
| CertificateTransfer | Yes (from) | Optional (to) |
| CertificateLoan | Yes (lender) | Optional (borrower) |
| SliceAnchor | Yes (holder) | None |
| All others | Yes | Per policy |

---

## 3. RhizoCrypt Integration

RhizoCrypt is the primary producer of LoamSpine entries through dehydration.

### 3.1 Dehydration Interface

```rust
/// Interface for RhizoCrypt to commit to LoamSpine
#[async_trait]
pub trait DehydrationTarget: Send + Sync {
    /// Append a session commit entry
    async fn commit_session(
        &self,
        spine_id: SpineId,
        session_id: SessionId,
        summary: DehydrationSummary,
        signer: &impl Signer,
    ) -> Result<EntryHash, LoamError>;
    
    /// Mark an entry as sliced (checked out)
    async fn mark_sliced(
        &self,
        spine_id: SpineId,
        entry_hash: EntryHash,
        slice_id: SliceId,
    ) -> Result<(), LoamError>;
    
    /// Clear slice mark
    async fn clear_slice_mark(
        &self,
        spine_id: SpineId,
        entry_hash: EntryHash,
        slice_id: SliceId,
    ) -> Result<(), LoamError>;
    
    /// Record slice checkout
    async fn record_slice_checkout(
        &self,
        spine_id: SpineId,
        slice_id: SliceId,
        session_id: SessionId,
        mode: SliceMode,
        signer: &impl Signer,
    ) -> Result<EntryHash, LoamError>;
    
    /// Record slice return
    async fn record_slice_return(
        &self,
        spine_id: SpineId,
        slice_id: SliceId,
        checkout_entry: EntryHash,
        waypoint_summary: Option<WaypointSummary>,
        signer: &impl Signer,
    ) -> Result<EntryHash, LoamError>;
}

impl DehydrationTarget for LoamSpine {
    async fn commit_session(
        &self,
        spine_id: SpineId,
        session_id: SessionId,
        summary: DehydrationSummary,
        signer: &impl Signer,
    ) -> Result<EntryHash, LoamError> {
        let spine = self.get_spine(spine_id).await?;
        
        let entry = EntryBuilder::new(EntryType::SessionCommit {
            session_id,
            session_type: summary.session_type.clone(),
            merkle_root: summary.merkle_root.clone(),
            summary: summary.clone(),
        })
        .build(&spine, spine.owner.clone(), signer)
        .await?;
        
        self.append_entry(spine_id, entry).await
    }
    
    // ... other implementations
}
```

### 3.2 Session Commit Flow

```
RhizoCrypt                              LoamSpine
    │                                       │
    │  Session resolving                    │
    │                                       │
    ├──── commit_session() ────────────────►│
    │     (SessionCommit entry)             │
    │                                       ├── Validate entry
    │                                       ├── Sign and append
    │◄──── EntryHash ──────────────────────┤
    │                                       │
    │  For each slice:                      │
    │                                       │
    ├──── record_slice_return() ───────────►│
    │     (SliceReturn entry)               │
    │                                       ├── Update certificate
    │◄──── EntryHash ──────────────────────┤
    │                                       │
    ├──── clear_slice_mark() ──────────────►│
    │                                       │
    │◄──── OK ─────────────────────────────┤
    │                                       │
```

---

## 4. SweetGrass Integration

SweetGrass queries LoamSpine for provenance and attribution data.

### 4.1 Query Interface

```rust
/// Interface for SweetGrass to query LoamSpine
#[async_trait]
pub trait ProvenanceSource: Send + Sync {
    /// Get entries by data hash
    async fn get_entries_for_data(
        &self,
        data_hash: ContentHash,
    ) -> Result<Vec<EntryRef>, LoamError>;
    
    /// Get certificate history
    async fn get_certificate_history(
        &self,
        cert_id: CertificateId,
    ) -> Result<CertificateHistory, LoamError>;
    
    /// Get entries by committer
    async fn get_entries_by_committer(
        &self,
        committer: &Did,
        limit: usize,
    ) -> Result<Vec<EntryRef>, LoamError>;
    
    /// Get session commit for a session
    async fn get_session_commit(
        &self,
        session_id: SessionId,
    ) -> Result<Option<Entry>, LoamError>;
    
    /// Get provenance chain for a certificate
    async fn get_provenance_chain(
        &self,
        cert_id: CertificateId,
    ) -> Result<ProvenanceChain, LoamError>;
    
    /// Get attribution data for a data hash
    async fn get_attribution(
        &self,
        data_hash: ContentHash,
    ) -> Result<AttributionData, LoamError>;
}

/// Entry reference (for queries)
#[derive(Clone, Debug)]
pub struct EntryRef {
    pub spine_id: SpineId,
    pub entry_hash: EntryHash,
    pub index: u64,
    pub entry_type: String,
    pub committer: Did,
    pub timestamp: u64,
}

/// Provenance chain
#[derive(Clone, Debug)]
pub struct ProvenanceChain {
    pub certificate: Certificate,
    pub mint_entry: Entry,
    pub transfers: Vec<Entry>,
    pub current_entry: Entry,
    pub agents: HashSet<Did>,
}

/// Attribution data
#[derive(Clone, Debug)]
pub struct AttributionData {
    pub data_hash: ContentHash,
    pub creators: Vec<Did>,
    pub contributors: Vec<Did>,
    pub sessions: Vec<SessionId>,
    pub certificates: Vec<CertificateId>,
    pub first_appearance: u64,
    pub last_update: u64,
}
```

### 4.2 Braid Anchoring

SweetGrass can anchor attribution braids in LoamSpine:

```rust
/// Anchor a SweetGrass braid
pub async fn anchor_braid(
    spine_id: SpineId,
    braid_id: BraidId,
    braid_hash: ContentHash,
    subject_hash: ContentHash,
    signer: &impl Signer,
    store: &impl SpineStore,
) -> Result<EntryHash, LoamError> {
    let spine = store.get_spine(spine_id).await?
        .ok_or(LoamError::SpineNotFound(spine_id))?;
    
    let entry = EntryBuilder::new(EntryType::BraidCommit {
        braid_id,
        braid_hash,
        subject_hash,
    })
    .build(&spine, spine.owner.clone(), signer)
    .await?;
    
    store.append_entry(spine_id, entry).await
}
```

---

## 5. NestGate Integration

NestGate stores large payloads referenced by LoamSpine entries.

### 5.1 Client Interface

```rust
/// NestGate client for LoamSpine
#[async_trait]
pub trait NestGateClient: Send + Sync {
    /// Store a payload
    async fn put(&self, data: Bytes) -> Result<PayloadRef, NestGateError>;
    
    /// Get a payload
    async fn get(&self, payload_ref: &PayloadRef) -> Result<Option<Bytes>, NestGateError>;
    
    /// Check if payload exists
    async fn exists(&self, payload_ref: &PayloadRef) -> Result<bool, NestGateError>;
    
    /// Get payload metadata
    async fn metadata(&self, payload_ref: &PayloadRef) -> Result<Option<PayloadMetadata>, NestGateError>;
}
```

### 5.2 Payload Usage

```rust
// When creating an entry with a large payload:

// 1. Store payload in NestGate
let payload_data = large_data_bytes;
let payload_ref = nestgate.put(payload_data).await?;

// 2. Reference in entry
let entry = EntryBuilder::new(EntryType::DataAnchor {
    data_hash: payload_ref.hash,
    mime_type: Some("application/octet-stream".into()),
    size: payload_ref.size,
})
.with_payload(payload_ref)
.build(&spine, committer, signer)
.await?;

// 3. Append to spine
let entry_hash = store.append_entry(spine.id, entry).await?;
```

---

## 6. Service Registry Integration

A service registry (Songbird, Consul, etcd, or any compatible HTTP registry) provides
service discovery and capability routing.

### 6.1 UPA Registration

```rust
/// Register LoamSpine with the service registry
pub async fn register_with_registry(
    loamspine: &LoamSpine,
    registry: &impl RegistryClient,
) -> Result<RegistrationReceipt, RegistryError> {
    let capabilities = vec![
        Capability::new("loamspine:spine:create"),
        Capability::new("loamspine:spine:read"),
        Capability::new("loamspine:spine:write"),
        Capability::new("loamspine:certificate:mint"),
        Capability::new("loamspine:certificate:transfer"),
        Capability::new("loamspine:certificate:loan"),
        Capability::new("loamspine:waypoint:anchor"),
        Capability::new("loamspine:proof:generate"),
    ];
    
    let service_info = ServiceInfo {
        name: "loamspine".to_string(),
        version: loamspine.version().to_string(),
        capabilities,
        endpoints: vec![
            // Pure Rust RPC: tarpc for primal-to-primal
            Endpoint::Tarpc {
                host: loamspine.tarpc_host(),
                port: loamspine.tarpc_port(),
            },
            // JSON-RPC 2.0 for external clients
            Endpoint::JsonRpc {
                base_url: loamspine.jsonrpc_url(),
            },
        ],
        health_check: Some(HealthCheck {
            endpoint: "/rpc".to_string(),
            method: "health.check".to_string(),
            interval: Duration::from_secs(30),
        }),
    };
    
    registry.register(service_info).await
}
```

### 6.2 Service Discovery

```rust
/// Service registry client interface (vendor-agnostic)
#[async_trait]
pub trait RegistryClient: Send + Sync {
    /// Register a service
    async fn register(&self, service: ServiceInfo) -> Result<RegistrationReceipt, RegistryError>;
    
    /// Discover services by capability
    async fn discover(&self, capability: &str) -> Result<Vec<ServiceEndpoint>, RegistryError>;
    
    /// Get specific service
    async fn get_service(&self, name: &str) -> Result<Option<ServiceInfo>, RegistryError>;
}
```

---

## 7. Federation Integration

LoamSpine supports replication between peers for community spines.

### 7.1 Sync Protocol

```rust
/// Sync protocol for replication
#[async_trait]
pub trait SyncProtocol: Send + Sync {
    /// Get sync status with a peer
    async fn get_sync_status(
        &self,
        peer: &PeerId,
        spine_id: SpineId,
    ) -> Result<SyncStatus, SyncError>;
    
    /// Push entries to peer
    async fn push_entries(
        &self,
        peer: &PeerId,
        spine_id: SpineId,
        entries: Vec<Entry>,
    ) -> Result<PushReceipt, SyncError>;
    
    /// Pull entries from peer
    async fn pull_entries(
        &self,
        peer: &PeerId,
        spine_id: SpineId,
        from_index: u64,
    ) -> Result<Vec<Entry>, SyncError>;
    
    /// Request full sync
    async fn request_sync(
        &self,
        peer: &PeerId,
        spine_id: SpineId,
    ) -> Result<SyncJob, SyncError>;
}

/// Sync status
#[derive(Clone, Debug)]
pub struct SyncStatus {
    pub spine_id: SpineId,
    pub local_height: u64,
    pub remote_height: u64,
    pub last_sync: Option<u64>,
    pub pending_entries: u64,
    pub state: SyncState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncState {
    Synced,
    Behind { entries: u64 },
    Ahead { entries: u64 },
    Diverged { at_index: u64 },
    Unknown,
}
```

### 7.2 Replication Flow

```
Node A                                  Node B
   │                                       │
   │ get_sync_status(B, spine)            │
   ├──────────────────────────────────────►│
   │                                       │
   │◄──────────────────────────────────────┤
   │ SyncStatus { local: 100, remote: 95 }│
   │                                       │
   │ push_entries(B, spine, [96..100])    │
   ├──────────────────────────────────────►│
   │                                       ├── Validate entries
   │                                       ├── Append to spine
   │◄──────────────────────────────────────┤
   │ PushReceipt { accepted: 5 }          │
   │                                       │
```

---

## 8. Adapter Pattern

All integrations use a common adapter pattern:

```rust
/// Generic primal adapter
pub struct PrimalAdapter<C> {
    client: C,
    config: AdapterConfig,
    metrics: AdapterMetrics,
    cache: Option<AdapterCache>,
}

impl<C> PrimalAdapter<C> {
    pub fn new(client: C, config: AdapterConfig) -> Self {
        Self {
            client,
            config,
            metrics: AdapterMetrics::default(),
            cache: config.cache_enabled.then(AdapterCache::new),
        }
    }
}

/// Adapter configuration
#[derive(Clone, Debug)]
pub struct AdapterConfig {
    pub retry: RetryConfig,
    pub circuit_breaker: CircuitBreakerConfig,
    pub timeout: Duration,
    pub cache_enabled: bool,
    pub cache_ttl: Duration,
}

/// Retry configuration
#[derive(Clone, Debug)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub exponential_base: f64,
}
```

---

## 9. Error Handling

### 9.1 Integration Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("BearDog error: {0}")]
    BearDog(#[from] BearDogError),
    
    #[error("RhizoCrypt error: {0}")]
    RhizoCrypt(String),
    
    #[error("SweetGrass error: {0}")]
    SweetGrass(String),
    
    #[error("NestGate error: {0}")]
    NestGate(#[from] NestGateError),
    
    #[error("Songbird error: {0}")]
    Songbird(#[from] SongbirdError),
    
    #[error("Sync error: {0}")]
    Sync(#[from] SyncError),
    
    #[error("Connection failed: {0}")]
    Connection(String),
    
    #[error("Timeout after {0:?}")]
    Timeout(Duration),
}
```

---

## 10. References

- [PURE_RUST_RPC.md](./PURE_RUST_RPC.md) — Pure Rust RPC philosophy
- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [API_SPECIFICATION.md](./API_SPECIFICATION.md) — tarpc + JSON-RPC API definitions
- [RhizoCrypt Specification](../../rhizoCrypt/specs/)
- [BearDog Specification](../../bearDog/specs/)
- [Songbird Specification](../../songBird/specs/)

---

*LoamSpine: Pure Rust, Primal Sovereignty, Permanent History.*

