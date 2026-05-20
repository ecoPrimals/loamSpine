# loamSpine — Public Chain Anchoring Architecture

**Date:** 2026-05-20
**Audit:** Wave 31 follow-up + WS-3 evolution
**License:** AGPL-3.0-or-later

---

## Summary

Comprehensive public chain anchoring architecture implemented: aggregate batch
anchoring via Merkle tree, compression pipeline spec, upstream documentation
propagation, and crypto-as-infrastructure philosophical alignment.

| Area | Deliverable | Status |
|------|-------------|--------|
| Spec | `specs/ANCHORING_ARCHITECTURE.md` (365 lines) | DONE |
| Core | `AggregateInclusionProof` + `generate_aggregate_proof` | DONE |
| Core | `PublicChainAnchor` extended (aggregate_root, inclusion_proof) | DONE |
| RPC | `anchor.publish_batch` (43rd method) | DONE |
| RPC | `anchor.verify` aggregate proof checking | DONE |
| Upstream | `whitePaper/gen4/ANCHORING_PIPELINE.md` updated | DONE |
| Upstream | `whitePaper/gen4/economics/NOVEL_FERMENT_TRANSCRIPTS.md` updated | DONE |
| Ecosystem | `wateringHole/ANCHORING_STANDARD.md` created | DONE |
| Self-knowledge | niche, neural_api, MCP, method_gate all updated | DONE |
| Docs | CHANGELOG, STATUS, API_SPECIFICATION, DATA_MODEL, sporeprint | DONE |

---

## Key Design Decisions

### Compression Pipeline

```
rhizoCrypt DAG (GBs) → dehydration summary → loamSpine entry (KBs)
  → spine state hash (32 bytes) → aggregate Merkle root (32 bytes)
    → single public chain transaction
```

N spine state hashes are batched into one Merkle root. Each spine gets an
`AggregateInclusionProof` (path + leaf_index) for independent verification
without the full batch.

### Crypto as Infrastructure

- Gas is postage, not currency speculation
- One transaction anchors unlimited spines (via Merkle aggregation)
- Community pooling: shared gas costs across springs
- L2 transfers available but not required
- No token, no wallet, no exchange dependency

### Novel Ferment Transcripts (not NFTs)

Digital objects whose value comes from accumulated fermentation history,
not artificial scarcity or blockchain tokens.

---

## Methods (43 total)

| Tier | Count |
|------|-------|
| stable | 39 |
| evolving | 2 |
| compat | 4 |

New: `anchor.publish_batch` (stable, Protected via JH-0).

---

## Tests

1,523 passing. Zero warnings. Zero compilation errors.

---

## Files Changed (loamSpine)

- `crates/loam-spine-core/src/proof.rs` — `compute_merkle_root` made pub, `AggregateInclusionProof`, `generate_aggregate_proof`
- `crates/loam-spine-core/src/entry/mod.rs` — `PublicChainAnchor` extended
- `crates/loam-spine-core/src/service/anchor.rs` — `anchor_batch`, verify aggregate
- `crates/loam-spine-core/src/service/mod.rs` — re-exports
- `crates/loam-spine-core/src/niche.rs` — 43rd method
- `crates/loam-spine-core/src/neural_api/mod.rs` — capabilities
- `crates/loam-spine-core/src/neural_api/mcp.rs` — MCP tool
- `crates/loam-spine-api/src/types/anchor.rs` — batch request/response types
- `crates/loam-spine-api/src/service/anchor_ops.rs` — batch handler
- `crates/loam-spine-api/src/jsonrpc/mod.rs` — dispatch wiring
- `crates/loam-spine-api/src/jsonrpc/method_gate.rs` — Protected classification
- `crates/loam-spine-core/src/entry/entry_tests.rs` — backward compat fixes
- `specs/ANCHORING_ARCHITECTURE.md` — new (365 lines)

## Files Changed (whitePaper)

- `gen4/architecture/ANCHORING_PIPELINE.md` — economics, crypto stance, status
- `gen4/economics/NOVEL_FERMENT_TRANSCRIPTS.md` — implementation status table

## Files Changed (wateringHole)

- `ANCHORING_STANDARD.md` — new ecosystem guidance
- `README.md` — index entry

---

## Upstream Action Items

| Consumer | What to Review |
|----------|---------------|
| primalSpring | New `anchor.publish_batch` method available for composition validation |
| rhizoCrypt | Aggregate anchoring enables batch dehydration → single chain tx |
| sweetGrass | Attribution braids can reference aggregate anchor receipts |
| All springs | `wateringHole/ANCHORING_STANDARD.md` for integration guidance |

---

## Remaining Horizon (Low Priority)

- Actual chain submission adapter (capability-discovered `"chain-anchor"` primal)
- RFC 3161 TSA live integration
- Community pooling coordination protocol

These are deferred to downstream demand — the architecture and RPC surface are ready.
