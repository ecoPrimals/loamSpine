<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Development Roadmap

**Current Version**: 0.9.16  
**Last Updated**: May 2, 2026

---

## Documentation changelog

- **April 16, 2026** — **bincode → rmp-serde (MessagePack)**: Storage and backup serialization migrated from `bincode` v1 to **`rmp-serde`**, eliminating **RUSTSEC-2025-0141**. The prior **bincode v1 → v2** migration plan is complete in spirit but **not** via bincode v2 — MessagePack is the chosen on-disk format.
- **April 16, 2026** — **biomeOS doc comments**: Literal **biomeOS** references in **production** doc comments genericized (**29 → 0**) for self-knowledge compliance.

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
- **Dependency evolution documented** — `specs/DEPENDENCY_EVOLUTION.md` tracks completed storage serialization (MessagePack via `rmp-serde`, superseding bincode v1), mdns evolution, sled deprecation/removal
- **Tests**: 1,397 (+85). Source files: 129. All under 1000 lines (max: 899). Coverage: 93.96% line / 92.60% region.

## v0.9.16 BTSP Phase 3 Negotiate Handler (May 2, 2026)

- **`btsp.negotiate` JSON-RPC method**: Returns `cipher: "null"` (plaintext fallback). primalSpring Phase 3 clients can negotiate cipher suites without `METHOD_NOT_FOUND`. Full encrypted framing (ChaCha20-Poly1305 AEAD + HKDF key derivation) deferred until BTSP provider exports session key material.
- **Zero new crypto deps**: Follows "delegate to Tower" philosophy — no `chacha20poly1305`, `hkdf`, or `sha2` added.
- **Tests**: 1,513 pass (+4). All gates green (clippy, fmt, deny).

## v0.9.16 Self-Contained Provenance Receipts (April 30, 2026)

- **Self-contained provenance receipts**: `CommitSessionResponse` now echoes the full session binding (`session_id`, `merkle_root`, `vertex_count`, `committer`) alongside the ledger anchor. When Tower signing is enabled, `tower_signature` is included in the receipt. Downstream consumers can trace DAG-to-ledger computation provenance from the receipt alone. Resolves Phase 56c "provenance chain for guideStone receipts."
- **`get_provenance_chain()` extended**: Now matches `SessionCommit` entries on `merkle_root` (relationship: `committed-from`), enabling provenance chain queries to traverse DAG session commits alongside data anchors and braid commits.
- **Tests**: 1,509 pass. All gates green (clippy, fmt, deny).

## v0.9.16 Tower-Signed Ledger Entries (April 28, 2026)

- **Tower-signed ledger entries**: `entry.append` and `session.commit` now sign entries via BearDog `crypto.sign_ed25519` when `BEARDOG_SOCKET` is set. Signature stored in entry metadata (`tower_signature`, `tower_signature_alg`). Follows NUCLEUS Two-Tier Crypto Model — loamSpine purpose: `ledger`. Standalone mode (no BearDog) unchanged.
- **Core API**: New `prepare_entry()` + `append_prepared_entry()` on `LoamSpineService` enable signing between entry creation and chain append. `append_entry()` delegates to these when no signing is needed.
- **BTSP tunnel consumption**: Documented as next evolution frontier per Two-Tier Crypto Model. No primal actively establishes persistent BTSP tunnels yet.
- **Tests**: 1,509 pass (+3 Tower signing tests, test file refactoring). Max `.rs` file: 783L. All gates green (clippy, fmt, deny).

## v0.9.16 PG-52 Verified Live + Provenance Receipt Enrichment (April 27, 2026)

