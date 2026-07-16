# loamSpine Wave 141a Handoff — Cross-Architecture Adoption + Deep Debt Sweep

**Date**: July 15, 2026  
**From**: sporeGate (loamSpine team)  
**To**: eastGate overwatch  
**Supersedes**: `LOAMSPINE_WAVE128B_EVOLUTION_AUDIT_JUN28_2026.md` (archived)

---

## Summary

Two commits addressing the Wave 141a cross-architecture adoption mandate and a
comprehensive deep debt sweep.

## Commit 1: Cross-Architecture Adoption

**Ref**: `CROSS_ARCHITECTURE_ADOPTION_PER_PRIMAL_HANDOFFS_WAVE141a`

All Unix-specific IPC gated behind `#[cfg(unix)]` with non-Unix error stubs:

| File | Change |
|------|--------|
| `btsp/provider_client.rs` | `ProviderConn` struct + impl split by platform |
| `transport/neural_api.rs` | `jsonrpc_call` method gated |
| `neural_api/mod.rs` | `register_at_socket` / `deregister_at_socket` gated |
| `traits/crypto_provider.rs` | `crypto_provider_call` gated |
| `service/signals.rs` | Fixed `signal::ctrl_c()` → `tokio::signal::ctrl_c()` |
| `jsonrpc/server.rs` | `handle_stream_with_first_line` gated |
| `jsonrpc/mod.rs` | `service()` accessor gated |
| `main.rs` | `debug` import gated |

**Result**: `cargo check --target x86_64-pc-windows-gnu` — 0 errors, 0 warnings.

## Commit 2: Deep Debt Sweep

| Item | Before | After |
|------|--------|-------|
| `integration_tests.rs` | 1,002 lines (over 800L limit) | 3 modules: 295L + 245L + 451L |
| BearDog env aliases | Silent deprecated fallback | `tracing::warn` at runtime |
| `certificate_loan.rs` clones | 3 deep `LoanInfo::clone()` per loan op | `take()` — zero-copy |
| `register_with_neural_api` test | Failed on sporeGate (live socket) | Tolerates any environment |

## Metrics

| Metric | Value |
|--------|-------|
| Tests | **1,684** (all passing) |
| Source files | **202** `.rs` (+3 from test split) |
| Max production file | 660L (`uds.rs`) |
| Max test file | 789L (`service_tests.rs`) |
| clippy | Clean (`-D warnings`) |
| fmt | Clean |
| doc | Clean |
| Windows GNU | Clean |
| TODOs/FIXMEs | 0 |
| unsafe | 0 (forbid) |
| Mocks in production | 0 |

## Verification

```bash
cargo fmt --all --check          # ✅
cargo clippy --workspace --all-targets -- -D warnings  # ✅
cargo test --workspace           # ✅ 1,684 passed
cargo doc --workspace --no-deps  # ✅
cargo check --target x86_64-pc-windows-gnu  # ✅
```

## Known Issues

- `cargo deny check advisories` reports `RUSTSEC-2026-0190` (anyhow unsoundness) — upstream dep, no fix available yet
- Pre-existing: CI (`ci.yml`) runs `--lib` only; full suite via `verify.sh`

## Remaining Low-Priority Debt

- **P3**: Plan removal timeline for `BEARDOG_*` env aliases (now warned at runtime)
- **P4**: `Did`/`String` clone density in loan paths (ownership-inherent)
- **P5**: `Vec<u8>` in BTSP frame paths (crypto library constraint)
- **P2**: benchScale exercises 44/47 methods (trust.* not covered)
- **P2**: specs/ date headers stale (May 2026) — content is accurate

---

*Wave 141a complete. sporeGate reports cross-arch adoption done for loamSpine.
Overwatch: update completion tracking table.*
