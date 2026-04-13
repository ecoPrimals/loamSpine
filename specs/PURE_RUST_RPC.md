<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# LoamSpine — Pure Rust RPC Specification

**Version**: 1.0.0  
**Status**: Active  
**Last Updated**: December 22, 2025

---

## 1. Philosophy

### 1.1 Primal Sovereignty

ecoPrimals embraces **primal sovereignty**—the principle that each primal controls its own destiny, data, and dependencies. This extends to our choice of RPC framework:

| Principle | Implementation |
|-----------|----------------|
| **Self-Sovereign** | No external tooling (protoc, protobuf) |
| **Pure Rust** | Lean into the Rust compiler |
| **No Vendor Lock-in** | No Google/corporate dependencies |
| **Human Dignity** | Simple tools that humans can understand |
| **Community-Driven** | Use crates the community maintains |

### 1.2 Why Not gRPC?

```
gRPC Problems:
❌ Requires protoc (C++ compiler, ~200MB binary)
❌ Requires protobuf (Google-maintained schema language)
❌ Non-Rust code generation (generated code is unidiomatic)
❌ Vendor lock-in (tied to Google ecosystem)
❌ Complex build process (build.rs + external tools)
❌ Breaks cargo-only builds (needs external dependencies)
❌ License concerns (Google-controlled evolution)
```

### 1.3 The ecoPrimals Way

```
Our Solution:
✅ Pure Rust (no C/C++ dependencies)
✅ Native serde serialization (community-standard)
✅ Rust procedural macros (tarpc #[service])
✅ No external tooling (cargo build is all you need)
✅ Full Rust compiler type checking
✅ Community-driven development
✅ Human-readable wire format (JSON-RPC)
```

---

## 2. Dual Protocol Architecture

### 2.1 Protocol Stack

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│              LoamSpineRpcService (business logic)           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────────┐    ┌────────────────────────────┐   │
│  │   tarpc (Binary)   │    │   JSON-RPC 2.0 (Text)      │   │
│  │                    │    │                            │   │
│  │  Primal ←→ Primal  │    │   External Clients         │   │
│  │  • RhizoCrypt      │    │   • Python                 │   │
│  │  • SweetGrass      │    │   • JavaScript             │   │
│  │  • BearDog         │    │   • curl/httpie            │   │
│  │  • Songbird        │    │   • Any language           │   │
│  │                    │    │                            │   │
│  │  High-performance  │    │   Universal access         │   │
│  │  Binary codec      │    │   Human-readable           │   │
│  └────────────────────┘    └────────────────────────────┘   │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│                    Transport Layer                           │
│              TCP (tarpc) │ HTTP (pure JSON-RPC)             │
├─────────────────────────────────────────────────────────────┤
│                    Serialization                             │
│                  serde + serde_json                         │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 When to Use Each Protocol

| Use Case | Protocol | Reason |
|----------|----------|--------|
| Primal-to-primal | **tarpc** | 10x faster, binary codec |
| Service mesh | **tarpc** | Type-safe, native Rust |
| Python/JS clients | **JSON-RPC** | Language agnostic |
| Debugging | **JSON-RPC** | Human-readable, curl-friendly |
| External integrations | **JSON-RPC** | Universal standard |
| Performance-critical | **tarpc** | Zero-copy, minimal overhead |

---

## 3. tarpc Service Definition

### 3.1 Why tarpc?

tarpc is a pure Rust RPC framework that uses Rust procedural macros:

```rust
// tarpc generates client and server code from this trait
#[tarpc::service]
pub trait LoamSpineRpc {
    async fn create_spine(request: CreateSpineRequest) -> Result<CreateSpineResponse, ApiError>;
    async fn get_spine(request: GetSpineRequest) -> Result<GetSpineResponse, ApiError>;
    // ... 16 more methods
}
```

**Benefits:**
- Type-safe at compile time (Rust compiler checks everything)
- Native async/await (no runtime code generation)
- Pluggable serialization (serde)
- Pluggable transport (TCP, Unix socket, in-memory)

### 3.2 Full Service Trait

