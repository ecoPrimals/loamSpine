<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# LoamSpine — Wave 22: Stadial Gate Readiness

**Date**: May 17, 2026
**From**: loamSpine team
**To**: primalSpring, downstream springs, lithoSpore, projectFOUNDATION
**Re**: Stadial universal checklist cleared — 23/23 items PASS

---

## Universal Standards Checklist: 23/23 PASS

### Runtime (5/5)
- [x] Health triad: `health.liveness`, `health.readiness`, `health.check`
- [x] UDS at `$XDG_RUNTIME_DIR/biomeos/loamspine.sock`
- [x] TCP via `LOAMSPINE_JSONRPC_PORT` / `--port`
- [x] `server` subcommand with `--port`
- [x] Standalone startup without `FAMILY_ID`/`NODE_ID`

### Discovery (4/4)
- [x] `capabilities.list` returns `{ capabilities, count, primal, methods, ... }`
- [x] `identity.get` canonical response (with `ecobin_grade`, `edition`, `capability_domain`)
- [x] `primal.announce` self-registration handler
- [x] All 40 methods follow `{domain}.{operation}` naming

### Security (7/7)
- [x] BTSP handshake when `BIOMEOS_FAMILY_ID` non-default
- [x] ChaCha20-Poly1305 + HKDF-SHA256 (`btsp-session-v1-c2s`/`s2c`)
- [x] `BIOMEOS_INSECURE=1` + family = refuse to start
- [x] `btsp.capabilities` registered
- [x] Zero metadata leakage
- [x] UDS-first (TCP opt-in only)
- [x] `deny.toml` bans `ring`, `openssl`, `aws-lc-sys`

### Build (4/4)
- [x] `edition = "2024"`
- [x] `notify-plasmidbin.yml` fires on push
- [x] musl-static clean
- [x] `cargo deny check` passes

### Documentation (3/3)
- [x] README version matches manifest (0.9.16)
- [x] CHANGELOG documents recent evolution
- [x] STATUS.md with stadial readiness section

---

## What we shipped

### New methods (40 total, up from 38)

| Method | Type | Description |
|--------|------|-------------|
| `btsp.capabilities` | Public | BTSP cipher/feature discovery |
| `primal.announce` | Public | Self-registration payload |

### Capability response enrichment

- `"count": N` field added to `capabilities.list`
- `"stability"` tier on each `provided_capabilities` entry
- `"ecobin_grade": "A+"` and `"capability_domain": "ledger"` in `identity.get`

### Security hardening

When `btsp_config` is active and a client sends plain JSON-RPC (no BTSP
handshake), the UDS handler now logs a security warning:
```
WARN: Plain JSON-RPC connection while BTSP is configured — client should
send BTSP handshake for protected operations
```
The connection is allowed (health probes need to work) but the warning
creates an audit trail. Protected methods are gated by the JH-0 MethodGate.

### Stability tiers

| Tier | Methods | Count |
|------|---------|-------|
| stable | spine.*, entry.*, certificate.*, proof.*, anchor.*, session.*, braid.*, bonding.*, btsp.*, lifecycle.*, health.*, auth.*, primal.* | 36 |
| evolving | slice.anchor, slice.checkout | 2 |
| compat | permanence.* | 4 |

---

## Per-primal audit items (resolved)

| Item | Resolution |
|------|-----------|
| ecobin_grade A+ → A++ gap | A+ confirmed. Gap is `seed_fingerprint` (build-time BLAKE3 of binary). Documented in STATUS.md. |
| DEPENDENCY_EVOLUTION.md | All items COMPLETE. tarpc/opentelemetry weight is LOW, monitored. |
| GAP-36 wire reconciliation | PASS — session aliases in `normalize_method`, handoff documented. |
| Hex string acceptance | PASS — `serde_content_hash`/`serde_opt_content_hash` modules with `deserialize_with` on all hash fields. |

---

## Composition gap #6: Hex string acceptance — CLOSED

loamSpine accepts both JSON byte arrays (`[1,2,...,32]`) and 64-char hex
strings (`"0102..."` with optional `0x` prefix) for all `ContentHash` and
`EntryHash` fields. Custom serde modules in `loam-spine-core/src/types.rs`.
Shared with rhizoCrypt — loamSpine's side is fully resolved.

---

## Stadial pairing

| Partner | Interface | Status |
|---------|-----------|--------|
| sweetGrass | `session.commit`, entry certificates | Ready |
| rhizoCrypt | `permanence.commit_session` dehydration | Ready |
| lithoSpore | `spine.get` ledger verification | Ready (awaiting lithoSpore) |
| projectFOUNDATION | Thread evidence permanence | Ready (awaiting projectFOUNDATION) |

---

## Metrics

- **40 methods**, all backed by live dispatch
- **1,522 tests**, all passing
- **90.9% line coverage**
- **Zero unsafe code** (`#![forbid(unsafe_code)]`)
- **Edition 2024**
- **ecoBin grade A+**
