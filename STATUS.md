<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Implementation Status

**Current Version**: 0.9.16  
**Last Updated**: August 7, 2026

---

## Overview

This document tracks implementation progress against the specification suite in [specs/00_SPECIFICATIONS_INDEX.md](specs/00_SPECIFICATIONS_INDEX.md).

---

## Implementation Status by Spec Area

| Spec | Status | Notes |
|------|--------|-------|
| [LOAMSPINE_SPECIFICATION.md](specs/LOAMSPINE_SPECIFICATION.md) | COMPLETE | Master spec implemented |
| [ARCHITECTURE.md](specs/ARCHITECTURE.md) | COMPLETE | Component layout matches spec |
| [DATA_MODEL.md](specs/DATA_MODEL.md) | COMPLETE | Entry, Spine, Chain, SpineConfig, EntryType (18+ variants incl. cross-gate trust + trust ledger IPC) |
| [PURE_RUST_RPC.md](specs/PURE_RUST_RPC.md) | COMPLETE | tarpc + pure JSON-RPC (hand-rolled), no gRPC/protobuf/jsonrpsee. Semantic naming. Protocol escalation (`IpcProtocol` negotiation). |
| [WAYPOINT_SEMANTICS.md](specs/WAYPOINT_SEMANTICS.md) | COMPLETE | `anchor_slice`, `checkout_slice`, `depart_slice`, `record_operation` implemented. `WaypointConfig` with `AttestationRequirement` (None/BoundaryOnly/AllOperations/Selective). `AttestationResult` for capability-discovered attestation providers. `PropagationPolicy`, `SliceTerms`, `SliceOperationType`, `WaypointSummary` types defined. `RelendingChain` with multi-hop sublend/return. `ExpirySweeper` for auto-return. |
| [CERTIFICATE_LAYER.md](specs/CERTIFICATE_LAYER.md) | COMPLETE | Core CRUD + loan/return + sublend + `verify_certificate` + `generate_provenance_proof` + escrow + `UsageSummary` integrated into `CertificateReturn` and `LoanRecord`. `WaypointSummary` re-used from waypoint module. Scyborg license schema. Certificate module: types, lifecycle, metadata, provenance, escrow, usage, tests. |
| [API_SPECIFICATION.md](specs/API_SPECIFICATION.md) | COMPLETE | 53 JSON-RPC methods (semantic naming), tarpc server. Spec updated to match implementation. |
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
| Tests | — | 1,791 (214 source files) |
| Concurrent testing | — | All tests concurrent (zero `#[serial]`), zero flaky storage tests |
| Coverage (llvm-cov) | 90%+ | 92.26% line / 89.50% branch / 92.56% region |
| `unsafe` in production | 0 | 0 (`#![forbid(unsafe_code)]`) |
| Clippy pedantic+nursery | 0 | 0 (including `missing_const_for_fn` at warn level) |
| Doc warnings | 0 | 0 |
| Max file size | < 800 lines | 677 max production (`main.rs`); 827 max test file (`service_tests.rs`) |
| Source files | — | 213 `.rs` files (+ 3 fuzz targets) |
| Edition | 2024 | 2024 |
| `#[allow]` in production | 0 | Zero. All suppressions use `#[expect(reason)]` or `#[cfg_attr]`-gated `#[expect]`. |
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
| `capability_registry.toml` | PASS | `config/capability_registry.toml` — 19 domains, 54 operations, 6 consumed capabilities |
| AGPL-3.0-or-later | PASS | SPDX headers on all 213 source files (+ 3 fuzz targets) |
| Scyborg triple license | PASS | `LICENSE` (AGPL-3.0), `LICENSE-ORC`, `LICENSE-CC-BY-SA` present. `CertificateType::scyborg_license()`, metadata builders, schema constants |
| Semantic naming | PASS | `capabilities.list` canonical + `primal.capabilities` alias per v2.1 standard |
| `health.liveness` | PASS | Returns `{"status": "alive"}` per Semantic Method Naming Standard v2.1 |
| PUBLIC_SURFACE | PASS | `CONTEXT.md` created, "Part of ecoPrimals" footer in README.md |
| Zero-copy | PASS | `Did` → `Arc<str>`, `DiscoveryClient.endpoint` → `Arc<str>`, `JsonRpcResponse.jsonrpc` → `Cow`, `capability_list()`/`mcp_tools_list()` → `LazyLock<Value>`, `HealthStatus` version/caps cached via `LazyLock`, `Bytes` for payloads, `[u8; 24]` stack keys, `tip_entry()` zero-copy persistence |
| MockTransport | PASS | `cfg(test|testing)` gated — no mock code in production binary |
| Socket Naming | PASS | G65 single-socket with protocol negotiation. C2 dual-socket retained for backward compat (`loamspine.sock` + `loamspine.tarpc.sock`). Family-scoped: `loamspine-{fid}.sock` / `loamspine-{fid}.tarpc.sock`. `ledger.sock` capability symlink, `permanence.sock` legacy symlink. `BIOMEOS_INSECURE` guard. All sockets cleaned on shutdown. |
| G65 Negotiation | PASS | `PROTOCOLS: tarpc,jsonrpc\n` → `PROTOCOL: tarpc\n`. Client preference wins. No negotiation = JSON-RPC fallback. Genetics prefix stripping composes with G65. |
| G66 Transport | PASS | `TransportListener` + generic protocol negotiation. Zero unconditional `UnixStream` in production. `#[cfg(unix)]` confined to transport layer. |
| BTSP Phase 1 | PASS | Family-scoped socket naming (`loamspine-{family_id}.sock`), `BIOMEOS_INSECURE` guard. |
| BTSP Phase 2 | PASS | Handshake-as-a-service via BTSP provider JSON-RPC. UDS listener gates on BTSP when `FAMILY_ID` is set. 4-step handshake (ClientHello/ServerHello/ChallengeResponse/HandshakeComplete). |
| BTSP Phase 3 | PASS | `btsp.negotiate` returns `cipher: "chacha20-poly1305"` (plus server nonce) when a Tower-provided handshake key is available; falls back to `cipher: "null"` for unauthenticated covalent bonds. **Transport verified**: after negotiate, UDS accept loop enters `handle_encrypted_stream` using `read_encrypted_frame`/`write_encrypted_frame` for all subsequent messages on that connection. |
| File size limit | PASS | All source files under 1000 lines. |
| Stadial parity gate | PASS | April 16, 2026 — storage backends reduced to redb (default) + memory; sled and SQLite removed; `hickory-resolver` 0.24→0.26; lockfile cleared of sled/libsqlite3-sys/rusqlite/instant/fxhash; `cargo deny` bans + advisories clean; dyn audit non-blocking (72 total usages). |

