<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# LoamSpine — API Specification

**Version**: 1.0.0  
**Status**: Active  
**Last Updated**: May 29, 2026

---

## 1. Overview

### 1.1 Pure Rust RPC Philosophy

LoamSpine uses **pure Rust RPC**—no gRPC, no protobuf, no C++ tooling.

| ❌ What We Don't Use | ✅ What We Use |
|---------------------|----------------|
| gRPC | tarpc (pure Rust) |
| protobuf | serde (native Rust) |
| protoc (C++ compiler) | cargo build |
| tonic | pure Rust JSON-RPC 2.0 (hand-rolled, no jsonrpsee) |
| prost | Native Rust types |

See [PURE_RUST_RPC.md](./PURE_RUST_RPC.md) for the full philosophy.

### 1.2 Dual Protocol Strategy

LoamSpine exposes two complementary APIs:

| Protocol | Use Case | Performance | Clients |
|----------|----------|-------------|---------|
| **tarpc** | Primal-to-primal | <1ms latency | Rust primals |
| **JSON-RPC 2.0** | External clients | ~5ms latency | Python, JS, curl |

Both APIs share the same service implementation and message types.

---

## 2. tarpc API (Binary RPC)

### 2.1 Service Trait

```rust
#[tarpc::service]
pub trait LoamSpineRpc {
    // ==================== Spine Management ====================
    
    /// Create a new spine
    async fn create_spine(request: CreateSpineRequest) -> Result<CreateSpineResponse, ApiError>;
    
    /// Get spine details
    async fn get_spine(request: GetSpineRequest) -> Result<GetSpineResponse, ApiError>;
    
    /// Seal spine (make read-only)
    async fn seal_spine(request: SealSpineRequest) -> Result<SealSpineResponse, ApiError>;
    
    // ==================== Entry Operations ====================
    
    /// Append an entry (Tower-signed via crypto.sign_ed25519 when TOWER_SIGNER_SOCKET is set)
    async fn append_entry(request: AppendEntryRequest) -> Result<AppendEntryResponse, ApiError>;
    
    /// Get entry by hash
    async fn get_entry(request: GetEntryRequest) -> Result<GetEntryResponse, ApiError>;
    
    /// Get spine tip
    async fn get_tip(request: GetTipRequest) -> Result<GetTipResponse, ApiError>;
    
    // ==================== Certificate Operations ====================
    
    /// Mint a new certificate
    async fn mint_certificate(request: MintCertificateRequest) -> Result<MintCertificateResponse, ApiError>;
    
    /// Get certificate
    async fn get_certificate(request: GetCertificateRequest) -> Result<GetCertificateResponse, ApiError>;
    
    /// Transfer certificate
    async fn transfer_certificate(request: TransferCertificateRequest) -> Result<TransferCertificateResponse, ApiError>;
    
    /// Loan certificate
    async fn loan_certificate(request: LoanCertificateRequest) -> Result<LoanCertificateResponse, ApiError>;
    
    /// Return loaned certificate
    async fn return_certificate(request: ReturnCertificateRequest) -> Result<ReturnCertificateResponse, ApiError>;
    
    // ==================== Waypoint Operations ====================
    
    /// Anchor slice at waypoint
    async fn anchor_slice(request: AnchorSliceRequest) -> Result<AnchorSliceResponse, ApiError>;
    
    /// Checkout slice from waypoint
    async fn checkout_slice(request: CheckoutSliceRequest) -> Result<CheckoutSliceResponse, ApiError>;
    
    // ==================== Proof Operations ====================
    
    /// Generate inclusion proof
    async fn generate_inclusion_proof(request: GenerateInclusionProofRequest) -> Result<GenerateInclusionProofResponse, ApiError>;
    
    /// Verify inclusion proof
    async fn verify_inclusion_proof(request: VerifyInclusionProofRequest) -> Result<VerifyInclusionProofResponse, ApiError>;
    
    // ==================== Integration ====================
    
    /// Commit session — returns self-contained provenance receipt with session binding + tower signature
    async fn commit_session(request: CommitSessionRequest) -> Result<CommitSessionResponse, ApiError>;
    
    /// Commit SweetGrass braid
    async fn commit_braid(request: CommitBraidRequest) -> Result<CommitBraidResponse, ApiError>;
    
    // ==================== Health ====================
    
    /// Health check
    async fn health_check(request: HealthCheckRequest) -> Result<HealthCheckResponse, ApiError>;

    // ==================== Auth (JH-0 Method Gate) ====================

    /// Check whether a method is allowed under the current auth mode
    async fn auth_check(params: { method: String }) -> Result<AuthCheckResponse, ApiError>;
    
    /// Return the current auth mode and public method classification
    async fn auth_mode() -> Result<AuthModeResponse, ApiError>;
    
    /// Return peer authentication status
    async fn auth_peer_info() -> Result<AuthPeerInfoResponse, ApiError>;
}
```