```rust
#[tarpc::service]
pub trait LoamSpineRpc {
    // ========================================================================
    // Spine Operations
    // ========================================================================
    
    /// Create a new spine.
    async fn create_spine(request: CreateSpineRequest) -> Result<CreateSpineResponse, ApiError>;
    
    /// Get a spine by ID.
    async fn get_spine(request: GetSpineRequest) -> Result<GetSpineResponse, ApiError>;
    
    /// Seal a spine (make immutable).
    async fn seal_spine(request: SealSpineRequest) -> Result<SealSpineResponse, ApiError>;

    // ========================================================================
    // Entry Operations
    // ========================================================================
    
    /// Append an entry to a spine.
    async fn append_entry(request: AppendEntryRequest) -> Result<AppendEntryResponse, ApiError>;
    
    /// Get an entry by hash.
    async fn get_entry(request: GetEntryRequest) -> Result<GetEntryResponse, ApiError>;
    
    /// Get the tip entry of a spine.
    async fn get_tip(request: GetTipRequest) -> Result<GetTipResponse, ApiError>;

    // ========================================================================
    // Certificate Operations
    // ========================================================================
    
    /// Mint a new certificate.
    async fn mint_certificate(request: MintCertificateRequest) -> Result<MintCertificateResponse, ApiError>;
    
    /// Transfer a certificate.
    async fn transfer_certificate(request: TransferCertificateRequest) -> Result<TransferCertificateResponse, ApiError>;
    
    /// Loan a certificate.
    async fn loan_certificate(request: LoanCertificateRequest) -> Result<LoanCertificateResponse, ApiError>;
    
    /// Return a loaned certificate.
    async fn return_certificate(request: ReturnCertificateRequest) -> Result<ReturnCertificateResponse, ApiError>;
    
    /// Get a certificate by ID.
    async fn get_certificate(request: GetCertificateRequest) -> Result<GetCertificateResponse, ApiError>;

    // ========================================================================
    // Waypoint Operations
    // ========================================================================
    
    /// Anchor a slice on a waypoint spine.
    async fn anchor_slice(request: AnchorSliceRequest) -> Result<AnchorSliceResponse, ApiError>;
    
    /// Checkout a slice from a waypoint.
    async fn checkout_slice(request: CheckoutSliceRequest) -> Result<CheckoutSliceResponse, ApiError>;

    // ========================================================================
    // Proof Operations
    // ========================================================================
    
    /// Generate an inclusion proof.
    async fn generate_inclusion_proof(request: GenerateInclusionProofRequest) -> Result<GenerateInclusionProofResponse, ApiError>;
    
    /// Verify an inclusion proof.
    async fn verify_inclusion_proof(request: VerifyInclusionProofRequest) -> Result<VerifyInclusionProofResponse, ApiError>;

    // ========================================================================
    // Health Operations
    // ========================================================================
    
    /// Health check.
    async fn health_check(request: HealthCheckRequest) -> Result<HealthCheckResponse, ApiError>;

    // ========================================================================
    // Integration Operations
    // ========================================================================
    
    /// Commit a RhizoCrypt session.
    async fn commit_session(request: CommitSessionRequest) -> Result<CommitSessionResponse, ApiError>;
    
    /// Commit a SweetGrass braid.
    async fn commit_braid(request: CommitBraidRequest) -> Result<CommitBraidResponse, ApiError>;
}
```

### 3.3 Server Implementation

```rust
/// tarpc server for LoamSpine.
pub struct LoamSpineTarpcServer {
    service: Arc<LoamSpineRpcService>,
}

impl LoamSpineRpc for LoamSpineTarpcServer {
    async fn create_spine(
        self,
        _: tarpc::context::Context,
        request: CreateSpineRequest,
    ) -> Result<CreateSpineResponse, ApiError> {
        self.service.create_spine(request).await
    }
    
    // ... implement all methods
}

/// Run the tarpc server.
pub async fn run_tarpc_server(
    addr: SocketAddr,
    service: LoamSpineRpcService,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = tarpc::serde_transport::tcp::listen(&addr, Json::default).await?;
    let server = LoamSpineTarpcServer::new(service);

    info!("🚀 LoamSpine tarpc server listening on {}", addr);

    listener
        .filter_map(|r| async { r.ok() })
        .map(server::BaseChannel::with_defaults)
        .max_channels_per_key(10, |t| t.transport().peer_addr().unwrap().ip())
        .map(|channel| {
            let server = server.clone();
            channel.execute(server.serve())
        })
        .flatten()
        .buffer_unordered(100)
        .for_each(|_| async {})
        .await;

    Ok(())
}
```

### 3.4 Client Usage

```rust
use tarpc::{client, context};

let transport = tarpc::serde_transport::tcp::connect(addr, Json::default).await?;
let client = LoamSpineRpcClient::new(client::Config::default(), transport).spawn();

let response = client.create_spine(
    context::current(),
    CreateSpineRequest {
        name: "my-spine".into(),
        owner: Did::new("did:key:z6MkOwner"),
        config: None,
    },
).await?;
```

---

## 4. JSON-RPC 2.0 Specification

### 4.1 Why JSON-RPC?

JSON-RPC 2.0 is the universal standard for RPC:
- Language agnostic (works with any HTTP client)
- Human-readable (easy to debug with curl)
- Simple specification (no complex tooling)
- Widely supported (Python, JS, Go, etc.)

### 4.2 Endpoint

```
POST /rpc HTTP/1.1
Host: localhost:8080
Content-Type: application/json

{
    "jsonrpc": "2.0",
    "method": "{domain}.{operation}",
    "params": { ... },
    "id": 1
}
```

### 4.3 Available Methods

Methods follow the `{domain}.{operation}` semantic naming standard
(see `wateringHole/SEMANTIC_METHOD_NAMING_STANDARD.md`).

