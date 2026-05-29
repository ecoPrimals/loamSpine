# loamSpine — Wave 55 Deep Debt: Primal Self-Knowledge Enforcement

**Date**: May 27, 2026  
**Version**: 0.9.16  
**Tests**: 1,528 (all passing, zero flaky)  
**Clippy**: Zero warnings  

---

## Summary

Wave 55 enforces primal self-knowledge by removing all BearDog-specific coupling
from production code and documentation. loamSpine now references capabilities
(tower signer, BTSP provider) instead of primal names.

---

## Changes

### BearDog Coupling Removed

| Before | After | Backward Compat |
|--------|-------|-----------------|
| `BEARDOG_SOCKET` | `TOWER_SIGNER_SOCKET` | `BEARDOG_SOCKET` accepted as deprecated fallback |
| `BEARDOG_FAMILY_SEED` | `BTSP_FAMILY_SEED` | `BEARDOG_FAMILY_SEED` accepted as deprecated fallback |
| `did:key:tower` | `TOWER_SIGNER_DID` env → `Did::anonymous()` | Config-driven |
| `did:key:unknown` | `Did::anonymous()` (`did:primal:anonymous`) | Sentinel with `is_anonymous()` predicate |
| Doc comments: "BearDog crypto.sign_ed25519" | "tower signer's crypto.sign_ed25519" | — |

### Dead Code and Visibility

- `IntoByteBuffer` trait removed (unused outside its defining file)
- 8 `pub` items tightened to `pub(crate)`: `manifest_dir`, `discover_manifests`, `find_by_capability`, `find_by_name`, `negotiate_protocol`, `resolve_primal_socket_with_env`, `DispatchOutcome::is_ok`
- `#[allow(clippy::unused_async)]` → `#[expect]` where configuration-stable (uds.rs)

### Storage Documentation

- `Postgres`/`Rocksdb` enum variants: "Roadmap — feature flag not yet defined"
- `LoamSpineService` doc: acknowledges in-memory storage with redb as standalone backend
- `testing` feature flag: documented as downstream-only

### Documentation Updated

- STATUS.md, WHATS_NEXT.md, KNOWN_ISSUES.md — date bumps, Wave 55 entries
- specs/API_SPECIFICATION.md — `BEARDOG_SOCKET` → `TOWER_SIGNER_SOCKET`
- specs/ARCHITECTURE.md — date bump (was December 2025)
- sporeprint/validation-summary.md — capability table updated

---

## Verification

```
cargo check --tests   → 0 errors, 0 warnings
cargo clippy --all-targets → 0 warnings
cargo test            → 1,528 passed, 0 failed
```

---

## For primalSpring Audit

- **No breaking changes** — deprecated env vars still accepted
- **Self-knowledge clean** — zero primal names in production code or doc comments
- All historical changelog entries preserved as fossil record
