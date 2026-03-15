# LoamSpine

**Permanence Layer -- Selective Memory & Loam Certificates**

[![License](https://img.shields.io/badge/license-AGPL--3.0--only-blue)]()
[![Version](https://img.shields.io/badge/version-0.8.8-blue)]()
[![Tests](https://img.shields.io/badge/tests-1%2C123%20passing-brightgreen)]()
[![Coverage](https://img.shields.io/badge/coverage-89.64%25%20line%20%7C%2091.71%25%20region-brightgreen)]()
[![Unsafe](https://img.shields.io/badge/unsafe-ZERO%20production-red)]()
[![Edition](https://img.shields.io/badge/edition-2024-blue)]()
[![ecoBin](https://img.shields.io/badge/ecoBin-compliant-green)]()

---

## Overview

LoamSpine is the **immutable, permanent ledger** of the ecoPrimals ecosystem. Named after loam -- the slow, anaerobic soil layer where organic matter compresses into permanent geological record -- LoamSpine serves as the canonical source of truth for all events, discoveries, and artifacts that matter.

**Key Concepts:**
- **Selective Permanence** -- Only deliberately committed data becomes permanent
- **Sovereign Spines** -- Each user controls their own history
- **Loam Certificates** -- Digital ownership with lending and provenance
- **Infant Discovery** -- Born with zero external knowledge, discovers at runtime
- **Capability-Based** -- "Who can sign?" not "Where is BearDog?"
- **NeuralAPI Integration** -- Registers with biomeOS for ecosystem orchestration
- **Provenance Trio** -- Coordinates with rhizoCrypt (ephemeral) and sweetGrass (attribution)

---

## Quick Start

```bash
# Build and test
cargo build --release
cargo test --workspace

# Run the service (UniBin)
cargo run --release --bin loamspine -- server

# With explicit ports
cargo run --release --bin loamspine -- server --tarpc-port 9001 --jsonrpc-port 8080

# UniBin introspection
cargo run --release --bin loamspine -- capabilities
cargo run --release --bin loamspine -- socket

# Quality checks
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo llvm-cov --workspace --summary-only
cargo deny check licenses bans sources

# Full verification
./verify.sh
```

---

## Architecture

**Pure Rust** -- No gRPC, no protobuf, no C/C++ tooling, no OpenSSL. Zero C dependencies (ecoBin compliant).

**Storage backends:** redb (default, pure Rust), memory, sqlite (feature-gated). sled is optional via `--features sled-storage`.

```
loamSpine/
├── bin/loamspine-service/     # UniBin: server | capabilities | socket
├── crates/
│   ├── loam-spine-core/       # Core library (55+ source files)
│   │   └── src/
│   │       ├── backup/            # Backup/restore
│   │       ├── capabilities.rs    # Capability definitions
│   │       ├── certificate/       # Loam Certificates (types, lifecycle, metadata, provenance, escrow, usage)
│   │       ├── config.rs          # Configuration
│   │       ├── discovery/         # Capability registry + DynSigner/DynVerifier
│   │       ├── discovery_client/  # HTTP discovery client + ResilientDiscoveryClient
│   │       ├── entry/             # Entry types (15+ variants, bincode canonical)
│   │       ├── infant_discovery/  # DNS-SRV, mDNS, registry discovery
│   │       ├── manager/           # Certificate manager
│   │       ├── neural_api.rs      # NeuralAPI / biomeOS integration
│   │       ├── proof.rs           # Inclusion + ownership proofs (Merkle/Blake3)
│   │       ├── resilience.rs      # Circuit breaker + retry policy (lock-free)
│   │       ├── service/           # Modular service layer
│   │       │   ├── lifecycle.rs   # Startup/shutdown + ServiceState + NeuralAPI
│   │       │   ├── certificate.rs # Certificate CRUD + verify + escrow + sublend
│   │       │   ├── expiry_sweeper.rs # Background expired-loan auto-return
│   │       │   ├── integration.rs # Trait implementations
│   │       │   ├── signals.rs     # Signal handling
│   │       │   └── waypoint.rs    # Anchoring, operations, departure, proofs
│   │       ├── spine.rs           # Spine structure
│   │       ├── storage/           # Storage backends (redb default, memory, sled optional, sqlite)
│   │       ├── sync/              # Sync engine (push/pull, peer discovery)
│   │       ├── temporal/          # Time tracking (moments, anchors)
│   │       ├── traits/            # Integration traits
│   │       ├── transport/         # IPC transports (HTTP, NeuralAPI, mock)
│   │       ├── waypoint.rs        # Waypoint types (config, attestation, relending chain)
│   │       └── trio_types.rs      # Provenance trio type bridging
│   └── loam-spine-api/        # RPC layer (14 source files)
│       └── src/
│           ├── jsonrpc/       # JSON-RPC 2.0 (semantic naming)
│           ├── tarpc_server.rs # Binary RPC (primal-to-primal)
│           ├── service/       # Domain-focused RPC ops
│           ├── health.rs      # Health checks
│           └── error.rs       # API errors
├── specs/                     # 11 specification documents
├── showcase/                  # Interactive demos (83 files)
└── fuzz/                      # Fuzz testing targets
```

**Dual Protocol:**
- **tarpc** -- High-performance binary RPC for primal-to-primal
- **JSON-RPC 2.0** -- Universal, language-agnostic for external clients and NeuralAPI (batch support)

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
| **Certificate** | `certificate.verify` | Verify integrity |
| **Certificate** | `certificate.lifecycle` | Ownership/loan history |
| **Waypoint** | `slice.anchor` | Anchor borrowed state |
| **Waypoint** | `slice.record_operation` | Record waypoint operation |
| **Waypoint** | `slice.depart` | Depart from waypoint |
| **Proof** | `proof.generate_inclusion` | Create proof |
| **Integration** | `session.commit` | rhizoCrypt commits |
| **Integration** | `commit.session` | Semantic alias (biomeOS routing) |
| **Integration** | `braid.commit` | sweetGrass commits |
| **Compat** | `permanent-storage.commitSession` | rhizoCrypt wire format |
| **Compat** | `permanent-storage.verifyCommit` | Verify via rhizoCrypt format |
| **Compat** | `permanent-storage.getCommit` | Retrieve via rhizoCrypt format |
| **Compat** | `permanent-storage.healthCheck` | Health for rhizoCrypt clients |
| **Health** | `health.check` | Service status |
| **Meta** | `capability.list` | List primal capabilities |

---

## Discovery

LoamSpine discovers services at runtime via **infant discovery** (zero knowledge at startup):

1. **NeuralAPI** -- biomeOS Unix socket IPC (preferred, capability-registered)
2. **Environment Variables** (`CAPABILITY_*_ENDPOINT`, `*_SERVICE_URL`)
3. **Service Registry** -- HTTP-based (Songbird, Consul adapter, etcd adapter)
4. **DNS SRV** -- RFC 2782 (`_signing._tcp.local`)
5. **mDNS** -- RFC 6762 (experimental, feature-gated)
6. **Development Fallback** (`localhost`, debug builds only)

---

## Quality

| Metric | Value |
|--------|-------|
| **Version** | 0.8.8 |
| **Edition** | 2024 |
| **Tests** | 1,123 passing |
| **Coverage** | 89.64% line, 91.71% region (llvm-cov) |
| **Clippy** | 0 warnings (pedantic + nursery, `-D warnings`) |
| **Unsafe Code** | 0 in production (`#![deny(unsafe_code)]`) |
| **Lint Exceptions** | 0 `#[allow]` in production (all `#[expect(reason)]`) |
| **Max File Size** | 955 lines (all < 1000) |
| **Source Files** | 112 `.rs` files across 2 crates + binary |
| **License** | AGPL-3.0-only |
| **SPDX Headers** | All source files |
| **ecoBin** | Zero C dependencies (pure Rust) |
| **cargo deny** | bans, licenses, sources pass |
| **UniBin** | `server`, `capabilities`, `socket` subcommands |
| **Mock isolation** | All mocks cfg-gated out of production |

---

## DevOps

```bash
# Docker
docker build -t loamspine .

# Verify everything
./verify.sh
```

---

## Specifications

Complete specifications in [specs/](./specs/):
- Core specification, architecture, data model
- Certificate layer, waypoint semantics
- API specification, service lifecycle
- Integration specification (provenance trio)

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

AGPL-3.0-only. See [LICENSE](./LICENSE).

---

**LoamSpine: Where memories become permanent.**
