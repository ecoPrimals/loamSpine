<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# LoamSpine — Architecture Specification

**Version**: 1.1.0  
**Status**: Active  
**Last Updated**: December 24, 2025

> **Implementation Note**: The LoamSpine codebase uses capability-based discovery.
> External services are discovered at runtime via environment variables and the
> `CapabilityRegistry`. No primal names are hardcoded in the source code.

---

## 1. Overview

LoamSpine is the **permanent ledger** of the ecoPrimals ecosystem. It provides immutable, sovereign storage for committed state—the "fossil record" where ephemeral DAG operations compress into permanent history.

### 1.1 Position in the Stack

```
┌─────────────────────────────────────────────────────────────────┐
│                         Applications                             │
│        (Games, Scientific Tools, Collaboration Apps)            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        SweetGrass 🌾                             │
│                    (Attribution Layer)                           │
│              Queries Spine, builds provenance braids            │
└─────────────────────────────────────────────────────────────────┘
                              │
            ┌─────────────────┼─────────────────┐
            │                 │                 │
            ▼                 ▼                 ▼
┌───────────────────┐ ┌─────────────────┐ ┌─────────────────────┐
│   RhizoCrypt 🔐   │ │  LoamSpine 🦴   │ │     NestGate 🏠     │
│   (Ephemeral DAG) │ │ (Permanent Lin) │ │  (Payload Storage)  │
│                   │ │                 │ │                     │
│ Dehydrates to ────┼─┤ Fossil record   │ │ Large blob storage  │
│ LoamSpine         │ │ Certificates    │ │ Content-addressed   │
│                   │ │ Slice anchoring │ │                     │
└───────────────────┘ └────────┬────────┘ └─────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                        BearDog 🐻                                │
│                   (Identity & Security)                          │
│              DIDs, Signatures, Policy Enforcement               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Songbird 🐦                               │
│                   (Service Discovery)                            │
│              UPA Registration, Capability Routing               │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Core Responsibilities

| Responsibility | Description |
|----------------|-------------|
| **Entry Storage** | Append-only storage of signed, hash-linked entries |
| **Spine Management** | Lifecycle of owned linear chains |
| **Certificate Layer** | Memory-bound objects with ownership tracking |
| **Waypoint Anchoring** | Local permanence for borrowed slices |
| **Verification** | Chain integrity, inclusion proofs |
| **Replication** | Federation sync between spines |
| **Rollups** | Compression of entry ranges |

---

## 2. Component Architecture

### 2.1 High-Level Components

```
┌─────────────────────────────────────────────────────────────────┐
│                      LoamSpine Service                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │  Spine Manager  │  │  Entry Writer   │  │   Certificate   │  │
│  │                 │  │                 │  │     Manager     │  │
│  │ Create/Seal/    │  │ Append/Verify   │  │                 │  │
│  │ Archive         │  │ Sign/Attest     │  │ Mint/Transfer   │  │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘  │
│           │                    │                    │           │
│           └────────────────────┼────────────────────┘           │
│                                │                                │
│  ┌─────────────────────────────▼─────────────────────────────┐  │
│  │                      Spine Core                            │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐    │  │
│  │  │   Entry     │  │   Chain     │  │    Proof        │    │  │
│  │  │   Store     │  │   Index     │  │   Generator     │    │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘    │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                │                                │
│  ┌─────────────────────────────▼─────────────────────────────┐  │
│  │                     Storage Layer                          │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐    │  │
│  │  │   SQLite    │  │ PostgreSQL  │  │     RocksDB     │    │  │
│  │  │   Store     │  │    Store    │  │      Store      │    │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘    │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                │                                │
│  ┌─────────────────────────────▼─────────────────────────────┐  │
│  │                  Replication Engine                        │  │
│  │                                                            │  │
│  │  Sync with peers, federated verification, conflict detect │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
                                 │
          ┌──────────────────────┼──────────────────────┐
          │                      │                      │
          ▼                      ▼                      ▼
    ┌──────────┐           ┌──────────┐          ┌───────────┐
    │ BearDog  │           │RhizoCrypt│          │ SweetGrass│
    │   🐻     │           │   🔐     │          │    🌾     │
    │ Signing  │           │ Commits  │          │  Braids   │
    └──────────┘           └──────────┘          └───────────┘
