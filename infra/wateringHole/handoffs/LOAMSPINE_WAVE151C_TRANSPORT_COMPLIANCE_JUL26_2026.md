# loamSpine — Wave 151c: TransportEndpoint Compliance + Deep Debt

**Date**: July 26, 2026  
**From**: loamSpine team (sporeGate/eastGate)  
**Wave**: 151c  
**Status**: COMPLETE

---

## Summary

All outbound IPC connections now route through `connect_transport(&TransportEndpoint)`,
completing the TransportEndpoint compliance that Wave 142b started. Previously,
`sync/mod.rs` (federation) and `discovery/mod.rs` (attestation) used raw
`TcpStream::connect(endpoint)`, bypassing platform-abstracted transport dispatch.

---

## Changes

### TransportEndpoint Compliance (P1)

- `sync/mod.rs::rpc_call` — evolved from raw `TcpStream::connect(endpoint)` to
  `connect_transport(&endpoint_from_addr(endpoint))`. Removes direct `tokio::net::TcpStream`
  import. TCP nodelay is now handled inside `connect_transport`.
- `discovery/mod.rs::jsonrpc_call` — same evolution. Raw `TcpStream` import removed.
- New helper: `transport::endpoint_from_addr("host:port")` → `TransportEndpoint::Tcp`.

### Error Visibility (P1)

- `discovery/mod.rs` — capability discovery (`signing`, `verification`) failures
  now log at `debug` level instead of being silently swallowed via `if let Ok(...)`.
- `sync/streaming.rs` — progress channel `send()` failures traced instead of
  `let _ =` swallowed (receiver may legitimately drop on client disconnect).
- `transport/stream.rs` — TCP `set_nodelay` failure traced at `trace` level.

### Dead Code Removal

- `jsonrpc/server.rs:302` — stale `let _ = &request_line` borrow removed.

### Test Coverage (11 new tests)

| Category | Count | Detail |
|----------|-------|--------|
| `generate_aggregate_proof` | 5 | Single leaf, two leaves, four leaves, odd count, out-of-bounds |
| `read_ndjson_stream_bounded` | 2 | Backpressure (limit hit), under-limit (no truncation) |
| `endpoint_from_addr` | 4 | Valid IPv4, localhost, missing port, invalid port |

### Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests | 1,723 | 1,731 |
| Source files | 210 | 210 |
| Lines of Rust | ~64,120 | ~64,292 |
| Raw `TcpStream::connect` in production | 2 | 0 |
| Silently swallowed discovery errors | 2 | 0 |
| Clippy warnings | 0 | 0 |
| Production `unwrap`/`expect` | 0 | 0 |

---

## Audit Findings (Remaining Low-Priority)

Items reviewed but not actioned (correct as-is or deferred):

1. **`#[allow(clippy::wildcard_imports)]`** in `service/mod.rs` and `tarpc_server.rs` —
   cannot use `#[expect]` because lint is conditionally unfulfilled in test target.
2. **`#[allow(dead_code)]`** on `send_heartbeat_with_retry` in `lifecycle.rs` —
   same conditional unfulfillment issue.
3. **Clone density** — top files (12 clones in `manager/mod.rs`) are structurally
   necessary (ownership transfers of `Did`, `Certificate` values). Not reducible
   without API redesign.
4. **Dead code stubs** ("pre-wired for provenance trio") — retained for Nest Atomic
   Phase 1 (loamSpine DAG ledger prototype).
5. **`Vec<u8>` → `bytes::Bytes`** — 21 candidates on wire paths. Deferred to
   dedicated zero-copy pass when NDJSON framing is redesigned.
6. **`dns-srv`/`mdns` features** — implemented but not enabled in CI or production
   binary. Documented as opt-in discovery backends.

---

## For Upstream Teams

No upstream demand signal changes from this wave.
