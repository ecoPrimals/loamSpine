<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Implementation Status

**Current Version**: 0.9.16  
**Last Updated**: April 27, 2026

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
| [API_SPECIFICATION.md](specs/API_SPECIFICATION.md) | COMPLETE | 37 JSON-RPC methods (semantic naming), tarpc server. Spec updated to match implementation. |
| [INTEGRATION_SPECIFICATION.md](specs/INTEGRATION_SPECIFICATION.md) | COMPLETE | Provenance trio, session/braid commit. `SyncProtocol` evolved to JSON-RPC/TCP sync engine with `push_to_peer`/`pull_from_peer` and graceful fallback. `ResilientDiscoveryClient` with circuit-breaker (Closed/Open/HalfOpen, lock-free atomics) and retry policy (exponential backoff with jitter). |
| [STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md) | PARTIAL | Memory and redb (default); sled and SQLite removed (stadial compliance). PostgreSQL, RocksDB not yet implemented. |
| [SERVICE_LIFECYCLE.md](specs/SERVICE_LIFECYCLE.md) | COMPLETE | `ServiceState` enum, startup/shutdown, NeuralAPI registration, signal handling, observable state via `watch` channel. |
| [COLLISION_LAYER_ARCHITECTURE.md](specs/COLLISION_LAYER_ARCHITECTURE.md) | PROPOSAL | Research spec. Hash collision layers bridging linear ↔ DAG. Validation experiments tracked in neuralSpring. |

---

## Discovery

| Mechanism | Status |
|-----------|--------|
| Environment variables | COMPLETE |
| DNS SRV | COMPLETE |
| Service registry HTTP | COMPLETE |
| mDNS-SD | Feature-gated (`mdns-sd` 0.19 — pure Rust, no async runtime dep) |

---

## Quality Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Tests | — | 1,506 (179 source files) |
| Concurrent testing | — | All tests concurrent (zero `#[serial]`), zero flaky storage tests |
| Coverage (llvm-cov) | 90%+ | 90.92% line / 89.09% branch / 92.92% region |
| `unsafe` in production | 0 | 0 (`#![forbid(unsafe_code)]`) |
| Clippy pedantic+nursery | 0 | 0 (including `missing_const_for_fn` at warn level) |
| Doc warnings | 0 | 0 |
| Max file size | < 800 lines | 605 max production (discovery_client/mod.rs); 783 max test file (chaos.rs) |
| Source files | — | 179 `.rs` files (+ 3 fuzz targets) |
| Edition | 2024 | 2024 |
| `#[allow]` in production | 4 | 2× `clippy::wildcard_imports` (tarpc macro; `#[expect]` unfulfilled in test target) + 2× `clippy::unused_async` (feature-conditional for dns-srv/mdns; `#[expect]` unfulfilled with `--all-features`) |
| `#[allow]` in tests | 0 | 0 (all migrated to `#[expect(reason)]` or removed as unfulfilled) |
| Unused dependencies | 0 | `serde_bytes` removed (confirmed unused) |
| Workspace-centralized deps | 100% | All shared deps defined in `[workspace.dependencies]` |
| `cargo deny check` | pass | advisories ok, bans ok, licenses ok, sources ok |
| Storage/backup serde | `rmp-serde` (MessagePack) | Replaced **`bincode` v1** for on-disk and backup payloads; **RUSTSEC-2025-0141** no longer applies (see `specs/DEPENDENCY_EVOLUTION.md`). |

---

## Standards Compliance

| Standard | Status | Notes |
|----------|--------|-------|
| UniBin | PASS | `loamspine server`, `capabilities`, `socket` subcommands |
| ecoBin | PASS | Zero C deps; blake3 `pure`; musl-static local + CI; `cargo build-x64` / `build-arm64` |
| AGPL-3.0-or-later | PASS | SPDX headers on all 179 source files (+ 3 fuzz targets) |
| Scyborg triple license | PASS | `LICENSE` (AGPL-3.0), `LICENSE-ORC`, `LICENSE-CC-BY-SA` present. `CertificateType::scyborg_license()`, metadata builders, schema constants |
| Semantic naming | PASS | `capabilities.list` canonical + `primal.capabilities` alias per v2.1 standard |
| `health.liveness` | PASS | Returns `{"status": "alive"}` per Semantic Method Naming Standard v2.1 |
| PUBLIC_SURFACE | PASS | `CONTEXT.md` created, "Part of ecoPrimals" footer in README.md |
| Zero-copy | PASS | `Did` → `Arc<str>`, `DiscoveryClient.endpoint` → `Arc<str>`, `JsonRpcResponse.jsonrpc` → `Cow`, `capability_list()`/`mcp_tools_list()` → `OnceLock<Value>`, `HealthStatus` version/caps cached via `OnceLock`, `Bytes` for payloads, `[u8; 24]` stack keys, `tip_entry()` zero-copy persistence |
| MockTransport | PASS | `cfg(test|testing)` gated — no mock code in production binary |
| Socket Naming | PASS | `loamspine.sock` / `loamspine-{fid}.sock` per `{primal}-{FAMILY_ID}.sock` convention. `ledger.sock` capability symlink, `permanence.sock` legacy symlink. `BIOMEOS_INSECURE` guard. Cleanup on shutdown. |
| BTSP Phase 1 | PASS | Family-scoped socket naming (`loamspine-{family_id}.sock`), `BIOMEOS_INSECURE` guard. |
| BTSP Phase 2 | PASS | Handshake-as-a-service via BTSP provider JSON-RPC. UDS listener gates on BTSP when `FAMILY_ID` is set. 4-step handshake (ClientHello/ServerHello/ChallengeResponse/HandshakeComplete). `BTSP_NULL` cipher only (Phase 3 encryption pending BTSP provider session key propagation). |
| File size limit | PASS | All source files under 1000 lines. |
| Stadial parity gate | PASS | April 16, 2026 — storage backends reduced to redb (default) + memory; sled and SQLite removed; `hickory-resolver` 0.24→0.26; lockfile cleared of sled/libsqlite3-sys/rusqlite/instant/fxhash; `cargo deny` bans + advisories clean; dyn audit non-blocking (72 total usages). |