```

### 2.2 Component Descriptions

#### Spine Manager
Manages the lifecycle of LoamSpine instances:
- **Create**: Initialize new spine with genesis entry
- **Configure**: Set replication, access, attestation policies
- **Seal**: Mark spine as read-only (no more entries)
- **Archive**: Move to cold storage
- **Transfer**: Change spine ownership

#### Entry Writer
Appends entries to spines:
- **Validate**: Check entry type, permissions
- **Sign**: Obtain BearDog signature
- **Link**: Compute hash chain
- **Append**: Store entry atomically
- **Index**: Update secondary indexes

#### Certificate Manager
Handles Loam Certificates (memory-bound objects):
- **Mint**: Create new certificate
- **Transfer**: Change certificate ownership
- **Loan**: Temporary transfer with return route
- **Return**: Process slice return from RhizoCrypt
- **Verify**: Check certificate authenticity and history

#### Spine Core
The heart of LoamSpine:
- **Entry Store**: Hash-addressed entry storage
- **Chain Index**: Previous/next links, height tracking
- **Proof Generator**: Inclusion and certificate proofs

#### Storage Layer
Pluggable backends:
- **SQLite**: Personal spines, portable
- **PostgreSQL**: Community spines, scalable
- **RocksDB**: High-performance local storage

#### Replication Engine
Federation support:
- **Sync**: Push/pull entries between peers
- **Verify**: Validate incoming entries
- **Detect**: Identify forks and conflicts
- **Resolve**: Apply conflict resolution policy

---

## 3. Data Flow

### 3.1 Entry Append Flow

```
     Application / RhizoCrypt
                │
                │ AppendEntry(spine, entry_type, payload)
                ▼
        ┌───────────────┐
        │  Entry Writer │
        └───────┬───────┘
                │
                │ 1. Validate entry type allowed
                │ 2. Check append permissions
                │ 3. Get current tip
                ▼
        ┌───────────────┐
        │   BearDog     │
        └───────┬───────┘
                │
                │ 4. Sign entry with committer key
                ▼
        ┌───────────────┐
        │  Spine Core   │
        └───────┬───────┘
                │
                │ 5. Compute entry hash
                │ 6. Link to previous entry
                │ 7. Assign index
                ▼
        ┌───────────────┐
        │ Storage Layer │
        └───────┬───────┘
                │
                │ 8. Persist entry
                │ 9. Update tip
                │ 10. Update indexes
                ▼
            EntryHash
```

### 3.2 Certificate Lifecycle Flow

```
  Publisher                              Retailer                               Customer
      │                                      │                                      │
      │ MintCertificate                      │                                      │
      ▼                                      │                                      │
 ┌─────────┐                                 │                                      │
 │Publisher│                                 │                                      │
 │  Spine  │                                 │                                      │
 └────┬────┘                                 │                                      │
      │                                      │                                      │
      │ TransferCertificate                  │                                      │
      ├─────────────────────────────────────►│                                      │
      │                                      ▼                                      │
      │                                 ┌─────────┐                                 │
      │                                 │Retailer │                                 │
      │                                 │  Spine  │                                 │
      │                                 └────┬────┘                                 │
      │                                      │                                      │
      │                                      │ TransferCertificate                  │
      │                                      ├─────────────────────────────────────►│
      │                                      │                                      ▼
      │                                      │                                 ┌─────────┐
      │                                      │                                 │Customer │
      │                                      │                                 │  Spine  │
      │                                      │                                 └────┬────┘
      │                                      │                                      │
      │                                      │                                      │ LoanCertificate
      │                                      │                                      │ (to friend)
      │                                      │                                      ▼
      │                                      │                                 ┌─────────┐
      │                                      │                                 │ Friend  │
      │                                      │                                 │Waypoint │
      │                                      │                                 └────┬────┘
      │                                      │                                      │
      │                                      │                                      │ [Uses item]
      │                                      │                                      │ [Loan expires]
      │                                      │                                      │
      │                                      │                                      │ ReturnCertificate
      │                                      │                                      ▼
      │                                      │                                 ┌─────────┐
      │                                      │                                 │Customer │
      │                                      │                                 │  Spine  │
      │                                      │                                 └─────────┘
