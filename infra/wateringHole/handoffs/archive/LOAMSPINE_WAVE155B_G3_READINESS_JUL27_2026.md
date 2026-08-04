# loamSpine — Wave 155b: G3 Readiness + Doc Accuracy

**Date**: July 27, 2026  
**From**: loamSpine team (eastGate)  
**Wave**: 155b  
**Status**: COMPLETE

---

## Summary

G3 (Loam Certificate minting validation) test readiness. Full certificate
lifecycle E2E with seal added. Dead code stubs realigned from "strandGate
deploy" to "Nest Atomic Phase 0 (G3)". Inline timeouts extracted to named
constants. Doc accuracy corrected across 7 files.

---

## Changes

### G3 Test Readiness

- New E2E integration test `full_certificate_lifecycle_with_seal`:
  mint → loan → return → transfer → seal → reject-on-sealed.
  Validates that sealed spines are permanently immutable.
- Closes the last certificate lifecycle coverage gap.

### Dead Code Alignment (Nest Atomic Phase 0)

Pre-wired discovery/protocol helpers updated to reflect G3 roadmap:

| Function | Old reason | New reason |
|----------|-----------|------------|
| `find_by_capability` | "strandGate deploy" | "Nest Atomic Phase 0 (G3)" |
| `find_by_name` | "strandGate deploy" | "Nest Atomic Phase 0 (G3)" |
| `negotiate_protocol` | "provenance trio IPC" | "Nest Atomic Phase 0 (G3)" |
| `resolve_primal_socket_with_env` | "provenance trio socket" | "Nest Atomic Phase 0 (G3)" |
| `find_by_capability_from` | "injectable variant" | "Nest Atomic Phase 0 (G3)" |
| `find_by_name_from` | "injectable variant" | "Nest Atomic Phase 0 (G3)" |

These functions are tested and ready to wire when Nest Atomic callers land.

### Configurable Timeouts

- Health probe: `Duration::from_secs(5)` → `HEALTH_PROBE_TIMEOUT` named constant.
- Attestation: inline `from_secs(5)` → `DEFAULT_IPC_TIMEOUT` (shared with framing).

### Doc Accuracy

| Fix | Files |
|-----|-------|
| "52-validation" → "20-phase, 44-method" | README, ARCHITECTURE, sporeprint |
| "Zero `#[allow]`" → "2 justified `#![allow]`" | README, CONTRIBUTING, sporeprint |
| Dead_code count "4" → "3" | README |
| KNOWN_ISSUES date Jul 21 → Jul 26 | KNOWN_ISSUES |

### Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests | 1,731 | 1,732 |
| Source files | 210 | 210 |
| Clippy warnings | 0 | 0 |
| Stale dead_code reasons | 6 | 0 |
| Inline production timeouts | 2 | 0 |

---

## Audit Findings (Deferred)

| Item | Priority | Rationale |
|------|----------|-----------|
| `Vec<u8>` → `Bytes` on 21 wire paths | P1 | Hot-path alloc; defer to dedicated zero-copy pass |
| Certificate loan clone refactor | P2 | 12 clones structurally required; helper pattern could cut 4-6 |
| Typed IPC errors (`IpcOperation` enum) | P2 | Duplicate format strings; improve programmatic matching |
| CI feature matrix (`dns-srv`, `mdns`, integration tests) | P0 | CI runs `--lib` only (1,589 of 1,732); full suite runs locally |
| G3: primalSpring mint authority validation | P0 | `MintingAuthority` exists but never populated or verified |

---

## For Upstream Teams

**nestGate**: G3 Phase 0 depends on nestGate CAS integration. loamSpine has
discovery helpers pre-wired and ready to call.

**primalSpring**: G3 mint authority validation scenario needed — loamSpine's
`MintingAuthority` struct exists but has no validation path yet.