---

## v0.9.16 Deep Debt Execution Pass (April 15, 2026)

- **Smart refactor large test files**: 4 test files >800 lines refactored by domain cohesion:
  - `sync/tests.rs` (885→692) + `tests_resilient.rs` (resilient sync + wire edge cases)
  - `tarpc_server_tests.rs` (806→306) + `tarpc_server_tests_compound.rs` (multi-step operations)
  - `cli_signer_tests.rs` (867→320) + `cli_signer_tests_integration.rs` (sign/verify flows, DynSigner/DynVerifier)
  - `discovery_client/tests.rs` (819→486) + `tests_transport.rs` (transport layer, status codes, resilient client)
- **Coverage push to 90.20%**: Targeted tests for:
  - `infant_discovery/backends.rs`: `capability_to_srv_name` catch-all arms (hyphenated, empty, single-hyphen)
  - `infant_discovery/mod.rs`: `from_explicit` branches, `from_env_or_default`, env_overrides, `InfantDiscovery::new()`
  - `neural_api/mod.rs`: `register_with_neural_api` (no socket), `deregister_from_neural_api`, capability/identity accessors, `validate_security_config_from_env`
  - `service/lifecycle.rs`: All `ServiceState` display+serde variants, start→running transition, stop→stopped transition
- **Spec doc fixes**:
  - `LOAMSPINE_SPECIFICATION.md`: Removed orphaned `ChainType` enum (stale markdown glitch from previous extraction)
  - `ARCHITECTURE.md`: Crate layout updated to reflect actual directory structure (module dirs, bin/ layout, fuzz/, graphs/)
  - `PURE_RUST_RPC.md`: tarpc version `0.34` → `0.37` with actual feature flags
- **All gates green**: `cargo fmt` PASS, `cargo clippy --all-targets --all-features -D warnings` PASS (0 warnings), `cargo doc` PASS (0 warnings), `cargo test` PASS, `cargo deny check` PASS.

## v0.9.16 Comprehensive Audit & Evolution Pass (April 12, 2026)