- **PG-52 VERIFIED LIVE**: primalSpring convergence validation confirmed double-BufReader fix working in live composition. Trio lifecycle (`create → append → seal`) operational. Stale plasmidBin confirmed as root cause — rebuilt and reharvested (blake3 `6403449f...`).
- **Provenance receipt enrichment**: `CommitSessionResponse` now returns `spine_id` + `committed_at` alongside `commit_hash` + `index`. `LoamCommitRef` likewise carries `committed_at`. Session commit responses are now self-contained provenance receipts for guideStone chain tracing. Backward-compatible (additive fields). API spec synchronized.
- **Remaining gap triage**: `ring` lockfile — Cargo.lock v4 artifact (not compiled, banned in `deny.toml`). NestGate bond wiring — loamSpine side complete (`bonding.ledger.*`), gap is upstream BearDog wire shape alignment.
- **Double-`BufReader` eliminated on post-BTSP path**: New `handle_stream_buffered` function accepts the existing `BufReader` directly instead of wrapping it in a second layer (previously `BufReader<BufReader<OwnedReadHalf>>`). Prevents potential residual-byte misalignment after BTSP handshake.
- **3 new UDS integration tests**: Persistent-connection trio lifecycle, BTSP-config coexistence, one-shot connection pattern (socat/nc composition script pattern).
- **Tests**: 1,506 pass (+3). All gates green (clippy, fmt, deny).
- **Action required**: plasmidBin binary rebuild for deployed compositions to pick up PG-07/PG-33/PG-52 fixes.

## v0.9.16 BTSP Connection Lifecycle Fix (April 24, 2026)

- **Persistent BearDog connection per handshake**: Replaced per-call `provider_call` (reconnect + `shutdown()`) with `ProviderConn` struct that holds a single UDS connection reused across all three relay calls (create → verify → negotiate). Per SOURDOUGH BTSP Relay Pattern §3.
- **Removed `writer.shutdown()` (primary bug)**: `provider_roundtrip` called `writer.shutdown()` after writing each request, sending EOF to BearDog. BearDog interpreted this as connection close and dropped the response — a race condition where `create` often succeeded (fast operation) but `verify` failed (slower, EOF propagated before response). Replaced with `flush()` only.
- **Read timeout added**: 10-second timeout on all provider reads to prevent indefinite hangs if BearDog drops the connection.
- **`crypto_provider.rs` same fix**: Removed identical `shutdown()` anti-pattern from crypto provider call path. Added read timeout.
- **Mock providers updated**: Both test mocks (`btsp_tests_integration.rs`, `tests_protocol_transport.rs`) now handle multiple requests per connection (loop-based), matching the persistent connection pattern.
- **Tests**: 1,503 pass. All gates green (clippy, fmt, deny).

## v0.9.16 BTSP HandshakeComplete Wire Fix (April 15, 2026)

- **`HandshakeComplete` now sends `"status":"ok"`**: primalSpring's BTSP client identifies `HandshakeComplete` by the `"status":"ok"` discriminator field. LoamSpine was sending `{"cipher":"...","session_id":"..."}` without `status`, causing the client to timeout waiting for the completion message. Fixed in both length-prefixed and NDJSON paths. Resolves Phase 45c "incomplete handshake" upstream debt — `ledger` capability BTSP should now PASS in guidestone.
- **Tests**: 1,503 (+1 wire-format assertion). All gates green.

## v0.9.16 BTSP Step 3→4 Verification Relay (April 23, 2026)

- **BearDog relay params aligned with `beardog_types::btsp::rpc`**: All three relay calls to BearDog corrected. `btsp.session.create` sends `family_seed` (base64-encoded) instead of `family_seed_ref: "env:FAMILY_SEED"`. Challenge generated by BearDog, not loamSpine — `generate_challenge()` removed. `btsp.session.verify` uses `session_token`/`response`/`preferred_cipher` instead of `session_id`/`client_response`/`server_ephemeral_pub`/`challenge`. `btsp.negotiate` uses `session_token`/`cipher` instead of `session_id`/`preferred_cipher`/`bond_type`. Response types aligned: `SessionCreateResult.session_token`+`challenge`, `NegotiateResult.accepted`.
- **`resolve_family_seed()` added**: Reads `FAMILY_SEED` (primary) or `BEARDOG_FAMILY_SEED` (fallback), base64-encodes the raw bytes for BearDog. Same pattern as sweetGrass.
- **5 new tests**: `resolve_family_seed` primary, fallback, precedence, missing error, hex roundtrip. Integration tests use `temp_env::with_var` for safe env mutation.
- **Tests**: 1,502 (+3 net). Mock providers aligned with BearDog response shapes (`session_token`, `challenge`, `accepted`).

## v0.9.16 BTSP Provider Socket Wired (April 22, 2026)