```

### 3.3 Replication Flow

```
     Spine A (Source)                         Spine B (Replica)
           │                                        │
           │ New entry appended                     │
           ▼                                        │
    ┌─────────────┐                                 │
    │  Entry      │                                 │
    │  Appended   │                                 │
    └──────┬──────┘                                 │
           │                                        │
           │ Replication trigger                    │
           ▼                                        │
    ┌─────────────┐                                 │
    │ Replication │                                 │
    │   Engine    │                                 │
    └──────┬──────┘                                 │
           │                                        │
           │ 1. Get summary of Spine B              │
           ├───────────────────────────────────────►│
           │                                        ▼
           │                                 ┌─────────────┐
           │◄────────────────────────────────│   Summary   │
           │                                 │ (height, tip)│
           │                                 └─────────────┘
           │                                        │
           │ 2. Compute delta                       │
           │ 3. Push missing entries                │
           ├───────────────────────────────────────►│
           │                                        ▼
           │                                 ┌─────────────┐
           │                                 │  Validate   │
           │                                 │  & Append   │
           │                                 └──────┬──────┘
           │                                        │
           │◄───────────────────────────────────────┤
           │         Sync receipt                   │
           │                                        │
```

---

## 4. Crate Structure

```
loamSpine/
├── Cargo.toml                    # Workspace manifest
├── crates/
│   ├── loam-spine-core/          # Core library (pure Rust)
│   │   ├── src/
│   │   │   ├── lib.rs            # Main entry, re-exports
│   │   │   ├── primal.rs         # Primal lifecycle traits (inlined, no sourdough)
│   │   │   ├── config.rs         # Configuration types
│   │   │   ├── error.rs          # Error types
│   │   │   ├── types.rs          # Core types (Did, Hash, etc.)
│   │   │   ├── entry.rs          # Entry data structure
│   │   │   ├── spine.rs          # Spine management
│   │   │   ├── certificate.rs    # Certificate layer
│   │   │   ├── proof.rs          # Inclusion proofs
│   │   │   ├── storage.rs        # Storage traits + in-memory impl
│   │   │   ├── manager.rs        # Certificate manager
│   │   │   └── integration.rs    # Primal integration (RhizoCrypt, SweetGrass)
│   │   └── Cargo.toml
│   │
│   ├── loam-spine-api/           # Pure Rust RPC layer (NO gRPC/protobuf)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── service.rs        # LoamSpineRpcService
│   │   │   ├── types.rs          # Request/Response types
│   │   │   ├── tarpc_server.rs   # tarpc server (primal-to-primal)
│   │   │   └── jsonrpc_server.rs # JSON-RPC 2.0 server (external clients)
│   │   └── Cargo.toml
│   │
│   ├── loam-spine-store/         # Storage backends (future)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── traits.rs         # EntryStore, SpineStore traits
│   │   │   ├── sqlite.rs         # SQLite implementation
│   │   │   ├── redb.rs           # redb embedded (default)
│   │   │   └── sled.rs           # Sled embedded (optional, sled-storage feature)
│   │   └── Cargo.toml
│   │
│   └── loam-spine-service/       # Runnable service
│       ├── src/
│       │   └── main.rs           # Service entry point
│       └── Cargo.toml
│
├── specs/                        # Specifications
├── showcase/                     # Demo applications
└── tests/                        # Integration tests
```

### 4.1 Why No gRPC/Protobuf?

LoamSpine follows the **Primal Sovereignty** principle:

| ❌ What We Don't Use | ✅ What We Use |
|---------------------|----------------|
| gRPC | tarpc (pure Rust) |
| protobuf/proto files | serde (native Rust) |
| protoc (C++ compiler) | cargo build only |
| tonic | pure JSON-RPC 2.0 (hand-rolled) |
| Generated code | Rust macros (compile-time safe) |

See [PURE_RUST_RPC.md](./PURE_RUST_RPC.md) for the full rationale.

---

## 5. Spine Hierarchy

### 5.1 Sovereign Spine Model

```
                    ┌────────────────┐
                    │   gAIa/Global  │
                    │    Commons     │  ← Eternal, global
                    └───────┬────────┘
                            │
         ┌──────────────────┼──────────────────┐
         │                  │                  │
         ▼                  ▼                  ▼
   ┌──────────┐       ┌──────────┐       ┌──────────┐
   │Community │       │Community │       │Community │  ← Federated
   │ Spine A  │       │ Spine B  │       │ Spine C  │
   └────┬─────┘       └────┬─────┘       └────┬─────┘
        │                  │                  │
   ┌────┼────┐        ┌────┼────┐        ┌────┼────┐
   │    │    │        │    │    │        │    │    │
   ▼    ▼    ▼        ▼    ▼    ▼        ▼    ▼    ▼
