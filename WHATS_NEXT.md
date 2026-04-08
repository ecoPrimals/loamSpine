<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Development Roadmap

**Current Version**: 0.9.16  
**Last Updated**: April 8, 2026

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
- **Smart file splits**: All source files under 1000 lines (max: 955)
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
- **AGPL-3.0-or-later** -- Aligned with wateringHole scyBorg guidance across all 119 source files
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
- **Coverage**: 91.03% function / 88.91% line / 84.61% region (1,206 tests)
- **Source files**: 119 → 121. All under 1000 lines (max: 955).

---

## v0.9.3 Completed (March 16, 2026)

- **tarpc 0.35 → 0.37** — Aligned with biomeOS, rhizoCrypt, sweetGrass trio partners
- **`DispatchOutcome<T>`** — Typed dispatch result separating protocol vs application errors (rhizoCrypt/airSpring pattern)
- **`OrExit<T>` trait** — Zero-panic startup validation for `Result` and `Option` (wetSpring V123 pattern)
- **`extract_rpc_error()`** — Centralized JSON-RPC error extraction; replaces inline pattern in `neural_api.rs`
- **`is_method_not_found()`** — Convenience method for JSON-RPC -32601 detection
- **NDJSON `StreamItem`** — Pipeline streaming type (`Data`/`Progress`/`End`/`Error`) for biomeOS coordination
- **`deny.toml` evolution** — `wildcards = "warn"`, tarpc 0.37 advisory ignores, banned C sys-crates
- **Coverage**: 91.03% function / 88.91% line / 84.61% region (1,206 tests)
- **Source files**: 121 → 122 (added `streaming.rs`). All under 1000 lines (max: 955).

---

## v0.9.4 Completed (March 16, 2026)

- **`is_timeout_likely()` + `is_application_error()`** — IpcPhase helpers matching sweetGrass pattern
- **`OrExit` wired into main.rs** — Zero-panic startup validation for bind address and lifecycle init
- **`operation_dependencies` + `cost_estimates`** — Top-level DAG/cost metadata in capability.list for Pathway Learner
- **`extract_capabilities()`** — Parse partner capability.list responses (4 formats: flat, object, nested, combined)
- **Manifest discovery** — `$XDG_RUNTIME_DIR/ecoPrimals/*.json` fallback (rhizoCrypt S16 pattern)
- **Proptest** — 4 property-based tests for IpcPhase, extract_rpc_error, DispatchOutcome
- **`deny.toml wildcards = "deny"`** — Tightened to match ecosystem standard
- **NeuralAPI IPC evolution** — Registration/deregistration/attestation evolved to structured `Ipc { phase, message }`
- **Coverage**: 90.89% function / 88.74% line / 84.51% region (1,221 tests)
- **Source files**: 122 → 123 (added `discovery/manifest.rs`). All under 1000 lines (max: 955).

---

## v0.9.5 Completed (March 17, 2026)

- **`DispatchOutcome` wired into JSON-RPC server dispatch** — `dispatch_typed` classifies protocol vs application errors; `outcome_to_response` maps back to JSON-RPC wire format
- **`StreamItem` wired into sync module** — `push_entries_streaming` and `pull_entries_streaming` emit Data/Progress/End/Error for pipeline coordination
- **`OrExit` tracing evolution** — `eprintln!` → `tracing::error!` for structured logging consistency
- **Zero-copy sync evolution** — `clone()` eliminated in `pull_from_peer` (ownership transfer via `remove()`) and `push_entries` (try-then-own pattern)
- **Smart refactor lifecycle.rs** — 888 → 442 + 444 lines (`lifecycle_tests.rs` via `#[path]`)
- **Storage error-path coverage** — 4 new sled tests: malformed keys, missing index entries, corrupted entry bytes
- **`#[expect]` lint refinement** — Removed unfulfilled expectations in jsonrpc, sync, and certificate test modules
- **Provenance trio types inlined** — `provenance-trio-types` crate removed; wire structs owned locally in `trio_types.rs`
- **Tests**: 1,226 (up from 1,221). Source files: 125 (up from 123). All under 1000 lines.

---

## v0.9.6 Completed (March 17, 2026)

