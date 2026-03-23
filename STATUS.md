<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# Implementation Status

**Current Version**: 0.9.7  
**Last Updated**: March 23, 2026

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
| [PURE_RUST_RPC.md](specs/PURE_RUST_RPC.md) | COMPLETE | tarpc + pure JSON-RPC (hand-rolled), no gRPC/protobuf/jsonrpsee. Semantic naming. Protocol escalation (`IpcProtocol` negotiation). |
| [WAYPOINT_SEMANTICS.md](specs/WAYPOINT_SEMANTICS.md) | COMPLETE | `anchor_slice`, `checkout_slice`, `depart_slice`, `record_operation` implemented. `WaypointConfig` with `AttestationRequirement` (None/BoundaryOnly/AllOperations/Selective). `AttestationResult` for capability-discovered attestation providers. `PropagationPolicy`, `SliceTerms`, `SliceOperationType`, `WaypointSummary` types defined. `RelendingChain` with multi-hop sublend/return. `ExpirySweeper` for auto-return. |
| [CERTIFICATE_LAYER.md](specs/CERTIFICATE_LAYER.md) | COMPLETE | Core CRUD + loan/return + sublend + `verify_certificate` + `generate_provenance_proof` + escrow + `UsageSummary` integrated into `CertificateReturn` and `LoanRecord`. `WaypointSummary` re-used from waypoint module. Scyborg license schema. Certificate module: types, lifecycle, metadata, provenance, escrow, usage, tests. |
| [API_SPECIFICATION.md](specs/API_SPECIFICATION.md) | COMPLETE | 28 JSON-RPC methods, tarpc server, semantic naming. Spec updated to match implementation. |
| [INTEGRATION_SPECIFICATION.md](specs/INTEGRATION_SPECIFICATION.md) | COMPLETE | Provenance trio, session/braid commit. `SyncProtocol` evolved to JSON-RPC/TCP sync engine with `push_to_peer`/`pull_from_peer` and graceful fallback. `ResilientDiscoveryClient` with circuit-breaker (Closed/Open/HalfOpen, lock-free atomics) and retry policy (exponential backoff with jitter). |
| [STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md) | PARTIAL | Memory, redb (default), sled (optional), SQLite (feature-gated, refactored to modular `sqlite/` directory). PostgreSQL, RocksDB not yet implemented. |
| [SERVICE_LIFECYCLE.md](specs/SERVICE_LIFECYCLE.md) | COMPLETE | `ServiceState` enum, startup/shutdown, NeuralAPI registration, signal handling, observable state via `watch` channel. |
| [COLLISION_LAYER_ARCHITECTURE.md](specs/COLLISION_LAYER_ARCHITECTURE.md) | PROPOSAL | Research spec. Hash collision layers bridging linear ↔ DAG. Validation experiments tracked in neuralSpring. |

---

## Discovery

| Mechanism | Status |
|-----------|--------|
| Environment variables | COMPLETE |
| DNS SRV | COMPLETE |
| Service registry HTTP | COMPLETE |
| mDNS | Feature-gated (real implementation via `mdns` crate) |

---

## Quality Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Tests | — | 1,232 |
| Coverage (llvm-cov) | 90%+ | 92.23% line / 90.46% region / 86.52% function |
| `unsafe` in production | 0 | 0 (`#![deny(unsafe_code)]`) |
| Clippy pedantic+nursery | 0 | 0 |
| Doc warnings | 0 | 0 |
| Max file size | < 1000 lines | 865 max (all 124 files under 1000) |
| Source files | — | 124 `.rs` files |
| Edition | 2024 | 2024 |
| `#[allow]` in production | 0 | 0 (all migrated to `#[expect(reason)]`) |
| `cargo deny check` | pass | advisories ok, bans ok, licenses ok, sources ok |

---

## Standards Compliance

