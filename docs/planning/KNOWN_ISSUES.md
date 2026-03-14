# Known Issues - LoamSpine

**Date**: March 14, 2026
**Version**: 0.8.2
**Status**: Production Ready

---

## Current State

As of v0.8.2 (March 14), the codebase passes all quality gates:

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Tests | 400+ | 744+ | PASS |
| Line Coverage | 90% | ~91% | PASS |
| Clippy (all targets, all features) | 0 warnings | 0 | PASS |
| Formatting | clean | clean | PASS |
| Documentation | compiles (-D warnings) | compiles | PASS |
| Unsafe Code | 0 | 0 | PASS |
| Max File Size | <1000 lines | 422 max | PASS |
| Source Files | — | 92 | — |
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
5. **Spec gaps**: Waypoint relending chain and expiry sweep; certificate `generate_provenance_proof`, escrow/`TransferConditions`; SyncProtocol federation.
6. **`#[allow]` attributes**: ~10 in production code (mostly `unused_self`, `unused_async` — architecturally justified stubs).

### Minor

1. **`thiserror` duplicate versions**: v1 and v2 coexist via transitive deps (non-blocking).
2. **`proc-macro-error` advisory**: Transitive dependency of optional `mdns` feature (not enabled by default).
3. ~~**`reqwest` pulls `ring`**~~: **RESOLVED** — `reqwest` replaced with `ureq` (pure Rust, no TLS); `ring` explicitly banned in `deny.toml`.
4. **Build infra**: Global `CARGO_TARGET_DIR` on noexec mount requires `CARGO_TARGET_DIR` env override.

### Resolved (March 14 — v0.8.2)

- Certificate storage evolved from `RwLock<HashMap>` to `CertificateStorage` trait + `InMemoryCertificateStorage`
- `must_use_candidate` lint enabled crate-wide (11 functions annotated)
- `discovery.rs` (783 lines) refactored into `discovery/{mod,dyn_traits,tests}.rs`
- `manager.rs` (783 lines) refactored into `manager/{mod,tests}.rs`
- All production files now under 422 lines (was 810 max)
- Waypoint types module: `WaypointConfig`, `PropagationPolicy`, `DepartureReason`, `WaypointSummary`, `SliceOperationType`, `SliceTerms`
- `record_operation` and `depart_slice` added to service
- `verify_certificate` with enum-based `VerificationCheck` results
- `certificate_lifecycle` for filtered entry history
- `MintInfo.entry` bug fixed (was `[0u8; 32]`, now actual entry hash)
- 25 new tests (719 → 744)

### Resolved (March 14 — v0.8.1)

- `Did` type evolved from `String` to `Arc<str>` for O(1) cloning
- `ServiceState` enum added per SERVICE_LIFECYCLE.md spec
- Observable lifecycle state via `tokio::sync::watch` channel
- Broken rustdoc links in `transport/mock.rs` fixed
- `storage_backend_availability` test now feature-aware
- `match_same_arms` lint resolved in `error.rs`
- `println!` in binary evolved to `writeln!(stdout())`
- Hardcoded `"loamspine"` strings replaced with `PRIMAL_NAME` constant
- `TransportResponse::from_static` for zero-copy mock responses
- `collect::<Vec<_>>()` anti-pattern eliminated
- Specs updated from `loamspine.camelCase` to semantic `domain.operation` naming
- Duplicate NeuralAPI registration code extracted

### Resolved (March 13 — v0.8.0)

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
- Test coverage raised from 88.64% to 90.62% (635 → 700+ tests)

### None Critical

No critical issues, blockers, or security concerns.

---

**Last Updated**: March 14, 2026
