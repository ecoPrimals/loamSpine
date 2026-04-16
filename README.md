<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# LoamSpine

**Permanence Layer -- Selective Memory & Loam Certificates**

[![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue)]()
[![Version](https://img.shields.io/badge/version-0.9.16-blue)]()
[![Tests](https://img.shields.io/badge/tests-1%2C442%20passing-brightgreen)]()
[![Coverage](https://img.shields.io/badge/coverage-90.9%25%20line-brightgreen)]()
[![Methods](https://img.shields.io/badge/JSON--RPC-37%20methods-blue)]()
[![Zero Copy](https://img.shields.io/badge/zero--copy-Arc%3Cstr%3E%20%7C%20Cow%20%7C%20OnceLock-green)]()
[![Unsafe](https://img.shields.io/badge/unsafe-ZERO%20(forbid)-red)]()
[![Edition](https://img.shields.io/badge/edition-2024-blue)]()
[![ecoBin](https://img.shields.io/badge/ecoBin-compliant-green)]()
[![scyBorg](https://img.shields.io/badge/scyBorg-triple%20license-blue)]()

---

## Overview

LoamSpine is the **immutable, permanent ledger** of the ecoPrimals ecosystem. Named after loam -- the slow, anaerobic soil layer where organic matter compresses into permanent geological record -- LoamSpine serves as the canonical source of truth for all events, discoveries, and artifacts that matter.

**Key Concepts:**
- **Selective Permanence** -- Only deliberately committed data becomes permanent
- **Sovereign Spines** -- Each user controls their own history
- **Loam Certificates** -- Digital ownership with lending and provenance
- **Infant Discovery** -- Born with zero external knowledge, discovers at runtime
- **Capability-Based** -- "Who can sign?" not "Where is a specific primal?"
- **NeuralAPI Integration** -- Registers with biomeOS for ecosystem orchestration
- **Provenance Trio** -- Coordinates with the ephemeral DAG and attribution capability primals

---

## Quick Start

```bash
# Build and test
cargo build --release
cargo test --workspace

# Run the service — UDS only (default, no port conflicts)
cargo run --release --bin loamspine -- server

# With explicit TCP ports (opt-in, --port aliases --jsonrpc-port per UniBin)
cargo run --release --bin loamspine -- server --port 8080 --tarpc-port 9001

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

**Pure Rust** -- No gRPC, no protobuf, no C/C++ tooling, no OpenSSL. Zero C dependencies (ecoBin compliant). Blake3 uses pure Rust mode (no C/asm). Builds as **musl-static** for portable container deployment via plasmidBin / benchScale.

**Storage backends:** redb (default, pure Rust) and memory.

```
loamSpine/
├── bin/loamspine-service/     # UniBin: server | capabilities | socket
├── crates/
│   ├── loam-spine-core/       # Core library
│   │   └── src/
│   │       ├── backup/            # Backup/restore
│   │       ├── btsp/              # BTSP Phase 2 handshake (wire, config, frame, provider_client, handshake)
│   │       ├── capabilities/       # Capability definitions (identifiers, types, parser)
│   │       ├── certificate/       # Loam Certificates (types, lifecycle, metadata, provenance, escrow, usage)
│   │       ├── config.rs          # Configuration
│   │       ├── discovery/         # Capability registry + DynSigner/DynVerifier
│   │       ├── discovery_client/  # HTTP discovery client + ResilientDiscoveryClient
│   │       ├── entry/             # Entry types (15+ variants, bincode canonical)
│   │       ├── infant_discovery/  # DNS-SRV, mDNS, registry discovery
│   │       ├── manager/           # Certificate manager
│   │       ├── niche.rs            # Primal self-knowledge (capabilities, deps, costs)
│   │       ├── primal_names.rs    # Centralized primal identifier constants
│   │       ├── neural_api/         # NeuralAPI / biomeOS integration (socket, MCP, identity)
│   │       ├── proof.rs           # Inclusion + ownership proofs (Merkle/Blake3)
│   │       ├── resilience.rs      # Circuit breaker + retry policy (lock-free)
│   │       ├── service/           # Modular service layer
│   │       │   ├── lifecycle.rs   # Startup/shutdown + ServiceState + NeuralAPI
│   │       │   ├── certificate.rs # Certificate core (mint, transfer, verify, proofs)
│   │       │   ├── certificate_loan.rs  # Loan lifecycle (loan, return, sublend)
│   │       │   ├── certificate_escrow.rs # Escrow (hold, release, cancel)
│   │       │   ├── expiry_sweeper.rs # Background expired-loan auto-return
│   │       │   ├── anchor.rs       # Public chain anchor (record + verify receipts)
│   │       │   ├── integration.rs # Trait implementations
│   │       │   ├── signals.rs     # Signal handling
│   │       │   └── waypoint.rs    # Anchoring, operations, departure, attestation, proofs
│   │       ├── spine.rs           # Spine structure
│   │       ├── storage/           # Storage backends (redb default, memory)
│   │       ├── sync/              # Sync engine (push/pull, peer discovery)
│   │       ├── temporal/          # Time tracking (moments, anchors)
│   │       ├── traits/            # Integration traits
│   │       ├── transport/         # IPC transports (HTTP, NeuralAPI, mock)
│   │       ├── waypoint.rs        # Waypoint types (config, attestation, relending chain)
│   │       └── trio_types.rs      # Provenance trio type bridging
│   └── loam-spine-api/        # RPC layer
│       └── src/
│           ├── jsonrpc/       # JSON-RPC 2.0 (semantic naming)
│           ├── tarpc_server.rs # Structured RPC (JSON-over-TCP, primal-to-primal)
│           ├── service/       # Domain-focused RPC ops
│           ├── health.rs      # Health checks
│           └── error.rs       # API errors
├── specs/                     # 13 specification documents
├── showcase/                  # Interactive demos (54 files)
└── fuzz/                      # Fuzz testing targets
```

**Dual Protocol:**
- **tarpc** -- High-performance structured RPC (JSON-over-TCP) for primal-to-primal
- **JSON-RPC 2.0** -- Universal, language-agnostic for external clients and NeuralAPI (batch support, HTTP/1.1 keep-alive)

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
| **Waypoint** | `slice.checkout` | Checkout a waypoint slice |
| **Waypoint** | `slice.record_operation` | Record waypoint operation |
| **Waypoint** | `slice.depart` | Depart from waypoint |
| **Proof** | `proof.generate_inclusion` | Create Merkle inclusion proof |
| **Proof** | `proof.verify_inclusion` | Verify Merkle inclusion proof |
| **Integration** | `session.commit` | Provenance session commits (aliases: `commit.session`, `provenance.commit`) |
| **Integration** | `braid.commit` | Attribution braid commits |
| **Compat** | `permanence.commit_session` | Ephemeral DAG wire compat |
| **Compat** | `permanence.verify_commit` | Verify via compat format |
| **Compat** | `permanence.get_commit` | Retrieve via compat format |
| **Compat** | `permanence.health_check` | Health for compat clients |
| **Bonding** | `bonding.ledger.store` | Store ionic bond record |
| **Bonding** | `bonding.ledger.retrieve` | Retrieve bond by ID |
| **Bonding** | `bonding.ledger.list` | List all bond IDs |
| **Anchor** | `anchor.publish` | Record public chain anchor receipt |
| **Anchor** | `anchor.verify` | Verify anchor against spine state |
| **Health** | `health.check` | Service status |
| **Health** | `health.liveness` | Liveness probe (`{"status":"alive"}`) |
| **Health** | `health.readiness` | Readiness probe |
| **Meta** | `capabilities.list` | List primal capabilities (Wire Standard L3) |
| **Meta** | `identity.get` | Primal identity (name, version, domain, license) |
| **MCP** | `tools.list` | MCP tool discovery |
| **MCP** | `tools.call` | MCP tool invocation |

---

## Discovery

LoamSpine discovers services at runtime via **infant discovery** (zero knowledge at startup):

1. **NeuralAPI** -- biomeOS Unix socket IPC (preferred, capability-registered)
2. **Environment Variables** (`CAPABILITY_*_ENDPOINT`, `*_SERVICE_URL`)
3. **Service Registry** -- HTTP-based (Consul adapter, etcd adapter)
4. **DNS SRV** -- RFC 2782 (`_signing._tcp.local`)
5. **mDNS** -- RFC 6762 (experimental, feature-gated)
6. **Development Fallback** (`localhost`, debug builds only)

### Socket Naming (PRIMAL_SELF_KNOWLEDGE_STANDARD §3)

| Mode | Socket Path |
|------|------------|
| **Development** (`BIOMEOS_INSECURE=1`) | `$XDG_RUNTIME_DIR/biomeos/permanence.sock` |
| **Production** (`BIOMEOS_FAMILY_ID=<fid>`) | `$XDG_RUNTIME_DIR/biomeos/permanence-<fid>.sock` |
| **Legacy symlink** | `loamspine.sock → permanence.sock` |

Security invariant: `BIOMEOS_INSECURE=1` + non-default `FAMILY_ID` → refuse to start.

---

## Quality

| Metric | Value |
|--------|-------|
| **Version** | 0.9.16 |
| **Edition** | 2024 |
| **Tests** | 1,442 passing (all concurrent, ~3s, zero flaky) |
| **Coverage** | 90.92% line / 89.09% branch / 92.92% region (llvm-cov) |
| **Clippy** | 0 warnings (pedantic + nursery + `missing_const_for_fn`, `-D warnings`) |
| **Unsafe Code** | 0 (`#![forbid(unsafe_code)]`) |
| **Lint Exceptions** | 4 `#[allow]` in production (2× tarpc macro, 2× feature-conditional async); tests all `#[expect(reason)]` |
| **Max File Size** | 605 max production; 783 max test file |
| **Source Files** | 178 `.rs` files across 2 crates + binary (+ 3 fuzz targets) |
| **License** | AGPL-3.0-or-later + ORC + CC-BY-SA-4.0 (scyBorg triple) |
| **SPDX Headers** | All source files |
| **ecoBin** | Zero C dependencies (pure Rust) |
| **cargo deny** | advisories, bans, licenses, sources all pass |
| **UniBin** | `server`, `capabilities`, `socket` subcommands |
| **Mock isolation** | All mocks cfg-gated out of production |

---

## Deployment

```bash
# musl-static build (ecoBin-compliant, for plasmidBin / benchScale)
cargo build-x64                # x86_64-unknown-linux-musl
cargo build-arm64              # aarch64-unknown-linux-musl

# Verify static linkage
file target/x86_64-unknown-linux-musl/release/loamspine
# → ELF 64-bit LSB executable, x86-64, statically linked, stripped

# Docker (musl-static, alpine runtime)
docker build -t loamspine .

# Verify everything
./verify.sh
```

Prerequisites for musl builds: `rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl` and `sudo apt install musl-tools gcc-aarch64-linux-gnu`.

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

scyBorg triple license:
- **Code**: AGPL-3.0-or-later — see [LICENSE](./LICENSE)
- **Game Mechanics**: ORC — see [LICENSE-ORC](./LICENSE-ORC)
- **Creative/Documentation**: CC-BY-SA-4.0 — see [LICENSE-CC-BY-SA](./LICENSE-CC-BY-SA)

---

## Part of ecoPrimals

This repo is part of the [ecoPrimals](https://github.com/ecoPrimals) sovereign
computing ecosystem — a collection of pure Rust binaries that coordinate via
JSON-RPC, capability-based routing, and zero compile-time coupling.

See [wateringHole](https://github.com/ecoPrimals/wateringHole) for ecosystem
documentation, standards, and the primal registry.

---

**LoamSpine: Where memories become permanent.**
