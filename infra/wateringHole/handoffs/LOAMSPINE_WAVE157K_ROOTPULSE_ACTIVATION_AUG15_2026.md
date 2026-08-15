# Handoff: loamSpine Wave 157k — rootPulse Step Handler Activation

**Date**: August 15, 2026  
**From**: westGate  
**To**: overwatch / eastGate / upstream  
**Wave**: 157k  
**Status**: rootPulse item #10 — loamSpine **DONE** (3/5 primals active)

---

## Summary

loamSpine now implements the permanence step in the `rootpulse_commit` provenance trio graph. Two new JSON-RPC methods provide graph-step-compatible inputs/outputs for biomeOS neuralAPI graph composition.

The implementation follows the `NESTGATE_ROOTPULSE_OVERSTEP_AAR` principle: loamSpine only implements `rootpulse.*` methods because permanence IS its core domain. The methods wrap existing `session.commit` logic into graph-compatible inputs — no foreign domain logic.

---

## New Methods

| Method | Description | Stability |
|--------|-------------|-----------|
| `rootpulse.ledger_commit` | Record provenance commit to dedicated spine. Accepts `{cas_ref, signed_provenance}` from upstream graph steps. Returns `ledger_ref` for downstream. | beta |
| `rootpulse.query_commit` | Query recent provenance commits (by wave, target, primal). Paginated. | beta |

**Wire alias**: `ledger.append` → `rootpulse.ledger_commit` (biomeOS graph step convention)

---

## Architecture Decision

```
rootpulse_commit graph flow:
  rhizoCrypt.dehydrate_state → bearDog.sign → nestGate.store → loamSpine.ledger_commit → sweetGrass.attribute
                                                                    ↑ we are here
```

loamSpine's step:
1. Parses `cas_ref` (hex → `[u8; 32]` content hash)
2. Auto-creates or reuses `rootpulse-provenance` spine (keyed to this gate)
3. Commits via existing `session.commit` core logic
4. Emits `GossipEvent::RootpulseCommit` to swarmVine mesh
5. Returns composite `ledger_ref` (`{spine_id}:{entry_hash_hex}:{index}`)

Graceful degradation: if `signed_provenance` contains a signature, it's recorded in metadata. If not, commit proceeds unsigned (same as existing session.commit behavior with missing signer).

---

## Gossip

5th `GossipEvent` variant added:

```
RootpulseCommit { spine_id, entry_hash, ledger_ref }
  → topic: "rootpulse.commit"
  → key: "rootpulse.commit:{gate_id}:{spine_id}"
```

Emitted on each successful `rootpulse.ledger_commit`. swarmVine mesh receives provenance head notifications for data availability and downstream consumers.

---

## Current Metrics

| Metric | Value |
|--------|-------|
| Tests | 1,865 |
| Source files | 223 `.rs` files (+ 3 fuzz targets) |
| JSON-RPC methods | 55 |
| tarpc methods | 37 |
| Domains | 20 (including new `rootpulse`) |
| Gossip events | 5 |
| MCP tools | +2 (`rootpulse_ledger_commit`, `rootpulse_query_commit`) |
| Unsafe code | Zero (`#![forbid(unsafe_code)]`) |
| TODOs/FIXMEs/HACKs | Zero |
| Clippy pedantic | Clean |

---

## Files Changed

| File | Change |
|------|--------|
| `crates/loam-spine-api/src/types/mod.rs` | Request/response types for rootPulse ops |
| `crates/loam-spine-api/src/service/rootpulse_ops.rs` | **NEW** — service handlers |
| `crates/loam-spine-api/src/service/mod.rs` | Module registration |
| `crates/loam-spine-api/src/jsonrpc/mod.rs` | Dispatch + `ledger.append` alias |
| `crates/loam-spine-api/src/jsonrpc/tests_rootpulse.rs` | **NEW** — 45 wire-level tests |
| `crates/loam-spine-core/src/niche.rs` | 2 methods, 1 domain, mappings, costs |
| `crates/loam-spine-core/src/gossip.rs` | `RootpulseCommit` variant + key format |
| `crates/loam-spine-core/src/neural_api/mcp.rs` | 2 MCP tool definitions + schemas |

---

## Upstream Signals

### For eastGate (biomeOS)

The `rootpulse_commit` graph definition can now wire `loamSpine.ledger_commit` as the permanence step. loamSpine accepts the standard graph-step contract:

```json
{
  "cas_ref": "<64-char hex hash from nestGate CAS>",
  "signed_provenance": { "signature": "...", "signer_did": "..." },
  "wave_id": "157k",
  "target_triple": "x86_64-unknown-linux-musl",
  "primal_name": "loamSpine"
}
```

### For ironGate (bearDog)

bearDog's rootPulse participation is translation-only: biomeOS calls existing `crypto.sign_ed25519` capability. No new handlers needed in bearDog.

### For ironGate/westGate (nestGate)

nestGate's rootPulse participation is translation-only: biomeOS calls existing `content.put` capability. No new handlers needed — the `content.put` translation entry in biomeOS is the remaining action (see item #13).

### Remaining (item #10)

| Primal | Status | Work Needed |
|--------|--------|-------------|
| rhizoCrypt | DONE (`fa35ed3`) | — |
| sweetGrass | DONE (`f31e1bc`) | — |
| loamSpine | **DONE** (this wave) | — |
| bearDog | Translation-only | biomeOS calls existing `crypto.sign_ed25519` |
| nestGate | Translation-only | biomeOS calls existing `content.put` |

---

## Blockers

None. loamSpine's rootPulse step is fully functional and tested. Upstream biomeOS graph wiring can proceed independently.

---

*Wave 157k. rootPulse permanence step ACTIVE. 3/5 primals done. Fermenter cultivating.*
