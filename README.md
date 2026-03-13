# LoamSpine

**Permanence Layer -- Selective Memory & Loam Certificates**

[![License](https://img.shields.io/badge/license-AGPL--3.0--only-blue)]()
[![Version](https://img.shields.io/badge/version-0.8.0-blue)]()
[![Tests](https://img.shields.io/badge/tests-549%20passing-brightgreen)]()
[![Coverage](https://img.shields.io/badge/line%20coverage-90.08%25-brightgreen)]()
[![Unsafe](https://img.shields.io/badge/unsafe-ZERO-red)]()

---

## Overview

LoamSpine is the **immutable, permanent ledger** of the ecoPrimals ecosystem. Named after loam -- the slow, anaerobic soil layer where organic matter compresses into permanent geological record -- LoamSpine serves as the canonical source of truth for all events, discoveries, and artifacts that matter.

**Key Concepts:**
- **Selective Permanence** -- Only deliberately committed data becomes permanent
- **Sovereign Spines** -- Each user controls their own history
- **Loam Certificates** -- Digital ownership with lending and provenance
- **Infant Discovery** -- Born with zero external knowledge, discovers at runtime
- **Capability-Based** -- "Who can sign?" not "Where is BearDog?"
- **Vendor Agnostic** -- Works with Songbird, Consul, etcd, or any RFC 2782 system

---

## Quick Start

```bash
# Build and test
cargo build --release
cargo test --workspace

# Run the service (UniBin subcommand)
cargo run --release --bin loamspine -- server

# With explicit ports
cargo run --release --bin loamspine -- server --tarpc-port 9001 --jsonrpc-port 8080

# Configuration (all optional -- sensible defaults)
export LOAMSPINE_TARPC_PORT=9001
export LOAMSPINE_JSONRPC_PORT=8080

# Capability discovery (auto-discovered if not set)
export CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT="http://localhost:8001"

# Quality checks
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo llvm-cov --workspace --summary-only
cargo deny check licenses bans sources

# Build docs
cargo doc --open --no-deps
```

---

## Architecture

**Pure Rust** -- No gRPC, no protobuf, no C/C++ tooling, no OpenSSL.

```
loamSpine/
├── bin/loamspine-service/     # UniBin: `loamspine server`
├── crates/
│   ├── loam-spine-core/       # Core library
│   │   └── src/
│   │       ├── backup.rs          # Backup/restore
│   │       ├── capabilities.rs    # Capability definitions
│   │       ├── certificate.rs     # Loam Certificates
│   │       ├── config.rs          # Configuration
│   │       ├── discovery.rs       # Capability registry
│   │       ├── discovery_client.rs # HTTP discovery client
│   │       ├── entry.rs           # Entry types (15+ variants)
│   │       ├── infant_discovery.rs # DNS-SRV, mDNS, registry discovery
│   │       ├── manager.rs         # Spine manager
│   │       ├── proof.rs           # Inclusion proofs
│   │       ├── service/           # Modular service
│   │       │   ├── lifecycle.rs   # Startup/shutdown with heartbeat
│   │       │   ├── infant_discovery.rs # Discovery orchestration
│   │       │   ├── certificate.rs # Certificate lifecycle
│   │       │   ├── integration.rs # Trait implementations
│   │       │   ├── signals.rs     # Signal handling
│   │       │   └── waypoint.rs    # Proof generation
│   │       ├── spine.rs           # Spine structure
│   │       ├── storage/           # Storage backends
│   │       │   ├── memory.rs      # In-memory
│   │       │   └── sled.rs        # Persistent (pure Rust)
│   │       ├── temporal/          # Time tracking
│   │       └── traits/            # Integration traits
│   │           ├── cli_signer.rs  # CLI-based signing
│   │           ├── signing.rs     # Signer, Verifier
│   │           └── commit.rs      # CommitAcceptor, SpineQuery
│   └── loam-spine-api/        # RPC layer
│       └── src/
│           ├── jsonrpc.rs     # JSON-RPC 2.0 (semantic naming)
│           ├── tarpc_server.rs # Binary RPC (primal-to-primal)
│           ├── service/       # Domain-focused RPC ops
│           ├── health.rs      # Health checks
│           └── error.rs       # API errors
├── specs/                     # Specifications
├── showcase/                  # Interactive demos
└── fuzz/                      # Fuzz testing
```

**Dual Protocol:**
- **tarpc** -- High-performance binary RPC for primal-to-primal
- **JSON-RPC 2.0** -- Universal, language-agnostic for external clients

---

## RPC API (Semantic Naming)

| Category | Method | Description |
|----------|--------|-------------|
| **Spine** | `spine.create` | Create sovereign ledger |
| **Spine** | `spine.get` | Get spine metadata |
| **Spine** | `spine.seal` | Make immutable |
| **Entry** | `entry.append` | Add entry to chain |
| **Entry** | `entry.get` | Query by hash |
| **Entry** | `entry.get_tip` | Get latest entry |
| **Certificate** | `certificate.mint` | Create ownership cert |
| **Certificate** | `certificate.transfer` | Transfer ownership |
| **Certificate** | `certificate.loan` | Temporary access |
| **Certificate** | `certificate.return` | End loan |
| **Certificate** | `certificate.get` | Query certificate |
| **Waypoint** | `slice.anchor` | Anchor borrowed state |
| **Proof** | `proof.generate_inclusion` | Create proof |
| **Integration** | `session.commit` | rhizoCrypt commits |
| **Integration** | `braid.commit` | sweetGrass commits |
| **Compat** | `permanent-storage.commitSession` | rhizoCrypt wire format |
| **Compat** | `permanent-storage.verifyCommit` | Verify via rhizoCrypt format |
| **Compat** | `permanent-storage.getCommit` | Retrieve via rhizoCrypt format |
| **Compat** | `permanent-storage.healthCheck` | Health for rhizoCrypt clients |
| **Health** | `health.check` | Service status |

---

## Discovery

LoamSpine discovers services at runtime via **infant discovery** (zero knowledge at startup):

1. **Environment Variables** (`CAPABILITY_*_ENDPOINT`, `*_SERVICE_URL`)
2. **Service Registry** -- HTTP-based (Songbird, Consul adapter, etcd adapter)
3. **DNS SRV** -- RFC 2782 (`_signing._tcp.local`)
4. **mDNS** -- RFC 6762 (experimental, feature-gated)
5. **Development Fallback** (`localhost`, debug builds only)

---

## Quality

| Metric | Value |
|--------|-------|
| **Version** | 0.8.0 |
| **Tests** | 549 passing |
| **Line Coverage** | 90.08% |
| **Clippy** | 0 warnings (all targets) |
| **Unsafe Code** | 0 (`#![forbid(unsafe_code)]`) |
| **Max File Size** | 863 lines |
| **License** | AGPL-3.0-only |
| **SPDX Headers** | All source files |
| **Pure Rust TLS** | rustls (no OpenSSL/native-tls) |
| **cargo deny** | bans, licenses, sources pass |
| **UniBin** | `loamspine server` subcommand |

---

## DevOps

```bash
# Docker
docker build -t loamspine .
docker-compose up -d

# Verify everything
./verify.sh
```

---

## Specifications

Complete specifications in [specs/](./specs/):
- Core specification, architecture, data model
- Certificate layer, waypoint semantics
- API specification, service lifecycle

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

AGPL-3.0-only. See [LICENSE](./LICENSE).

---

**LoamSpine: Where memories become permanent.**