- **`capabilities.list` canonical name** — JSON-RPC dispatcher now responds to `capabilities.list` (standard), `capability.list` (legacy), and `primal.capabilities` (alias)
- **`health.liveness` response standardized** — Returns `{"status": "alive"}` per Semantic Method Naming Standard v2.1 (was `{"alive": true}`)
- **CONTEXT.md created** — AI-discoverable context block per PUBLIC_SURFACE_STANDARD (65 lines)
- **"Part of ecoPrimals" footer** — Added to README.md per PUBLIC_SURFACE_STANDARD
- **`#[allow]` → `#[expect(reason)]` migration** — 30+ test files migrated; dead attributes removed where lints didn't fire
- **Smart refactor neural_api.rs** — 871 → 384 + 489 lines (`neural_api_tests.rs` via `#[path]`)
- **Tests**: 1,226. Source files: 126. All under 1000 lines (max: 489 in test files).

---

## v0.9.7 Completed (March 23, 2026)

- **`cargo deny check` passes clean** — All advisories, bans, licenses, sources ok
- **`deny.toml` accuracy** — Advisory comments corrected (fxhash/instant → sled, bincode → direct, opentelemetry_sdk → tarpc hard dep); mdns advisories documented
- **tarpc feature trimming** — `"full"` → explicit features; drops `serde-transport-bincode` (eliminates bincode v1 via tokio-serde)
- **`publish = false`** — All workspace crates marked private; `allow-wildcard-paths` for cargo-deny
- **Sync streaming coverage** — 7 new tests; sync module: 69.00% → 90.57% line coverage
- **`#[allow(deprecated)]` → `#[expect(deprecated, reason)]`** — Remaining test-only aliases migrated
- **Hardcoding eliminated** — `HTTPS_DEFAULT_PORT`, `external::*` constants in DNS SRV mapping
- **unsafe eliminated** — `infant_discovery` tests: `temp_env::with_vars` + phased `block_on`
- **Smart refactors** — `redb_tests.rs` split by domain; `jsonrpc/tests.rs` split by domain
- **Coverage**: 92.23% line / 90.46% region / 86.52% function (1,232 tests)
- **Source files**: 124. All under 1000 lines (max: 865).

---

## v0.9.8 Completed (March 23, 2026)

- **`normalize_method()`** — Absorbed from barraCuda v0.3.7; centralizes backward-compatible method alias resolution
- **`IpcPhase` → `IpcErrorPhase`** — Renamed with backward-compatible alias for ecosystem alignment
- **`extract_rpc_result` + `extract_rpc_result_typed`** — Typed JSON-RPC result extraction utilities
- **`SyncEngine` structured errors** — Evolved from flat `Network` errors to structured `IpcErrorPhase`
- **Cast lints denied at workspace level** — `cast_possible_truncation`, `cast_sign_loss`, `cast_precision_loss`, `cast_possible_wrap` — zero violations
- **9 new proptests** — Entry and Spine invariants (hash determinism, index sensitivity, genesis)
- **Cross-ecosystem absorption** — Patterns absorbed from review of 9 springs + 10 primals
- **Tests**: 1,247. Source files: 124. All under 1000 lines.

---

## v0.9.9 Completed (March 23, 2026)

- **`ResilientSyncEngine`** — Circuit-breaker + retry wrapper for SyncEngine federation outbound IPC
- **MCP `tools.list` / `tools.call`** — Model Context Protocol support for AI agent tool discovery and invocation (11 tools with `inputSchema`)
- **10 new certificate proptests** — Creation invariants, loan holder semantics, serde roundtrip, state transitions, loan terms builder
- **Niche self-knowledge expanded** — `tools.list` and `tools.call` in METHODS, SEMANTIC_MAPPINGS, COST_ESTIMATES
- **Zero debt audit confirmed** — Zero TODOs/FIXMEs, zero production mocks, all files under 1000 lines
- **Tests**: 1,256. Source files: 124. All under 1000 lines.

---

## v0.9.11 Completed (March 23, 2026)

- **`ChainError` sentinel → `Option`** — `HashMismatch` fields evolved from `[0u8; 32]` to idiomatic `Option<EntryHash>`
- **`ResilientAdapter::execute_classified`** — Selective retry with `is_transient` closure; permanent errors fail fast
- **MCP tool completeness** — Parity test + 7 missing method mappings in `mcp_tools_list`/`mcp_tool_to_rpc`
- **NeuralAPI naming fix** — `capability.list` consistency; `deregister` uses `extract_rpc_error`
- **`hickory-resolver` feature-gated** — New `dns-srv` feature (default-on); clean build without DNS SRV
- **NDJSON streaming** — `NDJSON_PROTOCOL_VERSION` + `read_ndjson_stream` async helper
- **CC-BY-SA-4.0 headers** — All 15 specs/ + 6 root markdown documentation files
- **Tests**: 1,283 (+27). Source files: 127. All under 1000 lines (max: 878).