### 2.2 Client Example

```rust
use tarpc::{client, context};
use tarpc::tokio_serde::formats::Json;

// Connect to tarpc server
let transport = tarpc::serde_transport::tcp::connect(addr, Json::default).await?;
let client = LoamSpineRpcClient::new(client::Config::default(), transport).spawn();

// Create a spine
let response = client.create_spine(
    context::current(),
    CreateSpineRequest {
        name: "my-history".to_string(),
        owner: Did::new("did:key:z6MkOwner"),
        config: None,
    },
).await?;

println!("Created spine: {:?}", response.spine_id);
```

### 2.3 Server Startup

```rust
use loam_spine_api::{run_tarpc_server, LoamSpineRpcService};

let service = LoamSpineRpcService::default_service();
let addr = "127.0.0.1:9001".parse()?;

run_tarpc_server(addr, service).await?;
```

---

## 3. JSON-RPC 2.0 API

### 3.1 Endpoint

```
POST /rpc HTTP/1.1
Host: localhost:8080
Content-Type: application/json
```

### 3.2 Available Methods

Methods follow the `{domain}.{operation}` semantic naming standard
(see `wateringHole/SEMANTIC_METHOD_NAMING_STANDARD.md`).

| Method | Description |
|--------|-------------|
| `spine.create` | Create a new spine |
| `spine.get` | Get spine by ID |
| `spine.list` | List all spine IDs (paginated) |
| `spine.seal` | Seal a spine |
| `entry.append` | Append entry to spine |
| `entry.get` | Get entry by hash |
| `entry.get_tip` | Get tip entry |
| `entry.list` | List entries in a spine (paginated, `start`/`limit`) |
| `certificate.mint` | Mint certificate |
| `certificate.get` | Get certificate |
| `certificate.transfer` | Transfer certificate |
| `certificate.loan` | Loan certificate |
| `certificate.return` | Return certificate |
| `slice.anchor` | Anchor slice |
| `slice.checkout` | Checkout slice |
| `proof.generate_inclusion` | Generate proof |
| `proof.verify_inclusion` | Verify proof |
| `anchor.publish` | Record a public chain anchor receipt on a spine |
| `anchor.publish_batch` | Aggregate batch anchor across N spines (Merkle aggregation) |
| `anchor.verify` | Verify an anchor receipt (single or aggregate) against spine state |
| `session.dehydrate` | Compute content-addressed summary of uncommitted entries (read-only) |
| `session.commit` | Commit session |
| `braid.commit` | Commit braid |
| `bonding.ledger.store` | Store a bond in the ionic bond ledger |
| `bonding.ledger.retrieve` | Retrieve a bond by ID |
| `bonding.ledger.list` | List all bond IDs |
| `health.check` | Health check |
| `health.liveness` | Liveness probe |
| `health.readiness` | Readiness probe |
| `identity.get` | Primal identity (name, version, domain, ecobin_grade) |
| `capabilities.list` | List capabilities with method count, stability tiers, cost estimates |
| `lifecycle.status` | Service lifecycle status (primal, version, status, auth_mode) |
| `btsp.negotiate` | BTSP Phase 3 cipher negotiation |
| `btsp.capabilities` | Lists supported BTSP ciphers, HKDF algorithm, info labels, frame format |
| `primal.announce` | Self-registration: returns primal identity, version, domain, full method list |
| `auth.check` | Check current auth status |
| `auth.mode` | Report current auth mode (permissive/enforced) |
| `auth.peer_info` | Report peer connection info |
| `tools.list` | MCP tool discovery |
| `tools.call` | MCP tool invocation |
| `permanence.commit_session` | Commit session (permanence alias) |
| `permanence.verify_commit` | Verify a commit |
| `permanence.get_commit` | Get a commit |
| `permanence.health_check` | Permanence health check |

#### Method Aliases

loamSpine normalizes method names before dispatch. Callers may use any alias; responses use the canonical name.

| Alias | Canonical Target |
|-------|-----------------|
| `commit.session`, `provenance.commit` | `session.commit` |
| `session.create`, `ledger.create` | `spine.create` |
| `session.state`, `session.get`, `ledger.state`, `ledger.get` | `spine.get` |
| `permanent-storage.commitSession` | `permanence.commit_session` |
| `permanent-storage.verifyCommit` | `permanence.verify_commit` |
| `permanent-storage.getCommit` | `permanence.get_commit` |
| `permanent-storage.healthCheck` | `permanence.health_check` |
| `capability.list`, `primal.capabilities` | `capabilities.list` |