---

## Stadial Readiness (May 24, 2026)

### Universal Checklist

| Area | Item | Status |
|------|------|--------|
| Runtime | Health triad (liveness/readiness/check) | PASS |
| Runtime | UDS at `$XDG_RUNTIME_DIR/biomeos/loamspine.sock` | PASS |
| Runtime | TCP fallback via `LOAMSPINE_JSONRPC_PORT` | PASS |
| Runtime | `server` subcommand with `--port` | PASS |
| Runtime | Standalone startup without FAMILY_ID | PASS |
| Discovery | `capabilities.list` with `count`, `primal`, `capabilities` | PASS |
| Discovery | `identity.get` canonical response | PASS |
| Discovery | `primal.announce` self-registration | PASS |
| Discovery | `{domain}.{operation}` method naming | PASS |
| Security | BTSP when FAMILY_ID is non-default | PASS |
| Security | ChaCha20-Poly1305 + HKDF-SHA256 | PASS |
| Security | BIOMEOS_INSECURE + FAMILY_ID refused | PASS |
| Security | `btsp.capabilities` registered | PASS |
| Security | Zero metadata leakage | PASS |
| Security | UDS-first, TCP opt-in | PASS |
| Security | deny.toml bans ring/openssl/aws-lc-sys | PASS |
| Build | `edition = "2024"` | PASS |
| Build | `notify-plasmidbin.yml` | PASS |
| Build | musl-static clean | PASS |
| Docs | README version matches | PASS |
| Docs | CHANGELOG recent | PASS |
| Docs | STATUS.md current status | PASS |

