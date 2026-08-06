<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# loamSpine — Wave 156n: tarpc Method Test Coverage + UDS E2E

**Date**: August 6, 2026  
**From**: sporeGate  
**Primal**: loamSpine  
**Status**: GREEN — all checks pass

---

## Summary

All 13 G64 tarpc methods now have dedicated tarpc-path tests. tarpc UDS server
has full E2E test coverage: real client→server round-trip over unix domain
socket, plus socket cleanup-on-drop verification.

## Changes

### New Tests (15)

**G64 tarpc method coverage (compound tests)**:
- `test_tarpc_append_entry_batch` — batch append 2 DataAnchor entries
- `test_tarpc_mint_certificate_batch` — batch mint 2 DataProvenance certs
- `test_tarpc_publish_anchor_batch` — aggregate anchor across 2 spines
- `test_tarpc_trust_anchor` — KeyExchange trust event anchoring

**G64 tarpc method coverage (lifecycle tests)**:
- `test_tarpc_list_spines` — list all spine IDs
- `test_tarpc_spine_status` — spine observability status
- `test_tarpc_list_entries` — paginated entry listing
- `test_tarpc_verify_certificate_not_found` — verify nonexistent cert
- `test_tarpc_certificate_lifecycle_not_found` — lifecycle error path
- `test_tarpc_certificate_history_not_found` — history error path
- `test_tarpc_trust_event_count` — zero-count baseline
- `test_tarpc_trust_query_empty` — empty query result
- `test_tarpc_negotiate_btsp` — cipher suite negotiation

**tarpc UDS E2E**:
- `run_tarpc_uds_server_accepts_client` — full round-trip: bind→connect→health_check→assert
- `tarpc_uds_handle_cleans_up_socket_on_drop` — socket file removal on Drop

### Files Modified

| File | Change |
|------|--------|
| `tarpc_server_tests_lifecycle.rs` | +11 tests (G64 methods + UDS E2E + cleanup) |
| `tarpc_server_tests_compound.rs` | +4 tests (batch + trust anchor) |
| Root docs (README, STATUS, CONTEXT, CONTRIBUTING, WHATS_NEXT, CHANGELOG) | Test count 1,755→1,770 |
| `sporeprint/validation-summary.md` | Test count updated |

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 1,770 |
| Clippy | 0 warnings (pedantic + nursery) |
| Fmt | Clean |
| Doc | Clean |

## Posture

loamSpine is tarpc-CONVERGED with full test coverage on all 37 tarpc methods.
C2 dual-socket shipped. Deep debt clean. G65 protocol negotiation gated on
sourDough reference implementation (C7).

---

*Wave 156n — 15 new tarpc tests. 1,770 tests total. All checks pass.*