### 3.3 Request Format

```json
{
    "jsonrpc": "2.0",
    "method": "{domain}.{operation}",
    "params": { ... },
    "id": <number>
}
```

### 3.4 Response Format

**Success:**
```json
{
    "jsonrpc": "2.0",
    "result": { ... },
    "id": <number>
}
```

**Error:**
```json
{
    "jsonrpc": "2.0",
    "error": {
        "code": -32000,
        "message": "error description"
    },
    "id": <number>
}
```

### 3.5 Example Requests

**Health Check:**
```bash
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "health.check",
    "params": { "include_details": true },
    "id": 1
  }'
```

**Create Spine:**
```bash
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "spine.create",
    "params": {
      "name": "my-history",
      "owner": { "value": "did:key:z6MkOwner" }
    },
    "id": 2
  }'
```

**Get Spine:**
```bash
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "spine.get",
    "params": {
      "spine_id": "01234567-89ab-cdef-0123-456789abcdef"
    },
    "id": 3
  }'
```

**Commit Session (RhizoCrypt Integration):**

> **Prerequisite:** A spine must exist before committing. Call `spine.create` first
> to obtain a `spine_id`, or use the compat method `permanence.commit_session`
> which auto-creates a spine per committer DID.

```bash
# Step 1: Create a spine (once per owner)
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "spine.create",
    "params": { "name": "rootpulse", "owner": { "value": "did:key:z6MkOwner" } },
    "id": 1
  }'
# → { "spine_id": "...", "genesis_hash": [...] }

# Step 2: Commit session to the spine
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "session.commit",
    "params": {
      "spine_id": "01234567-89ab-cdef-0123-456789abcdef",
      "session_id": "fedcba98-7654-3210-fedc-ba9876543210",
      "session_hash": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
      "vertex_count": 100,
      "committer": { "value": "did:key:z6MkCommitter" }
    },
    "id": 2
  }'
```

> **Alternative:** `permanence.commit_session` auto-creates a spine for the
> committer if one doesn't exist, without a separate `spine.create` call.
> Use this for workflows that don't need explicit spine management.

---

## 3.4 Entry Signing Contract

Callers do **not** provide entry signatures. Signing is handled internally:

| Aspect | Behavior |
|--------|----------|
| **Who signs** | LoamSpine calls the tower signer's `crypto.sign_ed25519` internally when `TOWER_SIGNER_SOCKET` is set |
| **What is signed** | The entry's canonical bytes (`entry.to_canonical_bytes()`) — includes all fields except metadata |
| **Where stored** | `entry.metadata["tower_signature"]` (base64 Ed25519) and `entry.metadata["tower_signature_alg"]` |
| **No Tower** | When `TOWER_SIGNER_SOCKET` is not set, entries are unsigned (standalone mode) |
| **`committer` field** | `Entry.committer` is always derived from `spine.owner`, **not** from the request. The `committer` field in `AppendEntryRequest` is optional and ignored. For `session.commit`, the request `committer` is embedded in the `SessionCommit` entry type. |
| **`committer` format** | DID string, e.g. `"did:key:z6MkOwner..."`. Set when creating the spine via `spine.create`. |
| **Hash fields** | All `ContentHash`/`EntryHash` fields accept both JSON byte arrays (`[1,2,...,32]`) and 64-char hex strings (`"0102..."` or `"0x0102..."`). |

The graph orchestrator (biomeOS/RootPulse) does **not** need to sign entries or
obtain signatures — LoamSpine handles this via Tower delegation. The orchestrator
only needs to call `spine.create` (once) and then `session.commit` or `entry.append`.

---

## 4. Message Types

### 4.1 Core Types

```rust
// Identifiers (native Rust types)
pub type SpineId = Uuid;           // UUIDv7
pub type EntryHash = [u8; 32];     // Blake3 hash — JSON: byte array OR 64-char hex string
pub type ContentHash = [u8; 32];   // Blake3 hash — JSON: byte array OR 64-char hex string
pub type CertificateId = Uuid;
pub type SliceId = [u8; 32];

// Semantic wrappers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Did { pub value: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature { pub bytes: Vec<u8> }
```

