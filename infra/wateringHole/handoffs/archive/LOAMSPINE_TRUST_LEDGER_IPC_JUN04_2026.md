# loamSpine — Trust Ledger IPC Wiring (June 4, 2026)

**From**: loamSpine (strandGate)  
**To**: primalSpring (eastGate), bearDog (southGate)  
**Wave**: 76 → forward evolution  
**Status**: COMPLETE

---

## Mission Response

primalSpring directive: "Wire ledger entries to bearDog — design the IPC path: when bearDog registers a trusted issuer, loamSpine anchors it as a permanent ledger entry."

## What Shipped

### Trust Ledger IPC — 3 new JSON-RPC methods

| Method | Wire Shape | Purpose |
|--------|-----------|---------|
| `trust.anchor` | `{ entry_type: EntryType }` → `{ entry_hash, index }` | Anchor any trust-domain entry as permanent ledger record |
| `trust.query` | `{ gate_did: Did }` → `{ events: [EntryType] }` | Query trust events by gate DID (all roles) |
| `trust.event_count` | `{}` → `{ count: u64 }` | Return number of anchored trust events |

### Architecture

- **Dedicated trust spine**: `did:primal:loamspine:trust-ledger` — lazily created on first `trust.anchor` call
- **Pattern**: Mirrors `bonding.ledger.*` — dedicated spine, typed EntryType append, in-memory query
- **Domain validation**: `trust.anchor` rejects non-trust EntryType variants
- **Method gate**: All 3 trust methods classified as `Protected` (require auth)

### IPC Path (bearDog → loamSpine)

```
bearDog registers trusted issuer
  → bearDog calls loamSpine via JSON-RPC:
    { method: "trust.anchor",
      params: { entry_type: { "TrustIssuerRegistration": {
        issuer_did: "did:key:z6Mk...",
        registering_gate: "did:key:z6MkSouthGate",
        trust_scope: "cross-gate",
        capabilities: ["signing", "verification"],
        expires_at: null
      }}}}
  → loamSpine ensures trust spine exists
  → loamSpine appends entry to trust spine
  → returns { entry_hash: [32 bytes], index: N }
```

Same path for `KeyExchange` and `TokenVerificationCrossGate` events.

### Deep Debt

- Last 2 `#[allow(` evolved to `#[expect(`. **Zero `#[allow(` remaining** in entire codebase.

### Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests | 1,583 | 1,600 |
| Source files | 194 | 198 |
| Methods | 44 | 47 |
| Domains | 15 | 16 |
| `#[allow(` | 2 | 0 |

### Files Changed

- `crates/loam-spine-core/src/service/trust_ledger.rs` — NEW: core trust ledger service
- `crates/loam-spine-core/src/service/mod.rs` — trust_ledger_spine field
- `crates/loam-spine-core/src/niche.rs` — trust domain, methods, mappings, costs
- `crates/loam-spine-core/src/neural_api/mod.rs` — trust methods in provided_capabilities
- `crates/loam-spine-core/src/neural_api/mcp.rs` — MCP trust tools
- `crates/loam-spine-api/src/types/trust.rs` — NEW: wire contract types
- `crates/loam-spine-api/src/service/trust_ops.rs` — NEW: API bridge
- `crates/loam-spine-api/src/jsonrpc/mod.rs` — trust dispatch wiring
- `crates/loam-spine-api/src/jsonrpc/tests_trust.rs` — NEW: 9 integration tests

### Validation

```
cargo check   — 0 errors
cargo clippy  — 0 warnings
cargo test    — 1,600 passed, 0 failed
```

---

*Next: bearDog wires `trust.anchor` calls into its `TrustedIssuerRegistry` registration flow.*