- **BTSP provider socket wired in static mode**: UDS accept loop restructured to always peek first byte via `BufReader::fill_buf()`, routing by wire format regardless of `btsp_config`. Fixes: when `BIOMEOS_FAMILY_ID` was set (static BTSP), NDJSON connections from primalSpring were misrouted to the binary length-prefixed handshake. Now `{` → NDJSON/JSON-RPC detection, non-`{` → binary BTSP.
- **`perform_server_handshake` split R/W**: Refactored from single `<S: AsyncReadExt + AsyncWriteExt>` to `<R, W>` (separate reader/writer). Matches `perform_ndjson_server_handshake` design and enables BufReader-based peek before binary handshake.
- **Provider resolution priority**: NDJSON BTSP path uses `btsp_config.provider_socket` when available, falling back to `resolve_btsp_provider()` (env vars). Static mode now carries its config into the NDJSON path.
- **2 new integration tests**: Full NDJSON handshake through `run_jsonrpc_uds_server` with `btsp_config = Some(...)` (regression for the exact bug), JSON-RPC fallthrough with BTSP configured.
- **Tests**: **1,499**. All gates green.

## v0.9.16 BTSP NDJSON Wire-Format Alignment & Deep Debt (April 21, 2026)

- **BTSP NDJSON auto-detection**: UDS accept loop now peeks the first line of each connection. When `"protocol":"btsp"` is detected (primalSpring-style newline-delimited JSON), routes to `perform_ndjson_server_handshake`. Resolves Phase 45b BTSP escalation gap. Existing length-prefixed BTSP unchanged.
- **NDJSON wire types**: `NdjsonClientHello` (with `protocol` discriminator), `NdjsonServerHello` (with `session_id`) — matches primalSpring `ecoPrimal/src/ipc/btsp_handshake.rs` format.
- **`handle_stream_with_first_line`**: New server entry point replays already-read first line into HTTP/NDJSON dispatch when the line is not BTSP.
- **Capability string unification**: `"permanence"`/`"ledger"` literals in `neural_api/mod.rs` CAPABILITIES and identity response → `primal_names::LEGACY_DOMAIN`/`CAPABILITY_DOMAIN`.
- **Path constant unification**: `"biomeos"` path segment in `network.rs` → `primal_names::BIOMEOS_SOCKET_DIR`.
- **12 new tests**: NDJSON wire type serde, primalSpring format compat, full handshake sequence (success + verify rejection + version mismatch), `is_btsp_ndjson` detection logic.
- **Tests**: **1,454**. All gates green. `cargo deny check` passes clean.

## v0.9.16 Stadial Parity Gate (April 16, 2026)

- **Storage**: Removed **sled** and **SQLite** backends; **redb** (default) + **memory** only. Source files **187 → 178** (9 backend files removed). Tests remain **1,442** (feature-gated sled/sqlite tests were never in the default count).
- **`hickory-resolver`**: **0.24 → 0.26** (`async-trait` dropped from `hickory-proto`; `hickory-net` still has it upstream).
- **Lockfile**: Cleared **sled**, **libsqlite3-sys**, **rusqlite**, **instant**, **fxhash**. Remaining upstream ghosts: **async-trait** (`hickory-net` 0.26), **ring** (optional features only).
- **`cargo deny`**: Bans + advisories clean. **Dyn audit**: 72 usages, all non-blocking per stadial gate.

## v0.9.16 Crypto Wire Adapter & Deep Debt Sweep (April 16, 2026)

- **`JsonRpcCryptoSigner` / `JsonRpcCryptoVerifier`**: Production signing path implementing `crypto.sign_ed25519` / `crypto.verify_ed25519` wire contract per `CRYPTO_WIRE_CONTRACT.md`. UDS NDJSON transport, base64 encoding, `const fn` constructors. `CliSigner` remains as development fallback.
- **Self-knowledge sweep**: Remaining hardcoded primal names (`airSpring`, `healthSpring`, `wetSpring`, `ludoSpring`, `neuralSpring`) in production doc comments genericized to ecosystem-capability language.
- **`#[allow(dead_code)]` evolved**: `SignResponse.algorithm` field now logged via `tracing::trace` instead of suppressed.
- **Dependency evolution notes**: `sled` (unmaintained; removed in stadial gate), storage/backup uses **`rmp-serde`** (not `bincode`), `mdns-sd` 0.19 replaces `mdns` 3.0 (`async-std` eliminated).
- **`cargo deny check`**: advisories OK, bans OK, licenses OK, sources OK.
- **Tests**: **1,442**. Source files: **178**. JSON-RPC methods: **37**. All gates green.

