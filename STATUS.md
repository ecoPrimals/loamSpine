<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Implementation Status

**Current Version**: 0.8.3  
**Last Updated**: March 14, 2026

---

## Overview

This document tracks implementation progress against the specification suite in [specs/00_SPECIFICATIONS_INDEX.md](specs/00_SPECIFICATIONS_INDEX.md).

---

## Implementation Status by Spec Area

| Spec | Status | Notes |
|------|--------|-------|
| [LOAMSPINE_SPECIFICATION.md](specs/LOAMSPINE_SPECIFICATION.md) | COMPLETE | Master spec implemented |
| [ARCHITECTURE.md](specs/ARCHITECTURE.md) | COMPLETE | Component layout matches spec |
| [DATA_MODEL.md](specs/DATA_MODEL.md) | COMPLETE | Entry, Spine, Chain, SpineConfig, EntryType (15+ variants) |
| [PURE_RUST_RPC.md](specs/PURE_RUST_RPC.md) | COMPLETE | tarpc + pure JSON-RPC (hand-rolled), no gRPC/protobuf/jsonrpsee. Semantic naming. |
| [WAYPOINT_SEMANTICS.md](specs/WAYPOINT_SEMANTICS.md) | PARTIAL | `anchor_slice`, `checkout_slice`, `depart_slice`, `record_operation` implemented. `WaypointConfig`, `PropagationPolicy`, `SliceTerms`, `SliceOperationType` types defined. Missing: relending chain, expiry sweep, Beardog attestation. |
| [CERTIFICATE_LAYER.md](specs/CERTIFICATE_LAYER.md) | PARTIAL | Core CRUD + loan/return + `verify_certificate` + `certificate_lifecycle`. Storage trait-backed. Missing: `generate_provenance_proof`, escrow/`TransferConditions`, `UsageSummary`. |
| [API_SPECIFICATION.md](specs/API_SPECIFICATION.md) | COMPLETE | 28 JSON-RPC methods, tarpc server, semantic naming. Spec updated to match implementation. |
| [INTEGRATION_SPECIFICATION.md](specs/INTEGRATION_SPECIFICATION.md) | PARTIAL | Provenance trio, session/braid commit. Missing: `SyncProtocol` (federation), `PrimalAdapter` retry/circuit-breaker. |
| [STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md) | PARTIAL | Memory, redb (default), sled (optional), SQLite (feature-gated). PostgreSQL, RocksDB not yet implemented. |
| [SERVICE_LIFECYCLE.md](specs/SERVICE_LIFECYCLE.md) | COMPLETE | `ServiceState` enum, startup/shutdown, NeuralAPI registration, signal handling, observable state via `watch` channel. |

---

## Discovery

| Mechanism | Status |
|-----------|--------|
| Environment variables | COMPLETE |
| DNS SRV | COMPLETE |
| Service registry HTTP | COMPLETE |
| mDNS | Stub only |

---

## Quality Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Tests | — | 809 |
| Coverage (llvm-cov) | 90%+ | 84.52% line / 78.88% branch |
| `unsafe` blocks | 0 | 0 |
| Clippy pedantic+nursery | 0 | 0 |
| Doc warnings | 0 | 0 |
| Max file size | < 1000 lines | 990 max |
| Source files | — | 96 `.rs` files, 35,352 lines |

---

## Standards Compliance

| Standard | Status | Notes |
|----------|--------|-------|
| UniBin | PASS | `loamspine server`, `capabilities`, `socket` subcommands |
| ecoBin | PASS | Zero C deps in default features |
| AGPL-3.0-only | PASS | SPDX headers on all 96 source files |
| Semantic naming | PASS | `{domain}.{operation}` per wateringHole standard |
| Zero-copy | PARTIAL | `Did` → `Arc<str>`, `Bytes` for payloads, `Cow<'static, str>` for config, zero-alloc JSON-RPC dispatch |
| MockTransport | PASS | `cfg(test\|testing)` gated — no mock code in production binary |
| File size limit | PASS | All files under 1000 lines (max: 990) |

---

## v0.8.1 Changes (March 14, 2026)

