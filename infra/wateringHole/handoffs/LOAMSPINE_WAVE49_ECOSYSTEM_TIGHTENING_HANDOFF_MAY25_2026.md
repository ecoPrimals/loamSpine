# loamSpine Wave 49 — Ecosystem Tightening Handoff

**Date**: May 25, 2026  
**Scope**: All 3 cleanup vectors from primalSpring Wave 49 audit  
**Reference**: `primalSpring/wateringHole/WAVE49_ECOSYSTEM_TIGHTENING.md`

---

## Audit Items — Resolution

### Vector A: Stale Deployment Patterns — RESOLVED

| Pattern | Location | Resolution |
|---------|----------|------------|
| `target/release/loamspine` | `infra/benchScale/validate_roundtrip.sh` | Updated: now checks `LOAMSPINE_BINARY` env var first, then `target/release`, then `target/debug`, then PATH via `command -v` |
| `target/release/loamspine` | `showcase/RUN_ME_FIRST.sh` | Removed: showcase fossilized |

No `which loamspine` or `cargo install` patterns found in any script.

### Vector B: wateringHole Consolidation — ALREADY COMPLIANT

`infra/wateringHole/handoffs/` is the only `wateringHole/` directory in the repo.
Contains 14 handoffs (May 5–25, 2026), all in the canonical location.
No stale local `wateringHole/` tree outside `infra/`.

### Vector C: Showcase Fossilization — COMPLETE

- **47 files** (420K) copied to `ecoPrimals/fossilRecord/primals/loamSpine/showcase_wave49/`
- `showcase/` directory replaced with `showcase/README.md` pointer
- Pointer documents: fossil record location, what was archived, active validation
  (benchScale), and runnable Rust examples that remain in crates

### Pipeline Debt: Tokio Runtime-in-Runtime — ALREADY FIXED

The audit notes "Tokio runtime-in-runtime panic on health probe." This was:

1. **LS-03** (v0.9.15): Nested `block_on()` in infant discovery → replaced with `tokio::spawn`
2. **PG-33** (v0.9.16): `mdns` 3.0 → `mdns-sd` 0.19 — structurally eliminates nested-runtime risk
3. **Wave 47**: Reported "Tokio double-runtime crash" was misdiagnosed — actual cause was
   `serve` vs `server` CLI mismatch in `plasmidBin/start_primal.sh`

Current state: **Zero `Runtime::new()` or `block_on()` in production code.**
Health handlers (`liveness`, `readiness`, `check`) are simple async methods with
no discovery, no I/O blocking, no nested runtimes.

---

## Verification Checklist

- [x] `showcase/` contains only `README.md` pointer to fossilRecord (per Wave 49 recipe)
- [x] Local `wateringHole/` handoffs already in `infra/wateringHole/handoffs/`
- [x] No `which loamspine` or `target/release/loamspine` in deployment scripts
- [x] `notify-plasmidbin.yml` active in `.github/workflows/`
- [x] Commit message references Wave 49 ecosystem tightening

---

## Current State

| Metric | Value |
|--------|-------|
| Tests | 1,528 (all concurrent, zero flaky) |
| Methods | 43 JSON-RPC (semantic naming) |
| Source files | 189 `.rs` |
| benchScale | 51 validations, 0 failures |
| Unsafe | 0 (`forbid(unsafe_code)`) |
| Pipeline debt | Zero open items |
