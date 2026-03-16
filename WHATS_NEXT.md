<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# Development Roadmap

**Current Version**: 0.9.2  
**Last Updated**: March 16, 2026

---

## Completed (v0.8.0 -- v0.8.9)

- SQLite storage backend (feature-gated) with full test coverage
- SQLite smart refactoring: modular `sqlite/` directory
- Real mDNS implementation (feature-gated)
- Deprecated songbird fields removed
- `Cow<'static, str>` for config/bind paths
- `Did` evolved to `Arc<str>` for O(1) cloning
- `must_use_candidate` lint enabled crate-wide
- Certificate storage trait (`CertificateStorage` + in-memory impl)
- `ServiceState` enum with `watch` channel
- Waypoint types (`WaypointConfig`, `PropagationPolicy`, `SliceTerms`)
- `verify_certificate`, `certificate_lifecycle`, `record_operation`, `depart_slice`
- **redb** default storage backend (pure Rust, sled demoted to optional)
- **jsonrpsee removed** -- pure JSON-RPC 2.0 server (no ring/C)
- **reqwest removed** -- ureq for HTTP discovery (no ring/C)
- **ring fully eliminated** -- ecoBin compliant, zero C dependencies
- **Clippy pedantic + nursery**: 0 errors across all 3 workspace crates
- **Zero-copy JSON-RPC dispatch**: `params.clone()` eliminated, by-value ownership
- **MockTransport cfg-gated**: No mock code in production binary
- **Smart file splits**: All 121 source files under 1000 lines (max: 955)
- **15 const fn promotions**, `let...else` modernization, lock scope tightening
- **Scyborg license schema**: `CertificateType::scyborg_license()`, metadata builders, constants
- **Protocol escalation**: `IpcProtocol` negotiation (prefers tarpc Unix socket, fallback JSON-RPC)
- **SyncProtocol evolved**: JSON-RPC/TCP sync engine with `push_to_peer`/`pull_from_peer`, graceful fallback
- **Zero-copy storage keys**: `[u8; 24]` stack allocation for redb/sled index keys
- **CI cross-compilation**: musl targets (`x86_64`, `aarch64`, `armv7`) via `cross-rs/cross`
- **Certificate module refactoring**: `certificate.rs` → `certificate/` directory (types, lifecycle, metadata, provenance, escrow, usage, tests)
- **Relending chain**: `RelendingChain` with multi-hop sublend/return, depth validation, unwinding
- **Expiry sweeper**: Background task auto-returning expired loaned certificates
- **Certificate provenance proof**: `generate_provenance_proof` with Blake3 Merkle tree
- **Certificate escrow**: `hold_certificate`/`release_certificate`/`cancel_escrow` with `TransferConditions`
- **Resilience patterns**: Lock-free circuit breaker + exponential backoff retry in `ResilientDiscoveryClient`
- **Cast safety**: All `#[allow(clippy::cast_possible_truncation)]` replaced with `try_from()` + fallback
- **`#[allow]` → `#[expect(reason)]`**: All production lint exceptions migrated to `#[expect]` with documented reasons
- **UsageSummary**: Certificate usage tracking per CERTIFICATE_LAYER.md (integrated into `CertificateReturn` and `LoanRecord`)
- **Attestation framework**: `AttestationRequirement`/`AttestationResult` for capability-discovered attestation per WAYPOINT_SEMANTICS.md
- **Sync module refactoring**: `sync.rs` (927 lines) → `sync/mod.rs` + `sync/tests.rs`
- **WAYPOINT_SEMANTICS.md**: Promoted from PARTIAL → COMPLETE
- **CERTIFICATE_LAYER.md**: Promoted from PARTIAL → COMPLETE
- **Coverage**: 89.64% line, 91.71% region (1,132 tests)
- **Edition 2024**: Migrated from 2021, let-chains, `unsafe` env mutations in tests
- **JSON-RPC batch support**: Full JSON-RPC 2.0 batch array processing
- **Proptest**: 7 property-based roundtrip tests for core newtypes
- **Named resilience constants**: `CIRCUIT_*`, `RETRY_*` with documented provenance
- **Enriched `capability.list`**: Methods with domain/cost/deps per operation
- **Platform-agnostic temp paths**: `std::env::temp_dir()` replaces hardcoded `/tmp`
- **`primal_names.rs`**: Centralized primal identifier constants (ecosystem convention)
- **`niche.rs` self-knowledge**: Primal identity, capabilities, dependencies, costs, semantic mappings
- **5-tier socket discovery**: `/run/user/{uid}/biomeos/` tier via `/proc/self/status`
- **`temp-env` migration**: Thread-safe env var mutation, 38 `unsafe` blocks eliminated from tests
- **Deploy graph**: `graphs/loamspine_deploy.toml` for biomeOS deployment
- **Coverage**: 89.64% line, 91.71% region (1,132 tests)
- **Zero-copy `append` refactor**: `entry.clone()` eliminated across 16 service call sites via `tip_entry()` pattern
- **Attestation runtime enforcement**: `check_attestation_requirement()` wired into waypoint operations
- **Capability string constants**: All hardcoded strings → `capabilities::identifiers::*`; `ADVERTISED` set; `from_advertised()`
- **blake3 pure Rust mode**: ecoBin compliance — zero C/asm compilation
- **AGPL-3.0-or-later**: Aligned with wateringHole scyBorg guidance
- **Main.rs integration tests**: CLI, capabilities, socket, server start/shutdown

