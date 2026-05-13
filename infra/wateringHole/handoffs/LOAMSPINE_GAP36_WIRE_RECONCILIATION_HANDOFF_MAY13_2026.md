<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# LoamSpine — GAP-36: Provenance Trio Wire Reconciliation

**Date**: May 13, 2026
**From**: loamSpine team
**To**: primalSpring, healthSpring, downstream consumers
**Re**: GAP-36 resolution — session aliases, lifecycle.status, METHODS drift

---

## What was reported

> healthSpring wired a complete Nest atomic validation scenario exercising
> loamSpine on the `ledger` capability. UDS handlers accept connections but
> "return no JSON-RPC payloads." Methods exercised: `session.create`,
> `session.state`.

> Registry defines `entry.append`/`entry.get` but the sweep exercises
> `session.create`/`session.state` on the `ledger` capability — reconcile
> which methods are the production surface.

---

## Root cause

**NOT an empty socket bug.** loamSpine's UDS server always returns well-formed
JSON-RPC responses for any valid request with an `id` field. The issue was a
**wire name mismatch**: healthSpring called `session.create` and
`session.state`, which are not native loamSpine method names. These hit the
`_` match arm and returned `-32601 method not found`.

loamSpine's native equivalents:
- `session.create` → `spine.create` (creates a ledger/spine)
- `session.state` → `spine.get` (queries spine state)

---

## What we shipped

### 1. Session aliases for ledger capability

`normalize_method` now routes downstream Nest sweep method names to native handlers:

| Alias | Target |
|-------|--------|
| `session.create` | `spine.create` |
| `session.state` | `spine.get` |
| `ledger.create` | `spine.create` |
| `ledger.get` | `spine.get` |
| `session.get` | `spine.get` |
| `ledger.state` | `spine.get` |

### 2. `lifecycle.status` handler

New public method (classified Public in JH-0 gate):

```json
{
  "primal": "loamspine",
  "version": "0.9.16",
  "status": "running",
  "auth_mode": "permissive"
}
```

### 3. METHODS drift fixed

**Removed** (advertised but no dispatch handler):
- `certificate.verify`
- `certificate.lifecycle`
- `slice.record_operation`
- `slice.depart`

These are planned v1 features. They were in `METHODS`, `provided_capabilities`,
`cost_estimates`, `operation_dependencies`, `SEMANTIC_MAPPINGS`, and MCP tools —
all cleaned.

**Added** (dispatched but not registered):
- `btsp.negotiate`
- `lifecycle.status`

### 4. Capability registration aligned

- `CAPABILITIES` array: `spine.query` → `spine.get`, `certificate.issue` → `certificate.mint`, `capability.list` → `capabilities.list`
- `cost_estimates`: Fixed `capability.list` key → `capabilities.list`, added `btsp.negotiate` and `lifecycle.status`
- `operation_dependencies`: Removed references to unimplemented `slice.record_operation` and `slice.depart`

---

## Method surface reconciliation

| Domain | Methods | Role |
|--------|---------|------|
| `spine.*` | `create`, `get`, `seal` | Ledger lifecycle (production surface) |
| `entry.*` | `append`, `get`, `get_tip` | Entry CRUD (production surface) |
| `session.*` | `commit` | Provenance trio session commit |
| `certificate.*` | `mint`, `transfer`, `loan`, `return`, `get` | Certificate lifecycle |
| `slice.*` | `anchor`, `checkout` | Waypoint slice operations |
| `proof.*` | `generate_inclusion`, `verify_inclusion` | Merkle proofs |
| `anchor.*` | `publish`, `verify` | Public chain anchoring |
| `bonding.ledger.*` | `store`, `retrieve`, `list` | Ionic bond ledger |
| `braid.*` | `commit` | Provenance trio braid commit |
| `btsp.*` | `negotiate` | BTSP Phase 3 cipher negotiation |
| `permanence.*` | `commit_session`, `verify_commit`, `get_commit`, `health_check` | Compat layer |
| `health.*` | `check`, `liveness`, `readiness` | Health probes (public) |
| `lifecycle.*` | `status` | Service status (public) |
| `auth.*` | `check`, `mode`, `peer_info` | JH-0 introspection (public) |
| Meta | `capabilities.list`, `identity.get`, `tools.list`, `tools.call` | Discovery (public) |

**38 methods total, all backed by live dispatch handlers.**

For downstream Nest sweeps: `session.create`/`session.state` will work via aliases.
For direct API consumers: `spine.create`/`spine.get` are the canonical names.

---

## Verification

- 1,522 tests passing
- `cargo clippy --all-targets --all-features` clean
- `cargo deny check` passing
- All `METHODS` entries have dispatch handlers
- All dispatch handlers are in `METHODS`
- MCP tools aligned with dispatch
- `semantic_mappings_target_valid_methods` test passes
- `cost_estimates_cover_key_methods` test passes
- `mcp_tools_cover_all_methods_in_capability_list` test passes

---

## Niche items (deferred — v1 targets)

- PostgreSQL/RocksDB storage backends
- `certificate.verify`, `certificate.lifecycle` (needs audit trail spec)
- `slice.record_operation`, `slice.depart` (needs attestation provider)
