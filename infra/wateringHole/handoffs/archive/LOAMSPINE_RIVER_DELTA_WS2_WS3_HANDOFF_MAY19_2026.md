# loamSpine — River Delta: WS-2 Cross-Spring Query + WS-3 Public Timestamping

**Date:** 2026-05-19
**Audit:** Upstream Gaps — River Delta (Springs) (May 19, 2026)
**License:** AGPL-3.0-or-later

---

## Summary

Two items from the River Delta audit routed to loamSpine:

| Gap | Priority | Action | Status |
|-----|----------|--------|--------|
| WS-2: Cross-Spring Data Exchange | HIGH (shared) | Added `spine.list` + `entry.list` RPC methods | DONE |
| WS-3: Public Chain Anchor | MEDIUM | Wrote exploration spec for RFC 3161 TSA | DONE |

---

## WS-2: Cross-Spring Data Exchange (loamSpine participation)

### Problem

External springs in the provenance trio cannot enumerate loamSpine data.
`permanence.get_commit` and `entry.get` require a specific `entry_hash` —
there is no way to discover what spines exist or iterate entries.

### Solution

Exposed existing internal capabilities as JSON-RPC methods:

**`spine.list`** — List all spine IDs
- Request: `{}` (no params)
- Response: `{ spine_ids: [uuid, ...], count: N }`
- Internal: delegates to `LoamSpineService::list_spine_ids()` → `SpineStorage::list_spines()`

**`entry.list`** — List entries in a spine (paginated)
- Request: `{ spine_id, start?: 0, limit?: 100 }`
- Response: `{ entries: [...], count: N, has_more: bool }`
- Internal: delegates to `SpineQuery::get_entries(spine_id, start, limit + 1)` with overflow detection for `has_more`

### Files changed

| File | Change |
|------|--------|
| `crates/loam-spine-core/src/service/mod.rs` | Added `list_spine_ids()` public method |
| `crates/loam-spine-api/src/types/mod.rs` | Added `ListSpinesRequest/Response`, `ListEntriesRequest/Response` types |
| `crates/loam-spine-api/src/service/spine_ops.rs` | Added `list_spines()` handler |
| `crates/loam-spine-api/src/service/entry_ops.rs` | Added `list_entries()` handler |
| `crates/loam-spine-api/src/jsonrpc/mod.rs` | Wired `spine.list` and `entry.list` in dispatch table |
| `crates/loam-spine-core/src/niche.rs` | Added to METHODS array (42 total) |
| `crates/loam-spine-core/src/neural_api/mod.rs` | Added to CAPABILITIES, provided_capabilities, cost_estimates |
| `crates/loam-spine-core/src/neural_api/mcp.rs` | Added MCP tool definitions and `mcp_tool_to_rpc` mapping |
| `crates/loam-spine-api/src/jsonrpc/method_gate.rs` | Test coverage for Protected classification |
| `crates/loam-spine-api/src/jsonrpc/tests_validation.rs` | 3 new tests |

### Remaining (biomeOS-owned)

biomeOS defines the `rootpulse.sync` NeuralAPI composition graph that
orchestrates cross-spring provenance exchange. loamSpine now provides the
query surface (`spine.list`, `entry.list`, `entry.get`, `entry.get_tip`)
needed for that composition.

---

## WS-3: Public Chain Anchor — Exploration Spec

### Current state

`anchor.publish` / `anchor.verify` are fully implemented with a
chain-agnostic `AnchorTarget` enum. loamSpine records anchor receipts;
actual submission is delegated to a capability-discovered `"chain-anchor"`
primal. No such primal exists yet.

### Exploration spec

Created `specs/PUBLIC_TIMESTAMPING.md` with multi-target anchoring strategy:

| Target | Role | Cost |
|--------|------|------|
| Bitcoin OP_RETURN | **Public immutability** — stamp to public record | Gas only ($0.10–$2) |
| Ethereum/L2 event | Higher-frequency public anchoring | Gas only ($0.001–$0.01) |
| RFC 3161 TSA | **Legal-grade timestamp** (ISO 18014-2) | Free |
| Data Commons (IPFS) | Content-addressed persistence | Free |
| Federated Spine | Cross-trust-domain verification | Free |

**Philosophy clarified:** Bitcoin/Ethereum are public immutable ledgers
used strictly for anchor publish — gas cost only, explicitly no other
crypto interaction. This mirrors cellMembrane/projectNUCLEUS using
available infrastructure (VPS, GitHub) on the path to full sovereignty.

### Implementation path (when prioritized)

1. `chain-anchor` capability primal for Bitcoin/Ethereum submission
2. Add `AnchorTarget::Rfc3161Tsa { tsa_url }` variant + built-in TSA client behind `tsa` feature flag
3. Optional IPFS persistence via `iroh` or HTTP gateway

No implementation pressure per the audit.

---

## Verification

```
cargo check        — PASS
cargo clippy       — PASS (0 new warnings)
cargo test         — PASS (1,523 tests, 0 failures)
```

---

## Documentation updated

- `CHANGELOG.md` — WS-2/WS-3 entry
- `STATUS.md` — 40 → 42 methods
- `specs/API_SPECIFICATION.md` — all 42 methods documented
- `specs/DATA_MODEL.md` — `ExternalAnchor` → `PublicChainAnchor`
- `specs/PUBLIC_TIMESTAMPING.md` — new exploration spec
- `specs/00_SPECIFICATIONS_INDEX.md` — added PUBLIC_TIMESTAMPING.md
