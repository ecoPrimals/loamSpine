+++
title = "loamSpine Validation Summary"
description = "Permanence ledger — 1,527 tests, 43 JSON-RPC methods, 189 source files, append-only Spines, Loam Certificates (Novel Ferment Transcripts), inclusion proofs, public chain anchoring, aggregate batch anchoring"
date = 2026-05-24

[taxonomies]
primals = ["loamspine"]
springs = []
+++

## Status

- **1,527 tests** (all passing), 0 failures, 0 ignored
- **43 JSON-RPC methods** across 12 domains (spine, entry, certificate, proof, anchor, session, braid, bonding, btsp, auth, lifecycle, permanence)
- **189 source files**, ~59,300 lines of Rust
- **3 workspace members**: `loam-spine-core`, `loam-spine-api`, `loamspine-service`
- **JH-0 ADOPTED** — method gate classifies all 43 methods as Public or Protected
- **BTSP Phase 3** — ChaCha20-Poly1305 AEAD, capability-discovered handshake key
- **ecoBin grade: A+** — zero C/C++ deps, `forbid(unsafe_code)`, edition 2024
- **Zero DEBT markers**, zero `#[allow]` without `reason`
- **Storage**: redb (default), in-memory (testing); sled/SQLite removed (stadial)
- **Stability tiers**: 37 stable, 2 evolving (slice), 4 compat (permanence legacy naming)

## Key Capabilities

| Domain | Methods | Description |
|--------|---------|-------------|
| Spine | `create`, `get`, `list`, `seal` | Append-only spine lifecycle |
| Entry | `append`, `get`, `get_tip`, `list` | Content-addressed entry management |
| Certificate | `mint`, `transfer`, `loan`, `return`, `get` | Memory-bound objects (Novel Ferment Transcripts) |
| Proof | `generate_inclusion`, `verify_inclusion` | Merkle inclusion proofs |
| Anchor | `publish`, `publish_batch`, `verify` | Public chain anchoring + aggregate batch (Bitcoin, Ethereum, RFC 3161, Data Commons) |
| Session | `commit` | Provenance trio integration (rhizoCrypt dehydration) |
| Braid | `commit` | Attribution braid integration (sweetGrass) |
| Bonding | `ledger.store`, `ledger.retrieve`, `ledger.list` | Ionic bond ledger |
| BTSP | `negotiate`, `capabilities` | Secure transport negotiation |
| Auth | `check`, `mode`, `peer_info` | JH-0 method gate introspection |
| Lifecycle | `status` | Service lifecycle |
| Health | `check`, `liveness`, `readiness` | Health probes |

## Provenance Trio Role

loamSpine is the **permanence layer** of the provenance trio:

```
rhizoCrypt (working DAG) → loamSpine (permanent ledger) → sweetGrass (attribution braid)
```

- `session.commit` receives dehydrated DAG sessions from rhizoCrypt
- `braid.commit` records attribution braids from sweetGrass
- `anchor.publish` stamps spine state to public immutable ledgers (Bitcoin OP_RETURN, Ethereum, RFC 3161 TSA)
- Loam Certificates are Novel Ferment Transcripts — value from accumulated history

## Recent Evolution (v0.9.16)

| Wave | What landed |
|------|-------------|
| Deep Debt Cleanup | Safe casts (`try_from`), dead code wiring (cipher tracing), test cohesion split (828→4 modules), 189 source files |
| Wave 47 | Deployment behavioral convergence — `serve`→`server` fix, `LOAMSPINE_DISCOVERY_ENABLED` env gate, `lifecycle.status` `uptime_s` |
| Wave 43 | Neural API `primal.announce` adoption — startup announce with capabilities, signal_tiers, cost_hints, latency_estimates |
| Anchoring Architecture | `anchor.publish_batch`, aggregation Merkle tree, ANCHORING_ARCHITECTURE.md, upstream propagation |
| Wave 22 (Stadial Gate) | `btsp.capabilities`, `primal.announce`, stability tiers, 40 methods |
| Stale Socket | TOCTOU-safe `unlink` before `bind`, PID file |
| River Delta (WS-2/WS-3) | `spine.list`, `entry.list`, `AnchorTarget::Rfc3161Tsa`, PUBLIC_TIMESTAMPING.md spec |
| Deep Debt | Typed `HexError`, `#[expect(reason)]` migration, test file splits |
| GAP-36 | Session alias wire reconciliation, `lifecycle.status` |

## Consumed Capabilities

| Capability | Provider | Role |
|------------|----------|------|
| `signing` | BearDog | Ed25519 entry signing |
| `discovery` | (capability-discovered) | mDNS / DNS-SRV primal discovery |
| `chain-anchor` | (not yet built) | External chain submission for anchor.publish |

## Specifications

| Spec | Status |
|------|--------|
| [LOAMSPINE_SPECIFICATION.md](../specs/LOAMSPINE_SPECIFICATION.md) | Complete |
| [API_SPECIFICATION.md](../specs/API_SPECIFICATION.md) | Complete (43 methods) |
| [DATA_MODEL.md](../specs/DATA_MODEL.md) | Complete |
| [CERTIFICATE_LAYER.md](../specs/CERTIFICATE_LAYER.md) | Complete |
| [ANCHORING_ARCHITECTURE.md](../specs/ANCHORING_ARCHITECTURE.md) | Complete |
| [PUBLIC_TIMESTAMPING.md](../specs/PUBLIC_TIMESTAMPING.md) | Exploration |
| [ARCHITECTURE.md](../specs/ARCHITECTURE.md) | Complete |

## See Also

- [STATUS.md](../STATUS.md) for detailed implementation progress
- [CHANGELOG.md](../CHANGELOG.md) for version history
- [specs/](../specs/) for the full specification suite