| Standard | Status | Notes |
|----------|--------|-------|
| UniBin | PASS | `loamspine server`, `capabilities`, `socket` subcommands |
| ecoBin | PASS | Zero C deps in default features; blake3 `pure` mode; musl cross-compile CI |
| AGPL-3.0-or-later | PASS | SPDX headers on all 124 source files |
| Scyborg license | PASS | `CertificateType::scyborg_license()`, metadata builders, schema constants |
| Semantic naming | PASS | `capabilities.list` canonical + `primal.capabilities` alias per v2.1 standard |
| `health.liveness` | PASS | Returns `{"status": "alive"}` per Semantic Method Naming Standard v2.1 |
| PUBLIC_SURFACE | PASS | `CONTEXT.md` created, "Part of ecoPrimals" footer in README.md |
| Zero-copy | PASS | `Did` → `Arc<str>`, `Bytes` for payloads, `Cow<'static, str>` for config, zero-alloc JSON-RPC dispatch, `[u8; 24]` stack keys for storage, `entry.clone()` eliminated — `tip_entry()` zero-copy persistence |
| MockTransport | PASS | `cfg(test|testing)` gated — no mock code in production binary |
| File size limit | PASS | All 124 files under 1000 lines (max: 865 in `certificate_tests.rs`). |

---

## v0.9.7 Dependency Hygiene & Coverage Evolution (March 23, 2026)

- **`cargo deny check` now passes clean**: advisories ok, bans ok, licenses ok, sources ok.
- **`deny.toml` accuracy**: Advisory comments corrected — `fxhash`/`instant` are sled deps (not tarpc); `bincode` v1 is direct dep (tarpc path eliminated); `opentelemetry_sdk` is tarpc 0.37 hard dep (not feature-gated). Three new mdns-related advisories (async-std, net2, proc-macro-error) documented as optional feature-gated.
- **tarpc feature trimming**: `features = ["full"]` replaced with explicit feature list dropping `serde-transport-bincode`. Eliminates bincode v1 via tokio-serde transitive path.
- **`publish = false`**: Added to all workspace crates (private, never published to crates.io). Satisfies cargo-deny wildcard ban with `allow-wildcard-paths`.
- **`libsqlite3-sys` ban wrapper**: `wrappers = ["rusqlite"]` allows the C dep only through the optional sqlite feature.
- **Sync streaming coverage**: 7 new tests for `push_entries_streaming` and `pull_entries_streaming` (success, failure fallback, requires-peers, empty state). Sync module line coverage: 69.00% → 90.57%.
- **`#[allow(deprecated)]` → `#[expect(deprecated, reason)]`**: Remaining two test-only deprecated aliases migrated.
- **Hardcoding eliminated**: Port 443 → `HTTPS_DEFAULT_PORT` constant; capability strings → `external::*` constants in infant discovery DNS SRV mapping.
- **unsafe eliminated**: All `infant_discovery` test `unsafe` env mutations migrated to `temp_env::with_vars` + phased `block_on` pattern.
- **Smart refactors**: `redb_tests.rs` (955 → 574 + 395 `redb_tests_cert_errors.rs`); `jsonrpc/tests.rs` (903 → 588 + 379 `tests_permanence_cert.rs`).
- **Coverage**: 91.67% → **92.23% line** / 89.87% → **90.46% region** / 86.21% → **86.52% function**.
- **Tests**: 1,226 → **1,232** (+6 net). Source files: 127 → **124**. All under 1000 lines.

---

## v0.9.6 Standards Compliance & Lint Evolution (March 17, 2026)

- **`capabilities.list` canonical method**: JSON-RPC dispatcher responds to `capabilities.list` (v2.1 standard), `capability.list` (legacy), and `primal.capabilities` (alias).
- **`health.liveness` standardized**: Returns `{"status": "alive"}` per Semantic Method Naming Standard v2.1 (was `{"alive": true}`).
- **CONTEXT.md**: AI-discoverable context block per PUBLIC_SURFACE_STANDARD (65 lines). Role, capabilities, boundaries.
- **"Part of ecoPrimals" footer**: Added to README.md per PUBLIC_SURFACE_STANDARD Layer 2.
- **`#[allow]` → `#[expect(reason)]` bulk migration**: 30+ test files migrated. Dead attributes removed where lints don't fire. `redundant_clone` attributes removed where clippy no longer triggers.
- **Smart refactor neural_api.rs**: 871 → 384 + 489 lines (`neural_api_tests.rs` via `#[path]`).
- **Tests**: 1,226. **Source files**: 125 → 126. All under 1000 lines.

---

## v0.9.5 Deep Debt Resolution & Idiomatic Evolution (March 17, 2026)

