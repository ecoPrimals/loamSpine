# loamSpine Wave 156e — spine.status Observability + Doc Debt

**Date**: August 5, 2026  
**From**: sporeGate  
**Wave**: 156e  
**Status**: GREEN — 1,752 tests, 0 warnings, 0 unsafe, 0 debt markers

---

## What Shipped

### 1. `spine.status` JSON-RPC Method (S6 Debt Item)

New observability endpoint — one call reports everything about a spine:
- **Structural**: spine_id, name, owner, state (Active/Sealed/Frozen/Archived), entry_count, tip_hash, genesis_hash, created_at, updated_at
- **Sessions**: All `SessionCommit` entries extracted with session_id, merkle_root, vertex_count, committer, committed_at timestamp, entry_index. Returned most-recent-first.

**Full-stack wiring:**
- API types: `SpineStatusRequest`, `SpineStatusResponse`, `SessionSummary`
- RPC handler: `spine_ops.rs` scans entries for `SessionCommit` variants
- JSON-RPC dispatch: `"spine.status" => rpc!(params, spine_status)`
- Niche: METHODS/SEMANTIC_MAPPINGS/COST_ESTIMATES (52→53)
- MCP: `spine_status` tool definition + tool-to-rpc mapping
- Capability registry: `status` added to spine domain

### 2. Broken Doc Link Fixes

- `append_entry_batch_with` — referenced nonexistent method. Rewritten to accurate description.
- `CertificateHistory::from_certificate_and_entries` — needed `crate::certificate::` prefix for rustdoc resolution.

### 3. Deep Debt Audit Confirmed Clean

| Dimension | Status |
|-----------|--------|
| unsafe | **ZERO** (forbid) |
| unwrap/expect (prod) | **ZERO** |
| TODO/FIXME/HACK | **ZERO** |
| dead_code suppression | **3** (all BTSP wire fields, justified) |
| `#[allow]` | **ZERO** (all `#[expect]` with reasons) |
| Hardcoding | Constants centralized, debug-only fallbacks |
| Mocks | All `cfg(test)`-gated |
| Blocking I/O | `spawn_blocking` used consistently |
| Files >800L (prod) | **ZERO** (largest: 670L `uds.rs`) |

---

## Verification

```
cargo fmt --all                 # clean
cargo clippy --workspace ...    # 0 warnings
cargo test --workspace          # 1,752 passed, 0 failed
cargo doc --workspace           # 0 warnings
```

---

## Files Touched

| File | Change |
|------|--------|
| `crates/loam-spine-api/src/types/mod.rs` | `SpineStatusRequest`, `SpineStatusResponse`, `SessionSummary` |
| `crates/loam-spine-api/src/service/spine_ops.rs` | `spine_status` handler |
| `crates/loam-spine-api/src/service/service_tests.rs` | 5 new tests |
| `crates/loam-spine-api/src/jsonrpc/mod.rs` | `spine.status` dispatch arm |
| `crates/loam-spine-core/src/niche.rs` | METHODS/SEMANTIC_MAPPINGS/COST_ESTIMATES 52→53 |
| `crates/loam-spine-core/src/neural_api/mcp.rs` | `spine_status` MCP tool + mapping |
| `crates/loam-spine-core/src/service/mod.rs` | Doc fix: `append_entry_batch_with` |
| `crates/loam-spine-core/src/service/certificate.rs` | Doc fix: `CertificateHistory` path |
| `config/capability_registry.toml` | `status` added to spine domain |
| Root docs (7 files) | Counts: 52→53 methods, 1,747→1,752 tests |

---

## Upstream Dependencies

None. This wave is internal feature work — all changes within loamSpine.

## Cross-Primal Impact

- **biomeOS**: Can now route `capability.call { domain: "spine", operation: "status" }` to loamSpine
- **squirrel**: `signal.dispatch` can target spine status queries via semantic mapping
- **sweetGrass**: `LedgerClient` can query spine status for observability dashboards
- **nestgate.io**: Dashboard can display per-spine health and session history

---

*Wave 156e — `spine.status` closes the S6 debt item from the overwatch punch list. loamSpine now has 53 JSON-RPC methods with full-stack wiring through every layer.*