## v0.9.16 Bond Persistence & Self-Knowledge Evolution (April 15, 2026)

- **Bond ledger persistence**: `bonding.ledger.store` / `bonding.ledger.retrieve` / `bonding.ledger.list` JSON-RPC methods implementing `STORAGE_WIRE_CONTRACT.md` for ionic bond state persistence. Dedicated append-only spine + in-memory `HashMap` index.
- **`BondLedgerRecord` entry type**: New `EntryType` variant for cross-primal contract persistence.
- **5 new JSON-RPC methods**: `bonding.ledger.store`, `bonding.ledger.retrieve`, `bonding.ledger.list`, `slice.checkout`, `proof.verify_inclusion` (37 total, was 32).
- **Self-knowledge evolution**: ~50 hardcoded primal name references (BearDog, rhizoCrypt, sweetGrass, NestGate, ToadStool) genericized to capability-based language. `BTSP_PROVIDER_PREFIX` from `"beardog"` to `"btsp-provider"`.
- **Capability Wire Standard**: Full L3 compliance — 37 methods, 10 capability groups (including bond-ledger), self-knowledge compliant.
- **Tests**: **1,434**. Source files: **186**. All gates green.

## v0.9.16 Hardcoding Evolution & Transport Refactor (April 11, 2026)

- **Registry path centralization**: `/health`, `/discover`, `/register`, `/heartbeat`, `/deregister` string fragments extracted from `discovery_client/mod.rs` into `constants::registry` module. Single source of truth for all registry HTTP paths.
- **BTSP provider socket naming**: Hardcoded `"beardog"` string literals in `btsp/config.rs` replaced with `BTSP_PROVIDER_PREFIX` constant. Protocol-level naming convention documented.
- **Smart refactor `jsonrpc/server.rs`** (529 lines) → TCP transport stays in `server.rs` (362 lines), UDS transport extracted to `uds.rs` (172 lines). Clean domain boundary: TCP/HTTP vs UDS+BTSP gating.
- **Tests**: 1,505 → **1,507** (+2 new: registry path validation, registry path distinctness). Source files: 169 → **170**. Full pipeline clean.

## v0.9.16 Deep Debt Pass 9 — BTSP Provider Decoupling & Modernization (April 14, 2026)

- **BTSP provider decoupling**: `beardog_client.rs` → `provider_client.rs` (module rename). `beardog_call` → `provider_call`, `beardog_socket` → `provider_socket` throughout handshake.rs. All "BearDog" error messages and doc comments evolved to "BTSP provider" — zero compile-time coupling to any specific signing primal. `BEARDOG_SOCKET` env var → `BTSP_PROVIDER_SOCKET` (checks BTSP_PROVIDER_SOCKET first, falls back to BEARDOG_SOCKET for backward compat). Unused `beardog_socket()` accessor removed.
- **`.into()` modernization**: `DEFAULT_BTSP_PROVIDER_PREFIX.to_string()` → `.into()`, `"LoamSpine".to_string()` → `.into()` (config default), `"Storage backend unavailable".to_string()` → `.into()` (health readiness), `fid.to_string()` → `fid.into()` (BTSP config).
- **Test naming evolved**: `spawn_mock_beardog` → `spawn_mock_provider`, `handshake_failure_beardog_unavailable` → `handshake_failure_provider_unavailable`.
- **Full 11-dimension audit**: Zero unsafe, zero production unwrap/expect, zero TODO/FIXME, zero production mocks, zero hardcoded primal names in production, all files under 1000 lines, zero stale `#[allow]`. **`bincode` v1 → `rmp-serde`** migration recorded as complete in `DEPENDENCY_EVOLUTION.md`.
- **Tests**: **1,396**. All gates green.

## v0.9.16 Deep Debt Pass 8 — provenance.commit Alias (April 14, 2026)

- **`provenance.commit` alias**: primalSpring benchScale (exp084) calls `provenance.commit` against loamSpine for replay attack validation. Method was returning `-32601 Method not found` because no such method existed in dispatch. Root cause: exp084 uses composition-level naming (`provenance.*`) while loamSpine's canonical method is `session.commit`. Fix: added `provenance.commit` to `normalize_method` alias table. 1 new integration test (`provenance_commit_alias_dispatches_to_session_commit`).
- **Tests**: 1,395→**1,396** (+1). All gates green.