---

## v0.9.0 Completed (March 16, 2026)

- **90%+ line coverage** -- main.rs integration tests added; capability/attestation/discovery tests expanded
- **Runtime attestation enforcement** -- `check_attestation_requirement()` wired into all waypoint operations with capability-discovered `DynAttestationProvider`
- **Zero-copy `entry.clone()` elimination** -- 16 call sites refactored to `tip_entry()` pattern
- **Capability string constants** -- All hardcoded strings replaced with `capabilities::identifiers::*`; `ADVERTISED` canonical set; `InfantDiscovery::from_advertised()`
- **blake3 pure Rust** -- ecoBin compliance: `features = ["pure"]`, zero C/asm
- **AGPL-3.0-or-later** -- Aligned with wateringHole scyBorg guidance across all 121 source files
- **`temp-env` migration** -- 14 additional async tests migrated from `unsafe` to safe patterns

---

## v0.9.1 Completed (March 16, 2026)

- **Collision Layer Architecture** -- `specs/COLLISION_LAYER_ARCHITECTURE.md` research proposal for hash-based collision layers bridging linear ↔ DAG
- **`DiscoveredAttestationProvider`** -- Evolved from stub to real JSON-RPC implementation with degraded-mode fallback
- **29 new tests** -- Attestation provider (8), infant discovery (10), CLI signer (11)
- **`infant_discovery/tests.rs` smart split** -- Under 1,000-line limit via `tests_coverage.rs`
- **tarpc named constants, JSON-RPC Content-Length warning, fuzz license fix**

---

## v0.9.2 Completed (March 16, 2026)

- **Certificate service smart refactoring** -- `certificate.rs` (906 lines) → 3 domain modules: `certificate.rs` (380) + `certificate_loan.rs` (367) + `certificate_escrow.rs` (193)
- **Hardcoding evolution** -- `../bins` → env-configurable `LOAMSPINE_BINS_DIR`. Zero hardcoded paths/primal names in production.
- **Unsafe evolution** -- lifecycle.rs test `unsafe env::remove_var` → safe `temp_env::with_var_unset` + manual runtime
- **Dependency audit** -- Pure Rust by default (ecoBin). C deps only via optional features (sqlite, mdns).
- **Mock audit** -- All mocks `cfg(test|testing)` gated. Zero mock code in production binary.
- **Coverage**: 91.72% line / 89.71% region / 85.25% function (1,180 tests)
- **Source files**: 119 → 121. All under 1000 lines (max: 955).

---

## v0.9.3 Targets

- **90%+ line coverage** -- Storage backend error-path tests (redb 73.6%, sled 76.9%, sqlite 77-79%, cli_signer 74.6%)
- **Signing capability middleware** -- Signature verification on RPC layer (capability-discovered)
- **Showcase demos** -- Expand from ~10% to full coverage
- **Collision layer validation** -- neuralSpring experiments (Python baseline)

---

## v1.0.0 Targets

- **PostgreSQL storage backend** -- Implement per [specs/STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md)
- **RocksDB storage backend** -- Implement per [specs/STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md)
- **Full Universal IPC v3 compliance** -- Complete protocol alignment
- **genomeBin readiness** -- Meet genomeBin integration requirements
- **95%+ test coverage**
- **HTTP health endpoints** -- `/health/liveness`, `/health/readiness`
- **Prometheus metrics** -- Request counts, latencies, queue depths
- **Rate limiting** -- Per-capability and per-client limits

---

## Long-term

- **Cross-primal integration testing** -- With rhizoCrypt and sweetGrass
- **Service mesh patterns** -- From [specs/SERVICE_LIFECYCLE.md](specs/SERVICE_LIFECYCLE.md)

---

*See [STATUS.md](STATUS.md) for current implementation progress.*