- **`DispatchOutcome` wired into JSON-RPC server**: `dispatch_typed` method classifies errors into `ProtocolError` vs `ApplicationError`; `outcome_to_response` maps back to JSON-RPC wire format. Ecosystem consistency with rhizoCrypt/airSpring.
- **`StreamItem` wired into sync**: `push_entries_streaming` and `pull_entries_streaming` emit `Data`/`Progress`/`End`/`Error` stream items for pipeline coordination.
- **`OrExit` tracing evolution**: `eprintln!` in `OrExit` trait replaced with `tracing::error!` for structured logging consistency.
- **Zero-copy sync evolution**: `entries_json.clone()` in `pull_from_peer` → `serde_json::Value::remove()` ownership transfer. `push_entries` clone elimination via try-then-own pattern.
- **Smart refactor lifecycle.rs**: `lifecycle.rs` (888 lines) → `lifecycle.rs` (442) + `lifecycle_tests.rs` (444). Uses `#[path]` pattern consistent with `certificate.rs`.
- **Storage error-path coverage**: 4 new sled tests covering malformed keys in `list_spines`/`list_certificates`, missing entries in index, and corrupted entry bytes.
- **`#[allow]` → `#[expect]` refinement**: Removed unfulfilled `expect_used`/`panic` expectations in `jsonrpc/mod.rs`, `sync/mod.rs`, and `certificate.rs` test modules.
- **Doc link fixes**: Fully qualified paths for `StreamItem` variants in sync module doc comments.
- **Tests**: 1,221 → 1,226 (+5). **Source files**: 123 → 125 (+`lifecycle_tests.rs`, +`sled` test additions). All under 1000 lines (max: 955).

---

## v0.9.2 Deep Debt Resolution & Idiomatic Evolution (March 16, 2026)

- **Certificate service smart refactoring**: `certificate.rs` (906 lines) → 3 domain-focused modules: `certificate.rs` (380 — core CRUD, verification, proofs), `certificate_loan.rs` (367 — loan lifecycle, sublend, auto-return), `certificate_escrow.rs` (193 — hold, release, cancel). No code duplication; clean `impl LoamSpineService` blocks per domain.
- **Hardcoding evolution**: `../bins` path in `cli_signer.rs` evolved to environment-configurable `LOAMSPINE_BINS_DIR` with fallback. Zero hardcoded paths remain in production code.
- **Unsafe code evolution**: `lifecycle.rs` test unsafe `env::remove_var` evolved to safe `temp_env::with_var_unset` + manual runtime pattern. `unsafe_code` allow removed from lifecycle test module.
- **Doc count alignment**: STATUS.md and WHATS_NEXT.md corrected from stale "114" to actual 121 source file count. Coverage metric corrected: 88.84% line / 84.46% region / 91.01% function.
- **Dependency audit**: All default-feature deps are pure Rust (ecoBin compliant). C dependencies only via optional features (sqlite, mdns). `tokio`/`redb` use system libc for I/O (unavoidable for networked services), but no bundled C code.
- **Mock audit**: All `MockSigner`, `MockVerifier`, `MockTransport` properly gated behind `#[cfg(any(test, feature = "testing"))]`. Zero mock code in production binary. All stubs evolved to real implementations.
- **Hardcoding audit**: Zero hardcoded primal names in production (2 self-identity `"LoamSpine"` references are correct). Zero hardcoded ports in production (dev defaults in `constants.rs` with env override). Zero TODO/FIXME/HACK. Zero `println!`/`eprintln!` in production (all tracing).
- **Source files**: 119 → 121. **All 1,190 tests pass**. Clippy pedantic+nursery clean. Zero doc warnings.

---

## v0.9.1 Deep Audit & Idiomatic Evolution (March 16, 2026)