## v0.9.16 Deep Debt Pass 7 — Doc Reconciliation & Debris Cleanup (April 12, 2026)

- **Root doc metrics reconciled**: Source files 176→**178** (README, STATUS, CONTEXT, CONTRIBUTING). Test badge 1,383→**1,395** (README). JSON-RPC method count reconciled to **32** across all docs (was 30 in STATUS, 36 in CONTRIBUTING; truth: `niche.rs` METHODS). Showcase file count 55→**54** (README). CHANGELOG 0.9.16 metrics corrected from stale intermediate snapshot.
- **Stale `phase1/` cross-repo links fixed**: 8 references across 5 files pointing to nonexistent `../../../phase1/<primal>/` updated to `../../<primal>/` (actual sibling layout). Primal casing corrected (songbird→songBird, toadstool→toadStool, nestgate→nestGate).
- **Build artifacts cleaned**: `cargo clean` removed 9,847 files / 6.4 GiB.
- **Debris scan clean**: No stale scripts, tracked build artifacts, TODO/FIXME in production, secrets, or redundant docs found.
- **Tests**: **1,395**. Source files: **178**. All gates green.

## v0.9.16 Deep Debt Pass 6 — Constants, Test Refactoring, Arc<str>, Modernization (April 12, 2026)

- **Discovery string literals → named constants**: `discovery_method::ENVIRONMENT/DNS_SRV/MDNS` and `srv_metadata::PRIORITY/WEIGHT/TARGET/PORT` modules in `constants.rs`. 3 new constant validation tests. All usages in `infant_discovery/mod.rs` and `backends.rs` updated.
- **Witness default constants**: `DEFAULT_WITNESS_KIND`/`DEFAULT_WITNESS_ENCODING` in `trio_types.rs`. 2 new tests.
- **Test file smart-refactoring**: `tests_protocol.rs` (956L) → `tests_protocol_transport.rs` (~430L) + `tests_protocol_wire.rs` (~500L). `discovery/tests.rs` (899L) → `tests_registry.rs` (~330L) + `tests_attestation.rs` (~570L). Split by domain, not arbitrary line count.
- **Arc<str> for retry closures**: `ResilientDiscoveryClient.discover_capability` and `advertise_self` parameters converted to `Arc<str>` — O(1) clone per retry instead of O(N) String allocation.
- **`.into()` modernization**: String literal `.to_string()` → `.into()` in error constructors across 4 files.
- **`health.check` empty params fix**: `HealthCheckRequest.include_details` now `#[serde(default)]`. `deser()` normalizes `null` params to `{}`.
- **Tests**: 1,390→**1,395** (+5 new). Source files: 176→**178**. Zero clippy/doc warnings.

## v0.9.16 Deep Debt Pass 5 — health.check Default & plasmidBin (April 12, 2026)

- **`health.check` accepts empty params**: `HealthCheckRequest.include_details` annotated with `#[serde(default)]`, defaulting to `false` when absent. Downstream consumers can call `health.check` with `{}` or `null` without error.
- **JSON-RPC `null` param normalization**: `deser()` function now normalizes `Value::Null` to empty object, preventing deserialization failures for methods expecting struct params.
- **plasmidBin/wateringHole updated**: Handoff and compliance docs updated for health.check fix.
- **Tests**: **1,390**. Source files: **176**. All gates green.

## v0.9.16 Deep Debt Pass 4 — Port Decoupling & Debris Cleanup (April 12, 2026)

