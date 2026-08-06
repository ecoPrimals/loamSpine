# loamSpine Wave 156h — G64 Cephalization: tarpc Convergence

**Date**: August 5, 2026  
**From**: sporeGate  
**Wave**: 156h  
**Status**: GREEN — 1,752 tests, 0 warnings, 0 unsafe, 37 tarpc + 53 JSON-RPC methods

---

## What Shipped

### tarpc Trait Expansion (24 → 37 domain methods)

All performance-critical domain operations now have tarpc parity with JSON-RPC.
Both protocols delegate to the same `LoamSpineRpcService` layer — identical behavior.

**13 new tarpc methods:**

| Category | Methods Added |
|----------|-------------|
| Spine | `list_spines`, `spine_status` |
| Entry | `append_entry_batch`, `list_entries` |
| Certificate | `mint_certificate_batch`, `verify_certificate`, `certificate_lifecycle`, `certificate_history` |
| Anchor | `publish_anchor_batch` |
| Trust | `trust_anchor`, `trust_query`, `trust_event_count` |
| BTSP | `negotiate_btsp` |

### Dual-Protocol Architecture

| Protocol | Methods | Purpose |
|----------|---------|---------|
| tarpc | 37 | Performance path — all domain ops for primal-to-primal composition |
| JSON-RPC | 53 | Full surface — discovery bootstrap, diagnostics, external clients |
| JSON-RPC-only | 16 | Meta/diagnostic: auth.*, tools.*, identity.*, permanence.*, health.liveness/readiness, lifecycle.status, capabilities.list, btsp.capabilities, primal.announce |

### G64 Posture

loamSpine is now **tarpc-converged** for Phase 1. Remaining cephalization work:
- **UDS binding**: `tarpc.sock` server binding (path builders exist, runtime binding deferred to UDS protocol convergence C2)
- **JH-0 gate**: tarpc calls bypass method gate — acceptable for local-trust UDS, needs gating for TCP

---

## Verification

```
cargo fmt --all                 # clean
cargo clippy --workspace ...    # 0 warnings
cargo test --workspace          # 1,752 passed, 0 failed
cargo doc --workspace           # 0 warnings
```

---

## Files Touched

| File | Change |
|------|--------|
| `crates/loam-spine-api/src/rpc.rs` | 13 new trait methods + imports |
| `crates/loam-spine-api/src/tarpc_server.rs` | 13 new server impl methods |
| `CHANGELOG.md` | Wave 156h entry |
| `STATUS.md` | Wave 156h section |
| `WHATS_NEXT.md` | Wave 156h entry |
| `sporeprint/validation-summary.md` | tarpc method count |

---

## Cross-Primal Impact

- **sweetGrass**: `LedgerClient` can now use tarpc for batch provenance ops (`append_entry_batch`, `commit_session`)
- **rhizoCrypt**: `dag.pipeline.ingest` dehydrate→commit path available over tarpc
- **biomeOS**: Orchestrator can route domain ops to tarpc socket for sub-ms composition
- **squirrel**: Signal dispatch can target tarpc for high-frequency trust queries

---

*Wave 156h — G64 cephalization moves loamSpine from "tarpc-wired" to "tarpc-converged". 37 typed domain methods on tarpc, 53 on JSON-RPC. Performance path and discovery path are now separate and complete.*