- **`StubAttestationProvider` → `DiscoveredAttestationProvider`**: Production stub evolved to real JSON-RPC implementation. Sends `attestation.request` to capability-discovered endpoint; falls back to local approval in degraded mode with tracing warning.
- **Attestation provider test coverage**: `register_attestation_provider`, `unregister_attestation_provider`, `request_attestation` (success, denial with reason, denial without reason, provider error) — 8 new tests.
- **Discovery test coverage**: `all_statuses_includes_attestation`, deprecated alias coverage expanded.
- **Infant discovery test coverage**: DNS SRV error/timeout paths, registry discovery failure, config clone/debug, method clone/debug, multi-capability cache independence — 10 new tests.
- **CLI signer test coverage**: `discover_binary` env fallthrough, nonexistent path, sign-after-binary-removal, verifier with true/false binaries, `verify_entry` delegation, accessor constants — 11 new tests.
- **tarpc server named constants**: `TARPC_MAX_CONCURRENT_REQUESTS` (100) and `TARPC_MAX_CHANNELS_PER_IP` (10) extracted from magic numbers.
- **JSON-RPC Content-Length warning**: Silent `unwrap_or(0)` replaced with `match` + `tracing::warn` on malformed headers.
- **`fuzz/Cargo.toml` license**: Added missing `license = "AGPL-3.0-or-later"`.
- **`#[allow]` → `#[expect(reason)]` migration**: Test modules for discovery, infant_discovery, and cli_signer_tests migrated to `#[expect(..., reason = "...")]`.

---

## v0.9.0 Deep Debt Resolution & ecoBin Evolution (March 16, 2026)

- **Zero-copy `append` refactor**: Eliminated `entry.clone()` across all 16 service layer call sites. `Spine::append()` takes ownership; callers use `spine.tip_entry()` for zero-copy persistence.
- **Capability string constants**: All hardcoded capability strings ("persistent-ledger", "certificate-manager") replaced with `capabilities::identifiers::loamspine::*` constants. Added `ADVERTISED` canonical set. `InfantDiscovery::from_advertised()` constructor.
- **Attestation runtime enforcement**: `check_attestation_requirement()` wired into `anchor_slice`, `record_operation`, `depart_slice`. Capability-discovered attestation provider with `DynAttestationProvider` trait, `StubAttestationProvider`, and graceful degradation.
- **blake3 pure Rust**: Switched to `features = ["pure"]` — zero C/asm compilation. Full ecoBin compliance confirmed.
- **AGPL-3.0-or-later**: Aligned all SPDX headers (119 source files) with wateringHole scyBorg guidance.
- **`temp-env` migration**: 14 additional async tests migrated from `unsafe` env mutation to `temp_env::with_vars` + manual runtime. Nested runtime issue resolved.
- **`CAPABILITIES.to_vec()` eliminated**: `neural_api.rs` uses `&[&str]` slice directly.
- **`.cargo/config.toml`**: Documented noexec mount workaround with env var override guidance.
- **`cfg_attr` conditional lint**: Discovery client `unreachable_code` lint expectation made feature-conditional.
- **`SpineConfig::waypoint_config`**: Added optional `WaypointConfig` to `SpineConfig` for attestation policies on waypoint spines.
- **Main.rs integration tests**: CLI parsing, capabilities JSON output, socket path, server start/shutdown via SIGINT.
- **`niche.rs` consumed capabilities**: Evolved from string literals to `capabilities::identifiers::external::*` constants.

---

## v0.8.9 Self-Knowledge, temp-env, Deploy Graph (March 15, 2026)

- **`primal_names.rs`**: Centralized primal identifier constants — ecosystem convention from groundSpring/wetSpring.
- **`niche.rs`**: Full primal self-knowledge module — 23 methods, 8 domains, 6 consumed capabilities, 4 optional deps, 21 cost estimates, semantic mappings.
- **5-tier socket discovery**: `/run/user/{uid}/biomeos/` tier via `/proc/self/status` (Linux). Applied to `constants/network.rs` and `neural_api.rs`.
- **`temp-env` migration**: 38 `unsafe` env mutation blocks eliminated from `constants/network.rs` and `neural_api.rs` tests. Thread-safe automatic save/restore.
- **Deploy graph**: `graphs/loamspine_deploy.toml` for biomeOS orchestration.
- **Tests**: 1,123 → 1,132 (+9). Source files: 112 → 114.

## v0.8.8 Cross-Spring Absorption & Edition 2024 (March 15, 2026)

