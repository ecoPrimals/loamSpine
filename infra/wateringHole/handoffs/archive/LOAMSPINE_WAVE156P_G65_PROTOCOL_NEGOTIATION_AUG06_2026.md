# Handoff: loamSpine Wave 156p — G65 Protocol Negotiation

**Date**: August 6, 2026  
**From**: sporeGate  
**To**: overwatch / eastGate / upstream  
**Wave**: 156p  
**HEAD**: (pending commit)

---

## Summary

G65 protocol negotiation shipped for loamSpine. Single-socket protocol
selection replaces C2 dual-socket as the primary connection model. Client
sends `PROTOCOLS: tarpc,jsonrpc\n`, server selects best mutual match and
responds `PROTOCOL: tarpc\n`. No negotiation = JSON-RPC (full backward
compatibility with existing clients).

Implemented following rhizoCrypt reference (convergent evolution, no shared
code). C2 dual-socket retained for backward compatibility until cellMembrane
drops `has_tarpc` tracking.

---

## Changes

### New Files

| File | Lines | Purpose |
|------|------:|---------|
| `crates/loam-spine-api/src/protocol_negotiation.rs` | ~350 | G65 module: `IpcProtocol` enum, `try_negotiate()`, `negotiate_client()`, `select_protocol()`, wire parsing, `NegotiationResult` |

### Modified Files

| File | Change |
|------|--------|
| `crates/loam-spine-api/src/tarpc_server.rs` | Added `serve_tarpc_connection()` — tarpc on already-negotiated stream via length-delimited + JSON serde framing |
| `crates/loam-spine-api/src/jsonrpc/uds.rs` | Restructured UDS handler: G65-first detection before stream split. `consume_genetics_prefix()` replaces `peek_first_protocol_byte()`. `dispatch_g65()` routes negotiated connections to tarpc binary or JSON-RPC |
| `crates/loam-spine-api/src/lib.rs` | `pub mod protocol_negotiation` registered |
| `crates/loam-spine-api/Cargo.toml` | `tokio-util` dependency added |
| `Cargo.toml` | `tokio-util` workspace dependency added |

### Test Coverage

| Category | Count | Details |
|----------|------:|---------|
| Protocol negotiation unit | 10 | Wire roundtrip, selection, parsing, display, format, duplex negotiate (tarpc + jsonrpc-only), not-negotiation passthrough, leftover bytes, oversized line |
| UDS genetics-prefix | 7 | Updated for `consume_genetics_prefix()` (riboCipher, mito-beacon, nuclear-sealed, passthrough, prefix-only, non-genetics, PROTOCOLS detection) |
| **Total new** | **13** | |
| **Running total** | **1,783** | All passing |

---

## Wire Protocol

```text
Client → Server: "PROTOCOLS: tarpc,jsonrpc\n"
Server → Client: "PROTOCOL: tarpc\n"
[Connection proceeds exclusively in selected protocol]
```

- No `PROTOCOLS:` line → JSON-RPC (backward compatible)
- Client preference order wins
- 256-byte max negotiation line (guard against abuse)
- Genetics prefix (0xEC/0xED/0xEE) composes cleanly with G65

---

## Architecture Decision: Stream Splitting

The UDS handler was restructured to check for G65 **before** splitting the
`UnixStream` into read/write halves. tarpc's `BaseChannel` requires a
full-duplex stream for its length-delimited framing. The `consume_genetics_prefix()`
function reads from the raw stream, and if the first real byte is `P`, the
unsplit stream is passed to `dispatch_g65()`. Only non-G65 connections split.

---

## Verification

```bash
cargo clippy --workspace --all-targets -- -D warnings  # clean
cargo fmt --all -- --check                              # clean
cargo test --workspace                                  # 1,783 passing
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps  # clean
```

---

## Cephalization Status

| Phase | loamSpine Status |
|-------|------------------|
| Phase 1 (JSON-RPC) | COMPLETE — 53 methods |
| Phase 2 (C2 dual-socket) | COMPLETE — retained for backward compat |
| **Phase 3 (G65)** | **SHIPPED** |
| tarpc methods | 37 (full domain parity) |

---

## What's Next

- C2 `.tarpc.sock` listener can be removed once cellMembrane evolves to
  G65-aware discovery (drops `has_tarpc` field)
- `consume_genetics_prefix()` + G65 compose cleanly; no further changes
  needed for BTSP Phase 3 or riboCipher evolution