- **Blocking redb in async fixed**: All `RedbSpineStorage`, `RedbEntryStorage`, `RedbCertificateStorage` trait impls now use `tokio::task::spawn_blocking` to prevent reactor stalls from synchronous disk I/O.
- **Resilience circuit-breaker semantics corrected**: `execute_classified` no longer counts permanent errors against the circuit breaker. Only transient failures increment failure count, preventing premature circuit trips on non-retryable errors.
- **RetryPolicy doc/code mismatch fixed**: Documentation now correctly states 10,000ms max delay (was 5,000ms).
- **Type safety evolved**: `PeerId` evolved from bare `type PeerId = String` to `PeerId(Arc<str>)` newtype with `Borrow<str>`, `From<&str>`, `From<String>`, `Display`. `LogLevel` evolved from loose `String` to validated `LogLevel` enum with serde + Display.
- **Zero-copy BTSP frames**: `read_frame` returns `bytes::Bytes` instead of `Vec<u8>` for zero-copy downstream processing.
- **BTSP magic numbers eliminated**: Handshake RPC IDs extracted to `rpc_id::SESSION_CREATE`, `SESSION_VERIFY`, `NEGOTIATE` constants.
- **NDJSON backpressure**: `read_ndjson_stream` now enforces `DEFAULT_NDJSON_MAX_ITEMS` (10,000) limit. New `read_ndjson_stream_bounded` for explicit control.
- **Owner index for O(1) lookup**: `LoamSpineService::ensure_spine` uses `owner_index: HashMap<Did, SpineId>` instead of scanning all spines.
- **Storage count methods evolved**: Redb `spine_count`, `entry_count`, `certificate_count` return `LoamSpineResult<usize>` instead of silently returning 0 on errors.
- **SpineBuilder clone cleanup**: `with_type`, `personal`, `waypoint` no longer produce redundant clones.
- **UniBin `--abstract` flag**: Added `--abstract` CLI flag for Linux abstract UDS namespace support per UniBin standard.
- **CLI config merge**: `run_server` now merges CLI-resolved ports/bind into `LoamSpineConfig.discovery` endpoints.
- **tarpc transport docs corrected**: All references to "binary RPC" updated to "structured RPC (JSON-over-TCP)" to accurately reflect the `Json::default` serde transport.
- **SPDX headers**: Added to 2 remaining test files (`error/tests.rs`, `types/tests.rs`). All 173 `.rs` files now have SPDX.
- **HTTP/1.1 keep-alive (connection-close fix)**: JSON-RPC TCP server evolved from single-shot HTTP (`Connection: close` after every response) to persistent HTTP/1.1 keep-alive loop. Multiple JSON-RPC requests on a single TCP connection now work without reconnection. Closes primalSpring audit item: "loamSpine connection closes after first response". `Connection: close` is still honored when the client requests it.
- **BTSP provider decoupled from hardcoded primal name**: `BTSP_PROVIDER_PREFIX` evolved from hardcoded `"beardog"` to env-configurable `BTSP_PROVIDER` with `"beardog"` default. `BtspHandshakeConfig.beardog_socket` → `provider_socket`. Socket name functions accept provider override parameter. All BTSP callers use capability-agnostic field names.
- **Smart test extraction (5 files)**: Inline `#[cfg(test)] mod tests` extracted to sibling `#[path]` modules: `streaming.rs` (354→203), `health.rs` (482→347), `service/mod.rs` (438→277), `config.rs` (370→285), `lib.rs` (532→374). Production code cohesion preserved.
- **Stale Songbird references removed**: All production doc comments referencing deprecated Songbird discovery primal evolved to generic capability-based language. Only test examples of `address_env_var("songbird")` remain (demonstrating the function works with any primal name).
- **Doc warning fixed**: Broken `read_ndjson_stream_with` intra-doc link → `read_ndjson_stream_bounded`.
- **LD-09 TCP opt-in (port 8080 crash fix)**: `loamspine server` no longer unconditionally binds `0.0.0.0:8080` for HTTP JSON-RPC. TCP transports (tarpc + JSON-RPC TCP) are now opt-in via `--port`/`--tarpc-port` CLI flags or `LOAMSPINE_JSONRPC_PORT`/`LOAMSPINE_TARPC_PORT`/`USE_OS_ASSIGNED_PORTS` env vars. UDS socket is always the primary transport. Follows ToadStool/barraCuda pattern. Resolves primalSpring audit item LD-09.
- **Root docs reconciled**: README, CONTEXT, CONTRIBUTING metrics aligned with STATUS.md truth (1,390 tests, 176 source files). Stale Songbird references removed from deploy graph and CONTEXT. Coverage badges updated.
- **traits/mod.rs test extraction**: Inline `#[cfg(test)] mod tests` (167 lines) extracted to `traits/mod_tests.rs`. Production module: 446→279 lines.
- **Magic number timeouts named**: `transport/http.rs` (`CONNECT_TIMEOUT`, `READ_TIMEOUT`), `infant_discovery/mod.rs` (`DNS_SRV_TIMEOUT`), `infant_discovery/backends.rs` (`MDNS_TIMEOUT`). All bare Duration literals in production code replaced with named constants.
- **Clone audit clean**: All production `.clone()` calls verified as Arc-based O(1) or structurally necessary. No unnecessary allocations in hot paths.
- **Capability-domain symlink**: `ledger.sock → loamspine.sock` created on bind, removed on shutdown. Enables `by_capability = "ledger"` routing in deploy graphs. Matches BearDog (`crypto.sock`), Songbird (`network.sock`), coralReef (`shader.sock`/`device.sock`) pattern. Socket naming now: primary `loamspine.sock`, capability `ledger.sock`, legacy `permanence.sock`. Wire Standard promoted to **full L3** in `CAPABILITY_WIRE_STANDARD.md` and `ECOSYSTEM_COMPLIANCE_MATRIX.md`.
- **plasmidBin metadata reconciled**: Version 0.9.13→0.9.16, domain `lineage`→`permanence`, capabilities reconciled to 22 live methods matching `niche.rs`, TCP opt-in flag, socket naming updated.
- **Ecosystem docs aligned**: `CAPABILITY_WIRE_STANDARD.md` loamSpine row promoted to L2 ✓ L3 ✓. `ECOSYSTEM_COMPLIANCE_MATRIX.md` transport line updated. `plasmidBin/manifest.lock` version and domain corrected.
- **Hardcoded port constants decoupled from production callers**: `DiscoveryConfig::default()` evolved to use `env_resolution` module (reads `LOAMSPINE_*_PORT` / `*_PORT` env vars) instead of raw `DEFAULT_TARPC_PORT`/`DEFAULT_JSONRPC_PORT` constants. `discovery_client::advertise_self()` port fallbacks similarly evolved. Constants remain only in doc examples and dev-mode fallback (cfg-gated to `debug_assertions`).
- **`health.check` accepts empty/null params (primalSpring audit)**: `HealthCheckRequest.include_details` now `#[serde(default)]` — defaults to `false` when absent. JSON-RPC `deser()` normalizes `null` params to `{}` per JSON-RPC 2.0 §4.2. Consumers can call `health.check` with `{}`, `null`, or omitted params without error. 2 new tests.
- **Discovery string literals → named constants**: `discovered_via` field values (`"environment"`, `"dns-srv"`, `"mdns"`) moved to `constants::discovery_method` module. DNS SRV metadata keys (`"priority"`, `"weight"`, `"target"`, `"port"`) moved to `constants::srv_metadata`. Eliminates scattered string literals, enables typo detection at compile time.
- **Witness default constants**: `DEFAULT_WITNESS_KIND` ("signature") and `DEFAULT_WITNESS_ENCODING` ("hex") extracted as named constants in `trio_types`. Serde default functions now reference the constants.
- **Test file smart-refactoring (>800L)**: `tests_protocol.rs` (956L) → `tests_protocol_transport.rs` + `tests_protocol_wire.rs`. `discovery/tests.rs` (899L) → `tests_registry.rs` + `tests_attestation.rs`. All splits by cohesive concern, not arbitrary line count.
- **Arc<str> for async retry closures**: `ResilientDiscoveryClient::discover_capability` and `advertise_self` evolved from `String` cloning to `Arc<str>` — O(1) clone per retry iteration instead of O(n) allocation.
- **`.into()` modernization**: Error constructors (`LoamSpineError::CapabilityUnavailable`, `::Network`, `::Internal`) and wire messages updated from `"literal".to_string()` to `"literal".into()` where the target type is `String`.
- **`provenance.commit` alias (primalSpring benchScale audit)**: `normalize_method` now maps `provenance.commit` → `session.commit`. primalSpring exp084 replay attack scenario can now reach LoamSpine's session commit handler instead of getting `-32601 Method not found`. 1 new integration test.
- **BTSP provider decoupling**: `beardog_client.rs` → `provider_client.rs`. `beardog_call` → `provider_call`. `beardog_socket` params → `provider_socket`. All "BearDog" error messages and doc comments evolved to "BTSP provider". `BEARDOG_SOCKET` env → `BTSP_PROVIDER_SOCKET` (backward compat preserved). `beardog_socket()` accessor removed (was unused). Zero compile-time coupling to BearDog identity.
- **`.into()` modernization**: `DEFAULT_BTSP_PROVIDER_PREFIX.to_string()` → `.into()`. `"LoamSpine".to_string()` → `.into()` in config default. `"Storage backend unavailable".to_string()` → `.into()` in health readiness.
- **All gates green**: `cargo fmt` PASS, `cargo clippy --all-targets --all-features -D warnings` PASS (0 warnings), `cargo doc` PASS (0 warnings), `cargo test` PASS (1,442 tests, 0 failures), `cargo deny check` PASS.

