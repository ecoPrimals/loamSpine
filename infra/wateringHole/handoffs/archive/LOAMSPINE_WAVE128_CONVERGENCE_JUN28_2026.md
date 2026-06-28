<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# loamSpine — Wave 128 Convergence: Provenance Depth + Debt Cleanup

**Date**: June 28, 2026
**Author**: sporeGate Overwatch
**Wave**: 128
**Supersedes**: LOAMSPINE_COVERAGE_PUSH_CONFIG_REFRESH_JUN20_2026

---

## Summary

Wave 128 convergence pass addressing the P1 Nest provenance depth requirement
(ledger → 5+), production doc terminology cleanup (BearDog → generic BTSP
provider), and targeted coverage push on env_resolution and crypto_provider.

## Changes

### P1: Nest Provenance Depth (ledger → 5+)

- **`ProvenanceLink`** evolved with `depth: u32` field tracking chain position
- **`get_provenance_chain()`** now:
  - Matches `PublicChainAnchor` (`state_hash`) → "chain-anchored" relationship
  - Matches `CertificateMint` (entry hash) → "certified-by" relationship
  - Sorts results by `(timestamp, index)` for deterministic ordering
  - Assigns `depth` (0-indexed) to each link
- 6-link cross-spine integration test verifying depth ≥ 5 with all relationship types
- Additional tests: certificate mint provenance, deterministic ordering

### P4: Doc Terminology Cleanup

BearDog → generic BTSP provider in all production doc comments:
- `btsp/wire.rs` — 7 doc comments genericized
- `btsp/handshake.rs` — 2 doc comments
- `btsp/phase3.rs` — 1 module doc
- `btsp/provider_client.rs` — 1 struct doc
- `loam-spine-api/src/service/mod.rs` — 1 method doc
- `loam-spine-api/src/service/btsp_ops.rs` — 1 method doc
- `KNOWN_ISSUES.md` — 1 entry cleaned

Backward-compat env vars (`BEARDOG_SOCKET`, `BEARDOG_FAMILY_SEED`) preserved.

### P3: Coverage Push (+17 tests)

| Area | Tests Added |
|------|-------------|
| `env_resolution.rs` | 11 (family_seed fallback chain ×4, tower_signer_socket ×3, key constants, cache_ttl, auth_mode, signer_did) |
| `crypto_provider.rs` | 4 (error response handling, verify_entry delegation, request counter ×2) |
| `integration_tests.rs` | 2 (5+ depth chain, certificate mint provenance) |

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 1,669 |
| Line coverage | ~92%+ |
| Source files | 199 |
| JSON-RPC methods | 47 |
| `#[allow]` in prod | 0 |
| `unsafe` in prod | 0 (forbidden) |
| Files >800L | 0 |
| TODO/FIXME/HACK | 0 |
| cargo fmt | PASS |
| cargo clippy | PASS (0 warnings) |
| cargo doc | PASS |
| cargo deny | PASS |

## Gaps for Upstream

| Item | Owner | Priority |
|------|-------|----------|
| Signing capability middleware (RPC layer signature verification) | loamSpine | v0.10.0 |
| `checksums.toml` for genomeBin/depot | loamSpine | LOW |
| Persistent BTSP tunnels for ledger replication | ecosystem | FUTURE |
| PostgreSQL/RocksDB backends | loamSpine | v1.0.0 (demand-driven) |

## Verification

```
cargo fmt --all --check    → PASS
cargo clippy --workspace   → 0 warnings
cargo test --workspace     → 1,669 passed, 0 failed
cargo doc --workspace      → PASS
cargo deny check           → bans ok, licenses ok, sources ok
```