### Stability Tiers

All 53 methods have stability annotations in `capabilities.list` response:
- **stable**: spine, entry, certificate, proof, anchor, session, braid, bonding, trust, btsp, lifecycle, health, auth, primal, capabilities, identity, tools (47 methods)
- **evolving**: slice (2 methods)
- **compat**: permanence (4 methods — legacy naming)

### Degradation Behavior

When loamSpine is unavailable:
- **Provenance trio** (rhizoCrypt → loamSpine → sweetGrass): DAG sessions can still complete, but permanent certificates cannot be minted. rhizoCrypt retains dehydration summaries for later commit when loamSpine returns. No data loss.
- **Entry signing**: If `TOWER_SIGNER_SOCKET` is also unavailable, entries are stored unsigned. Signature metadata field is empty. Entries can be signed retroactively.
- **Health probes**: Downstream discovery marks loamSpine as unhealthy. Composition fallback to cached capability lists.
- **Bonding ledger**: Ionic bond contracts cannot be persisted. Bond negotiation fails gracefully — bonds are not established until ledger confirms storage.
- **Slice anchoring**: Waypoint operations fail. Slices remain local and can be anchored when service returns.

### Downstream Pairing

| Consumer | Dependency | Priority |
|----------|-----------|----------|
| sweetGrass | Braid permanence — `session.commit` + entry certificates | HIGH |
| rhizoCrypt | Dehydration target — DAG Merkle roots committed via `permanence.commit_session` | HIGH |
| primalSpring | Guidestone validation — ledger state via `spine.get` | MEDIUM |
| healthSpring | Clinical data pipeline — Nest atomic validation via `session.create`/`session.state` aliases | MEDIUM |
| lithoSpore | Ledger verification — USB deployment evidence chain | STADIAL |
| projectFOUNDATION | Immutable evidence — thread lineage permanence | STADIAL |

### ecoBin Grade: A+

Gap to A++: `seed_fingerprint` (build-time BLAKE3 hash of the released binary). All other criteria met: zero C deps, `#![forbid(unsafe_code)]`, blake3 pure, deny.toml bans, musl-static, edition 2024.

---

### Wave 156j: C2 Dual-Socket Pattern — tarpc UDS Server (August 6, 2026)

- **tarpc UDS server**: `loamspine.tarpc.sock` now binds alongside `loamspine.sock` — binary tarpc framing for sub-ms primal-to-primal composition over UDS. Follows the C2 dual-socket pattern shipped by songBird and petalTongue.
- **Socket path derivation**: `tarpc_socket_from_jsonrpc()` derives `{stem}.tarpc.sock` from the JSON-RPC socket path. `resolve_tarpc_socket_path()` for env-based resolution.
- **Full lifecycle**: tarpc UDS server starts alongside JSON-RPC UDS, has cooperative shutdown (`TarpcUdsHandle::stop()`), and cleans up `.tarpc.sock` on drop.
- **Protocol negotiation ready**: `negotiate_protocol_from()` (pre-wired since G3) will now find the `.tarpc.sock` and escalate to tarpc automatically.
- **3 new tests**: socket path derivation (basic, with family, relative path).
- **Metrics**: 1,787 tests, 212 source files, 53 JSON-RPC + 37 tarpc methods (both UDS + TCP), all checks clean.

---

### Wave 156h: G64 Cephalization — tarpc Convergence (August 5, 2026)

- **tarpc trait expanded**: 24 → 37 domain methods — full parity on all performance-critical operations. Added 13 methods: `list_spines`, `spine_status`, `append_entry_batch`, `list_entries`, `mint_certificate_batch`, `verify_certificate`, `certificate_lifecycle`, `certificate_history`, `publish_anchor_batch`, `trust_anchor`, `trust_query`, `trust_event_count`, `negotiate_btsp`.
- **Design**: 37 tarpc domain methods + 16 JSON-RPC-only diagnostic/meta methods = 53 total. tarpc carries performance; JSON-RPC carries discovery/diagnostics.
- **Metrics**: 1,752 tests, 211 source files, 53 JSON-RPC + 37 tarpc methods, all checks clean.

---