## v0.9.16 Deep Debt Overhaul & Dependency Evolution (April 11, 2026)

- **BTSP challenge evolution**: `generate_challenge_placeholder()` (time-nanos deterministic) replaced with `generate_challenge()` using `blake3(uuid_v7_a || uuid_v7_b)` — 148+ bits of OS-sourced entropy per challenge, zero new dependencies.
- **Smart refactor `btsp.rs`**: 697-line monolith → `btsp/` directory module with 5 focused submodules: `wire.rs` (types), `config.rs` (env-driven config + BearDog resolution), `frame.rs` (length-prefixed I/O), `beardog_client.rs` (JSON-RPC delegation), `handshake.rs` (protocol logic). All production modules under 500 lines.
- **Unused dependency removed**: `serde_bytes` (crate `0.11`) removed from `loam-spine-core` — confirmed unused in all `.rs` files via grep + Cargo analysis.
- **Workspace dependency centralization**: `loam-spine-core`, `loam-spine-api` (internal crates), `tarpc`, `futures`, `clap`, `bytes`, `url`, `bincode` promoted to `[workspace.dependencies]`. Member crates reference via `workspace = true`. Eliminates duplicated version pins and path declarations.
- **Storage test isolation fixed**: All sled two-phase-open tests (corruption, malformed keys, cross-spine iteration) rewritten to use `from_db()` constructors — eliminates sled lock contention under parallel execution. `SledSpineStorage::from_db`, `SledEntryStorage::from_db`, `SledCertificateStorage::from_db` constructors added. SQLite `open_connection` evolved to enable WAL mode + 5s busy timeout. redb tests migrated from manual `remove_dir_all` to `tempfile::tempdir()` lifecycle. **3 consecutive full parallel runs: 1,504 tests, 0 failures.**
- **`#[allow]` audit**: 4 production `#[allow]` documented as irreducible — `#[expect]` causes `unfulfilled-lint-expectations` in test/all-features targets due to macro/feature interaction.
- **Tests**: 1,373 → **1,504**. Source files: 167 → **169** (btsp module split +6, btsp.rs deleted -1, net +5). Zero clippy warnings. Zero doc warnings. `cargo deny check` pass.

## v0.9.16 Deep Debt Cleanup & Evolution Pass (April 9, 2026)

- **Smart refactor `infant_discovery/mod.rs`**: Extracted mDNS backend functions (`mdns_discover_impl`, `parse_mdns_response`, `capability_to_srv_name`) into `backends.rs` (158 lines). Module reduced 711→570 lines. All production files now under 700 lines.
- **Zero-copy JSON-RPC extraction**: Eliminated `.clone()` in `extract_rpc_result_typed` and `parse_beardog_response` — replaced `serde_json::from_value(result.clone())` with borrowing `T::deserialize(result)`. Zero allocation on hot JSON-RPC deserialization path.
- **Resilience retry path**: Removed `err_msg.clone()` from retry loop — log then move instead of clone-for-log.
- **tarpc/opentelemetry advisory documented**: Added `RUSTSEC-2026-0007` tracking to `DEPENDENCY_EVOLUTION.md` with upstream blocker and mitigation plan.
- **Coverage expansion**: 10 new tests — 6 for `EphemeralProvenance`/`Attestation` serde roundtrips (temporal module previously uncovered), 4 for `StorageResultExt` trait (error module previously untested directly).
- **Tests**: 1,363 → **1,373**. Source files: 166 → **167**. Zero clippy warnings. Zero doc warnings. All files under 1000 lines.

## v0.9.16 BTSP Phase 2 Handshake Integration (April 9, 2026)

