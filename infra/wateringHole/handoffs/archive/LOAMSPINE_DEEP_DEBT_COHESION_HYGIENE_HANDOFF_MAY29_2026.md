<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# loamSpine — Deep Debt: Test Cohesion, Dependency Hygiene, Pure Rust Default

**Date**: May 29, 2026
**Wave**: Post-Wave 60 deep debt pass
**Status**: COMPLETE
**Artifact**: `primals/loamSpine` @ `main`

---

## Summary

Internal deep debt cleanup following Wave 60 (`session.dehydrate`). Three focus areas:

### 1. Test Cohesion — jsonrpc test split

`crates/loam-spine-api/src/jsonrpc/tests.rs` (876 lines) refactored into 5 domain-cohesive modules:

| Module | Lines | Domain |
|--------|-------|--------|
| `tests.rs` | 120 | Shared helpers, server creation, health, core capabilities/identity |
| `tests_spine_entry.rs` | 233 | Spine lifecycle, entry CRUD, certificates, slices |
| `tests_session.rs` | 194 | Session (dehydrate/commit) and braid operations |
| `tests_proof_anchor.rs` | 155 | Inclusion proofs, public chain anchoring |
| `tests_wire_errors.rs` | 195 | JSON-RPC protocol error handling and edge cases |

### 2. Dependency Hygiene

- **Removed** unused `anyhow` from `loam-spine-api/Cargo.toml`
- **Hoisted** `base64` to workspace `[workspace.dependencies]` (single source of truth for version)

### 3. Pure Rust Default Build

- **`dns-srv` removed from default features** in `loam-spine-core/Cargo.toml`
- Default build is now zero-C: `hickory-resolver` and `ring` only enter via opt-in `dns-srv` feature
- DNS SRV tests feature-gated with `#[cfg(feature = "dns-srv")]`
- `#[allow(unused_mut)]` added for conditionally-mutated variable in infant discovery

---

## Metrics

| Metric | Value |
|--------|-------|
| Source files | 193 `.rs` (+ 3 fuzz targets) |
| Tests (with dns-srv) | 1,533 |
| Tests (default build) | 1,530 |
| JSON-RPC methods | 44 (38 stable, 2 evolving, 4 compat) |
| Clippy warnings | 0 |
| C dependencies (default) | 0 |

---

## Validation

- `cargo check` — clean
- `cargo clippy --all-targets` — zero warnings
- `cargo test --workspace` — 1,530 pass (default)
- `cargo test --workspace --features dns-srv` — 1,533 pass
- benchScale roundtrip — 52 validations, 44 methods, 19 phases

---

## For primalSpring Audit

All 10 deep-debt dimensions pass. Default build is pure Rust (zero C deps). `dns-srv` is opt-in for deployments that need DNS SRV discovery. No regressions from Wave 60 `session.dehydrate` implementation.
