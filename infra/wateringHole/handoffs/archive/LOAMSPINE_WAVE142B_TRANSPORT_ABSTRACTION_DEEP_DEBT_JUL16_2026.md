<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# LoamSpine Wave 142b: Silicon Atheism Phase 2 + Deep Debt

**Date**: July 16, 2026  
**Wave**: 142b  
**From**: sporeGate loamSpine team  
**To**: eastGate overwatch (primalSpring)

---

## Summary

Phase 2 of Silicon Atheism complete for loamSpine. All outbound IPC clients
migrated from raw `#[cfg(unix)] UnixStream` to trait-based `TransportEndpoint`
dispatch via new `TransportStream` enum + `connect_transport()`. Deep debt sweep:
custom base64 replaced, blocking fs wrapped in spawn_blocking, clone reduction,
doc drift fixed.

---

## Changes Delivered

### Wave 142b (July 16, 2026)

| Item | Detail |
|------|--------|
| `transport/stream.rs` (NEW) | `TransportStream` enum (UDS/TCP), `connect_transport()` dispatch, `endpoint_from_path()` |
| `transport/framing.rs` (NEW) | NDJSON + length-prefixed IPC helpers — `write_ndjson_request`, `read_ndjson_response`, `ndjson_rpc_call`, `write_length_prefixed`, `read_length_prefixed`, `length_prefixed_rpc_call` |
| `btsp/provider_client.rs` | `ProviderConn` now uses `TransportStream` halves + NDJSON framing. All `#[cfg]` removed. |
| `traits/crypto_provider.rs` | `crypto_provider_call` uses `connect_transport` + `ndjson_rpc_call`. All `#[cfg]` removed. |
| `transport/neural_api.rs` | `jsonrpc_call` uses `connect_transport` + `length_prefixed_rpc_call`. Custom `base64_decode` (40L) replaced with workspace `base64` crate. |
| `neural_api/mod.rs` | `register_at_socket`/`deregister_at_socket` use `connect_transport` + `length_prefixed_rpc_call`. All `#[cfg]` removed. Doc comments genericized. |
| `jsonrpc/uds.rs` | Blocking `std::fs` calls (`create_dir_all`, `remove_file`) wrapped in `spawn_blocking`. |
| `main.rs` | PID file write, symlink creation/removal, shutdown cleanup wrapped in `spawn_blocking`. |
| `service/integration.rs` | Clone reduction: `committer` moved directly, `owner` extracted after `save_spine`, `holder` moved on last use. |
| `btsp/handshake.rs` | Clone reduction: `BtspSession` built once, fields borrowed for `HandshakeComplete`. |
| `service/trust_ledger.rs` | Doc comment genericized (removed external primal name). |

### Wave 141a (July 15, 2026 — now archived)

- Cross-architecture `#[cfg(unix)]` gating for Windows GNU target
- Integration test refactor (1002L → 3 modules)
- BearDog env deprecation warnings
- Clone reduction in certificate_loan.rs

---

## Metrics

| Metric | Value |
|--------|-------|
| Tests | **1,697** (all passing) |
| Source files | **204** `.rs` (+3 fuzz targets) |
| JSON-RPC methods | **47** |
| Coverage | 92.26% line |
| Max production file | 660L (`uds.rs`) |
| Max test file | 789L (`service_tests.rs`) |
| `#[cfg(unix)]` outbound blocks | **0** (all migrated to `TransportEndpoint`) |
| `cargo check --target x86_64-pc-windows-gnu` | Clean (0 errors, 0 warnings) |

---

## Remaining Work (loamSpine-local)

| Item | Priority | Notes |
|------|----------|-------|
| Inbound server Named Pipe (Windows) | P2 | Future: `\\.\pipe\ecoPrimals-{stem}` |
| `main.rs` TRANSPORT_ENDPOINT wiring | P2 | Make functional, not log-only |
| JSON-RPC response `Vec<u8>` → `Bytes` | P2 | Wire layer optimization |
| benchScale trust.* methods (44/47) | P2 | 3 trust methods uncovered |
| Spec date headers refresh | P3 | Several from Dec 2025 |
| SERVICE_LIFECYCLE.md Songbird → generic | P3 | Capability-based terminology |

---

## Upstream Gaps for Overwatch

| Gap | Affects | Notes |
|-----|---------|-------|
| benchScale 44/47 methods | Validation | trust.anchor, trust.query, trust.event_count not in roundtrip |
| Spec date headers | Documentation | 6 specs still show Dec 2025 dates; content is accurate |
| SERVICE_LIFECYCLE.md | Documentation | References Songbird as discovery — should be capability-based |

---

*Wave 142b: Phase 2 transport abstraction complete. All outbound IPC is
`TransportEndpoint`-dispatched. Deep debt sweep: base64, spawn_blocking,
clone reduction, doc genericization. 1,697 tests, 204 source files.*