- **BTSP handshake-as-a-service**: New `btsp` module in `loam-spine-core` implements the consumer side of BTSP Phase 2. LoamSpine delegates all cryptographic operations to BearDog via JSON-RPC (`btsp.session.create`, `btsp.session.verify`, `btsp.negotiate`). Zero crypto dependencies added.
- **UDS listener gated**: `run_jsonrpc_uds_server` accepts `Option<BtspHandshakeConfig>`. When `BIOMEOS_FAMILY_ID` is set (non-default), every incoming UDS connection must complete the 4-step BTSP handshake before JSON-RPC methods are exposed. Without `FAMILY_ID`, behavior is identical to pre-BTSP (raw newline-delimited JSON-RPC).
- **Wire format**: 4-byte big-endian length-prefixed frames per `BTSP_PROTOCOL_STANDARD.md`. Wire types: `ClientHello`, `ServerHello`, `ChallengeResponse`, `HandshakeComplete`, `HandshakeError`.
- **Capability-discovered BearDog**: Socket path resolved via `BEARDOG_SOCKET` env, `$BIOMEOS_SOCKET_DIR/beardog-{family_id}.sock`, or platform fallback. No primal names hardcoded.
- **Consumed capability registered**: `"btsp"` added to `CONSUMED_CAPABILITIES`, `DEPENDENCIES`, `capabilities::identifiers::external`, and `primal-capabilities.toml`.
- **28 new tests**: Pure function tests (config derivation, socket resolution, frame I/O), wire type serde roundtrips, mock BearDog integration tests (success, verify rejection, cipher rejection, BearDog unavailable, version mismatch).
- **Resolves**: primalSpring audit BTSP debt ("BTSP handshake stub always returns Err — wired as a function but not connected to BearDog IPC yet").

## v0.9.16 Deep Debt Module Evolution (April 7, 2026)

- **Smart module refactoring (6 large files)**: `types.rs` (819→directory), `error.rs` (777→directory), `neural_api.rs` (735→directory), `infant_discovery/mod.rs` (cache extraction), `constants/network.rs` (env_resolution extraction), `sync/mod.rs` (streaming extraction). All refactored by domain semantics, not arbitrary line splits.
- **StorageResultExt for SQLite**: `to_storage_err` standalone function eliminated; all 3 SQLite modules (`entry.rs`, `certificate.rs`, `spine.rs`) migrated to `.storage_err()` / `.storage_ctx()` trait methods.
- **Parse helper DRY**: `integration_ops.rs` — 6 duplicated parse-and-map-err patterns extracted to `parse_uuid()` and `parse_content_hash()`.
- **Hardcoding removal**: "Songbird" literal removed from `niche.rs` external dependency description.
- **Deploy graph aligned**: `graphs/loamspine_deploy.toml` 0.9.15 → 0.9.16 with `anchor.publish`/`anchor.verify`.
- **Coverage push**: 18 new tests across `DiscoveryCache`, `certificate_loan` expired paths, tarpc server delegation.
- **Tests**: 1,280 → **1,298**. Source files: 136 → **148**. Zero clippy warnings. Zero `cc` crate in build graph.

## v0.9.16 musl-static Deployment (April 7, 2026)

- **ecoBin deployment debt resolved**: `.cargo/config.toml` now defines `[target.x86_64-unknown-linux-musl]` and `[target.aarch64-unknown-linux-musl]` with `musl-gcc` linker, `+crt-static`, `-static`, `relocation-model=static` (prevents musl ≤1.2.2 static-PIE segfault). Matches nestgate/biomeOS reference pattern.
- **Dockerfile converted to musl-static**: Builder stage installs `musl-tools`, builds with `--target x86_64-unknown-linux-musl`. Runtime stage: `alpine:3.20` (was `debian:bookworm-slim`). Binary is fully statically linked, stripped, 4.3M.
- **Release profile added**: `[profile.release]` with `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true` — aligns with rhizoCrypt.
- **Convenience aliases**: `cargo build-x64` and `cargo build-arm64` for one-command musl builds.
- **Verified**: `file` confirms `ELF 64-bit LSB executable, x86-64, statically linked, stripped`; `ldd` confirms `not a dynamic executable`.
- **Showcase cleanup**: `03-songbird-discovery/` and songbird helper scripts archived to `fossilRecord/loamspine/showcase-songbird-apr2026/` — deprecated since v0.9.15 API removal.
- **Unblocks**: benchScale container deployment and plasmidBin musl-first harvest for loamSpine.

## v0.9.16 Storage Error Evolution & Smart Refactoring (April 6, 2026)

- **`StorageResultExt` trait**: New extension trait on `Result<T, E: Display>` providing `.storage_err()` and `.storage_ctx("context")` methods. Eliminates ~85 verbose `.map_err(|e| LoamSpineError::Storage(e.to_string()))` closures.
- **redb.rs evolution**: 54 closure-based error conversions replaced with trait methods (628 → 512 lines, -18%).
- **sled.rs evolution**: 31 closure-based error conversions replaced with trait methods (519 → 461 lines, -11%).
- **Smart test extraction**: Three production files refactored below 500 lines via `#[path]` test extraction:
  - `resilience.rs`: 789 → 421 lines (368 lines → `resilience_tests.rs`)
  - `proof.rs`: 759 → 384 lines (375 lines → `proof_tests.rs`)
  - `service/mod.rs` (API): 796 → 137 lines (659 lines → `service_tests.rs`)
- **Source files**: 129 → **136** `.rs` files. All 1,280 tests pass. Zero clippy warnings. Zero doc warnings.

## v0.9.16 Public Chain Anchor (April 6, 2026)

