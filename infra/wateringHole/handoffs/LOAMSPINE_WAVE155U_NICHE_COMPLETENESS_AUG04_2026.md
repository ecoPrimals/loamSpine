# loamSpine Wave 155u — Niche Completeness + MCP Batch Tools

**Date**: August 4, 2026  
**From**: sporeGate  
**Wave**: 155u  
**Status**: GREEN — 1,747 tests, 0 warnings, 0 unsafe, 0 debt markers

---

## What Shipped

### 1. Semantic Mapping Completeness (33 → 52)

Every method in `METHODS` now has an orchestrator-routable semantic mapping in `SEMANTIC_MAPPINGS`. Previously 19 methods were missing mappings, meaning `capability.call { domain, operation }` would silently fail for those operations.

**Added mappings:**
- Batch ops: `append_entry_batch` → `entry.append_batch`, `mint_certificate_batch` → `certificate.mint_batch`
- Certificate introspection: `verify_certificate`, `certificate_lifecycle`, `certificate_history`
- Listing: `list_spines` → `spine.list`, `list_entries` → `entry.list`
- Infrastructure probes: `health_liveness`, `health_readiness`, `lifecycle_status`
- Auth introspection: `auth_check`, `auth_mode`, `auth_peer_info`
- BTSP: `btsp_negotiate`, `btsp_capabilities`
- Permanence compat: 4 legacy naming methods

### 2. Cost Estimate Completeness (32 → 52)

Every method now has scheduling hints for the ecosystem orchestrator. Key additions:
- `entry.append_batch`: 10ms (amortized batch I/O)
- `certificate.mint_batch`: 15ms (amortized batch I/O)
- `session.dehydrate`: 3ms
- All infrastructure probes: 1ms
- All auth introspection: 1ms

### 3. MCP Batch Tool Exposure

Added `entry_append_batch` and `certificate_mint_batch` MCP tool definitions to `mcp_tools_list()`. AI agents can now discover and invoke batch operations via `tools/list` → `tools/call`.

### 4. Doc Alignment

| Doc | Change |
|-----|--------|
| README.md | Badge 48 → 52 methods |
| STATUS.md | Header July 28 → Aug 4, method counts aligned to 52 |
| CONTEXT.md | Method list includes batch + cert introspection, count 48 → 52 |
| CHANGELOG.md | Wave 155u entry added |
| WHATS_NEXT.md | Header July 27 → Aug 4, Wave 155u entry |
| KNOWN_ISSUES.md | Date July 26 → Aug 4 |
| sporeprint/validation-summary.md | Full refresh: 1,747 tests, 52 methods, G31 section |

### 5. Housekeeping

- `primal-capabilities.toml` annotated with canonical registry reference (`config/capability_registry.toml`)
- 3 pre-155n handoffs archived (155b, 155b+, 155d) → 47 total in archive
- Forgejo remote URL fixed to port 2222 (was defaulting to port 22)
- 2 pending commits from Wave 155n successfully pushed to golgiBody

---

## Verification

```
cargo fmt --all                 # clean
cargo clippy --workspace ...    # 0 warnings
cargo test --workspace          # 1,747 passed, 0 failed
cargo doc --workspace           # 0 warnings
```

---

## Files Touched

| File | Change |
|------|--------|
| `crates/loam-spine-core/src/niche.rs` | SEMANTIC_MAPPINGS 33→52, COST_ESTIMATES 32→52 |
| `crates/loam-spine-core/src/neural_api/mcp.rs` | 2 batch MCP tool definitions |
| `README.md` | Badge 48→52 |
| `STATUS.md` | Date, method counts, Wave 155u section |
| `CONTEXT.md` | Method list + count |
| `CHANGELOG.md` | Wave 155u entry |
| `WHATS_NEXT.md` | Date, Wave 155u entry |
| `KNOWN_ISSUES.md` | Date |
| `sporeprint/validation-summary.md` | Full refresh |
| `primal-capabilities.toml` | Canonical registry note |

---

## Upstream Dependencies

None. This wave is internal completeness — all changes are within loamSpine.

## Cross-Primal Impact

- **biomeOS**: Can now route `capability.call` to all 52 loamSpine methods (was missing 19)
- **squirrel**: `signal.dispatch` can target batch operations via semantic mappings
- **rhizoCrypt/sweetGrass**: Batch pipeline callers now have MCP tool exposure for AI-assisted orchestration

---

*Wave 155u — niche completeness closes the last gap between loamSpine's actual method surface and its orchestrator-visible capabilities.*