- **Hardcoded port constants decoupled**: `DiscoveryConfig::default()` evolved from raw `DEFAULT_TARPC_PORT`/`DEFAULT_JSONRPC_PORT` to `env_resolution` module (reads `LOAMSPINE_*_PORT` > `*_PORT` > default). `discovery_client::advertise_self()` fallbacks similarly evolved. Constants remain only in doc examples and cfg-gated dev fallback.
- **Showcase consolidation**: Duplicate `SHOWCASE_QUICK_REFERENCE_CARD.md` (126 lines) removed — `QUICK_REFERENCE.md` (306 lines) is the canonical reference. Index and entry point updated.
- **`.gitignore` hardened**: Added `.vscode/`, `.idea/`, `coverage/`, `htmlcov/`, `*.lcov`, `*.rs.bk` patterns.
- **Full 11-dimension debt audit clean**: Zero unsafe, zero production unwrap/expect, zero TODO/FIXME, zero production mocks, zero hardcoded primal names, zero files over 1000 lines (Rust), zero archive directories, zero IDE debris, zero stale scripts, zero build artifacts, zero coverage artifacts.
- **Tests**: **1,388**. Source files: **176**. All gates green.

## v0.9.16 Ecosystem Validation & Domain Symlink (April 12, 2026)

- **Capability-domain symlink**: `ledger.sock → loamspine.sock` created on bind, removed on shutdown. Enables `by_capability = "ledger"` routing in deploy graphs. Socket naming now: primary `loamspine.sock`, capability `ledger.sock`, legacy `permanence.sock`. Matches BearDog/Songbird/coralReef pattern.
- **`CAPABILITY_DOMAIN` constant**: `primal_names.rs` — new `CAPABILITY_DOMAIN = "ledger"` constant. `socket.rs` — `capability_domain_socket_name()` and `resolve_capability_symlink_path()` functions. 5 new tests.
- **Wire Standard promoted**: `CAPABILITY_WIRE_STANDARD.md` loamSpine row updated to L2 ✓ L3 ✓ (full compliance — `methods`, `identity.get`, `provided_capabilities`, `consumed_capabilities`, `cost_estimates`, `operation_dependencies`).
- **Compliance matrix updated**: `ECOSYSTEM_COMPLIANCE_MATRIX.md` loamSpine transport and discovery entries corrected to reflect domain symlink, Wire L2+L3, TCP opt-in.
- **plasmidBin reconciled**: `metadata.toml` version 0.9.13→0.9.16, domain `lineage`→`permanence`, capabilities reconciled to 22 live methods, TCP opt-in, socket naming. `manifest.lock` version and domain corrected.
- **Tests**: **1,388**. Source files: **176**. Zero clippy/doc warnings.

## v0.9.16 Deep Debt & Evolution Pass 3 (April 12, 2026)

- **traits/mod.rs test extraction**: Inline `#[cfg(test)] mod tests` (167 lines, 12 tests) extracted to `traits/mod_tests.rs`. Production module: 446→279 lines.
- **Magic number timeouts named**: `CONNECT_TIMEOUT`/`READ_TIMEOUT` (http.rs), `DNS_SRV_TIMEOUT` (infant_discovery), `MDNS_TIMEOUT` (backends.rs). All bare `Duration` literals in production code replaced with named constants.
- **Clone audit clean**: All production `.clone()` confirmed Arc-based O(1) or structurally necessary. No unnecessary allocations in hot paths.
- **LD-09 TCP opt-in**: `loamspine server` no longer binds `0.0.0.0:8080` unconditionally. TCP transports opt-in via `--port`/`--tarpc-port` or env vars. UDS-first by default.
- **Showcase Songbird references cleaned**: Capability table and tarpc description updated to generic language.
- **Root docs reconciled**: All metrics aligned across README, CONTEXT, CONTRIBUTING, STATUS (1,383 tests, 176 source files).
- **Tests**: **1,383**. Source files: **176**. Zero clippy/doc warnings. `cargo deny check` PASS.

## v0.9.16 Deep Debt & Evolution Pass 2 (April 12, 2026)

- **HTTP/1.1 keep-alive**: Connection-close bug fixed — JSON-RPC TCP server now supports persistent HTTP connections (primalSpring audit item resolved).
- **BTSP provider decoupled**: Hardcoded `"beardog"` → env-configurable `BTSP_PROVIDER` with default. `provider_socket` replaces `beardog_socket`.
- **Smart test extraction (5 files)**: `streaming.rs`, `health.rs`, `service/mod.rs`, `config.rs`, `lib.rs` — inline tests extracted to `#[path]` siblings.
- **Stale Songbird references removed**: All production doc comments evolved to capability-based language.
- **Doc warning fixed**: Broken `read_ndjson_stream_with` intra-doc link.
- **Root docs reconciled**: README, CONTEXT, CONTRIBUTING metrics aligned with STATUS.md.
- **Tests**: **1,382**. Source files: **175**. Zero clippy/doc warnings. `cargo deny check` PASS.