### Wave 156e: spine.status Observability + Doc Debt (August 5, 2026)

- **`spine.status` JSON-RPC method**: New observability endpoint — reports entry count, tip hash, genesis hash, state (Active/Sealed/etc.), creation/update timestamps, and all `SessionCommit` entries with session IDs, Merkle roots, vertex counts, and committers. Sessions returned most-recent-first.
- **Full-stack wiring**: API types (`SpineStatusRequest`, `SpineStatusResponse`, `SessionSummary`), RPC handler, JSON-RPC dispatch, niche registration (METHODS/SEMANTIC_MAPPINGS/COST_ESTIMATES 52→53), MCP tool definition + tool-to-rpc mapping, capability registry.
- **Broken doc link fixes**: Fixed `append_entry_batch_with` (nonexistent method ref) and `CertificateHistory::from_certificate_and_entries` (needed full path).
- **5 new tests**: basic info, session reporting, not-found error, sealed state, multi-session reverse ordering.
- **Metrics**: 1,752 tests, 211 source files, 53 JSON-RPC methods, all checks clean (fmt + clippy + doc + test).

---

### Wave 155u: Niche Completeness + MCP Batch Tools (August 4, 2026)

- **Semantic mapping completeness**: `SEMANTIC_MAPPINGS` expanded from 33 to 52 entries — 100% coverage of `METHODS`. Added batch ops (`append_entry_batch`, `mint_certificate_batch`), certificate introspection (`verify_certificate`, `certificate_lifecycle`, `certificate_history`), listing ops (`list_spines`, `list_entries`), infrastructure probes (`health_liveness`, `health_readiness`, `lifecycle_status`), auth introspection (`auth_check`, `auth_mode`, `auth_peer_info`), BTSP (`btsp_negotiate`, `btsp_capabilities`), and permanence compat layer (4 methods).
- **Cost estimate completeness**: `COST_ESTIMATES` expanded from 32 to 52 entries — 100% coverage. Batch ops: `entry.append_batch` 10ms, `certificate.mint_batch` 15ms (amortized I/O). `session.dehydrate` 3ms added.
- **MCP tool exposure**: `entry_append_batch` and `certificate_mint_batch` MCP tool definitions added to `mcp_tools_list` — AI agents can now discover and invoke batch operations.
- **Metrics**: 1,752 tests, 211 source files, all checks clean.

---

### Wave 150t: Health Probe Honesty + Entry Path Coverage (July 21, 2026)

- **Health probe evolution**: `readiness()` now wraps storage probe in 5s timeout — returns `ready: false` on storage lock timeout instead of hardcoding `ready: true`. `health_check()` now reports `Unhealthy` with storage component detail on timeout instead of hardcoding `Healthy`. Both methods now truly async (await timeout).
- **Entry path test coverage**: 5 new tests for `prepare_entry`/`append_prepared_entry` error paths — missing spine, sealed spine, roundtrip with metadata injection, wrong spine append, append on sealed spine. These are the tower-signing delegation code paths that previously lacked direct unit tests.
- **Health probe tests**: 4 new tests — storage detail reporting, readiness probe storage count, liveness probe, permanence health endpoint counts.
- **Stale comment fix**: Cargo.toml `dns-srv` feature comment corrected — `hickory-resolver 0.26` is pure Rust (no `ring` dependency in current tree).
- **Metrics**: 1,711 tests, 208 source files, ~63,470 lines, all checks clean.

---

### Wave 149b: Dimensional Self-Audit + Test File Splits (July 18, 2026)

- **Self-audit at Wave 149b standard**: All 10 dimensions assessed. GAP-036 (socket naming) PASS, GAP-038 (stale UDS cleanup) PASS, 0 prod unwrap/expect, 0 debt markers, 0 unsafe, 0 `#[allow]` in production.
- **Test file splits**: `chaos.rs` (783L → 2 modules): fault injection (525L) + stress/concurrency `chaos_stress.rs` (260L). `lifecycle_tests.rs` (779L → 2 modules): core lifecycle (546L) + heartbeat/state `lifecycle_tests_heartbeat.rs` (230L). All test files now under 760L.
- **Fuzz safety**: `#![forbid(unsafe_code)]` added to all 3 fuzz targets for parity with crate roots.
- **`--abstract` flag honesty**: CLI flag now warns it's pre-wired rather than silently accepting.
- **Metrics**: 1,702 tests, 208 source files, max test 753L (`tests_validation.rs`).

