# Handoff: loamSpine Wave 156s — G66 Transport Abstraction

**Date**: August 6, 2026  
**From**: sporeGate  
**To**: overwatch / eastGate / upstream  
**Wave**: 156s  
**HEAD**: (pending commit)

---

## Summary

G66 transport abstraction shipped for loamSpine. Silicon deism eliminated:
zero unconditional `UnixStream` usage in production code. Protocol
negotiation and tarpc serving are now generic over `AsyncRead + AsyncWrite
+ Unpin`. Server-side `TransportListener` added, symmetric to the existing
`connect_transport()`.

---

## Changes

### `loam-spine-core` — Transport Layer

| Component | Change |
|-----------|--------|
| `TransportListener` | NEW — server-side listener enum (`Uds`/`Tcp`). `bind(&TransportEndpoint)` dispatches to platform. `accept()` returns `TransportStream`. `#[cfg(unix)]` on UDS variant + bind. |
| `bind_local()` | NEW — UDS bind with stale socket cleanup, parent dir creation. Non-unix stub returns error. |

### `loam-spine-api` — Protocol Negotiation + tarpc

| Component | Before | After |
|-----------|--------|-------|
| `try_negotiate()` | `stream: &mut UnixStream` | `stream: &mut S where S: AsyncRead + AsyncWrite + Unpin` |
| `negotiate_client()` | `stream: &mut UnixStream` | `stream: &mut S where S: AsyncRead + AsyncWrite + Unpin` |
| `serve_tarpc_connection()` | `#[cfg(unix)]`, `UnixStream` | Generic `<S: AsyncRead + AsyncWrite + Unpin + Send + 'static>`, no cfg gate |

### Tests

| Category | Count | Detail |
|----------|------:|--------|
| TransportListener TCP accept | 1 | bind → accept → roundtrip |
| TransportListener UDS accept | 1 | `#[cfg(unix)]` bind → accept |
| TransportListener mesh relay error | 1 | bind returns error |
| G65 negotiate TCP (tarpc) | 1 | Replaced UDS version |
| G65 negotiate TCP (jsonrpc) | 1 | Replaced UDS version |
| G65 not-negotiation TCP | 1 | Replaced UDS version |
| G65 leftover bytes TCP | 1 | Replaced UDS version |
| G65 oversized line TCP | 1 | Replaced UDS version |
| G65 negotiate UDS roundtrip | 1 | `#[cfg(unix)]` regression guard |
| **Net new** | **4** | |
| **Total** | **1,787** | All passing |

---

## Silicon Deism Scorecard

| Location | Before G66 | After G66 |
|----------|-----------|----------|
| `protocol_negotiation.rs` public API | `UnixStream` (unconditional) | Generic `<S>` |
| `serve_tarpc_connection()` | `#[cfg(unix)]` + `UnixStream` | Generic, no cfg gate |
| `TransportListener` (server bind) | None — `UnixListener` hard-wired in `uds.rs` | `TransportListener::bind()` dispatches |
| G65 tests | `UnixStream::pair()` (all) | TCP-based (5) + UDS guard (1) |

---

## Verification

```bash
cargo clippy --workspace --all-targets -- -D warnings  # clean
cargo fmt --all -- --check                              # clean
cargo test --workspace                                  # 1,787 passing
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps  # clean
```

---

## What's Next

- UDS handler (`uds.rs`) can be evolved to accept `TransportStream` in a
  future wave when TCP inbound serving is needed (currently UDS-only server
  is correct for the unix gate deployment)
- `TransportListener` enables future TCP-based service mode without
  duplicating the accept loop
