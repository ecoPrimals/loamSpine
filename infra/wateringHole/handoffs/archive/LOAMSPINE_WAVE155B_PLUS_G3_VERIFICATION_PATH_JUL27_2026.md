# loamSpine — Wave 155b+: G3 Verification Path + RPC Surface

**Date**: July 27, 2026  
**From**: loamSpine team (eastGate)  
**Wave**: 155b+  
**Status**: COMPLETE

---

## Summary

G3 MintingAuthority validation path evolution. `verify_certificate` upgraded
from storage-existence-only to semantic integrity checks. Two new JSON-RPC
methods (`certificate.verify`, `certificate.lifecycle`) expose verification
and provenance history over the wire. `MintInfo::with_authority` builder
prepares the delegated minting path for Nest Atomic Phase 0.

---

## Changes

### Semantic Certificate Verification

`verify_certificate` now performs 6 progressive checks (up from 4):

| # | Check | New? | What it validates |
|---|-------|------|-------------------|
| 1 | `Exists` | — | Certificate record in storage |
| 2 | `SpineExists` | — | Associated spine in storage |
| 3 | `MintEntryExists` | — | Mint entry hash resolves |
| 4 | `MintEntryValid` | **NEW** | Entry type is `CertificateMint` with matching `cert_id` |
| 5 | `OwnerConsistent` | **NEW** | Mint entry `initial_owner` matches certificate minter |
| 6 | `ChainValid` | evolved | Now requires all 5 checks + location entry |

`CertificateVerification` and `VerificationCheck` derive `Serialize`/`Deserialize`
for wire transport.

### New JSON-RPC Methods

| Method | Request | Response | Gate |
|--------|---------|----------|------|
| `certificate.verify` | `{ certificate_id }` | `{ exists, valid, checks_passed }` | Protected |
| `certificate.lifecycle` | `{ certificate_id }` | `{ count, entries }` | Protected |

### Delegated Minting Path

- `MintInfo::with_authority(MintingAuthority)` builder for Nest Atomic G3
- `certificate` module promoted to `pub mod` for cross-crate verification types

### Observability

- mDNS `let _ = daemon.shutdown()` (3 sites) evolved to `tracing::trace!` on error

### Tests

- 4 new tests: semantic verification checks (core), lifecycle ordering (core),
  verify RPC (API), lifecycle RPC (API)
- **Total: 1,736 tests**, 210 source files, all checks clean

---

## Remaining G3 Work

| Priority | Item | Status |
|----------|------|--------|
| P0 | Wire `MintingAuthority` validation into `mint_certificate` | Next wave |
| P0 | Trust ledger lookup for authority DID | Next wave |
| P1 | Route certificate entries through Tower signing | Next wave |
| P1 | Connect verify to `JsonRpcCryptoVerifier` | Next wave |
| P2 | Populate `CertificateHistory`/`OwnershipRecord` | Schema-ready |
| P2 | primalSpring G3 mint authority scenario | Coordinate |

---

## Verification

```
cargo fmt --all --check       # clean
cargo clippy -D warnings      # zero warnings
cargo test --workspace        # 1,736 passed, 0 failed
cargo doc --workspace -D warnings  # clean
```

---

## Files Changed

| File | Change |
|------|--------|
| `loam-spine-core/src/service/certificate.rs` | Semantic verify, `Serialize`/`Deserialize` derives |
| `loam-spine-core/src/service/mod.rs` | `certificate` module → `pub mod` |
| `loam-spine-core/src/certificate/provenance.rs` | `MintInfo::with_authority` builder |
| `loam-spine-core/src/service/certificate_tests.rs` | 2 new tests |
| `loam-spine-core/src/infant_discovery/backends.rs` | mDNS shutdown tracing |
| `loam-spine-core/src/service/infant_discovery.rs` | mDNS shutdown tracing |
| `loam-spine-api/src/types/certificate.rs` | Verify + lifecycle request/response types |
| `loam-spine-api/src/service/certificate_ops.rs` | verify + lifecycle RPC handlers |
| `loam-spine-api/src/jsonrpc/mod.rs` | Method dispatch for `certificate.verify`, `certificate.lifecycle` |
| `loam-spine-api/src/service/service_tests.rs` | 2 new API tests |
