<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# loamSpine Wave 155d — Structural Extraction + Schema Evolution

**Date**: July 28, 2026  
**From**: sporeGate → wateringHole  
**Wave**: 155d  
**Status**: Internal deep debt (Tower-independent work during Tower Atomic hardening phase)

---

## Context

Wave 155d reoriented priority: Tower Atomic hardening first, Nest Atomic (G3
IPC wiring) deferred until Tower is proven stable on existing gates. loamSpine's
G3 items (`MintingAuthority` validation, discovery caller wiring) wait for Tower
stability. This wave focuses on internal structural debt that improves code
quality independent of cross-primal sequencing.

---

## What Shipped

### 1. Entry Module Structural Extraction

**Problem**: `entry/mod.rs` at 648 lines combined the `Entry` struct, its
methods, and 21 `EntryType` variants + `AnchorTarget` + `SpineConfig` +
`SpineType` in a single file. Approaching the 800L soft limit.

**Solution**: Smart extraction — types that define *what entries contain* moved
to `entry/types.rs` (437L), while the `Entry` struct and its *methods* remain in
`entry/mod.rs` (227L). All existing imports preserved via `pub use` re-exports.
Test modules get types via `#[cfg(test)]` imports in the parent.

| File | Before | After |
|------|--------|-------|
| `entry/mod.rs` | 648L | 227L |
| `entry/types.rs` | — | 437L |

Zero downstream changes required — all `use crate::entry::EntryType` paths
continue to work.

### 2. Schema Orphan Evolution

**Problem**: `CertificateHistory`, `OwnershipRecord`, `LoanRecord`, and
`AcquisitionType` were defined but never constructed anywhere in the codebase.
They represented a future API that was never implemented.

**Solution**: Implemented `CertificateHistory::from_certificate_and_entries()`
which parses raw lifecycle entries into structured typed records:

- **Ownership records** track each owner with `AcquisitionType` (Mint,
  Transfer), timestamps, and entry hashes
- **Loan records** correlate `CertificateLoan` entries with their
  `CertificateReturn` via a pending-loan index
- `CertificateHistory` now derives `Serialize`/`Deserialize` for wire transport

### 3. New RPC Surface

- **`certificate.history`** — Returns structured `CertificateHistoryResponse`
  with typed ownership and loan records (vs `certificate.lifecycle` which returns
  raw entries). Total: **48 JSON-RPC methods**.
- **`LoamSpineService::certificate_history()`** — Core service method that
  fetches certificate + lifecycle entries and builds structured history.

### 4. Tests

3 new tests:
- `certificate_history_returns_typed_records` — mint + transfer → 2 ownership
  records with proper `AcquisitionType` and timestamp tracking
- `certificate_history_tracks_loan_records` — mint + loan + return → 1
  ownership + 1 completed loan record with correlation
- `certificate_history_rpc_returns_typed_records` — API-level RPC test

**Total**: 1,739 tests, 211 source files, all checks clean.

---

## What's Deferred (Wave 155d posture)

Per overwatch directive, these await Tower Atomic stability:

- **G3 `MintingAuthority` validation** in `mint_certificate()` — requires
  cross-primal trust anchor (rhizoCrypt)
- **Discovery caller wiring** — `find_by_capability`, `negotiate_protocol` into
  Nest Atomic IPC callers
- **Cross-primal certificate provenance** — rhizoCrypt → loamSpine
  `certificate.verify` integration

---

## File Changes

| File | Change |
|------|--------|
| `crates/loam-spine-core/src/entry/types.rs` | **NEW** — extracted types |
| `crates/loam-spine-core/src/entry/mod.rs` | Reduced to Entry struct + re-exports |
| `crates/loam-spine-core/src/certificate/mod.rs` | `CertificateHistory` impl |
| `crates/loam-spine-core/src/service/certificate.rs` | `certificate_history()` method |
| `crates/loam-spine-core/src/service/certificate_tests.rs` | 2 new tests |
| `crates/loam-spine-api/src/types/certificate.rs` | History request/response types |
| `crates/loam-spine-api/src/service/certificate_ops.rs` | RPC handler |
| `crates/loam-spine-api/src/service/service_tests.rs` | 1 new test |
| `crates/loam-spine-api/src/jsonrpc/mod.rs` | Dispatch entry |
| Root docs (README, STATUS, CONTEXT, CHANGELOG, WHATS_NEXT) | Updated counts |

---

## Verification

```
cargo fmt --all          ✓ (no changes)
cargo clippy --workspace ✓ (0 warnings, pedantic + nursery)
cargo test --workspace   ✓ (1,739 passed, 0 failed)
cargo doc --workspace    ✓ (clean)
```