## v0.9.15 Completed (March 31, 2026)

- **LS-03 startup panic fixed** — Nested `block_on()` inside running async runtime → `tokio::spawn`. Provenance trio pipeline unblocked.
- **`--port` flag** — UniBin-standard CLI alias for `--jsonrpc-port`
- **Deprecated API removal** — Songbird aliases and `advertise_loamspine` removed; dead code eliminated
- **Self-knowledge enforcement** — `primal_names.rs` stripped to `SELF_ID`/`BIOMEOS`/`BIOMEOS_SOCKET_DIR` only; config `"songbird"` alias removed
- **tokio features narrowed** — `"full"` → explicit feature list for faster compile times
- **Smart refactor `jsonrpc/tests.rs`** — Split into `tests.rs` (610) + `tests_protocol.rs` (526)
- **Dependency evolution documented** — `specs/DEPENDENCY_EVOLUTION.md` tracks bincode v2, mdns evolution, sled deprecation
- **Tests**: 1,397 (+85). Source files: 129. All under 1000 lines (max: 899). Coverage: 93.96% line / 92.60% region.

## v0.9.16 Deep Debt Module Evolution Sprint 2 (April 8, 2026)

- **Smart refactor `jsonrpc/mod.rs`** (773 lines) → 3 focused modules: `wire.rs` (82 lines — wire types & error codes), `server.rs` (428 lines — TCP/UDS transport infrastructure), `mod.rs` (285 lines — dispatch logic only). Each module has a single responsibility.
- **Smart refactor `capabilities.rs`** (587 lines) → `capabilities/` directory: `mod.rs` (107 lines — identifier constants & re-exports), `types.rs` (235 lines — enum definitions & impls), `parser.rs` (129 lines — response parser), `tests.rs` (116 lines).
- **mDNS service discovery stub evolved**: `try_mdns_discovery()` from always-`None` stub to real async implementation using `spawn_blocking` + `mdns::discover::all`. Queries `_discovery._tcp.local` on LAN, parses SRV records. Feature-gated under `mdns`.
- **Lint audit**: All 2 `#[allow(` suppressions verified as correctly feature-conditional. All `#[expect(` suppressions have documented reasons.
- **Tests**: 1,304 pass. Source files: **152**. Zero clippy warnings.

## v0.9.16 Capability Wire Standard L2/L3 (April 8, 2026)

- **Wire Standard L2 compliance**: `capabilities.list` response reshaped per Capability Wire Standard v1.0. `methods` promoted from array of objects to flat string array (primary biomeOS routing signal). All 32 callable methods now advertised (previously 24, missing health/permanence/tools/identity).
- **Wire Standard L3 (composable)**: `provided_capabilities` grouping (9 domain groups), `consumed_capabilities` declaration, `cost_estimates` and `operation_dependencies` (already present, retained).
- **`identity.get` method**: New JSON-RPC method returning `{primal, version, domain, license}`. Cached via `OnceLock`. MCP tool `identity_get` added.
- **Niche evolution**: `METHODS` uses canonical `capabilities.list` (was `capability.list`). `identity.get` and `permanence.*` methods added.
- **Deploy graph aligned**: All 32 methods registered in `loamspine_deploy.toml`.
- **Tests**: 1,301 → **1,304**. Zero clippy warnings.

## v0.9.16 Deep Debt Module Evolution (April 7, 2026)