## v0.9.16 Deep Debt Overhaul & Dependency Evolution (April 11, 2026)

- **BTSP challenge evolved**: `generate_challenge_placeholder()` (timestamp-derived) replaced with `generate_challenge()` using `blake3` + `uuid::Uuid::now_v7()` — 148+ bits OS-sourced entropy. Zero new dependencies.
- **Smart refactor `btsp.rs`** (696 lines) → `btsp/` module directory with 5 submodules: `wire.rs` (types), `config.rs` (BearDog socket resolution), `frame.rs` (length-prefixed I/O), `beardog_client.rs` (JSON-RPC delegation), `handshake.rs` (4-step protocol). All production modules now under 581 lines.
- **Dependency cleanup**: `serde_bytes` removed (unused). `bytes`, `url`, `bincode`, `tarpc`, `futures`, `clap`, `loam-spine-core`, `loam-spine-api` centralized to `[workspace.dependencies]`.
- **Storage test isolation fixed**: Sled `from_db` constructors eliminate lock contention (10 tests). SQLite WAL mode + busy timeout. redb `tempfile::tempdir()` + explicit `drop` (5 tests). Zero flaky storage tests.
- **`#[allow]` audit**: `#[expect]` attributes that caused `unfulfilled-lint-expectations` in `--all-features` builds reverted to `#[allow]` with documented reasons.
- **Tests**: 1,373 → **1,507** (+134). Source files: 167 → **170**. Zero clippy warnings. Full pipeline clean (fmt, clippy, doc, deny).

## v0.9.16 Deep Debt Cleanup & Evolution Pass (April 9, 2026)

- **Smart refactor `infant_discovery/mod.rs`**: Extracted mDNS backend functions (`mdns_discover_impl`, `parse_mdns_response`, `capability_to_srv_name`) into `backends.rs` (158 lines). Module reduced 711→570 lines. All production files now under 700 lines.
- **Zero-copy JSON-RPC extraction**: Eliminated `.clone()` in `extract_rpc_result_typed` and `parse_beardog_response` — replaced `serde_json::from_value(result.clone())` with borrowing `T::deserialize(result)`.
- **Resilience retry path**: Removed `err_msg.clone()` from retry loop — log then move.
- **tarpc/opentelemetry advisory documented**: Added `RUSTSEC-2026-0007` to `DEPENDENCY_EVOLUTION.md`.
- **Coverage expansion**: 10 new tests (temporal types, StorageResultExt trait).
- **Tests**: 1,363 → **1,373**. Source files: 166 → **167**. Zero clippy warnings.

## v0.9.16 BTSP Phase 2 Handshake Integration (April 9, 2026)

- **BTSP handshake-as-a-service**: New `btsp` module in `loam-spine-core` implements the consumer side of BTSP Phase 2. LoamSpine delegates all crypto to BearDog via JSON-RPC (`btsp.session.create`, `btsp.session.verify`, `btsp.negotiate`). Zero crypto dependencies added.
- **UDS listener gated**: `run_jsonrpc_uds_server` accepts `Option<BtspHandshakeConfig>`. When `BIOMEOS_FAMILY_ID` is set, every UDS connection must complete the 4-step BTSP handshake before JSON-RPC methods are exposed.
- **Wire format**: 4-byte big-endian length-prefixed frames per `BTSP_PROTOCOL_STANDARD.md`. Wire types: `ClientHello`, `ServerHello`, `ChallengeResponse`, `HandshakeComplete`, `HandshakeError`.
- **Capability-discovered BearDog**: Socket path resolved via env → family fallback → platform default. No primal names hardcoded.
- **Consumed capability registered**: `"btsp"` in capabilities, niche, and `primal-capabilities.toml`.
- **28 new tests**: Config derivation, socket resolution, frame I/O, wire serde roundtrips, mock BearDog integration (success, verify rejection, cipher rejection, BearDog unavailable, version mismatch).
- **Tests**: 1,316 → **1,363**. Source files: 163 → **166**. Zero clippy warnings.

## v0.9.16 Deep Debt Module Evolution Sprint 2 (April 8, 2026)

