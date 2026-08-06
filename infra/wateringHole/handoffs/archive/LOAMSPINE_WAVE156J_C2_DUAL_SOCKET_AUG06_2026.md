<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# loamSpine — Wave 156j: C2 Dual-Socket Pattern

**Date**: August 6, 2026  
**From**: sporeGate  
**To**: overwatch (eastGate)

---

## Summary

loamSpine now binds a **tarpc UDS server** alongside the existing JSON-RPC UDS server, implementing the **C2 dual-socket pattern** that songBird (C1a) and petalTongue (C1b+C2) have already shipped.

## What Changed

### tarpc UDS Server (`tarpc_server.rs`)

- `run_tarpc_uds_server()` — binds `tarpc::serde_transport::unix::listen` on `.tarpc.sock`
- `TarpcUdsHandle` — cooperative shutdown + socket cleanup on drop
- Backpressure: `max_concurrent_requests` from `TarpcServerConfig`
- Stale socket cleanup on startup (same pattern as JSON-RPC UDS)

### Socket Path Derivation (`neural_api/socket.rs`)

- `tarpc_socket_from_jsonrpc()` — derives `{stem}.tarpc.sock` from JSON-RPC socket path
- `resolve_tarpc_socket_path()` — env-based resolution (reads LOAMSPINE_SOCKET, XDG_RUNTIME_DIR, FAMILY_ID)
- Exported from `neural_api` module

### Server Wiring (`main.rs`)

- tarpc UDS server starts alongside JSON-RPC UDS on every startup
- Startup log shows both socket paths
- Cooperative shutdown: both UDS servers stopped before lifecycle teardown
- Socket cleanup: both `.sock` and `.tarpc.sock` removed on graceful shutdown

### Protocol Negotiation

- `negotiate_protocol_from()` (pre-wired since G3) will now find the `.tarpc.sock` and escalate automatically
- Primals connecting to loamSpine get tarpc binary framing without configuration

## File Changes

| File | Change |
|------|--------|
| `crates/loam-spine-api/src/tarpc_server.rs` | +131 lines: `TarpcUdsHandle`, `run_tarpc_uds_server`, `run_tarpc_uds_server_with_config` |
| `crates/loam-spine-api/src/lib.rs` | Export `TarpcUdsHandle`, `run_tarpc_uds_server` |
| `crates/loam-spine-core/src/neural_api/socket.rs` | +18 lines: `tarpc_socket_from_jsonrpc`, `resolve_tarpc_socket_path` |
| `crates/loam-spine-core/src/neural_api/mod.rs` | Export new functions |
| `bin/loamspine-service/main.rs` | +37 lines: tarpc UDS startup, shutdown, cleanup |
| `crates/loam-spine-core/src/neural_api/tests_socket.rs` | +21 lines: 3 new tests |

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests | 1,752 | 1,755 |
| JSON-RPC methods | 53 | 53 |
| tarpc methods | 37 | 37 |
| UDS sockets | 1 (JSON-RPC) | 2 (JSON-RPC + tarpc) |
| clippy | 0 warnings | 0 warnings |
| fmt | clean | clean |
| doc | clean | clean |
| unsafe | 0 (forbid) | 0 (forbid) |

## Socket Layout (C2 Pattern)

```
$XDG_RUNTIME_DIR/biomeos/
├── loamspine.sock          # JSON-RPC 2.0 (discovery, diagnostics, external clients)
├── loamspine.tarpc.sock    # tarpc binary framing (primal-to-primal composition)
├── loamspine.pid           # PID file for liveness checks
├── ledger.sock → loamspine.sock     # capability symlink
└── permanence.sock → loamspine.sock # legacy symlink
```

With family ID:
```
├── loamspine-{fid}.sock
├── loamspine-{fid}.tarpc.sock
├── loamspine-{fid}.pid
├── ledger-{fid}.sock → loamspine-{fid}.sock
└── permanence-{fid}.sock → loamspine-{fid}.sock
```

## C2 Status Across Ecosystem

| Primal | C2 Status |
|--------|-----------|
| songBird | DONE (C1a) |
| petalTongue | DONE (C1b+C2) |
| **loamSpine** | **DONE (this wave)** |
| Others | Pending |

## For Upstream

- loamSpine is now **tarpc-CONVERGED + dual-socket C2** — the most complete cephalization posture.
- Protocol negotiation (`negotiate_protocol_from`) works bidirectionally — any primal can discover loamSpine's tarpc socket.
- Next C2 candidates: sourDough (reference impl), then remaining primals per the blurb priority.

---

*Wave 156j — C2 dual-socket shipped. 1,755 tests. All checks clean.*
