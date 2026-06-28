<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# loamSpine — Wave 128b: Evolution Audit + Branch Coverage Push

**Date**: June 28, 2026
**Author**: sporeGate Overwatch
**Wave**: 128
**Supersedes**: LOAMSPINE_WAVE128_CONVERGENCE_JUN28_2026

---

## Summary

Full evolution audit against all long-term targets: large files, unsafe code,
hardcoded primal names, mocks in production, blocking I/O in async, external
dependencies, clone patterns, Vec<u8> vs Bytes. Followed by targeted evolution
actions and a branch coverage push (+15 tests) hitting previously uncovered
match arms, error branches, and aggregate anchor paths.

## Changes

### Evolution: Hardcoded Primal Names

Songbird → generic orchestrator in production doc comments:
- `transport/endpoint.rs` — 3 doc comments genericized (module doc, struct doc, MeshRelay variant)

### Evolution: Blocking I/O in Async

- `lib.rs:299` — `std::fs::create_dir_all` in async `start()` wrapped with `tokio::task::spawn_blocking`

### Evolution: Lint Tightening

- `Cargo.toml` — `clippy::todo` and `clippy::unimplemented` added as `deny`
- `Cargo.toml` — `rust-version = "1.85"` added for Edition 2024 MSRV clarity

### Coverage Push (+15 tests)

| Area | Tests Added | Branches Hit |
|------|-------------|-------------|
| `jsonrpc/tests_protocol_wire.rs` | 1 (normalize_method ledger/session aliases) | 6 |
| `jsonrpc/method_gate.rs` | 5 (AuthMode::from_env enforced, permissive, unrecognized, case-insensitive, unset) | 5 |
| `btsp_tests.rs` | 1 (parse_response type mismatch) | 1 |
| `constants/env_resolution.rs` | 4 (discovery_cache_ttl invalid + valid, discovery_enabled false/true variants) | 8+ |
| `service/anchor_tests.rs` | 3 (anchor_batch minimum guard, batch success + aggregate verify, verify_anchor no aggregate) | 10+ |
| `service/integration_tests.rs` | 1 (get_attribution with contributors) | 3 |

### Evolution Targets Evaluated (No Change Needed)

| Target | Decision | Rationale |
|--------|----------|-----------|
| Vec<u8> → Bytes in BTSP encrypt/decrypt | Skip | Consumed by ref immediately; Bytes wrapper adds complexity without zero-copy benefit |
| Clone reduction in certificate_loan.rs | Skip | Clones are small domain types (Did, LoanTerms) in infrequent ledger ops |
| Blocking I/O in UDS server startup | Skip | One-shot init before async work; microsecond ops don't warrant spawn_blocking |
| discovery/manifest.rs dead code | Correct | Pre-wired for strandGate deploy with proper #[expect(dead_code)] |

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 1,684 |
| Line coverage | ~92%+ |
| Source files | 199 |
| JSON-RPC methods | 47 |
| `#[allow]` in prod | 0 |
| `unsafe` in prod | 0 (forbidden) |
| Files >800L | 0 production (1 test: 950L integration_tests.rs) |
| TODO/FIXME/HACK | 0 |
| Hardcoded primal names in prod | 0 (BearDog env vars preserved as backward-compat) |
| cargo fmt | PASS |
| cargo clippy | PASS (0 warnings) |
| cargo doc | PASS |
| cargo deny | PASS |

## Gaps for Upstream

| Item | Owner | Priority |
|------|-------|----------|
| Signing capability middleware (RPC layer signature verification) | loamSpine | v0.10.0 |
| HTTP health endpoints (/health/liveness, /health/readiness) | loamSpine | v1.0.0 |
| Prometheus metrics (request counts, latencies) | loamSpine | v1.0.0 |
| `checksums.toml` for genomeBin/depot | loamSpine | LOW |
| PostgreSQL/RocksDB backends | loamSpine | v1.0.0 (demand-driven) |
| Persistent BTSP tunnels for ledger replication | ecosystem | FUTURE |

## Verification

```
cargo fmt --all --check    → PASS
cargo clippy --workspace   → 0 warnings
cargo test --workspace     → 1,684 passed, 0 failed
cargo doc --workspace      → PASS
cargo deny check           → bans ok, licenses ok, sources ok
```