┌────┐┌────┐┌────┐ ┌────┐┌────┐┌────┐ ┌────┐┌────┐┌────┐
│User││User││User│ │User││User││User│ │User││User││User│  ← Personal
│  1 ││  2 ││  3 │ │  4 ││  5 ││  6 │ │  7 ││  8 ││  9 │
└────┘└────┘└────┘ └────┘└────┘└────┘ └────┘└────┘└────┘
```

### 5.2 Spine Types

| Type | Ownership | Scope | Replication | Use Case |
|------|-----------|-------|-------------|----------|
| **Personal** | Individual | Private | Optional | Personal history |
| **Professional** | Individual | Public | Optional | Work portfolio |
| **Community** | Group | Federated | Required | Shared history |
| **Waypoint** | Individual | Private | Never | Borrowed state |
| **Public** | Individual | Global | Full | Verified claims |

---

## 6. Thread Model

### 6.1 Async Runtime

LoamSpine uses Tokio as its async runtime with pure Rust RPC:

```
┌───────────────────────────────────────────────────────────────┐
│                     Tokio Runtime                             │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐   │
│  │tarpc Server │  │  JSON-RPC   │  │  Background Tasks   │   │
│  │  (binary)   │  │ (pure JSON-RPC) │  │                     │   │
│  │             │  │             │  │  - Replication sync │   │
│  │Primal ↔ Prim│  │External API │  │  - Rollup sweep     │   │
│  └──────┬──────┘  └──────┬──────┘  │  - Archive move     │   │
│         │                │         │  - Metrics emit     │   │
│         └────────┬───────┘         └─────────────────────┘   │
│                  │                                            │
│                  ▼                                            │
│  ┌────────────────────────────────────────────────────────┐  │
│  │              Request Handler Pool                       │  │
│  │                                                         │  │
│  │   Each request handled on Tokio task                   │  │
│  │   DB operations use connection pool                    │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### 6.2 Concurrency Model

| Component | Concurrency Strategy |
|-----------|---------------------|
| Spine Manager | `RwLock<HashMap<SpineId, SpineHandle>>` |
| Entry Store | Connection pool (SQLite/PG), or sharded (RocksDB) |
| Chain Index | Per-spine locks, concurrent reads |
| Certificate Manager | Per-certificate locks |
| Replication | Single writer per spine, concurrent sync tasks |

---

## 7. Error Handling

### 7.1 Error Hierarchy

```rust
#[derive(Debug, thiserror::Error)]
pub enum LoamSpineError {
    // Spine errors
    #[error("Spine not found: {0}")]
    SpineNotFound(SpineId),
    
    #[error("Spine sealed: {0}")]
    SpineSealed(SpineId),
    
    #[error("Spine archived: {0}")]
    SpineArchived(SpineId),
    
    // Entry errors
    #[error("Entry not found: {0:?}")]
    EntryNotFound(EntryHash),
    
    #[error("Entry validation failed: {0}")]
    EntryValidation(String),
    
    #[error("Hash chain broken at index {0}")]
    ChainBroken(u64),
    
    // Certificate errors
    #[error("Certificate not found: {0}")]
    CertificateNotFound(CertificateId),
    
    #[error("Certificate not owned by {0}")]
    CertificateNotOwned(Did),
    
    #[error("Certificate is loaned")]
    CertificateLoaned,
    
    // Signature errors
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Missing required attestation")]
    MissingAttestation,
    
    // Storage errors
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    // Integration errors
    #[error("BearDog error: {0}")]
    BearDog(String),
    
    #[error("RhizoCrypt error: {0}")]
    RhizoCrypt(String),
    
    // Internal errors
    #[error("Internal error: {0}")]
    Internal(String),
}
```

---

## 8. Observability

### 8.1 Metrics

```rust
// Spine metrics
loamspine_spines_total: Gauge
loamspine_spines_by_state: Gauge { state = "active|sealed|archived" }
loamspine_spine_height: Gauge { spine_id }

// Entry metrics
loamspine_entries_appended_total: Counter
loamspine_entry_append_latency_seconds: Histogram
loamspine_entries_per_spine: Histogram

// Certificate metrics
loamspine_certificates_total: Counter
loamspine_certificate_transfers_total: Counter
loamspine_certificate_loans_active: Gauge

// Proof metrics
loamspine_proofs_generated_total: Counter
loamspine_proof_generation_latency_seconds: Histogram
loamspine_proofs_verified_total: Counter

// Replication metrics
loamspine_sync_entries_pushed_total: Counter
loamspine_sync_entries_pulled_total: Counter
loamspine_sync_latency_seconds: Histogram
loamspine_sync_conflicts_total: Counter

// Storage metrics
loamspine_storage_bytes_total: Gauge
loamspine_storage_read_latency_seconds: Histogram
loamspine_storage_write_latency_seconds: Histogram
```

