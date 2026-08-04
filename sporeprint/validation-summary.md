+++
title = "loamSpine Validation Summary"
description = "Permanence ledger — 1,747 tests, 52 JSON-RPC methods, 211 source files, append-only Spines, Loam Certificates (Novel Ferment Transcripts), inclusion proofs, public chain anchoring, aggregate batch anchoring, batch entry append, batch certificate mint, cross-gate trust ledger IPC, TransportEndpoint compliance, BTSP ClientHello handshake, capability_registry.toml, cross-architecture #[cfg(unix)] parity, MCP batch tool exposure"
date = 2026-08-04

[taxonomies]
primals = ["loamspine"]
springs = []
+++

## Status

- **1,747 tests** (all passing), 0 failures, 0 ignored
- **52 JSON-RPC methods** across 19 domains (spine, entry, certificate, proof, anchor, session, braid, bonding, trust, btsp, auth, lifecycle, health, capabilities, identity, tools, primal, permanence)
- **211 source files**, ~65,000 lines of Rust
- **3 workspace members**: `loam-spine-core`, `loam-spine-api`, `loamspine-service`
- **JH-0 ADOPTED** — method gate classifies all 52 methods as Public or Protected
- **BTSP Phase 2+3** — ClientHello handshake (client + server), ChaCha20-Poly1305 AEAD, capability-discovered handshake key
- **ecoBin grade: A+** — zero C/C++ deps, `forbid(unsafe_code)`, edition 2024
- **Zero DEBT markers** — zero TODO/FIXME/HACK in production code
- **Zero `#[allow]`** — all suppressions use `#[expect(reason)]` or `#[cfg_attr]`-gated
- **Zero unsafe** — `#![forbid(unsafe_code)]` on all crates + fuzz targets
- **Storage**: redb (default), in-memory (testing); sled/SQLite removed (stadial)
- **Stability tiers**: 46 stable, 2 evolving (slice), 4 compat (permanence legacy naming)
- **Semantic mappings**: 52/52 (100% — every method routable by orchestrator)
- **Cost estimates**: 52/52 (100% — every method has scheduling hints)
- **MCP tools**: 36 tools exposed via `tools/list` (including batch operations)

## Key Capabilities

| Domain | Methods | Description |
|--------|---------|-------------|
| Spine | `create`, `get`, `list`, `seal` | Append-only spine lifecycle |
| Entry | `append`, `append_batch`, `get`, `get_tip`, `list` | Content-addressed entry management |
| Certificate | `mint`, `mint_batch`, `transfer`, `loan`, `return`, `get`, `verify`, `lifecycle`, `history` | Memory-bound objects (Novel Ferment Transcripts) |
| Proof | `generate_inclusion`, `verify_inclusion` | Merkle inclusion proofs |
| Anchor | `publish`, `publish_batch`, `verify` | Public chain anchoring + aggregate batch (Bitcoin, Ethereum, RFC 3161, Data Commons) |
| Session | `dehydrate`, `commit` | Provenance trio integration (content-addressed dehydration for rootPulse signing, then commit) |
| Braid | `commit` | Attribution braid integration (sweetGrass) |
| Bonding | `ledger.store`, `ledger.retrieve`, `ledger.list` | Ionic bond ledger |
| Trust | `anchor`, `query`, `event_count` | Cross-gate trust event anchoring (tower IPC) |
| BTSP | `negotiate`, `capabilities` | Secure transport negotiation |
| Auth | `check`, `mode`, `peer_info` | JH-0 method gate introspection |
| Lifecycle | `status`, `primal.announce` | Service lifecycle + self-registration |
| Health | `check`, `liveness`, `readiness` | Health probes |
| Capabilities | `list` | Capability discovery (Wire Standard L3) |
| Identity | `get` | Primal identity |
| Tools | `list`, `call` | MCP tool discovery and invocation |
| Compat | `permanence.*` (4) | Legacy naming compatibility |

## Provenance Trio Role

loamSpine is the **permanence layer** of the provenance trio:

```
rhizoCrypt (working DAG) → loamSpine (permanent ledger) → sweetGrass (attribution braid)
```

- `session.dehydrate` computes content-addressed summary of uncommitted entries (read-only)
- `session.commit` receives dehydrated DAG sessions from rhizoCrypt
- `braid.commit` records attribution braids from sweetGrass
- `anchor.publish` stamps spine state to public immutable ledgers (Bitcoin OP_RETURN, Ethereum, RFC 3161 TSA)
- Loam Certificates are Novel Ferment Transcripts — value from accumulated history

## G31 Batch Provenance Pipeline

- `entry.append_batch` — append N entries in one RPC call, amortized I/O (1 read + 1 write)
- `certificate.mint_batch` — mint N certificates in one RPC call, same amortization
- Target: ~30 ms/object → ~3 ms/object amortized for bulk ingestion
- Addresses 12× throughput gap identified in westGate provenance × acquisition divergence

## Recent Evolution (v0.9.16)

| Wave | What landed |
|------|-------------|
| Wave 155u (Aug 4) | Semantic mappings 33→52, cost estimates 32→52, MCP batch tools, doc alignment |
| Wave 155n (Aug 3) | G31 batch provenance: `entry.append_batch`, `certificate.mint_batch`, CLI `--bind` |
| Wave 155f (Jul 28) | Entry extraction, `certificate.history`, BTSP handshake dedup |
| Wave 155b+ (Jul 27) | G3 verification path, semantic certificate checks, delegated minting |
| Wave 151c (Jul 26) | TransportEndpoint compliance, error visibility, endpoint parsing |
| Wave 151b (Jul 26) | BTSP ClientHello handshake (4-step, HMAC-SHA256) |
| Wave 150t (Jul 21) | Health probe honesty (5s timeout), entry path coverage |
| Wave 149b (Jul 18) | Dimensional self-audit, test file splits, fuzz safety |
| Wave 143b (Jul 16) | Transport endpoint functional wiring, framing edge cases |
| Wave 142b (Jul 16) | Silicon atheism phase 2, async fs hygiene, clone reduction |
| Wave 141a (Jul 15) | Cross-architecture `#[cfg(unix)]` parity, integration test splits |

## Consumed Capabilities

| Capability | Provider | Role |
|------------|----------|------|
| `signing` | Tower signer (capability-discovered) | Ed25519 entry signing |
| `discovery` | (capability-discovered) | mDNS / DNS-SRV primal discovery |
| `chain-anchor` | (not yet built) | External chain submission for anchor.publish |

## Specifications

| Spec | Status |
|------|--------|
| [LOAMSPINE_SPECIFICATION.md](../specs/LOAMSPINE_SPECIFICATION.md) | Complete |
| [API_SPECIFICATION.md](../specs/API_SPECIFICATION.md) | Complete (52 methods) |
| [DATA_MODEL.md](../specs/DATA_MODEL.md) | Complete |
| [CERTIFICATE_LAYER.md](../specs/CERTIFICATE_LAYER.md) | Complete |
| [ANCHORING_ARCHITECTURE.md](../specs/ANCHORING_ARCHITECTURE.md) | Complete |
| [PUBLIC_TIMESTAMPING.md](../specs/PUBLIC_TIMESTAMPING.md) | Exploration |
| [ARCHITECTURE.md](../specs/ARCHITECTURE.md) | Complete |

## See Also

- [STATUS.md](../STATUS.md) for detailed implementation progress
- [CHANGELOG.md](../CHANGELOG.md) for version history
- [specs/](../specs/) for the full specification suite