- `Did` evolved from `String` to `Arc<str>` for O(1) cloning across RPC boundaries
- `ServiceState` enum added per SERVICE_LIFECYCLE.md spec (Starting → Ready → Running → Stopping → Stopped)
- Observable state via `tokio::sync::watch` channel for health probes
- Broken rustdoc links in `transport/mock.rs` fixed
- `storage_backend_availability` test now feature-aware (`--all-features` passes)
- `match_same_arms` lint resolved in `error.rs` (merged `InvalidData` into common arm)
- `println!` in binary replaced with `writeln!(stdout())` for explicit error handling
- Hardcoded `"loamspine"` strings in `discovery_client` replaced with `PRIMAL_NAME` constant
- `TransportResponse::from_static` added for zero-copy mock responses
- `collect::<Vec<_>>()` anti-pattern eliminated in example
- Specs (`API_SPECIFICATION.md`, `PURE_RUST_RPC.md`) updated from `loamspine.camelCase` to `domain.operation` semantic naming
- Duplicate NeuralAPI registration code extracted to `register_neural_api()` helper

---

## v0.8.2 Changes (March 14, 2026)

- **Certificate storage trait**: Evolved from raw `RwLock<HashMap>` to `CertificateStorage` trait + `InMemoryCertificateStorage` implementation
- **`must_use_candidate` lint enabled**: Removed crate-level `#[allow]`, added `#[must_use]` to 11 functions
- **Smart refactoring**: `discovery.rs` (783 lines) → `discovery/{mod,dyn_traits,tests}.rs` (337+117+345); `manager.rs` (783 lines) → `manager/{mod,tests}.rs` (299+422). All files now under 422 lines
- **Waypoint types module**: `WaypointConfig`, `PropagationPolicy`, `DepartureReason`, `WaypointSummary`, `SliceOperationType`, `SliceTerms` types defined per spec
- **Waypoint operations**: `record_operation` and `depart_slice` added to service
- **Certificate verification**: `verify_certificate` returns `CertificateVerification` with enum-based `VerificationCheck`s
- **Certificate lifecycle**: `certificate_lifecycle` returns filtered entry history for a certificate
- **Mint fix**: `MintInfo.entry` now set to the actual entry hash (was `[0u8; 32]`)
- **Test count**: 719 → 744 (+25 tests covering new storage, waypoint, and certificate features)

---

## v0.8.3 Quality & Pedantic Audit (March 14, 2026)

- **Clippy pedantic + nursery clean**: 67 errors → 0 across all 3 workspace crates
- **`significant_drop_tightening`**: 26 lock guard scoping issues fixed with `drop()` and block scoping
- **`const fn` promotion**: 15 functions made `const` (identifiers, accessors, constructors)
- **Missing `# Errors` docs**: 10 public Result-returning functions documented
- **`let...else` modernization**: 6 match blocks rewritten to idiomatic `let...else`
- **`MockTransport` cfg-gated**: No longer compiled into production binary
- **Dead field removed**: `SpineSyncState.last_sync_ns` (never read)
- **Zero-copy JSON-RPC**: `params.clone()` eliminated — `dispatch` takes ownership, `handle_request` takes by value
- **SQLite storage tests**: 16 new tests (was 0% coverage)
- **HTTP transport tests**: 12 new tests with mini-server for success/error paths
- **Neural API tests**: 5 new env-var resolution tests
- **CLI signer tests**: 10 new DynSigner/DynVerifier trait object tests
- **Smart file splits**: `storage/tests.rs` (1261→892+370), `cli_signer.rs` (1002→332+673)
- **All files under 1000 lines**: Max file is 990 lines (was 1261)
- **Test count**: 771 → 809 (+38 tests)
- **Coverage**: 80.52% → 84.52% line coverage (llvm-cov)
- **`cargo fmt`**: Clean (was 6 files with drift)
- **`cargo doc`**: Zero warnings

---

## v0.8.2+ Pure Rust Evolution (March 14, 2026)

- **redb default storage**: Added `RedbStorage` (pure Rust embedded DB) as default backend; `sled` demoted to optional feature (`sled-storage`)
- **jsonrpsee removed**: Replaced with hand-rolled pure JSON-RPC 2.0 server (TcpListener + newline-delimited JSON + HTTP POST); eliminates transitive `ring` dependency
- **reqwest removed**: Replaced with `ureq` (pure Rust, no TLS, no ring) for `discovery-http` feature; HTTPS routes through BearDog/Songbird TLS stack
- **ecoBin compliant**: Zero C/C++/assembly dependencies in default feature set; `ring` fully eliminated
- **Test count**: 744 → 739 (5 tests removed during jsonrpsee/reqwest migration — stale integration stubs)

---

*See [WHATS_NEXT.md](WHATS_NEXT.md) for the development roadmap.*
