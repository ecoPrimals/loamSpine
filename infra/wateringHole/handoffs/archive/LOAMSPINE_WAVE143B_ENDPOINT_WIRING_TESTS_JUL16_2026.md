<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# loamSpine — Wave 143b Handoff

**Date**: July 16, 2026  
**Wave**: 143b  
**From**: loamSpine team on sporeGate  
**To**: eastGate overwatch / primalSpring

---

## Summary

Wave 143b wires `TRANSPORT_ENDPOINT` to functional dispatch (was log-only),
splits the largest remaining test file (789L), and adds 7 framing edge-case
tests for UDS + TCP roundtrip coverage.

## Changes

### `TRANSPORT_ENDPOINT` Functional Dispatch

`main.rs` now uses the parsed `TransportEndpoint` to drive server startup:

- **UDS endpoint**: injected socket path overrides `--socket` flag and env resolution.
- **TCP endpoint**: injected host overrides `--bind-address`, injected port overrides `--port`.
- **TCP auto-enable**: TCP server starts automatically when a TCP endpoint is injected (no need for explicit `--port` flag).

### Test File Split

`service_tests.rs` (789L → 3 focused modules):

| Module | Domain | Lines |
|--------|--------|-------|
| `service_tests.rs` | Core spine CRUD, certificates, proof | 388 |
| `service_tests_integration.rs` | `permanent_storage.*`, `commit_session` | 270 |
| `service_tests_btsp.rs` | BTSP negotiate, key derivation interop | 111 |

### Framing Edge-Case Tests

7 new tests in `transport/framing.rs`:

| Test | What it covers |
|------|----------------|
| `length_prefixed_zero_length_frame` | Server sends 0-length response |
| `length_prefixed_server_disconnect` | Server drops connection after reading request |
| `ndjson_server_sends_empty_line_then_response` | NDJSON with string-typed result |
| `ndjson_roundtrip_via_uds` | Full NDJSON call over Unix domain socket |
| `length_prefixed_roundtrip_via_uds` | Full length-prefixed call over Unix domain socket |

(UDS tests are `#[cfg(unix)]`.)

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests | 1,697 | 1,704 |
| Source files | 204 | 206 |
| Max test file | 789L (`service_tests.rs`) | 779L (`lifecycle_tests.rs`) |
| Largest production file | 660L (`uds.rs`) | 660L (`uds.rs`) |

## All Checks

- `cargo fmt --all --check` — PASS
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS (0 warnings)
- `cargo test --workspace` — 1,704 tests, 0 failures
- `cargo doc --workspace --no-deps` — PASS
- `cargo check --target x86_64-pc-windows-gnu` — PASS

## Phase 2 Status Note

**loamSpine outbound Transport already shipped in Wave 142b** — `TransportStream`
enum, `connect_transport()` dispatch, NDJSON and length-prefixed framing helpers,
all 4 outbound IPC clients migrated. Wave 143b blurb lists loamSpine as "TODO"
but this reflects stale state. Inbound server-side `listen_transport` abstraction
is P2 future work.

## Remaining Debt (P2+)

- Inbound server `listen_transport` abstraction (`uds.rs` → `TransportEndpoint` for listen)
- Dead code wiring: `manifest.rs` discovery helpers, `negotiate_protocol()`, `AUTH_ERROR` const
- Health probes: evolve `unused_async` to truly async (storage/discovery probes)

---

*Wave 143b: Endpoint wiring + test coverage. loamSpine Phase 2 outbound transport COMPLETE.*