---

### Wave 143b: Transport Endpoint Wiring + Test Coverage (July 16, 2026)

- **`TRANSPORT_ENDPOINT` functional dispatch**: `main.rs` wired to use injected `TransportEndpoint` for server startup — UDS path override, TCP host:port and bind address from launcher/orchestrator. Previously log-only.
- **Test file split**: `service_tests.rs` (789L → 3 modules): core spine/cert/proof (388L), `permanent_storage.*`/`commit_session` integration (270L), BTSP negotiate/key-derivation (111L).
- **Framing edge-case tests**: 7 new tests — zero-length frame, server disconnect, NDJSON string result, UDS roundtrip (NDJSON + length-prefixed).
- **Metrics**: 1,702 tests, 206 source files, max production 660L (`uds.rs`), max test 779L (`lifecycle_tests.rs`).

---

### Wave 142b: Silicon Atheism Phase 2 + Deep Debt (July 16, 2026)

- **Phase 2 transport abstraction**: Custom `base64_decode` (40L hand-rolled) replaced with workspace `base64` crate. All outbound IPC clients (`provider_client.rs`, `crypto_provider.rs`, `neural_api.rs`, `neural_api/mod.rs`) migrated to `TransportStream` + framing helpers (prior wave). `urlencoding_encode` retained (14L, no dependency needed).
- **Async fs hygiene**: All blocking `std::fs` calls in async functions wrapped in `tokio::task::spawn_blocking` — `uds.rs` startup (`create_dir_all`, `remove_file`), `main.rs` PID file write, symlink creation/removal, shutdown cleanup.
- **Clone reduction**: `integration.rs` and `handshake.rs` — 4 gratuitous `.clone()` calls eliminated via move semantics (partial struct moves, field extraction before last use). `commit_session` committer moved directly. `checkout_slice` owner extracted after save. BTSP session built once, fields borrowed for `HandshakeComplete`.
- **Doc drift**: Production comments referencing `biomeOS` → generic "orchestrator". `trust_ledger.rs` `"e.g. bearDog"` → `"a signing primal"`.
- **Metrics**: 1,702 tests, 206 source files, max production 660L (`uds.rs`).

---

### Wave 141a: Cross-Architecture Adoption + Deep Debt Sweep (July 15, 2026)

- **Cross-architecture**: All Unix-specific IPC (`UnixStream`, `tokio::signal::unix`) gated behind `#[cfg(unix)]` with non-Unix error stubs. `cargo check --target x86_64-pc-windows-gnu` clean (0 errors, 0 warnings). BTSP `ProviderConn`, NeuralAPI registration, `crypto_provider_call`, UDS JSON-RPC server, PID files, capability symlinks all platform-gated.
- **Integration test refactor**: `integration_tests.rs` (1,002 lines, over 800L limit) split into 3 domain-focused modules: `integration_tests_spine_ops.rs` (295L), `integration_tests_slice_mgr.rs` (245L), `integration_tests_provenance.rs` (451L). Source files 199 → 202.
- **BearDog deprecation**: `BEARDOG_FAMILY_SEED` and `BEARDOG_SOCKET` env aliases now emit `tracing::warn` at runtime, guiding operators to canonical `LOAMSPINE_*` / `TOWER_SIGNER_SOCKET` names.
- **Clone reduction**: `certificate_loan.rs` uses `active_loan.take()` instead of deep clones during ownership transfer — zero-copy per loan operation.
- **Test reliability**: `register_with_neural_api` test now tolerates live NeuralAPI socket environments (sporeGate, eastGate).
- **Metrics**: 1,684 tests, 202 source files, max production 660L (`uds.rs`), max test 779L (`lifecycle_tests.rs`).

---

*For complete historical changelog, see [CHANGELOG.md](CHANGELOG.md).*

---

*See [WHATS_NEXT.md](WHATS_NEXT.md) for the development roadmap.*