- **Edition 2024**: Migrated from edition 2021 (Rust 1.92 supports it). 19 `collapsible_if` patterns modernized to let-chains. `env::set_var`/`remove_var` wrapped in `unsafe` blocks in 7 test files. `env_set!`/`env_rm!` macros introduced for test ergonomics. `unsafe_code` lint: `forbid` → `deny` to allow `#[allow(unsafe_code)]` in test modules only.
- **JSON-RPC batch**: Full JSON-RPC 2.0 batch array support per spec — empty batch error, notification suppression, mixed batch processing.
- **Proptest**: 7 property-based roundtrip tests for core newtypes (`Did`, `SpineId`, `ContentHash`, `Signature`, `ByteBuffer`).
- **Named constants**: Circuit breaker and retry thresholds extracted to `{DOMAIN}_{METRIC}_{QUALIFIER}` named constants with documented provenance.
- **Enriched `capability.list`**: Response includes `version`, `methods` array with `method`/`domain`/`cost`/`deps` per operation.
- **Platform-agnostic paths**: Hardcoded `/tmp` → `std::env::temp_dir()` in socket resolution fallback paths.
- **Cleanup**: Stale showcase `IMPLEMENTATION_STATUS.md` removed. Showcase index aligned with actual directory structure. Dockerfile updated for edition 2024. Broken `ROOT_DOCS_INDEX.md` reference removed.
- **Test count**: 1,114 → 1,123 (+9 tests)

---

## v0.8.7 Spec Completion & Idiomatic Evolution (March 15, 2026)

- **UsageSummary**: `UsageSummary` type with builder API, integrated into `CertificateReturn` entry type and `LoanRecord` provenance. `WaypointSummary` re-used from waypoint module. Per CERTIFICATE_LAYER.md spec.
- **Attestation framework**: `AttestationRequirement` (None/BoundaryOnly/AllOperations/Selective) added to `WaypointConfig`. `AttestationResult` struct for capability-discovered attestation providers. No hardcoded primal names — attestation discovered at runtime via `"attestation"` capability. Per WAYPOINT_SEMANTICS.md spec.
- **`#[allow]` → `#[expect(reason)]` migration**: All production `#[allow(...)]` attributes replaced with `#[expect(..., reason = "...")]` for documented lint exceptions. Removed stale `#[allow(async_fn_in_trait)]` from `dyn_traits.rs` (methods were already `Pin<Box<dyn Future>>`).
- **Sync module refactoring**: `sync.rs` (927 lines) → `sync/mod.rs` (405) + `sync/tests.rs` (505). Clear separation of production code and test infrastructure.
- **JSON-RPC server coverage**: `ServerHandle::local_addr()` for OS-assigned port testing. 6 new TCP integration tests covering raw TCP, HTTP POST, method-not-found, parse error, shutdown, and spine creation over TCP. `jsonrpc/mod.rs` coverage: 51% → 92%.
- **Certificate error-path tests**: 5 new tests for return-not-loaned, wrong-borrower-return, transfer/loan nonexistent, verify nonexistent.
- **Dependency audit**: Default features pure Rust (zero C deps). `libsqlite3-sys` only via feature-gated `sqlite`. `hickory-resolver` pure Rust. `linux-raw-sys` is Rust syscall bindings (not C).
- **Capability discovery**: `primal-capabilities.toml` updated with `attestation` optional dependency and enhanced port documentation.
- **Coverage**: 89.30% → 89.64% line, 91.26% → 91.71% region (+22 tests). Remaining gap is binary entry point `main.rs` (150 lines at 0% — inherently untestable via unit tests).
- **Test count**: 1,092 → 1,114 (+22 tests)
- **Specs promoted**: WAYPOINT_SEMANTICS.md and CERTIFICATE_LAYER.md both promoted from PARTIAL → COMPLETE.

---

## v0.8.6 Deep Debt & Feature Completion (March 15, 2026)