| Method | Description |
|--------|-------------|
| `spine.create` | Create a new spine |
| `spine.get` | Get spine by ID |
| `spine.seal` | Seal a spine |
| `entry.append` | Append entry |
| `entry.get` | Get entry by hash |
| `entry.get_tip` | Get tip entry |
| `certificate.mint` | Mint certificate |
| `certificate.transfer` | Transfer certificate |
| `certificate.loan` | Loan certificate |
| `certificate.return` | Return certificate |
| `certificate.get` | Get certificate |
| `health.check` | Health check |
| `session.commit` | Commit session (dehydration) |
| `braid.commit` | Commit braid (attribution) |

### 4.4 Example Requests

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

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "Healthy",
    "report": {
      "name": "LoamSpine",
      "version": "0.1.0",
      "status": "Healthy",
      "uptime_secs": 3600,
      "components": [{ "name": "storage: 5 spines", "status": "Healthy" }]
    }
  },
  "id": 1
}
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

**Python Client:**
```python
import requests

class LoamSpineClient:
    def __init__(self, url="http://localhost:8080/rpc"):
        self.url = url
        self.id = 0
    
    def _call(self, method, params):
        self.id += 1
        response = requests.post(self.url, json={
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": self.id
        })
        return response.json()["result"]
    
    def health_check(self, include_details=True):
        return self._call("health.check", {"include_details": include_details})
    
    def create_spine(self, name, owner):
        return self._call("spine.create", {
            "name": name,
            "owner": {"value": owner}
        })

# Usage
client = LoamSpineClient()
print(client.health_check())
```

---

## 5. Message Types

### 5.1 Native Rust Types

All message types are native Rust structs with serde derives:

```rust
/// Request to create a new spine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSpineRequest {
    pub name: String,
    pub owner: Did,
    pub config: Option<SpineConfig>,
}

/// Response from creating a spine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSpineResponse {
    pub spine_id: SpineId,
    pub genesis_hash: EntryHash,
}
```

### 5.2 Core Type Definitions

```rust
// Identifiers
pub type SpineId = Uuid;           // UUIDv7
pub type EntryHash = [u8; 32];     // Blake3 hash
pub type ContentHash = [u8; 32];   // Blake3 hash
pub type CertificateId = Uuid;
pub type SliceId = [u8; 32];

// Semantic types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Did(String);            // did:key:z6Mk...

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature(Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamp(u64);         // Nanoseconds since epoch
```

---

## 6. Performance Comparison

| Metric | tarpc (Binary) | JSON-RPC (Text) | Notes |
|--------|----------------|-----------------|-------|
| **Latency** | <1ms | ~5ms | Network round-trip |
| **Throughput** | 100K req/s | 20K req/s | Per connection |
| **Message Size** | 50-100 bytes | 200-500 bytes | For typical requests |
| **Parse Overhead** | Minimal | JSON parsing | serde is fast |
| **Debugging** | Harder | Easy (curl) | Trade-off |

---

## 7. Dependencies

```toml
[dependencies]
# Pure Rust RPC - no protobuf, no gRPC, no C++ tooling
tarpc = { version = "0.34", features = ["full"] }

# JSON-RPC 2.0 for external clients (hand-rolled implementation, no jsonrpsee)

# Serialization - native Rust serde (no protobuf)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## 8. Migration from gRPC

If you had a previous gRPC implementation:

| gRPC Component | Pure Rust Replacement |
|----------------|----------------------|
| `.proto` files | Rust traits with `#[tarpc::service]` |
| `tonic` | `tarpc` |
| `prost` | `serde` |
| `protoc` | Not needed (cargo build) |
| `build.rs` proto compile | Not needed |
| Generated code | Rust macros (type-safe) |

---

## 9. Alignment with Primal Ecosystem

### 9.1 Consistency with Songbird

LoamSpine follows the same RPC philosophy as Songbird (see `../../songBird/specs/TARPC_JSON_RPC_PROTOCOL_SPEC.md`):

```
✅ Pure Rust (no C/C++ dependencies)
✅ tarpc for primal-to-primal (binary, fast)
✅ JSON-RPC for external clients (universal)
✅ Native serde serialization
✅ No vendor lock-in
```

### 9.2 Inter-Primal Communication

```
┌─────────────┐     tarpc      ┌─────────────┐
│  RhizoCrypt │───────────────►│  LoamSpine  │
│             │  commit_session │             │
└─────────────┘                 └──────┬──────┘
                                       │ tarpc
                                       │ get_entry
                                       ▼
                               ┌─────────────┐
                               │  SweetGrass │
                               └─────────────┘
```

---

## 10. References

- [Songbird tarpc Specification](../../songBird/specs/TARPC_JSON_RPC_PROTOCOL_SPEC.md)
- [tarpc Documentation](https://docs.rs/tarpc/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification) — Hand-rolled implementation (no jsonrpsee)

---

*LoamSpine: Pure Rust, Primal Sovereignty, No Vendor Lock-in.*