- **Smart module refactoring (6 large files)**: `types.rs` (819 lines) → `types/` directory (`mod.rs`, `anchor.rs`, `certificate.rs`, `permanent_storage.rs`, `tests.rs`). `error.rs` (777 lines) → `error/` directory (`mod.rs`, `ipc.rs`, `dispatch.rs`, `storage_ext.rs`, `tests.rs`). `neural_api.rs` (735 lines) → `neural_api/` directory (`mod.rs`, `socket.rs`, `mcp.rs`, `tests.rs`). `infant_discovery/mod.rs` → extracted `cache.rs` with `DiscoveryCache` struct. `constants/network.rs` → extracted `env_resolution.rs` for environment-reading facades. `sync/mod.rs` → extracted `streaming.rs` for NDJSON progress reporting.
- **StorageResultExt evolution**: SQLite storage modules (`entry.rs`, `certificate.rs`, `spine.rs`) migrated from standalone `to_storage_err` function to `StorageResultExt` trait methods (`.storage_err()`, `.storage_ctx("context")`). The old function is fully removed.
- **Parse helper extraction**: `integration_ops.rs` — duplicated `parse::<uuid::Uuid>().map_err(...)` and `hex_to_content_hash().map_err(...)` patterns (6 call sites) extracted to `parse_uuid()` and `parse_content_hash()` helpers.
- **Hardcoding removal**: "Songbird/Consul/etcd" literal in `niche.rs` replaced with generic "service registry (mDNS / DNS-SRV / etcd)".
- **Documentation**: Doc comments added to `sqlite/common.rs` (5 functions) and `serde_opt_bytes` module.
- **Dependency audit**: Verified `cc` crate does not leak into default build graph.
- **Coverage push**: 18 new tests — 8 `DiscoveryCache` direct unit tests, 5 `certificate_loan` expired-return paths (auto_return disabled, no-expiry, expired success, chain unwind, nonexistent), 5 tarpc server tests (config, custom-config bind, commit_session, commit_braid, get_certificate_not_found).
- **Deploy graph aligned**: `graphs/loamspine_deploy.toml` bumped from 0.9.15 to 0.9.16 with `anchor.publish`/`anchor.verify` capabilities.
- **Tests**: 1,280 → **1,298**. Source files: 136 → **148**. All under 1000 lines. Zero clippy warnings.

## v0.9.16 musl-static Deployment (April 7, 2026)

- **ecoBin deployment debt resolved** — `.cargo/config.toml` musl targets (`x86_64` + `aarch64`) with `relocation-model=static` (nestgate/biomeOS pattern). Dockerfile converted from glibc to musl-static alpine. `[profile.release]` with LTO + strip. Binary: 4.3M statically linked.
- **Showcase cleanup** — `03-songbird-discovery/` archived to fossilRecord (deprecated since v0.9.15). Renumbered `04-inter-primal/` → `03-inter-primal/`.

## v0.9.16 Storage Error Evolution & Smart Refactoring (April 6, 2026)

- **`StorageResultExt` trait** — Extension trait on `Result<T, E: Display>` providing `.storage_err()` and `.storage_ctx("context")` — eliminates ~85 verbose `.map_err(|e| LoamSpineError::Storage(e.to_string()))` closures across redb and sled backends.
- **redb.rs evolution** — 54 closure-based error conversions → trait methods (628 → 512 lines, -18%).
- **sled.rs evolution** — 31 closure-based error conversions → trait methods (519 → 461 lines, -11%).
- **Smart test extraction** — Three production files refactored below 500 lines via `#[path]` test extraction:
  - `resilience.rs`: 789 → 421 (tests → `resilience_tests.rs`)
  - `proof.rs`: 759 → 384 (tests → `proof_tests.rs`)
  - `service/mod.rs` (API): 796 → 137 (tests → `service_tests.rs`)
- **Source files**: 129 → **136**. All under 1000 lines. 1,280 tests pass. Zero clippy warnings.

## v0.9.16 Public Chain Anchor (April 6, 2026)

- **Public chain anchor** — `EntryType::PublicChainAnchor` + `AnchorTarget` enum for external provenance verification. Anchors spine state hashes to any append-only ledger (Bitcoin, Ethereum, federated spines, data commons). LoamSpine records receipts only — chain submission is capability-discovered (`"chain-anchor"`).
- **JSON-RPC + tarpc** — `anchor.publish` and `anchor.verify` wired through both transports.
- **Capability advertisement** — `"public-anchoring"` provided, `"chain-anchor"` consumed. MCP tools, neural API, niche all updated.
- **Closes Gap 4** from wetSpring NUCLEUS handoff — provenance braids are now externally verifiable; wetSpring Tier 3 `verify_url` can link to `anchor.verify`.
- **1,280 tests** — 10 new anchor tests.

## v0.9.16 Deep Debt & Zero-Copy (April 1--2, 2026)