### 8.2 Health Checks

```rust
impl PrimalHealth for LoamSpine {
    async fn check_health(&self) -> HealthReport {
        HealthReport::new("loamspine")
            .with_status(self.compute_status().await)
            .with_component("spine_manager", self.spine_manager.health())
            .with_component("entry_store", self.entry_store.health())
            .with_component("replication", self.replication.health())
            .with_metric("active_spines", self.active_spine_count())
            .with_metric("total_entries", self.total_entry_count())
    }
}
```

---

## 9. Security Model

### 9.1 Authentication

- All API requests require BearDog authentication
- Spine creation requires valid DID
- Entry signing is mandatory (not optional like RhizoCrypt)

### 9.2 Authorization

| Operation | Required Permission |
|-----------|---------------------|
| Create spine | `loamspine:spine:create` |
| Append entry | `loamspine:spine:{id}:write` |
| Read entry | `loamspine:spine:{id}:read` |
| Seal spine | `loamspine:spine:{id}:admin` |
| Mint certificate | `loamspine:certificate:mint` |
| Transfer certificate | Certificate owner only |

### 9.3 Data Protection

- All entries are signed and hash-linked
- Chain integrity verifiable by anyone
- Certificates have verifiable provenance
- Replication validates all incoming entries

---

## 10. Deployment Modes

### 10.1 Embedded Mode

LoamSpine runs in-process with the application:

```rust
let loam = LoamSpine::embedded()
    .with_store(SqliteStore::open("loam.db")?)
    .build()?;

let spine = loam.create_spine(owner_did, config).await?;
```

### 10.2 Service Mode

LoamSpine runs as a standalone service:

```bash
loam-spine-service \
    --config /etc/loamspine/config.toml \
    --grpc-addr 0.0.0.0:50052 \
    --rest-addr 0.0.0.0:8081
```

### 10.3 Federated Mode

Multiple LoamSpine instances replicate via tarpc:

```
┌─────────────────┐  tarpc   ┌─────────────────┐
│   LoamSpine     │─────────▶│   LoamSpine     │
│    Node A       │◀─────────│    Node B       │
└────────┬────────┘          └────────┬────────┘
         │                            │
         └─────────────┬──────────────┘
                       │
                       ▼
                ┌─────────────┐
                │  Songbird   │
                │  Discovery  │
                └─────────────┘
```

---

## 11. Design Principles

### 11.1 Primal Sovereignty

LoamSpine adheres to the **Primal Sovereignty** principle:

| Principle | Application |
|-----------|-------------|
| **Self-Sovereign** | No external tooling (protoc, C++ compilers) |
| **Pure Rust** | All dependencies are Rust-native |
| **No Vendor Lock-in** | No Google/corporate protocol dependencies |
| **Human Dignity** | Simple tools that humans can understand |
| **Cargo-Only Builds** | `cargo build` is all you need |

### 11.2 Pure Rust Stack

```
✅ tarpc     → Binary RPC (Rust-native)
✅ pure JSON-RPC 2.0 (hand-rolled, no jsonrpsee)
✅ serde     → Serialization (community standard)
✅ blake3    → Hashing (Rust-native)
✅ tokio     → Async runtime (Rust-native)
✅ ureq      → HTTP client (pure Rust, no TLS, no ring)

❌ gRPC      → Requires protoc (C++ compiler)
❌ protobuf  → Google-controlled protocol
❌ reqwest   → Replaced with ureq (pure Rust)
❌ ring      → Eliminated from dependency tree
❌ tonic     → gRPC implementation (C++ deps)
❌ prost     → Protobuf codegen (external tooling)
```

---

## 12. References

- [PURE_RUST_RPC.md](./PURE_RUST_RPC.md) — Pure Rust RPC philosophy
- [LOAMSPINE_SPECIFICATION.md](./LOAMSPINE_SPECIFICATION.md) — Full specification
- [DATA_MODEL.md](./DATA_MODEL.md) — Data structures
- [WAYPOINT_SEMANTICS.md](./WAYPOINT_SEMANTICS.md) — Waypoint spines
- [CERTIFICATE_LAYER.md](./CERTIFICATE_LAYER.md) — Memory-bound objects
- [API_SPECIFICATION.md](./API_SPECIFICATION.md) — Pure Rust API definitions

---

*LoamSpine: Pure Rust, Primal Sovereignty, Permanent History.*

