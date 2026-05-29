<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# loamSpine — Wave 60 Session Dehydration Handoff

**Date**: May 29, 2026
**Wave**: 60
**Status**: COMPLETE

---

## Upstream Target: `session.dehydrate`

### What was requested

primalSpring Wave 60 audit specified one new method for loamSpine:

| Method | Signal Graph | What It Does |
|--------|-------------|--------------|
| `session.dehydrate` | rootpulse.commit | Serialize session state for content-addressed storage |

Priority: MEDIUM — rootPulse needs this to dehydrate before signing.

### What was implemented

`session.dehydrate` is a read-only JSON-RPC method that computes a blake3
content-addressed summary of a spine's uncommitted entries.

**Flow**: `session.dehydrate` → (sign hash) → `session.commit`

**Request** (`DehydrateSessionRequest`):
- `spine_id`: Target spine UUID
- `session_id`: Caller-assigned session UUID
- `committer`: DID of the requester
- `session_type`: Optional label (default: `"session"`)

**Response** (`DehydrateSessionResponse`):
- `spine_id`, `session_id`, `committer` (echoed)
- `session_hash`: Blake3 hash (content address)
- `entry_count`: Number of uncommitted entries included
- `dehydrated_at`: Timestamp
- `session_type`: Label (echoed)

**Hash computation**: `blake3(session_id || spine_id || entry_hash_1 || ... || entry_hash_n)`
where entries are those since the last `SessionCommit` entry (or since genesis).

### Wiring

| Layer | File | Change |
|-------|------|--------|
| Types | `crates/loam-spine-api/src/types/mod.rs` | `DehydrateSessionRequest` + `DehydrateSessionResponse` |
| Service | `crates/loam-spine-api/src/service/integration_ops.rs` | `dehydrate_session()` implementation |
| JSON-RPC | `crates/loam-spine-api/src/jsonrpc/mod.rs` | `"session.dehydrate" => rpc!(params, dehydrate_session)` |
| tarpc | `crates/loam-spine-api/src/rpc.rs` + `tarpc_server.rs` | `dehydrate_session` trait method + server impl |
| MCP | `crates/loam-spine-core/src/neural_api/mcp.rs` | `session_dehydrate` tool |
| Niche | `crates/loam-spine-core/src/niche.rs` | Added to `METHODS` + `SEMANTIC_MAPPINGS` |
| Capabilities | `crates/loam-spine-core/src/neural_api/mod.rs` | Session group: `["dehydrate", "commit"]`, cost estimate, dependency |

### Tests

3 new integration tests:
- `test_jsonrpc_dehydrate_session` — basic dehydration of a fresh spine
- `test_jsonrpc_dehydrate_then_commit` — full pipeline: create → append → dehydrate → commit
- `test_jsonrpc_dehydrate_idempotent` — repeated calls return same hash

benchScale Phase 7 updated: dehydrate → commit flow with real hash handoff.

---

## DH-1: /tmp Hardcoding Compliance

loamSpine is **not listed** in the Wave 60 DH-1 offenders. Confirmed clean:

- Zero `/tmp` literals in production Rust code
- Socket resolution: `$XDG_RUNTIME_DIR` → `/run/user/{uid}/biomeos/` → `std::env::temp_dir()/biomeos/`
- All test `/tmp` usage is fixture paths in `#[cfg(test)]` modules

---

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Methods | 43 | 44 |
| Tests | 1,528 | 1,533 |
| Clippy warnings | 0 | 0 |
| Compilation warnings | 0 | 0 |
