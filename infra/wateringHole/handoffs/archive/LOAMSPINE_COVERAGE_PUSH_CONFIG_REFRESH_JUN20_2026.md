<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# loamSpine — Coverage Push & Config Refresh

**Date**: June 20, 2026
**Author**: sporeGate Overwatch
**Supersedes**: LOAMSPINE_DEEP_DEBT_AUDIT_JUN19_2026, LOAMSPINE_FRAGO_WAVE114_JUN16_2026

---

## Summary

Coverage push targeting lowest-covered production files, deploy graph refresh,
capability registry evolution, and dependency analysis. All deps confirmed
pure Rust. Zero actionable debt remaining.

## Changes

### Coverage Push (+31 tests)

| File | Before | After | Tests Added |
|------|--------|-------|-------------|
| `btsp/provider_client.rs` | 46.85% | 64.96% | 4 (`parse_response` paths) |
| `btsp/frame.rs` | 56.32% | 66.67% | 5 (roundtrip, oversized read/write, ser/deser) |
| `proof.rs` | 68.50% | — | 9 (ProvenanceProof, CertificateProof, VerificationResult, HistorySummary) |
| `types.rs` | 82.96% | 88.34% | 13 (PeerId, Did anonymous, Signature base64, PayloadRef, Timestamp ordering) |

### Config Refresh

- **Deploy graph** (`graphs/loamspine_deploy.toml`): Updated to June 2026. Phase 5 operation: `lifecycle.register` → `primal.announce`. 47 methods in capability list (was incomplete). Trust domain added.
- **Capability registry** (`config/capability_registry.toml`): Bare `health` operation added. Health domain description updated.
- **benchScale** (`infra/benchScale/validate_roundtrip.sh`): Bare `health` validation added to Phase 1 Health Triad.

### Dependency Analysis

All 17 direct deps are pure Rust. Zero C dependencies in default build.
`cargo deny`: bans ok, licenses ok, sources ok.

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 1,652 |
| Line coverage | 92.26% |
| Branch coverage | 89.50% |
| Region coverage | 92.56% |
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
cargo test --workspace     → 1,652 passed, 0 failed
cargo doc --workspace      → PASS
cargo deny check           → bans ok, licenses ok, sources ok
```
