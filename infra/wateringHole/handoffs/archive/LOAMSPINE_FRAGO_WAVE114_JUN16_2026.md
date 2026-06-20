# loamSpine FRAGO — Wave 114: Genetics-Layer Mito-Beacon Adoption

**Date**: 2026-06-16
**From**: loamSpine (strandGate)
**Re**: Wave 114 — Genetics-layer wiring COMPLETE (was 9/11, now 10/11)

---

## Shipped: Full eukaryotic genetics signal acceptance

loamSpine now accepts **all three** genetics-layer signal bytes per the eukaryotic model:

| Signal | Byte | Status |
|--------|------|--------|
| riboCipher clear | `0xEC` | ✅ (shipped Wave 113, `617838f`) |
| MitoBeacon obfuscated | `0xED` | ✅ **NEW** |
| Nuclear sealed | `0xEE` | ✅ **NEW** |

### Wire behavior

Any connection beginning with a byte in `0xEC..=0xEE` followed by a version byte has the 2-byte prefix stripped, then normal protocol auto-detection proceeds (JSON-RPC, BTSP NDJSON, or binary BTSP). Plain JSON-RPC callers without prefix continue to work unchanged.

### Implementation

- `GENETICS_SIGNAL_RANGE` constant (`0xEC..=0xEE`) in `jsonrpc/uds.rs`
- `peek_first_protocol_byte()` matches range instead of single `0xEC` byte
- Structured tracing: logs signal name (`riboCipher-clear` / `mito-beacon` / `nuclear-sealed`) + version byte
- 3 new tests: mito-beacon strip, nuclear-sealed strip, non-genetics boundary (0xEB not stripped)

### Previous Wave 113 items (still shipped)

- Bare `health` JSON-RPC method → `{status: "ok", primal: "loamspine", version}`
- `OnceLock` → `LazyLock` evolution (zero `OnceLock` remaining)

### Metrics

- **Tests**: 1,621 | **Files**: 199 | **Methods**: 48 | **Coverage**: 90.9%
- **Zero**: P1, clippy warnings, unsafe, TODOs, debt

### Verification

```bash
# Plain health probe:
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' \
  | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/biomeos/loamspine.sock

# With riboCipher prefix:
printf '\xEC\x01{"jsonrpc":"2.0","method":"health","params":{},"id":1}\n' \
  | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/biomeos/loamspine.sock

# With mito-beacon prefix:
printf '\xED\x01{"jsonrpc":"2.0","method":"health","params":{},"id":1}\n' \
  | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/biomeos/loamspine.sock
```

---

**Status**: Wave 114 genetics-layer adoption COMPLETE. loamSpine is 10th of 11 primals to ship. Only squirrel remains.
