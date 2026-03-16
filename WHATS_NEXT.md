<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Development Roadmap

**Current Version**: 0.8.9  
**Last Updated**: March 15, 2026

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
- **Smart file splits**: All 114 source files under 1000 lines (max: 955)
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

---

## v0.9.0 Targets

- **90%+ line coverage** -- Currently 89.64%; remaining gap is binary entry point `main.rs` (150 lines, 0%). DNS SRV/mDNS network paths have limited testability.
- **Signing capability middleware** -- Signature verification on RPC layer (capability-discovered)
- **Showcase demos** -- Expand from ~10% to full coverage
- **Runtime attestation integration** -- Wire `AttestationRequirement` checks into waypoint operation flow with capability-discovered attestation providers

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
