# Known Issues - LoamSpine

**Date**: March 13, 2026
**Version**: 0.8.0
**Status**: Production Ready

---

## Current State

As of v0.8.0 (March 13 deep debt pass), the codebase passes all quality gates:

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Tests | 400+ | 610 | PASS |
| Line Coverage | 90% | 90%+ | PASS |
| Clippy (all targets) | 0 warnings | 0 | PASS |
| Formatting | clean | clean | PASS |
| Documentation | compiles | compiles | PASS |
| Unsafe Code | 0 | 0 | PASS |
| Max File Size | <1000 lines | 899 max | PASS |
| License | AGPL-3.0-only | AGPL-3.0-only | PASS |
| SPDX Headers | all files | all files | PASS |
| cargo deny (bans, licenses, sources) | pass | pass | PASS |
| ecoBin (zero C deps) | pass | pass | PASS |
| UniBin subcommands | server, capabilities, socket | all present | PASS |

---

## Remaining Technical Debt

### Minor

1. **mDNS experimental**: Feature-gated behind `mdns` feature; stub implementation returns empty when feature enabled.
2. **`main.rs` 0% coverage**: Binary entry point; tested via integration/showcase demos, not unit tests.
3. **`thiserror` duplicate versions**: v1 and v2 coexist via transitive deps (non-blocking).
4. **Storage backends**: `Sqlite`, `Postgres`, `Rocksdb` enum variants defined but not implemented (planned work).
5. **Showcase demos**: 10% complete (2/21 demos fully implemented); remaining are documented/scaffolded.
6. **`proc-macro-error` advisory**: Transitive dependency of optional `mdns` feature (not enabled by default).

### Resolved (March 13)

- `songbird` deprecated alias removed (was: scheduled for v1.0.0 removal)
- `unwrap_or_default()` in production code eliminated (entry.rs, transport/neural_api.rs, cli_signer.rs)
- `u32 as usize` network casts replaced with `try_from`
- `MockTransport` dead code warnings resolved via `cfg(test)` gating
- `entry.rs` JSON serialization replaced with deterministic `bincode` + sorted metadata

### None Critical

No critical issues, blockers, or security concerns.

---

**Last Updated**: March 13, 2026
