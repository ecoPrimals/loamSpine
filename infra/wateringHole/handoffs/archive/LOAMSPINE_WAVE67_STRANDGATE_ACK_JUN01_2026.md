<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# loamSpine — Wave 67: strandGate Provenance Gate Ack

**Date**: June 1, 2026
**Wave**: 67
**Status**: ACK — mountain ready, deploy blocked on Phase 1 mesh
**Artifact**: `primals/loamSpine` @ `main`
**Impulse**: `2026-06-01T13-32_eastGate__wave67-strandgate-provenance-compute-gate-deploy.toml`

---

## Context

Wave 67 glacial cutover assigns loamSpine to **strandGate** as part of the provenance trio (rhizoCrypt + loamSpine + sweetGrass). Hardware is ready (Dual EPYC 7452, 256GB ECC). Deployment is blocked on Phase 1 mesh validation (southGate P0: Songbird socket, biomeOS `capability.call`, bearDog S4).

## Mountain Status

loamSpine is **production-ready** and **VPS-deployed** in the Nest Atomic stack on cellMembrane/golgiBody.

| Metric | Value |
|--------|-------|
| Version | 0.9.16 |
| Tests | 1,533 (1,530 default, +3 with `dns-srv`) |
| JSON-RPC methods | 44 (38 stable, 2 evolving, 4 compat) |
| Source files | 193 `.rs` (+ 3 fuzz targets) |
| Clippy warnings | 0 |
| Unsafe code | 0 (`#![forbid(unsafe_code)]`) |
| C dependencies (default) | 0 (pure Rust) |
| Coverage | 90.92% line |

## Wave 67 Debt Resolution

| Item | Status |
|------|--------|
| `#[allow(dead_code)]` → `#[expect(dead_code)]` | **DONE** — 4 pre-wired strandGate deploy entry points evolved from silent suppression to documented expectation |
| `#[allow]` audit | **CLEAN** — 4 production `#[allow]` (all with `reason`), 0 without |
| TODO/FIXME in code | **CLEAN** — zero |
| `.unwrap()` in production | **CLEAN** — zero |
| `as` casts in production | **CLEAN** — zero (all `try_from`) |
| Hardcoded `/tmp` | **CLEAN** — zero in production |
| Test suite | **PASS** — 1,530 default, 1,533 with dns-srv |

## Pre-Wired for strandGate Deploy

Four functions evolved to `#[expect(dead_code)]` — these are env-reading entry points that will be wired when strandGate deploys:

1. `resolve_primal_socket_with_env` — provenance trio socket resolution
2. `negotiate_protocol` — tarpc/JSON-RPC IPC negotiation
3. `find_by_capability` — capability-based primal discovery
4. `find_by_name` — name-based primal discovery

When wired, `#[expect]` will emit a compiler warning reminding to remove the annotation.

## Open Upstream Items (Not Blocking Glacial)

| Item | Status | Source |
|------|--------|--------|
| Provenance trio wiring (`content.put` → loamSpine ledger) | Blocked on mesh | strandGate impulse |
| WS-3 public chain anchor | Glacial goal, spec exists | `specs/PUBLIC_TIMESTAMPING.md` |
| sweetGrass braid integration with sporePrint | Post-deploy | `GLACIAL_CUTOVER_PLAN.md` |
| Cross-gate compute dispatch from biomeGate | Post-deploy | `GLACIAL_CUTOVER_PLAN.md` |

## Critical Path

```
southGate P0 fixes → Phase 1 mesh (3+ gates) → strandGate deploy → provenance trio wiring
```

loamSpine awaits strandGate deployment. No mountain debt blocks glacial.
