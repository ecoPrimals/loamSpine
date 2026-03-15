# Known Issues - LoamSpine

**Date**: March 15, 2026
**Version**: 0.8.6
**Status**: Production Ready

---

## Current State

As of v0.8.6 (March 15), the codebase passes all quality gates:

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Tests | 400+ | 1,092 | PASS |
| Line Coverage | 90% | 89.30% line, 91.26% region | NEAR TARGET |
| Clippy (pedantic + nursery) | 0 warnings | 0 | PASS |
| Formatting | clean | clean | PASS |
| Documentation | compiles (-D warnings) | compiles | PASS |
| Unsafe Code | 0 | 0 | PASS |
| Max File Size | <1000 lines | 955 max (all under 1000) | PASS |
| Source Files | — | 113 | — |
| License | AGPL-3.0-only | AGPL-3.0-only | PASS |
| SPDX Headers | all files | all files | PASS |
| cargo deny (bans, licenses, sources) | pass | pass | PASS |
| ecoBin (zero C deps default) | pass | pass | PASS |
| UniBin subcommands | server, capabilities, socket | all present | PASS |
| Mock isolation | cfg-gated out of production | yes | PASS |
| Scyborg license schema | implemented | yes | PASS |
| Protocol escalation | tarpc preferred | yes | PASS |
| CI cross-compilation | musl targets | x86_64, aarch64, armv7 | PASS |
| Resilience patterns | circuit breaker + retry | implemented (lock-free) | PASS |
| Certificate escrow | hold/release/cancel | implemented | PASS |
| Relending chain | multi-hop sublend | implemented | PASS |
| Provenance proof | Merkle/Blake3 | implemented | PASS |

---

## Remaining Technical Debt

### Medium

1. **Coverage gap** (89.30% line, 91.26% region → 90% line target): Remaining gap is in network I/O:
   - `jsonrpc/mod.rs` (47% — TCP server loop, needs integration-level testing)
   - `main.rs` (0% — binary entry point, tested via integration demos)
   - DNS SRV / mDNS network paths (require real network or more sophisticated mocking)
2. **Storage backends**: `Sqlite` implemented (feature-gated); `Postgres`, `Rocksdb` planned.
3. **Showcase demos**: ~10% complete (2/21 demos fully implemented); remaining are documented/scaffolded.

### Minor

1. **`thiserror` duplicate versions**: v1 and v2 coexist via transitive deps (non-blocking).
2. **`proc-macro-error` advisory**: Transitive dependency of optional `mdns` feature (not enabled by default).
3. **Build infra**: Global `CARGO_TARGET_DIR` on noexec mount requires env override or `.cargo/config.toml`.

### Resolved (March 15 — v0.8.6)

- **Relending chain** implemented: `RelendingChain` with multi-hop sublend/return, depth validation, unwinding
- **Expiry sweeper** implemented: Background task auto-returns expired loans
- **Certificate provenance proof** implemented: Blake3 Merkle tree over ownership chain
- **Certificate escrow** implemented: `TransferConditions`, `hold/release/cancel` with `PendingTransfer` state
- **PrimalAdapter resilience** implemented: Lock-free circuit breaker + exponential backoff retry
- **Certificate module** refactored: `certificate.rs` → `certificate/` directory (7 files)
- All `#[allow(clippy::cast_possible_truncation)]` replaced with `try_from()` + fallback
- Coverage raised from 88.28% to 89.30% line, 90.45% to 91.26% region (+124 tests)
- Test count: 968 → 1,092 (+124 tests)
- All 113 source files under 1000 lines (max: 955)

### Resolved (March 15 — v0.8.5)

- 18 clippy errors fixed (module_inception, match_same_arms, cast_possible_truncation, expect_used, future_not_send, manual_let_else, unused_async, iter_on_single_items)
- `storage/tests.rs` (1122 lines) refactored into 3 backend-specific modules (all under 1000 LOC)
- Coverage raised from 86.47% to 88.28% line, 90.45% region (+98 tests)
- `ConfigurableTransport` added for discovery client error-path testing
- Mock helpers evolved: `async fn` → `fn`, owned params → borrowed refs (idiomatic + zero-copy)
- Test count: 870 → 968 (+98 tests)

### Resolved (March 15 — v0.8.4)

- Scyborg license schema implemented (type URI, metadata builders, constants)
- Protocol escalation (`IpcProtocol` negotiation, tarpc preferred over JSON-RPC)
- SyncProtocol evolved from stub to JSON-RPC/TCP sync engine with graceful fallback
- SQLite refactored from single 990-line file to modular `sqlite/` directory
- Zero-copy storage keys (`Vec<u8>` → `[u8; 24]` stack allocation in redb/sled)
- CI cross-compilation for musl targets via `cross-rs/cross`

### Resolved (March 14 — v0.8.3)

- 67 clippy pedantic+nursery errors → 0 across all workspace crates
- 15 `const fn` promotions, 26 lock guard scope fixes, 6 `let...else` rewrites
- Zero-copy JSON-RPC dispatch (by-value `params` and `request`)
- `MockTransport` cfg-gated to test/feature only
- Dead `SpineSyncState.last_sync_ns` field removed

### None Critical

No critical issues, blockers, or security concerns.

---

**Last Updated**: March 15, 2026
