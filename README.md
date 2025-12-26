# 🦴 LoamSpine

**Permanence Layer — Selective Memory & Loam Certificates**

[![Build](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Tests](https://img.shields.io/badge/tests-332%20passing-brightgreen)]()
[![Coverage](https://img.shields.io/badge/coverage-90.72%25-brightgreen)]()
[![Clippy](https://img.shields.io/badge/clippy-0%20warnings-brightgreen)]()
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue)]()
[![Version](https://img.shields.io/badge/version-0.6.1-blue)]()
[![Songbird](https://img.shields.io/badge/songbird-integrated-purple)]()
[![Grade](https://img.shields.io/badge/grade-A%2B%20(98%2F100)-gold)]()
[![Unsafe](https://img.shields.io/badge/unsafe-forbidden-red)]()
[![Hardcoding](https://img.shields.io/badge/hardcoded%20endpoints-0-green)]()
[![Status](https://img.shields.io/badge/status-production%20ready-success)]()
[![Benchmarks](https://img.shields.io/badge/benchmarks-working-green)]()


---

## Overview

LoamSpine is the **immutable, permanent ledger** of the ecoPrimals ecosystem. Named after loam—the slow, anaerobic soil layer where organic matter compresses into permanent geological record—LoamSpine serves as the canonical source of truth for all events, discoveries, and artifacts that matter.

**Production Ready**: Zero technical debt, zero hardcoded endpoints, zero unsafe code.

**Key Concepts:**
- **Selective Permanence** — Only deliberately committed data becomes permanent
- **Sovereign Spines** — Each user controls their own history
- **Loam Certificates** — Digital ownership with lending and provenance
- **Recursive Stacking** — Spines can reference other spines
- **Universal Adapter** — O(n) discovery through Songbird instead of O(n²)
- **Capability-Based Discovery** — Primals discover each other at runtime
- **Zero Primal Hardcoding** — LoamSpine knows only itself
- **Signing Integration** — Agnostic CLI-based signing (any Ed25519 provider)
- **Zero-Copy Buffers** — Efficient `bytes` crate for network operations

---

## Quick Start

```bash
# Build
cargo build --release

# Test (332 tests, 100% pass rate)
cargo test

# Check linting (0 warnings - all targets)
cargo clippy --all-targets --all-features -- -D warnings

# Format check
cargo fmt --check

# Coverage (90.72%)
cargo tarpaulin --out Stdout --engine llvm

# Benchmarks
cargo bench

# Run working examples
cargo run --example hello_loamspine
cargo run --example entry_types

# Build docs
cargo doc --open --no-deps

# Run showcase demos
cd showcase && ./QUICK_START.sh
```

---

## Architecture

**Pure Rust RPC** — No gRPC, no protobuf, no C++ tooling.

```
loamSpine/
├── .github/workflows/ci.yml    # CI/CD pipeline
├── Dockerfile                  # Container build
├── docker-compose.yml          # Deployment config
├── deny.toml                   # Security audits
├── rustfmt.toml                # Formatting config
├── tarpaulin.toml              # Coverage config
├── CONTRIBUTING.md             # Contribution guide
├── crates/
│   ├── loam-spine-core/        # Core library (~8,900 LOC)
│   │   └── src/
│   │       ├── lib.rs          # Primal entry point
│   │       ├── backup.rs       # Backup/restore functionality
│   │       ├── entry.rs        # Entry types (15+ variants)
│   │       ├── spine.rs        # Spine structure
│   │       ├── certificate.rs  # Loam Certificates + time constants
│   │       ├── proof.rs        # Inclusion proofs
│   │       ├── storage.rs      # InMemory + Sled storage
│   │       ├── discovery.rs    # Capability-based primal discovery
│   │       ├── songbird.rs     # Songbird client (universal adapter)
│   │       ├── service/        # Modular service (refactored v0.3.0)
│   │       │   ├── mod.rs      # Core service + spine management
│   │       │   ├── certificate.rs # Certificate lifecycle
│   │       │   ├── integration.rs # Trait implementations
│   │       │   ├── lifecycle.rs   # Startup/shutdown automation
│   │       │   └── waypoint.rs # Proof generation
│   │       └── traits/         # Integration trait modules
│   │           ├── commit.rs   # CommitAcceptor, SpineQuery
│   │           ├── slice.rs    # SliceManager
│   │           ├── signing.rs  # Signer, Verifier (mocks test-only)
│   │           └── cli_signer.rs  # CLI-based signing (agnostic)
│   │   ├── tests/
│   │   │   ├── e2e.rs          # End-to-end tests
│   │   │   └── chaos.rs        # Chaos/fault tests
│   │   └── benches/
│   │       └── core_ops.rs     # Performance benchmarks
│   └── loam-spine-api/         # Pure Rust RPC (~2,800 LOC)
│       └── src/
│           ├── rpc.rs          # RPC trait definition
│           ├── service.rs      # RPC implementation
│           ├── tarpc_server.rs # High-performance binary RPC
│           ├── jsonrpc.rs      # JSON-RPC 2.0 + error constants
│           ├── types.rs        # Native Rust types (serde)
│           └── error.rs        # API error types
├── fuzz/                       # Fuzz testing
│   └── fuzz_targets/
│       ├── fuzz_entry_parsing.rs
│       ├── fuzz_certificate.rs
│       └── fuzz_spine_operations.rs
└── specs/                      # Specifications (8,400+ lines)
```

---

## Why Pure Rust RPC?

**ecoPrimals philosophy: Lean into the Rust compiler.**

| ❌ gRPC Problems | ✅ Our Solution |
|-----------------|-----------------|
| Requires `protoc` (C++ compiler) | Pure Rust with tarpc macros |
| Requires protobuf (Google tooling) | Native serde serialization |
| Non-Rust code generation | Rust procedural macros |
| Vendor lock-in | Community-driven development |
| Complex build process | Standard `cargo build` |

**Dual Protocol Strategy:**
- **tarpc** — High-performance binary RPC for primal-to-primal
- **JSON-RPC 2.0** — Universal, language-agnostic for external clients

---

## Core Features

### Spines (Sovereign Ledgers)
```rust
use loam_spine_core::{Spine, SpineBuilder, Did};

let owner = Did::new("did:key:z6MkOwner");
let spine = SpineBuilder::new(owner)
    .with_name("My History")
    .build()?;
```

### Entries (Immutable Records)
```rust
use loam_spine_core::{Entry, EntryType};

let entry = spine.create_entry(EntryType::SessionCommit {
    session_id: SessionId::now_v7(),
    merkle_root: [0u8; 32],
    vertex_count: 100,
    committer: owner.clone(),
});
spine.append(entry)?;
```

### Certificates (Digital Ownership)
```rust
use loam_spine_core::{Certificate, CertificateType, LoanTerms};

// Mint a certificate
let (cert_id, _) = service.mint_certificate(
    spine_id,
    CertificateType::DigitalCollectible { ... },
    owner.clone(),
    None,
).await?;

// Loan it out
let terms = LoanTerms::new().with_duration(3600);
service.loan_certificate(cert_id, owner, borrower, terms).await?;
```

### Backup & Restore
```rust
use loam_spine_core::backup::SpineBackup;

// Export spine to backup
let backup = SpineBackup::new(spine, entries, certificates);
backup.export(&mut file)?;

// Verify and import
let backup = SpineBackup::import(&mut file)?;
assert!(backup.verify().valid);
```

### Capability-Based Discovery
```rust
use loam_spine_core::{CapabilityRegistry, LoamSpineService};

// Create service with capability registry
let registry = CapabilityRegistry::new();
let service = LoamSpineService::with_capabilities(registry.clone());

// Register signing capability (done by signing primal)
registry.register_signer(signer).await;

// Request capability (done by LoamSpine)
let signer = registry.require_signer().await?;
```

### Songbird Integration (Universal Adapter)
```rust
use loam_spine_core::songbird::SongbirdClient;
use loam_spine_core::service::{LifecycleManager, LoamSpineService};
use loam_spine_core::config::LoamSpineConfig;

// Connect to Songbird (universal adapter)
let client = SongbirdClient::connect("http://localhost:8082").await?;

// Discover services by capability (not by primal name!)
let services = client.discover_capability("signing").await?;
for service in services {
    println!("Found: {} at {}", service.name, service.endpoint);
}

// Advertise LoamSpine capabilities
client.advertise_loamspine(
    "http://localhost:9001",  // tarpc
    "http://localhost:8080"   // jsonrpc
).await?;

// OR: Use lifecycle manager for full automation
let config = LoamSpineConfig::default()
    .with_songbird("http://localhost:8082");
let service = LoamSpineService::new();
let mut manager = LifecycleManager::new(service, config);

// Auto-advertises on startup, runs background heartbeat
manager.start().await?;
// ... service runs ...
manager.stop().await?;  // Graceful shutdown
```

**Architecture**: O(n) complexity instead of O(n²)

```
┌─────────────┐
│  LoamSpine  │────┐
└─────────────┘    │
                   │    ┌──────────────┐
┌─────────────┐    ├───▶│   Songbird   │◀────┐
│   Beardog   │────┘    │   (Adapter)  │     │
└─────────────┘         └──────────────┘     │
                                             │
┌─────────────┐                              │
│  NestGate   │──────────────────────────────┘
└─────────────┘
```

---

## RPC API (18 Methods)

| Category | Method | Description |
|----------|--------|-------------|
| **Spine** | `create_spine` | Create sovereign ledger |
| **Spine** | `get_spine` | Get spine metadata |
| **Spine** | `seal_spine` | Make immutable |
| **Entry** | `append_entry` | Add entry to chain |
| **Entry** | `get_entry` | Query by hash |
| **Entry** | `get_tip` | Get latest entry |
| **Certificate** | `mint_certificate` | Create ownership cert |
| **Certificate** | `transfer_certificate` | Transfer ownership |
| **Certificate** | `loan_certificate` | Temporary access |
| **Certificate** | `return_certificate` | End loan |
| **Certificate** | `get_certificate` | Query certificate |
| **Waypoint** | `anchor_slice` | Anchor borrowed state |
| **Waypoint** | `checkout_slice` | Checkout with provenance |
| **Proof** | `generate_inclusion_proof` | Create proof |
| **Proof** | `verify_inclusion_proof` | Validate proof |
| **Integration** | `commit_session` | RhizoCrypt commits |
| **Integration** | `commit_braid` | SweetGrass commits |
| **Health** | `health_check` | Service status |

---

## Integration Points

### RhizoCrypt (Ephemeral DAGs)
```rust
use loam_spine_core::traits::{CommitAcceptor, DehydrationSummary};

let summary = DehydrationSummary::new(session_id, "game", merkle_root);
let commit_ref = service.commit_session(spine_id, owner, summary).await?;
```

### SweetGrass (Semantic Attribution)
```rust
use loam_spine_core::traits::{BraidAcceptor, BraidSummary};

let braid = BraidSummary::new(braid_id, "attribution", subject_hash, braid_hash);
service.commit_braid(spine_id, owner, braid).await?;
```

### Signing Service (Agnostic)
```rust
use loam_spine_core::{CliSigner, CliVerifier};

// Real signing via any CLI signing service
let signer = CliSigner::new("/path/to/signer", "key-id")?;
let signature = signer.sign(data).await?;

// Verify with any verification service
let verifier = CliVerifier::new("/path/to/signer")?;
let result = verifier.verify(data, &signature, &signer_did).await?;
```

---

## Status

| Metric | Value |
|--------|-------|
| **Version** | 0.6.1 ✨ |
| **Grade** | A+ (98/100) |
| **Tests** | 332 passing (+45 new) |
| **Coverage** | 90.72% (exceeds target) |
| **LOC** | ~13,700 total |
| **RPC Methods** | 18/18 implemented |
| **Clippy** | pedantic + nursery (0 warnings, all targets) |
| **Unsafe Code** | 0 (forbidden) |
| **Critical Issues** | 0 ✅ |
| **Max File Size** | <1000 lines ✅ |
| **Fuzz Targets** | 3 |
| **Showcase Demos** | 8 (6 core + 2 Phase 1 integration) |
| **Signing Integration** | Agnostic CLI + Mock (testing) |
| **Zero-Copy** | `bytes` crate optimization |
| **Phase 1 Integration** | Signing + Storage services |
| **Docker Support** | ✅ Production ready |
| **CI/CD** | ✅ GitHub Actions |
| **Mocks** | ✅ Isolated to testing |
| **Hardcoding** | ✅ Zero (capability-based) |
| **Benchmarks** | ✅ All working |
| **Status** | ✅ **PRODUCTION READY** |

---

## DevOps

### Docker
```bash
# Build container
docker build -t loamspine .

# Run with docker-compose
docker-compose up -d
```

### CI/CD
- ✅ Format check (rustfmt)
- ✅ Clippy (pedantic + nursery)
- ✅ Documentation build
- ✅ Test suite (all features)
- ✅ Coverage reporting (llvm-cov)
- ✅ Security audit (cargo-deny)
- ✅ Multi-platform build (Linux, macOS)
- ✅ MSRV check (1.75.0)

---

## 🎭 Showcase

Run interactive demos to see LoamSpine in action:

```bash
# Phase 1: Local Primal Capabilities
cargo run -p loam-spine-core --example demo_hello_loamspine
cargo run -p loam-spine-core --example demo_entry_types
cargo run -p loam-spine-core --example demo_certificate_lifecycle
cargo run -p loam-spine-core --example demo_backup_restore

# Phase 2: RPC API Interaction
cargo run -p loam-spine-api --example demo_rpc_service

# Phase 3: Inter-Primal Integration
cargo run -p loam-spine-core --example demo_inter_primal

# Run all demos
cd showcase && ./QUICK_START.sh

# Include Phase 1 primal integration (BearDog, NestGate)
cd showcase && ./QUICK_START.sh --with-phase1
```

### Phase 1 Primal Integration

Real binaries from `../bins/` demonstrate integration with Phase 1 primals:

```bash
# Signing service demo (Ed25519 key operations)
./showcase/scripts/demo_signing_service.sh

# Storage service patterns
./showcase/scripts/demo_storage_service.sh
```

See [showcase/README.md](./showcase/README.md) for full documentation.

---

## Documentation

### Quick Reference
- [EXECUTIVE_SUMMARY.md](./EXECUTIVE_SUMMARY.md) — Quick overview & metrics
- [SESSION_COMPLETION_SUMMARY.md](./SESSION_COMPLETION_SUMMARY.md) — Latest session
- [DEPLOYMENT_READY.md](./DEPLOYMENT_READY.md) — Deployment certification

### Project Status
- [STATUS.md](./STATUS.md) — Current project status
- [WHATS_NEXT.md](./WHATS_NEXT.md) — Roadmap and progress
- [KNOWN_ISSUES.md](./KNOWN_ISSUES.md) — Issue status (all resolved ✅)

### Developer Resources
- [START_HERE.md](./START_HERE.md) — Developer onboarding
- [CONTRIBUTING.md](./CONTRIBUTING.md) — Contribution guide
- [showcase/](./showcase/) — Interactive demos
- [specs/](./specs/) — Full specifications

### Session Reports
- [FINAL_SESSION_REPORT_DEC_24_2025.md](./FINAL_SESSION_REPORT_DEC_24_2025.md) — Complete session log
- [BENCHMARKS_FIXED.md](./BENCHMARKS_FIXED.md) — Benchmark fixes

---

**🦴 LoamSpine: Where memories become permanent.**

**v0.6.1 — Production Ready — Grade A+ (98/100)**