- **Concurrent test evolution** — All seven phases completed: full suite is concurrent (**~3s**), **zero `#[serial]`** (was 121), **`serial_test`** and **`temp_env`** removed from the workspace.
- **Inner/outer function pattern** — Pure inner functions for dependency injection; public APIs remain thin env wrappers where needed.
- **Deterministic time control** — `tokio::time::pause()` + `advance()` replace wall-clock sleeps in affected tests.
- **Dynamic ports** — **`portpicker`** for integration tests to avoid port collisions under parallel execution.
- **Zero-copy evolution** — `DiscoveryClient.endpoint` → `Arc<str>`, `JsonRpcResponse.jsonrpc` → `Cow<'static, str>` (`const fn success()`), `capability_list()`/`mcp_tools_list()` → `OnceLock<Value>`, `HealthStatus` version/caps cached via `OnceLock`.
- **Hardcoding elimination** — `advertise_self` capabilities → `ADVERTISED` constants; protocol/metadata strings → `constants::protocol`/`constants::metadata` modules.
- **Structured errors** — `HealthError` enum replaces `Result<_, String>` on health checks.
- **`as` cast elimination** — All remaining production casts evolved to `From`/`try_from`.
- **1,270 tests** — Consolidated from 1,397 (redundant trivial tests dropped); all concurrent.
- **Coverage**: 91.96% line / 87.07% region / 93.39% function.

## v0.9.14 Completed (March 24, 2026)

- **`const fn` promotions** — 11 functions promoted; workspace `missing_const_for_fn` evolved from `allow` to `warn` (zero warnings)
- **`#[non_exhaustive]` forward compatibility** — 14 public enums protected against downstream match breakage
- **`DiscoveryProtocol` disambiguation** — Infant discovery naming collision resolved (46 references)
- **`TarpcServerConfig` configurable** — Hardcoded server limits evolved to runtime-configurable struct
- **Smart refactor `sled_tests.rs`** — 954 → 725 + 206 lines (certificate tests extracted as domain module)
- **Tests**: 1,312. Source files: 131. All under 1000 lines (max: 885). Coverage: 92.11% line.

## v0.9.13 Completed (March 24, 2026)

- **JSON-RPC 2.0 spec compliance** — `process_request` rewritten: validates `jsonrpc: "2.0"`, suppresses notification responses (missing/null `id`), correct `INVALID_REQUEST` error codes
- **Serialization safety** — `unwrap_or_default()` replaced with `serialize_response()` + `tracing::error!` logging fallback
- **Zero-copy Signature deserialization** — Custom `ByteBufferVisitor` eliminates `Vec<u8>` intermediary for binary codecs
- **Idiomatic API evolution** — `impl Into<String>` on `JsonRpcResponse::error()`, `TimeMarker::branch()`/`tag()`
- **Smart refactors** — `spine.rs` 854 → 438 lines, `waypoint.rs` 815 → 511 lines (test extraction, production code unchanged)
- **Tests**: 1,312. Source files: 127 → 130 (+3 extracted test files). All under 1000 lines.

## v0.9.12 Completed (March 24, 2026)

- **`#![forbid(unsafe_code)]`** — Evolved from `deny` to `forbid` workspace-wide per wateringHole ecoBin standard
- **Coverage push 89.59% → 90.02%** — 29 new tests across redb, sled, sqlite, types, trio_types, waypoint, streaming, transport
- **Clippy all-targets clean** — Fixed 8 errors in sqlite/tests.rs (unused variables, redundant closures)
- **scyBorg triple license** — Added `LICENSE-ORC` and `LICENSE-CC-BY-SA` alongside existing AGPL-3.0 `LICENSE`
- **Spec smart-refactor** — `LOAMSPINE_SPECIFICATION.md` 1521 → 1089 lines (deduplicated data model + appendix)
- **Tests**: 1,312 (+29). Source files: 124. All under 1000 lines (max: 954).

---

## v0.10.0 Targets

- **Signing capability middleware** — Signature verification on RPC layer (capability-discovered)
- **Showcase demos** — Expand from ~10% to full coverage
- **Collision layer validation** — neuralSpring experiments (Python baseline)
- **mdns crate evolution** — `mdns` 3.0 uses discontinued async-std/net2; evaluate `mdns-sd` or `hickory-resolver` mDNS (see `specs/DEPENDENCY_EVOLUTION.md`)
- **bincode v1 → v2** — Storage format migration for RUSTSEC-2025-0141 resolution (see `specs/DEPENDENCY_EVOLUTION.md`)
- **`ValidationHarness`/`ValidationSink`** — Structured validation pattern from biomeOS (partially addressed via `execute_classified` is_transient pattern in v0.9.11)

---

## v1.0.0 Targets

- **PostgreSQL storage backend** -- Implement per [specs/STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md)
- **RocksDB storage backend** -- Implement per [specs/STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md)
- **Full Universal IPC v3 compliance** -- Complete protocol alignment
- **genomeBin readiness** -- musl-static resolved (v0.9.16); remaining: checksums.toml musl triple + PIE verification
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