- **External provenance verification**: `EntryType::PublicChainAnchor` + `AnchorTarget` enum. Records anchor receipts from any append-only ledger (Bitcoin, Ethereum, federated spines, data commons). Actual chain submission is delegated to a capability-discovered `"chain-anchor"` primal.
- **JSON-RPC + tarpc**: `anchor.publish` and `anchor.verify` methods wired through both transports.
- **Capability advertisement**: `"public-anchoring"` provided capability, `"chain-anchor"` consumed capability. MCP tools, neural API, niche self-knowledge all updated.
- **Closes Gap 4** from wetSpring NUCLEUS handoff: provenance braids are now externally verifiable. wetSpring's Tier 3 `verify_url` can link to `anchor.verify`.
- **10 new tests**: Entry serde roundtrip, domain, waypoint exclusion, service methods (roundtrip, latest, missing spine, non-anchor, target serde), JSON-RPC dispatch (publish + verify).
- **Tests**: 1,270 → **1,280**. All checks pass (fmt, clippy, test, doc).

## v0.9.16 Deep Debt Evolution & Zero-Copy Hardening (April 1--2, 2026)

- **Zero-copy evolution**: `DiscoveryClient.endpoint` evolved from `String` to `Arc<str>` for O(1) clone in resilient adapter retry loops. `JsonRpcResponse.jsonrpc` evolved from `String` to `Cow<'static, str>` with `success()` promoted to `const fn`. `capability_list()` and `mcp_tools_list()` cached with `OnceLock` — return `&'static serde_json::Value`. `HealthStatus` version/capabilities cached with `OnceLock` via `cached_version()`/`cached_capabilities()`.
- **Hardcoding elimination**: `advertise_self` capabilities replaced with `capabilities::identifiers::loamspine::ADVERTISED`. Protocol identifiers centralized in `constants::protocol::{TARPC, JSONRPC, HEALTH_PATH}`. Metadata values centralized in `constants::metadata::{LANGUAGE, RPC_STYLE, STORAGE_BACKEND}`.
- **Structured errors**: `HealthError` enum with `thiserror` replaces `Result<_, String>` on `check_health`/`check_readiness`.
- **`as` cast elimination**: All remaining production `as usize`/`as char`/`as u64` casts evolved to `usize::from()`, `char::from()`, `u64::try_from()`.
- **Test extraction**: `transport/neural_api.rs` inline tests → `transport/neural_api_tests.rs` (328 lines).
- **`primal-capabilities.toml`**: Version bumped to 0.9.16.
- **Coverage**: 91.96% line / 87.07% region / 93.39% function. All checks pass (fmt, clippy, test, doc, deny).

## v0.9.16 Concurrent Test Evolution (April 1, 2026)

- **Seven-phase concurrent test evolution completed**: Removed workspace dependencies on `serial_test` and `temp_env`; **`#[serial]` tests: 121 → 0** across the codebase.
- **Inner/outer function pattern**: Pure `resolve_*` / `_from` / `_with` inner functions for config injection; thin env-reading wrappers as the outer public API (`constants/network.rs`, `neural_api.rs`, infant discovery, `manifest.rs`, `cli_signer.rs`, `lifecycle.rs`).
- **`DiscoveryConfig.env_overrides`**: `HashMap<String, String>` for test config injection without mutating process environment.
- **`CliSigner::discover_binary_from`**: Pure function with explicit `signer_path` / `bins_dir` parameters.
- **Deterministic timing**: Eight test sites that used sleeps now use `tokio::time::pause()` + `advance()` for deterministic async time.
- **Dynamic ports**: `portpicker` crate for integration tests — no fixed-port collisions under full concurrency.
- **Test consolidation**: **1,397 → 1,270** tests (trivial env-read tests removed). **Source files**: 129 (unchanged). **Max file**: 899 lines (unchanged). Full suite runs in **~3 seconds** with all tests concurrent.
- **Quality**: Clippy pedantic + nursery, **0 warnings** (`-D warnings`). All tests pass on three consecutive runs.

## v0.9.15 Deep Debt & Evolution: LS-03 Fix, Self-Knowledge, Coverage Push (March 31, 2026)

- **LS-03 startup panic fixed**: `block_on()` inside running async runtime → `tokio::spawn`. Provenance trio pipeline unblocked.
- **`--port` flag**: UniBin-standard CLI alias for `--jsonrpc-port`.
- **Deprecated API removal**: Songbird aliases (`discover_from_songbird`, `advertise_to_songbird`, `heartbeat_songbird`) and `advertise_loamspine` removed with all tests.
- **Self-knowledge enforcement**: `primal_names.rs` stripped to `SELF_ID`, `BIOMEOS`, `BIOMEOS_SOCKET_DIR` only — external primal names removed. Serde `"songbird"` alias removed from config.
- **tokio features narrowed**: `"full"` → explicit feature list — faster compile times, smaller dependency footprint.
- **Smart refactor `jsonrpc/tests.rs`**: Extracted `tests_protocol.rs` (526 lines) for protocol-level tests. Both files under 1,000 lines.
- **Dependency evolution documented**: `specs/DEPENDENCY_EVOLUTION.md` tracks completed **`bincode` v1 → `rmp-serde` (MessagePack)** migration, `mdns` crate evolution, `sled → redb` completion (sled later removed in stadial gate).
- **85 new tests**: UDS server, protocol-level JSON-RPC, lifecycle state transitions, discovery manifest, CLI signer, neural API edge cases.
- **Tests**: 1,312 → **1,397** (+85). **Source files**: 131 → **129**. All under 1000 lines (max: 899). **Coverage**: 93.96% line / 92.60% region. Clippy pedantic+nursery: 0 warnings. Doc warnings: 0.

