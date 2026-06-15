# loamSpine FRAGO — Wave 113: Health + riboCipher + LazyLock Evolution

**Date**: 2026-06-14 (updated 2026-06-15)
**From**: loamSpine (strandGate)
**Re**: Wave 113 — Both P2 items COMPLETE, depot rebuild needed for VPS compliance

---

## Status: BOTH P2 ITEMS SHIPPED — VPS Binary Stale

The Wave 113 blurb (Jun 15) still shows loamSpine as `❌ timeout` for riboCipher and `⚠️ -32601` for health. This is a **depot staleness issue**, not a code gap. Both items were shipped Jun 14:

- `617838f` — `health` method + riboCipher prefix acceptance
- `26f41b2` — `OnceLock` → `LazyLock` idiomatic evolution

**Action needed**: cellMembrane depot rebuild for loamSpine from HEAD. Once deployed, VPS probes will see both compliant.

---

## Completed: Both P2 items

### 1. Bare `health` JSON-RPC method

```json
{"jsonrpc":"2.0","method":"health","params":{},"id":1}
→ {"status":"ok","primal":"loamspine","version":"0.9.16"}
```

- Classified as `Public` in MethodGate (no auth/BTSP required)
- cellMembrane probes will now get 200 OK instead of -32601
- Does NOT replace `health.check` / `health.liveness` / `health.readiness` (those remain for detailed probing)

### 2. riboCipher `[0xEC, 0x01]` prefix acceptance

- UDS connections beginning with `[0xEC, 0x01]` have the 2-byte signal stripped
- Normal protocol auto-detection proceeds on remaining bytes
- Plain JSON-RPC callers (without prefix) continue to work unchanged
- BTSP handshake after riboCipher prefix also works

### 3. OnceLock → LazyLock evolution

- All global caches (capability list, identity response, MCP tools, version, capabilities) evolved from `OnceLock` + `get_or_init()` to `LazyLock` (stabilized Rust 1.80, we're on 1.92)
- Zero `OnceLock` remaining in production code

### Implementation details

- `peek_first_protocol_byte()` helper in `jsonrpc/uds.rs` — extracted for function length compliance
- MethodGate updated: `"health"` is Public alongside `health.*` prefix
- 4 new tests (1 dispatch + 3 UDS protocol)

### Metrics

- **Tests**: 1,618 | **Files**: 199 | **Methods**: 48 | **Coverage**: 90.9%
- **Zero**: P1, clippy warnings, unsafe, TODOs, `OnceLock`, debt

### Verification

```bash
# Plain health probe:
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' \
  | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/biomeos/loamspine.sock

# With riboCipher prefix:
printf '\xEC\x01{"jsonrpc":"2.0","method":"health","params":{},"id":1}\n' \
  | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/biomeos/loamspine.sock
```

---

## Deep Debt Audit (Jun 14 — all clear)

| Dimension | Status |
|-----------|--------|
| Unsafe code | ZERO (`forbid(unsafe_code)` workspace-level) |
| Files > 800L | ZERO (largest: 648L `entry/mod.rs`) |
| TODOs/FIXMEs | ZERO in production |
| Hardcoding | ZERO (no addresses/ports/primal names in prod) |
| Mocks in production | ZERO (all `#[cfg(test)]` gated) |
| `#[allow(` in prod | ZERO (1 on test module only) |
| `Option<&String>` | ZERO |
| External C deps | ZERO (`blake3` uses `pure` feature) |
| `OnceLock` | ZERO (evolved to `LazyLock`) |
| Modern idioms | 45 `let...else`/`is_some_and`; 260 `.into()`; 70 `pub(crate)` |

---

## Remaining evolution (glacial, non-blocking)

- Public timestamping anchor (WS-3 spec exists, timeline open)
- Transport Phase 2: outbound `connect_transport()` (blocked on songBird `ipc.resolve`)

---

**Status**: Wave 113 P2 items RESOLVED. Zero remaining gaps. Depot rebuild will bring VPS compliance to 15/15 for loamSpine.