- **Smart refactor `jsonrpc/mod.rs`** (773 lines) → 3 focused modules: `wire.rs` (82 lines — wire types & error codes), `server.rs` (428 lines — TCP/UDS transport infrastructure), `mod.rs` (285 lines — dispatch logic only). Each module has a single responsibility.
- **Smart refactor `capabilities.rs`** (587 lines) → `capabilities/` directory: `mod.rs` (107 lines — identifier constants & re-exports), `types.rs` (235 lines — enum definitions & impls), `parser.rs` (129 lines — response parser), `tests.rs` (116 lines).
- **mDNS-SD service discovery**: `try_mdns_discovery()` evolved from stub to real async implementation via `mdns-sd` 0.19. Queries `_discovery._tcp.local.` on LAN. Feature-gated under `mdns`. (Previously used `mdns` 3.0 + `async-std`; migrated April 20, 2026.)
- **Lint audit**: All 2 `#[allow(` suppressions verified as correctly feature-conditional. All `#[expect(` suppressions have documented reasons.
- **Tests**: 1,304 pass. Source files: **152**. Zero clippy warnings.

## v0.9.16 Capability Wire Standard L2/L3 (April 8, 2026)

- **Wire Standard L2 compliance**: `capabilities.list` response reshaped per Capability Wire Standard v1.0. `methods` promoted from array of objects to flat string array (primary biomeOS routing signal). All 32 callable methods now advertised (previously 24, missing health/permanence/tools/identity).
- **Wire Standard L3 (composable)**: `provided_capabilities` grouping (9 domain groups), `consumed_capabilities` declaration, `cost_estimates` and `operation_dependencies` (already present, retained).
- **`identity.get` method**: New JSON-RPC method returning `{primal, version, domain, license}`. Cached via `OnceLock`. MCP tool `identity_get` added.
- **Niche evolution**: `METHODS` uses canonical `capabilities.list` (was `capability.list`). `identity.get` and `permanence.*` methods added.
- **Deploy graph aligned**: All 32 methods registered in `loamspine_deploy.toml`.
- **Tests**: 1,301 → **1,304**. Zero clippy warnings.

## v0.9.16 GAP-MATRIX-12 Socket Naming Compliance (April 8, 2026)

- **Ecosystem convention socket naming**: Primary socket uses `loamspine.sock` / `loamspine-{family_id}.sock` per `{primal}-{FAMILY_ID}.sock` convention. Capability symlink: `ledger.sock → loamspine.sock`. Legacy symlink: `permanence.sock → loamspine.sock` (backward compat). `"ledger"` added to CAPABILITIES for `discover_by_capability("ledger")` support.
- **`BIOMEOS_INSECURE` security guard**: `validate_security_config()` rejects startup when `BIOMEOS_INSECURE=1` is combined with a non-default `FAMILY_ID` (family-scoped sockets require BTSP authentication).
- **Socket cleanup on shutdown**: Primary socket, capability symlink, and legacy symlink all removed on graceful exit.
- **Tests**: 1,304 → **1,316** (+12 new: domain naming, legacy symlink, security config validation). Zero clippy warnings.

## v0.9.16 Deep Debt Smart Refactoring Sprint 3 (April 8, 2026)

- **`certificate_tests.rs` split** (1,060 → 535 + 525): Only file over 1,000 lines split by domain — core CRUD/provenance tests vs escrow/expiry/return edge cases.
- **6 production file test extractions**: Inline `mod tests {}` blocks extracted to dedicated `*_tests.rs` files via `#[path]`:
  - `service/waypoint.rs` (627 → 250 production)
  - `service/infant_discovery.rs` (662 → 448 production)
  - `constants/network.rs` (585 → 325 production)
  - `trio_types.rs` (591 → 442 production)
  - `types.rs` (568 → 380 production)
  - `entry/mod.rs` (617 → 530 production, proptests merged into single test file)
- **Max production file**: 711 lines (infant_discovery/mod.rs — tests already external).
- **Source files**: 152 → **163**. **Tests**: 1,316 (unchanged). Zero clippy warnings.

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
- **~~mdns crate evolution~~** — **COMPLETE** (April 20, 2026): `mdns` 3.0 replaced with `mdns-sd` 0.19; async-std/net2/proc-macro-error eliminated; 3 RUSTSEC advisories removed
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