## v0.9.14 Deep Audit Execution: Idiomatic Evolution & Forward Compatibility (March 24, 2026)

- **`const fn` promotions**: 11 functions promoted to `const fn` across `UsageSummary::is_empty`, `default_contribution_weight`, `AttestationRequirement::is_required`, `RelendingChain::new`, `CircuitBreaker::new`, `RetryPolicy::new`/`max_retries_value`, `ResilientAdapter::new`, `ExpirySweeper::new`, `StreamItem::data`, `hash_u32`. Workspace lint `missing_const_for_fn` evolved from `allow` to `warn` — zero warnings.
- **`#[non_exhaustive]` forward compatibility**: Added to 14 public enums: `LoamSpineError`, `IpcErrorPhase`, `ApiError`, `ServerError`, `CircuitState`, `SpineState`, `PrimalState`, `HealthStatus`, `ServiceState`, `CapabilityStatus`, `PropagationPolicy`, `AttestationRequirement`, `DepartureReason`, `SliceOperationType`. Cross-crate `From<LoamSpineError>` match updated with catch-all arm.
- **`DiscoveryProtocol` disambiguation**: Infant discovery `DiscoveryMethod` renamed to `DiscoveryProtocol` to resolve naming collision with `config::DiscoveryMethod` (46 references across 3 files).
- **`TarpcServerConfig` configurable**: Hardcoded `TARPC_MAX_CONCURRENT_REQUESTS` and `TARPC_MAX_CHANNELS_PER_IP` evolved to `TarpcServerConfig` struct with `run_tarpc_server_with_config()`. Backward-compatible `run_tarpc_server()` uses defaults.
- **Smart refactor `sled_tests.rs`**: 954 → 725 lines + `sled_tests_certificate.rs` (206 lines). Certificate storage tests extracted as cohesive domain module following established `redb_tests_cert_errors.rs` pattern.
- **Tests**: 1,312 (unchanged). **Source files**: 130 → **131** (+1 extracted test file). All under 1000 lines (max: 885 in `sync/tests.rs`). **Coverage**: 92.11% line / 90.33% region / 87.83% function. Clippy pedantic+nursery: 0 warnings. Doc warnings: 0.

## v0.9.13 JSON-RPC 2.0 Compliance, Zero-Copy & Smart Refactors (March 24, 2026)

- **JSON-RPC 2.0 spec compliance**: `process_request` rewritten to parse as `Value` first, enabling proper notification detection (no response for missing/null `id`), `jsonrpc: "2.0"` version validation (`INVALID_REQUEST` for mismatches), and correct error codes (empty batch returns `-32600` not `-32700`). Batch items also validated.
- **Serialization safety**: Replaced all `serde_json::to_vec(&response).unwrap_or_default()` (silent failure) with `serialize_response()` helper that logs via `tracing::error!` and returns a hard-coded JSON-RPC internal error fallback.
- **HTTP notification support**: Notifications return `204 No Content` over HTTP, empty response on raw TCP. Transport-level replies are clean.
- **Zero-copy Signature deserialization**: Custom `ByteBufferVisitor` replaces `Vec<u8>` intermediary in `Signature::deserialize`. Binary codecs (bincode, postcard) now receive owned bytes directly via `visit_byte_buf`; JSON falls back through `visit_seq` as before.
- **Idiomatic API evolution**: `JsonRpcResponse::error()` accepts `impl Into<String>` (eliminates `.to_string()` on literal messages). `TimeMarker::branch()` and `tag()` accept `impl Into<String>` for name/created_by parameters.
- **Smart refactors**: `spine.rs` 854 → **438 lines** (tests → `spine_tests.rs` + `spine_proptests.rs`). `waypoint.rs` 815 → **511 lines** (tests → `waypoint_tests.rs`). Production code cohesion preserved; only test modules extracted.
- **Tests**: 1,312 (unchanged). **Source files**: 127 → **130** (+3 extracted test files). All under 1000 lines (max: 954). Clippy pedantic+nursery: 0 warnings. Doc warnings: 0.

## v0.9.12 Deep Audit Execution & Coverage Push (March 24, 2026)

- **`#![forbid(unsafe_code)]`**: Evolved from `#![deny(unsafe_code)]` to `#![forbid(unsafe_code)]` in `loam-spine-core` and workspace-level lints per wateringHole ecoBin standard. Zero unsafe code in entire codebase.
- **Coverage 89.59% → 90.02%**: 29 new tests across redb (corrupt entry via index, short index key), sled (corrupt data in get_spine/get_entry/get_certificate, cross-spine iteration), sqlite (temporary() constructors, flush, get_entry None), types (From<String>, Signature::default, Timestamp::Display, ByteBuffer from &str), trio_types (default weight, as_str accessors), waypoint (RelendingChain::new, DepartureReason::Relend display, SliceOperationType::name variants), streaming (empty line skipping), transport (from_bytes zero-copy, SuccessTransport::default).
- **Clippy all-targets clean**: Fixed 8 errors in sqlite/tests.rs (2 unused variables → underscore prefix, 6 redundant closures → `PoisonError::into_inner` method reference).
- **Scyborg triple license files**: Added `LICENSE-ORC` and `LICENSE-CC-BY-SA` for complete ORC + CC-BY-SA-4.0 compliance alongside existing AGPL-3.0 `LICENSE`.
- **Spec smart-refactor**: `LOAMSPINE_SPECIFICATION.md` reduced from 1521 → 1089 lines by deduplicating §3 Data Model (430 lines of struct definitions → summary + cross-reference to `DATA_MODEL.md`) and Appendix A (→ cross-reference to `CERTIFICATE_LAYER.md`). Fixed duplicate §3 numbering. `DATA_MODEL.md` (1441) and `CERTIFICATE_LAYER.md` (1133) kept intact as cohesive single-domain reference specs.
- **Tests**: 1,283 → **1,312** (+29). **Source files**: 124. All under 1000 lines (max: 954 in `sled_tests.rs`).

