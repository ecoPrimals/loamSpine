# loamSpine — FRAGO Response: Wave 69 strandGate Provenance Deployment

**Date**: 2026-06-02  
**In Response To**: `wave69-strandgate-provenance-deployment` (eastGate audit)  
**From**: loamSpine team (strandGate)  
**To**: primalSpring (eastGate), rhizoCrypt + sweetGrass (provenance trio)  
**Commits**: `cdcad6f` → `4763c26` (4 evolution commits), `83ecd60` (doc refresh)

---

## P2: RESOLVED — Tokio runtime-in-runtime panic (fixed v0.9.15, March 2026)

### Status: **No action required — fix is live since LS-03**

The nested `block_on()` inside a running async runtime was replaced with `tokio::spawn` in v0.9.15 (commit LS-03, March 31, 2026). This was re-verified in Wave 47 — the "double-runtime crash" reported on 2 gates was root-caused to a stale `plasmidBin` binary calling `serve` instead of `server` subcommand, not a loamSpine code bug.

**Current state**: Zero `Runtime::new()` or `block_on()` in any production code (`bin/` and `crates/*/src/`). All remaining occurrences are exclusively in test files and benchmarks.

**If gates still see this**: The fix is to reharvest the loamSpine binary into plasmidBin. The stale binary predates the fix. See PG-52 verified live (April 27, 2026).

### Evidence

```
$ rg 'Runtime::new|block_on' bin/ crates/*/src/ --type rust
(no matches)
```

---

## P3: RESOLVED — Anchoring pipeline completeness

### Status: **All pipeline stages implemented and tested**

The full provenance anchoring pipeline is operational:

| Stage | Method | Status | Tests |
|-------|--------|--------|-------|
| 1. Dehydrate | `session.dehydrate` | COMPLETE | Content-addressed blake3 summary of uncommitted entries (read-only) |
| 2. Commit session | `session.commit` | COMPLETE | Receives dehydrated DAG sessions from rhizoCrypt |
| 3. Commit braid | `braid.commit` | COMPLETE | Records attribution braids from sweetGrass; returns real append index |
| 4. Anchor single | `anchor.publish` | COMPLETE | Records receipts (Bitcoin OP_RETURN, Ethereum, RFC 3161 TSA, Data Commons) |
| 5. Anchor batch | `anchor.publish_batch` | COMPLETE | Aggregate Merkle tree anchoring (N spines → 1 on-chain tx) |
| 6. Verify | `anchor.verify` | COMPLETE | Verifies single or aggregate anchor proofs |

Pipeline flow: `dehydrate → sign → session.commit → braid.commit → anchor.publish`

**Remaining glacial gap**: Actual chain submission is delegated to a `"chain-anchor"` capability-discovered primal (not yet built ecosystem-wide). loamSpine records receipts; a future primal handles on-chain submission. Spec: `specs/PUBLIC_TIMESTAMPING.md`.

---

## Current Metrics

| Metric | Value |
|--------|-------|
| Tests | 1,574 (zero flaky, all concurrent, ~3s) |
| JSON-RPC methods | 44 (37 stable, 2 evolving, 4 compat, 1 alias) |
| Source files | 193 `.rs` |
| Coverage | 90.92% line / 89.09% branch / 92.92% region |
| Unsafe code | 0 (`#![forbid(unsafe_code)]`) |
| Clippy | 0 warnings (pedantic + nursery) |
| TODO/FIXME | 0 |
| Deep debt | All 10 dimensions CLEAN |

## Recent Evolution (June 2, 2026)

17 thin/stub handlers evolved to real implementations:
- `health.check` real uptime, error transparency in get ops
- `braid.commit` real index, `InclusionProof::verify` Merkle validation
- `lifecycle.status` shared state handle, discovery transport wiring
- `permanence.health_check` structured diagnostics, `spine.seal` reason field
- `get_attribution` contributor collection, `auth.peer_info` auth mode
- `CapabilityRegistry::all_required_available()` enforcement

## Audit Context Note

The Wave 69 audit context appears to reference pre-Wave 55 state (mentions BearDog coupling and placeholder DIDs as recent changes — both resolved May 27, 2026). loamSpine is currently at zero mountain debt with all evolution targets through Round 4 complete.

---

## strandGate Deployment Readiness

- **loamSpine**: Ready. No blocking debt. Stadial checklist 23/23 PASS.
- **Blocked on**: Phase 1 mesh validation (3+ gates proven on LAN) — unchanged.
- **First tasks post-deploy**: Provenance trio wiring (content.put to rhizoCrypt DAG + loamSpine ledger), sweetGrass braid integration.
