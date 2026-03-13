# Known Issues - LoamSpine

**Date**: March 12, 2026
**Version**: 0.8.0
**Status**: Production Ready

---

## Current State

As of v0.8.0, the codebase passes all quality gates:

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Tests | 400+ | 510+ | PASS |
| Line Coverage | 90% | 90.08% | PASS |
| Clippy (all targets, all features) | 0 warnings | 0 | PASS |
| Formatting | clean | clean | PASS |
| Documentation | compiles | compiles | PASS |
| Unsafe Code | 0 | 0 | PASS |
| Max File Size | <1000 lines | 863 max | PASS |
| License | AGPL-3.0-only | AGPL-3.0-only | PASS |
| SPDX Headers | all files | all files | PASS |
| cargo deny (bans, licenses, sources) | pass | pass | PASS |
| Pure Rust deps (no openssl/native-tls) | pass | pass | PASS |

---

## Remaining Technical Debt

### Minor

1. **Deprecated config fields**: `songbird_enabled`, `songbird_endpoint` in `DiscoveryConfig` -- scheduled for removal in v1.0.0.
2. **mDNS experimental**: Feature-gated behind `mdns` feature; stub implementation returns empty when feature enabled.
3. **`main.rs` 0% coverage**: Binary entry point; tested via integration/showcase demos, not unit tests.
4. **`thiserror` duplicate versions**: v1 and v2 coexist via transitive deps (non-blocking, `cargo deny` warns).

### None Critical

No critical issues, blockers, or security concerns.

---

## v0.8.0 Changes (This Release)

- UniBin compliance: binary renamed to `loamspine`, subcommand structure (`loamspine server`)
- AGPL-3.0-only LICENSE file, SPDX headers on all source files
- Semantic JSON-RPC method naming (`spine.create`, `certificate.mint`, etc.)
- DNS-SRV discovery activated in default config
- `service.rs` refactored into domain-focused modules
- `#[allow]` cleanup and `cast_possible_truncation` evolved to `try_into()`
- Service registry discovery evolved from warning stub to real HTTP-based implementation
- `reqwest` switched from `native-tls` to `rustls-tls` for pure Rust TLS (ecoBin compliance)
- `deny.toml` updated: `AGPL-3.0-only`, `CDLA-Permissive-2.0` licenses allowed
- Environment-touching tests serialized with `#[serial]` to prevent race conditions
- Coverage pushed from 87% to 90.08% with targeted tests across 8 files
- 510+ tests (up from 495), 90.08% line coverage (up from 87%)
- Version aligned to 0.8.0 across workspace

---

**Last Updated**: March 12, 2026
