<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# loamSpine Wave 155n — G31 Batch Provenance Pipeline

**Date**: August 3, 2026
**From**: sporeGate → wateringHole
**Wave**: 155n
**Status**: G31 batch operations — loamSpine side complete. Awaits cross-primal coordination (sweetGrass, rhizoCrypt).

---

## Context

Wave 155n (NUCLEUS convergence) identifies G31 as a P1 item: coordinated
cross-primal batch ops for 10× faster bulk ingestion. loamSpine's current
~30 ms/object is dominated by per-call spine I/O — each `entry.append` and
`certificate.mint` reads the spine, appends, writes the spine. For PDB's
220K structures across 38 datasets, this is the bottleneck.

---

## What Shipped

### 1. `entry.append_batch` (JSON-RPC)

Append N entries to a spine in one RPC call. Core optimization:

- **1 spine read** instead of N
- **N entry creates** from the spine's evolving in-memory state (correct chain indexing)
- **N entry saves** + **1 spine save** instead of N+N

Request/response types: `AppendEntryBatchRequest` (spine_id + Vec<EntryType>),
`AppendEntryBatchResponse` (Vec<BatchEntryResult> with per-entry hash + index, count).

### 2. `certificate.mint_batch` (JSON-RPC)

Mint N certificates in one RPC call with same amortization pattern:

- 1 spine read, N entry creates + appends, N certificate creates
- Batch entry persistence + 1 spine save + N certificate saves

Request/response types: `MintCertificateBatchRequest` (spine_id + Vec<BatchMintItem>),
`MintCertificateBatchResponse` (Vec<BatchMintResult> with cert_id + mint_hash, count).

### 3. Performance Target

| Operation | Before (per-object) | After (batch amortized) |
|-----------|-------------------|----------------------|
| entry.append | ~30 ms | ~3 ms |
| certificate.mint | ~30 ms | ~3 ms |

Amortization comes from eliminating N-1 spine reads and N-1 spine writes.
Storage saves are still per-entry but sequential with no intervening I/O.

### 4. CLI Standardization

`--bind` alias for `--bind-address` — biomeOS P2 for uniform primal startup
during NUCLEUS composition lifecycle.

### 5. Tests

7 new tests:
- `append_entry_batch_chains_correctly` — 5 entries chain with correct indices
- `append_entry_batch_empty_is_noop` — empty batch returns empty vec
- `append_entry_batch_sealed_spine_fails` — sealed spine rejects batch
- `mint_certificate_batch_creates_all` — 10 certificates with unique IDs
- `mint_certificate_batch_empty_is_noop` — empty batch returns empty vec
- `append_entry_batch_rpc_returns_ordered_results` — API-level batch with monotonic indices
- `mint_certificate_batch_rpc_creates_all` — API-level batch mint with unique IDs

**Total**: 1,747 tests, 211 source files, 50 JSON-RPC methods.

---

## What's Needed for G31 Completion

loamSpine's batch surface is ready. G31 requires cross-primal coordination:

1. **sweetGrass** batch braid commit — `braid.commit_batch` for bulk attribution
2. **rhizoCrypt** batch DAG anchoring — `dag.anchor_batch` for bulk provenance chains
3. **Coordinated caller** (nestGate or biomeOS signal graph) — orchestrates
   `certificate.mint_batch` → `braid.commit_batch` → `dag.anchor_batch` as a
   single provenance pipeline per dataset

---

## File Changes

| File | Change |
|------|--------|
| `crates/loam-spine-core/src/service/mod.rs` | `append_entry_batch()` |
| `crates/loam-spine-core/src/service/certificate.rs` | `mint_certificate_batch()` |
| `crates/loam-spine-core/src/service/service_mod_tests.rs` | 3 new tests |
| `crates/loam-spine-core/src/service/certificate_tests.rs` | 2 new tests |
| `crates/loam-spine-core/src/niche.rs` | 2 new methods |
| `crates/loam-spine-core/src/neural_api/mcp.rs` | 2 new MCP mappings |
| `crates/loam-spine-api/src/types/mod.rs` | Batch entry request/response types |
| `crates/loam-spine-api/src/types/certificate.rs` | Batch mint request/response types |
| `crates/loam-spine-api/src/service/entry_ops.rs` | `append_entry_batch()` handler |
| `crates/loam-spine-api/src/service/certificate_ops.rs` | `mint_certificate_batch()` handler |
| `crates/loam-spine-api/src/service/service_tests.rs` | 2 new tests |
| `crates/loam-spine-api/src/jsonrpc/mod.rs` | 2 dispatch entries |
| `config/capability_registry.toml` | 2 new operations |
| `bin/loamspine-service/main.rs` | `--bind` alias |
| Root docs | Updated counts |

---

## Verification

```
cargo fmt --all          ✓ (no changes)
cargo clippy --workspace ✓ (0 warnings, pedantic + nursery)
cargo test --workspace   ✓ (1,747 passed, 0 failed)
cargo doc --workspace    ✓ (clean)
```
