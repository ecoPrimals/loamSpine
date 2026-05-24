<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# LoamSpine Deep Debt Cleanup Handoff — May 24, 2026

## Summary

Comprehensive deep debt audit and cleanup pass covering safe casts, dead code wiring,
API ergonomics, and test module cohesion. The audit confirmed the codebase is
exceptionally clean across all 10 standard dimensions.

## Changes

### Safe Cast (btsp/frame.rs)
- **Before**: `let mut buf = BytesMut::zeroed(len as usize)` — sole production `as` cast
- **After**: `usize::try_from(len)` with proper `LoamSpineError::Ipc` on platform capacity overflow
- Consistent with workspace-level `deny(cast_possible_truncation)` policy

### Dead Code Wiring (btsp/wire.rs + handshake.rs)
- `SessionVerifyResult::cipher` was deserialized from Tower response but never used
- Removed `#[expect(dead_code)]` marker
- Wired into `tracing::debug!(cipher = ?verify.cipher, ...)` on both handshake paths
  (length-prefixed and NDJSON) for protocol observability

### API Ergonomics (service/mod.rs)
- `register_btsp_session` evolved from `session_id: String` to `session_id: impl Into<String>`
- Callers can pass `&str`, `String`, or `Arc<String>` without explicit conversion

### Test Cohesion Split (neural_api/tests.rs — 828 lines)
Split into 4 domain-focused modules:
- `tests.rs` — Capability list and primal identity (~130 lines)
- `tests_socket.rs` — Socket resolution, domain naming, security config (~200 lines)
- `tests_registration.rs` — Register/deregister, Wave 43 announce validation (~280 lines)
- `tests_mcp.rs` — MCP tool coverage and parity checks (~130 lines)

Lint expectations narrowed per module (e.g., `tests_socket.rs` doesn't use `expect()`,
so only `clippy::unwrap_used` is expected).

## Audit Confirmed Clean

| Dimension | Finding |
|-----------|---------|
| Unsafe code | 0 — `forbid(unsafe_code)` workspace-level |
| TODO/FIXME | 0 |
| Production mocks | 0 — all `cfg(test\|testing)` gated |
| Production unwrap/expect | 0 — workspace-level `deny` |
| Production `as` casts | 0 (was 1, now fixed) |
| println/eprintln | 0 in production — all tracing |
| Hardcoded ports/paths | Dev defaults with release guards only |
| C/C++ dependencies | 0 in production (libfuzzer-sys in fuzz only) |

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Source files | 186 | 189 (+3 test modules) |
| Tests | 1,527 | 1,527 |
| Max test file | 828 | 787 |
| Clippy warnings | 0 | 0 |

## Files Changed

- `crates/loam-spine-core/src/btsp/frame.rs` — safe cast
- `crates/loam-spine-core/src/btsp/wire.rs` — dead_code removed
- `crates/loam-spine-core/src/btsp/handshake.rs` — cipher tracing (2 paths)
- `crates/loam-spine-api/src/service/mod.rs` — impl Into<String>
- `crates/loam-spine-core/src/neural_api/mod.rs` — test module registration
- `crates/loam-spine-core/src/neural_api/tests.rs` — trimmed to capability domain
- `crates/loam-spine-core/src/neural_api/tests_socket.rs` — NEW
- `crates/loam-spine-core/src/neural_api/tests_registration.rs` — NEW
- `crates/loam-spine-core/src/neural_api/tests_mcp.rs` — NEW

## Upstream Action Items

- **primalSpring**: Audit confirms loamSpine meets all 10 deep debt dimensions
- **wateringHole**: No standard changes needed