## v0.9.11 Feature gating, MCP completeness & streaming evolution (March 23, 2026)

- **`ChainError` sentinel hash → `Option<EntryHash>`**: `HashMismatch { expected, actual }` fields evolved from `[0u8; 32]` sentinel to idiomatic `Option<EntryHash>`.
- **`ResilientAdapter::execute_classified`**: New method accepting `is_transient` closure for selective retries — permanent errors fail fast, transient errors trigger backoff.
- **MCP tool completeness**: New test enforcing parity between `capability_list()` and MCP tool mappings; 7 missing methods added to `mcp_tools_list` and `mcp_tool_to_rpc` (`spine.seal`, `entry.get_tip`, `certificate.verify`, `slice.anchor`, `session.commit`, etc.).
- **NeuralAPI naming fix**: `capability_list` → `capability.list` consistency in `mcp_tool_to_rpc`; `deregister_from_neural_api` now uses `extract_rpc_error` for structured error handling.
- **`hickory-resolver` feature-gated**: New `dns-srv` feature (default-on); builds clean with `--no-default-features --features redb-storage`. Reduces binary size and compile time for deployments without DNS SRV.
- **NDJSON streaming evolution**: `NDJSON_PROTOCOL_VERSION` constant + `read_ndjson_stream` async helper for `StreamItem` parsing from any `AsyncBufRead`.
- **CC-BY-SA-4.0 headers**: All 15 `specs/` + 6 root markdown files now have correct scyBorg documentation license SPDX headers.
- **`mcp_tools_list` lint**: `#[expect(clippy::too_many_lines)]` for declarative MCP schema (justified — pure data).
- **Tests**: 1,256 → **1,283** (+27). **Source files**: 127 (unchanged). All under 1000 lines (max: 878 in `sync/tests.rs`).

## v0.9.10 Deep debt resolution & lint pedantry (March 23, 2026)

- **Doc warnings fixed**: 3 unresolved intra-doc links in `sync/mod.rs` and `discovery_client/mod.rs` resolved with fully-qualified `crate::` paths.
- **`#[allow]` → `#[expect(reason)]` in tests**: 20 test/bench/example files migrated. Unfulfilled expectations removed (lints that don't fire in test/bench/example targets are not suppressed). Production `#[allow]` for tarpc wildcard imports documented.
- **Hardcoded `/tmp/` paths eliminated**: 6 test paths in `lib.rs` evolved to `tempfile::tempdir()` for CI safety and parallel test isolation.
- **STATUS.md accuracy**: `#[allow]` metric corrected to reflect 2 justified production exceptions.
- **`cargo deny check`**: Verified passing via local binary (advisories ok, bans ok, licenses ok, sources ok).
- **Tests**: 1,256 (unchanged). **Coverage**: unchanged. **Source files**: 124 (unchanged). All under 1000 lines.

## v0.9.9 ResilientSyncEngine, MCP tools, certificate proptests (March 23, 2026)

- **`ResilientSyncEngine`**: SyncEngine wrapped with circuit-breaker + retry for federation outbound IPC.
- **MCP `tools.list` / `tools.call`**: AI agents (Squirrel/biomeOS) can now discover and invoke LoamSpine operations via Model Context Protocol.
- **Certificate lifecycle proptests**: 10 new property tests (creation invariants, loan holder semantics, serde roundtrip, state transitions, loan terms builder).
- **Niche self-knowledge**: `tools.list` and `tools.call` added to METHODS, SEMANTIC_MAPPINGS, and COST_ESTIMATES.
- Zero TODOs, zero mocks in production, all files under 1000 lines, zero hardcoded addresses.

## v0.9.8 IPC alignment, cast lints & ecosystem absorption (March 23, 2026)

- **`normalize_method()` absorbed** from barraCuda v0.3.7.
- **`IpcPhase` → `IpcErrorPhase`**: Renamed with backward-compatible alias.
- **`extract_rpc_result` + `extract_rpc_result_typed`**: New utilities for typed RPC result handling.
- **`SyncEngine`**: Evolved from flat `Network` errors to structured `IpcErrorPhase`.
- **Cast lints denied at workspace level**: `cast_possible_truncation`, `cast_sign_loss`, `cast_precision_loss`, `cast_possible_wrap` — zero violations.
- **Proptest**: 9 new property tests for Entry and Spine invariants.
- **Ecosystem patterns**: Absorbed from 9 springs + 10 primals review.
- **wateringHole docs**: `PRIMAL_REGISTRY.md` and `LOAMSPINE_LEVERAGE_GUIDE.md` updated.
- **provenance-trio-types**: Blocker documented as resolved.
- **Tests**: 1,232 → **1,247** (+15 net). **Coverage**: unchanged from v0.9.7 (92.23% line / 90.46% region / 86.52% function). **Source files**: 124 (unchanged). **Max file**: 865 lines (unchanged).

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