### 4.2 Spine Messages

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSpineRequest {
    pub name: String,
    pub owner: Did,
    pub config: Option<SpineConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSpineResponse {
    pub spine_id: SpineId,
    pub genesis_hash: EntryHash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSpineRequest {
    pub spine_id: SpineId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSpineResponse {
    pub found: bool,
    pub spine: Option<Spine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealSpineRequest {
    pub spine_id: SpineId,
    pub sealer: Did,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealSpineResponse {
    pub success: bool,
    pub seal_hash: Option<EntryHash>,
}
```

### 4.3 Entry Messages

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntryRequest {
    pub spine_id: SpineId,
    pub entry_type: EntryType,
    pub committer: Option<Did>,  // optional — Entry.committer derived from spine owner
    pub payload: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntryResponse {
    pub entry_hash: EntryHash,
    pub index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEntryRequest {
    pub spine_id: SpineId,
    pub entry_hash: EntryHash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEntryResponse {
    pub found: bool,
    pub entry: Option<Entry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTipRequest {
    pub spine_id: SpineId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTipResponse {
    pub tip_hash: EntryHash,
    pub entry: Entry,
    pub height: u64,
}
```

### 4.4 Certificate Messages

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintCertificateRequest {
    pub spine_id: SpineId,
    pub cert_type: CertificateType,
    pub owner: Did,
    pub metadata: Option<CertificateMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintCertificateResponse {
    pub certificate_id: CertificateId,
    pub mint_hash: EntryHash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferCertificateRequest {
    pub certificate_id: CertificateId,
    pub from: Did,
    pub to: Did,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanCertificateRequest {
    pub certificate_id: CertificateId,
    pub lender: Did,
    pub borrower: Did,
    pub terms: LoanTerms,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnCertificateRequest {
    pub certificate_id: CertificateId,
    pub returner: Did,
}
```

### 4.5 Health Messages

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckRequest {
    pub include_details: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: HealthStatus,
    pub report: Option<HealthReport>,
}
```

### 4.6 Integration Messages

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSessionRequest {
    pub spine_id: SpineId,
    pub session_id: Uuid,
    pub session_hash: ContentHash,
    pub vertex_count: u64,
    pub committer: Did,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSessionResponse {
    // Ledger anchor
    pub spine_id: SpineId,
    pub commit_hash: EntryHash,
    pub index: u64,
    pub committed_at: Timestamp,
    // Session binding (echoed from request)
    pub session_id: Uuid,
    pub merkle_root: ContentHash,
    pub vertex_count: u64,
    pub committer: Did,
    // Tower signature (when TOWER_SIGNER_SOCKET is set)
    pub tower_signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitBraidRequest {
    pub spine_id: SpineId,
    pub braid_id: Uuid,
    pub braid_hash: ContentHash,
    pub subjects: Vec<Did>,
    pub committer: Did,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitBraidResponse {
    pub commit_hash: EntryHash,
    pub index: u64,
}
```

---

## 5. Error Handling

### 5.1 Error Types

```rust
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ApiError {
    #[error("spine not found: {0}")]
    SpineNotFound(String),

    #[error("entry not found: {0}")]
    EntryNotFound(String),

    #[error("certificate not found: {0}")]
    CertificateNotFound(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("spine is sealed: {0}")]
    SpineSealed(String),

    #[error("certificate already exists: {0}")]
    CertificateExists(String),

    #[error("not certificate owner: {0}")]
    NotCertificateOwner(String),
}
```

### 5.2 JSON-RPC Error Codes

| Code | Meaning |
|------|---------|
| -32000 | Application error (ApiError) |
| -32600 | Invalid Request |
| -32601 | Method not found |
| -32602 | Invalid params |
| -32603 | Internal error |

---

## 6. Authentication

All API calls require tower authentication:

```
Authorization: Bearer <tower-token>
X-Tower-DID: did:key:z6Mk...
```

---

## 7. Rate Limiting

| Endpoint Category | Rate Limit |
|-------------------|------------|
| Spine management | 100 req/min |
| Entry operations | 1,000 req/min |
| Certificate operations | 500 req/min |
| Waypoint operations | 500 req/min |
| Proof generation | 100 req/min |

---

## 8. Performance

| Protocol | Latency | Throughput |
|----------|---------|------------|
| tarpc (binary) | <1ms | 100K req/s |
| JSON-RPC 2.0 | ~5ms | 20K req/s |

---

## 9. References

- [PURE_RUST_RPC.md](./PURE_RUST_RPC.md) — RPC philosophy
- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [DATA_MODEL.md](./DATA_MODEL.md) — Data structures
- [INTEGRATION_SPECIFICATION.md](./INTEGRATION_SPECIFICATION.md) — Primal integrations

---

*LoamSpine: Pure Rust APIs for Sovereign Permanence.*
