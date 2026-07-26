<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# loamSpine — Wave 151b: BTSP ClientHello Handshake

**Date**: July 26, 2026  
**From**: loamSpine team (eastGate)  
**Wave**: 151b — BTSP standard evolution  
**Status**: COMPLETE

---

## Summary

Evolved loamSpine from BTSP server-only to full BTSP client + server.
Client-side 4-step handshake wired into both outbound bearDog connection
paths: Tower signer (`crypto_provider_call`) and BTSP provider relay
(`ProviderConn::connect`).

Previously, loamSpine connected to bearDog via plain JSON-RPC. With
sporeGate deploying `BEARDOG_UDS_REQUIRE_BTSP=1`, loamSpine needed
client-side BTSP handshake to maintain crypto access in Nest Atomic.

---

## What Changed

### Evolved: `btsp_client.rs` (upstream) + new integration tests

| Item | Detail |
|------|--------|
| Public API | `perform_client_handshake(stream) → Result<BtspClientSession, BtspClientError>` |
| HMAC | `HMAC-SHA256(family_seed.trim().as_bytes(), base64_decode(challenge))` |
| Cipher | Requests `chacha20_poly1305` |
| Env gate | `btsp_strict_mode_expected()` — checks `BEARDOG_UDS_REQUIRE_BTSP=1` or `BTSP_STRICT_MODE=1` |
| Seed resolution | Fixed to songBird standard: `FAMILY_SEED` → `BTSP_FAMILY_SEED` → `BEARDOG_FAMILY_SEED` |

### Wired: Both outbound paths

| Path | File | Integration |
|------|------|-------------|
| Tower signer | `traits/crypto_provider.rs` | Handshake before `ndjson_rpc_call` (upstream) |
| BTSP provider | `btsp/provider_client.rs` | Handshake before splitting reader/writer (new) |

Both paths: if handshake fails, log warning and proceed with plain JSON-RPC.

### New tests: 5 integration tests with mock bearDog server

| Test | Scenario |
|------|----------|
| `client_handshake_success` | Full 4-step handshake with HMAC verification |
| `client_handshake_rejected_by_server` | Server sends HandshakeError after ChallengeResponse |
| `client_handshake_no_family_seed` | Missing FAMILY_SEED returns NoFamilySeed |
| `client_handshake_server_sends_error_on_hello` | Server rejects at ServerHello step |
| `client_handshake_server_disconnects` | Server drops connection after ClientHello |

---

## Verification

```
cargo fmt --all --check     → CLEAN
cargo clippy --workspace    → 0 warnings
cargo test --workspace      → 1,723 passed, 0 failed
cargo doc --workspace       → 0 warnings
cargo check --target x86_64-pc-windows-gnu → CLEAN
```

---

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests | 1,715 | 1,723 |
| Source files | 209 | 210 |
| BTSP server | Phase 2+3 | Phase 2+3 |
| BTSP client | Phase 2 (1 path) | Phase 2 (both paths) |

---

## BTSP Status

| Role | Status |
|------|--------|
| Server-side handshake (incoming) | COMPLETE (Phase 2+3) |
| Client-side handshake (outbound) | **COMPLETE** (Phase 2, both paths) |
| Phase 3 encrypted transport | COMPLETE (ChaCha20-Poly1305) |

**loamSpine is BTSP-compliant for Nest Atomic.**

---

*Wave 151b: loamSpine BTSP client handshake DONE. 1,723 tests.
Ready for Nest Atomic Phase 1 (append-only DAG ledger).*
