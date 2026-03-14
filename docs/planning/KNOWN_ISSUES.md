# Known Issues - LoamSpine

**Date**: March 14, 2026
**Version**: 0.8.1
**Status**: Production Ready

---

## Current State

As of v0.8.1 (March 14), the codebase passes all quality gates:

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Tests | 400+ | 700 | PASS |
| Line Coverage | 90% | 90.62% | PASS |
| Clippy (all targets, all features) | 0 warnings | 0 | PASS |
| Formatting | clean | clean | PASS |
| Documentation | compiles | compiles | PASS |
| Unsafe Code | 0 | 0 | PASS |
| Max File Size | <1000 lines | 810 max | PASS |
| Source Files | — | 88 | — |
| License | AGPL-3.0-only | AGPL-3.0-only | PASS |
| SPDX Headers | all files | all files | PASS |
| cargo deny (bans, licenses, sources) | pass | pass | PASS |
| ecoBin (zero C deps default) | pass | pass (ring via opt feature) | PASS |
| UniBin subcommands | server, capabilities, socket | all present | PASS |

---

## Remaining Technical Debt

### Medium

1. **Remaining uncovered code** (9.38%): Concentrated in areas requiring live external services:
   - `main.rs` (0% — binary entry point, tested via integration demos)
   - `neural_api.rs` (51% — requires live NeuralAPI socket)
   - `jsonrpc/mod.rs` (68% — macro-generated trait impls)
   - `infant_discovery.rs` (81% — DNS SRV and mDNS paths)
2. **Storage backends**: `Sqlite` implemented (feature-gated); `Postgres`, `Rocksdb` planned.
3. **mDNS**: Real implementation via `mdns` crate v3.0. Requires network access for LAN discovery.
4. **Showcase demos**: ~10% complete (2/21 demos fully implemented); remaining are documented/scaffolded.

### Minor

1. **`thiserror` duplicate versions**: v1 and v2 coexist via transitive deps (non-blocking).
2. **`proc-macro-error` advisory**: Transitive dependency of optional `mdns` feature (not enabled by default).
3. **`reqwest` pulls `ring`**: Only via optional `discovery-http` feature; default path uses pure-Rust `tower-atomic`.

### Resolved (March 13)

- v0.7.0 deprecated items removed (`songbird_enabled`, `songbird_endpoint`, `with_songbird`, `DependencyHealth::songbird`)
- `DiscoveryMethod::Songbird` constant removed (use `ServiceRegistry`)
- `MockTransport` dead code warnings resolved (made `pub`, re-exported for downstream testing)
- `u32 as u8` base64 truncation cast resolved (explicit `#[allow]` with invariant proof)
- Redundant clone in `http.rs` test resolved
- `TransportResponse.body` evolved from `Vec<u8>` to `Bytes` (zero-copy)
- Hardcoded `"../bins/beardog"` paths in `cli_signer.rs` replaced with `CliSigner::discover_binary()`
- `"did:key:anonymous"` default removed; commits now require explicit committer DID
- `unwrap_or_default()` in production code eliminated
- `u32 as usize` network casts replaced with `try_from`
- `entry.rs` JSON serialization replaced with deterministic `bincode` + sorted metadata
- `Entry.metadata` evolved from `HashMap` to `BTreeMap` for inherent canonical ordering
- `tarpc_server.rs` split: production code (240 lines) + test file (810 lines) — under 1000-line limit
- Test coverage raised from 88.64% to 90.62% (635 → 700 tests) covering lifecycle, spine, entry, tarpc, integration, backup paths

### None Critical

No critical issues, blockers, or security concerns.

---

**Last Updated**: March 14, 2026