- **Relending chain**: `RelendingChain` with `RelendingLink`, multi-hop sublend/return, depth validation (`can_sublend`), unwinding (`return_at`), `current_holder()` tracking
- **Expiry sweeper**: `ExpirySweeper` background task with configurable interval, auto-returns expired loaned certificates, full relending chain unwinding
- **Provenance proof**: `CertificateOwnershipProof` with `compute_merkle_root()` using Blake3, Merkle tree over mint+transfer entry hashes, `verify()` method
- **Certificate escrow**: `TransferConditions`, `EscrowCondition` (Payment/Signature/Time), `hold_certificate`/`release_certificate`/`cancel_escrow` with `PendingTransfer` state
- **Resilience patterns**: `CircuitBreaker` (Closed/Open/HalfOpen, lock-free `AtomicU8`/`AtomicU32`/`AtomicU64`), `RetryPolicy` (exponential backoff with jitter), `ResilientDiscoveryClient` wrapping discovery operations
- **Certificate module refactoring**: 915-line `certificate.rs` → `certificate/` module directory (mod.rs, types.rs, lifecycle.rs, metadata.rs, provenance.rs, escrow.rs, tests.rs)
- **Cast safety**: All `#[allow(clippy::cast_possible_truncation)]` replaced with `try_from()` + fallback
- **File size compliance**: All 113 `.rs` files under 1000 lines (max: 955). Test files split: `redb_tests_coverage.rs`, `tests_validation.rs`, `certificate_tests.rs`
- **Coverage**: 88.28% → 89.30% line, 90.45% → 91.26% region (+124 tests)
- **Test count**: 968 → 1,092 (+124 tests across jsonrpc, redb, sled, lifecycle, certificate, resilience, waypoint, proof, escrow)
- **Version**: `primal-capabilities.toml` bumped to 0.8.6

---

## v0.8.5 Comprehensive Audit & Evolution (March 15, 2026)

- **Clippy clean**: Fixed 18 clippy errors (module_inception, match_same_arms, cast_possible_truncation, expect_used, future_not_send, manual_let_else, unused_async, iter_on_single_items)
- **Storage test refactoring**: `storage/tests.rs` (1122 lines) → 3 backend-specific modules: `tests.rs` (~340), `redb_tests.rs` (~340), `sled_tests.rs` (~340). All under 1000 LOC.
- **Coverage boost**: 86.47% → 88.28% line, 90.45% region (+98 tests across sqlite, infant_discovery, sync, jsonrpc, redb, discovery_client)
- **Idiomatic Rust evolution**: `unused_async` removed from mock helpers, `let...else` patterns, `HashSet::from()` constructors, `Sync` bounds on test generics, borrowed `&serde_json::Value` where ownership unnecessary
- **ConfigurableTransport**: New test-only transport for discovery client error-path coverage
- **Zero-copy improvements**: Mock helper functions take `&serde_json::Value` by reference instead of owned
- **Test count**: 870 → 968 (+98 tests)

---

## v0.8.4 Changes (March 15, 2026)

- **Scyborg license schema**: `CertificateType::scyborg_license()`, `CertificateMetadata::with_scyborg_license()`, schema constants (`SCYBORG_LICENSE_TYPE_URI`, `SCYBORG_LICENSE_SCHEMA_VERSION`)
- **Protocol escalation**: `IpcProtocol` enum, `negotiate_protocol()` preferring tarpc Unix socket, `resolve_primal_socket()` / `resolve_primal_tarpc_socket()` path builders
- **SyncProtocol evolved**: From local-only stub to JSON-RPC/TCP sync engine with `rpc_call()`, `push_to_peer()`, `pull_from_peer()`, `best_peer_endpoint()`, graceful fallback to local queues
- **SQLite smart refactoring**: 990-line `sqlite.rs` → modular `sqlite/` directory (`mod.rs`, `common.rs`, `spine.rs`, `entry.rs`, `certificate.rs`, `tests.rs`) totaling 939 lines
- **Zero-copy storage keys**: `Vec<u8>` index keys in redb/sled evolved to `[u8; 24]` stack allocation
- **Coverage boost**: 84.52% → 86.47% line coverage (+61 tests across neural_api, transport, infant_discovery, storage)
- **CI cross-compilation**: GitHub Actions job for musl targets (`x86_64`, `aarch64`, `armv7`) via `cross-rs/cross`
- **Infant discovery coverage**: Expanded tests for cache logic, config parsing, capability-to-SRV mapping, fresh checks
- **Neural API coverage**: Mock Unix socket server tests for register/deregister/error paths (57% → 88%)
- **Transport coverage**: Mock server tests for `jsonrpc_call`, `get_with_query`, `post_json`, base64 edge cases (70% → 92%)
- **Test count**: 809 → 870 (+61 tests)

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
