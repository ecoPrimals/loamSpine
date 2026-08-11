# Handoff: loamSpine Wave 157g — Deep Debt Audit + G72 + Gossip

**Date**: August 10, 2026  
**From**: sporeGate  
**To**: overwatch / eastGate / upstream  
**Wave**: 157g  
**HEAD**: `18f7239`

---

## Summary

Five waves shipped since last handoff (G66, Aug 6):

1. **G68 Platform Substrate** (L1 links + L2 permissions) — zero `PermissionsExt` / `std::os::unix::fs::symlink` outside platform layer
2. **Vertebrate Evolution Self-Audit** — 54/54 RPC methods verified, `persist_tip` helper (18 call sites), attestation IPC consolidated
3. **Gossip Injection** — `GossipEvent` enum (4 events), `GossipEmitter` (swarmVine mesh), fire-and-forget hooks at all persist points
4. **G72 Dependency Pandemic** — `url` crate excised, `chacha20poly1305` 0.10→0.11, RustCrypto stack unified
5. **Deep Debt Audit** — test file splits (all <800L), comprehensive hygiene audit

---

## Current Metrics

| Metric | Value |
|--------|-------|
| Tests | 1,820 |
| Source files | 218 `.rs` files (+ 3 fuzz targets) |
| JSON-RPC methods | 53 |
| tarpc methods | 37 |
| Largest source file | 753L (test file) |
| Unsafe code | Zero (`#![forbid(unsafe_code)]`) |
| TODOs/FIXMEs/HACKs | Zero |
| Production unwrap/expect | Zero |
| Production mocks | Zero (all `#[cfg(test)]`) |
| Clippy pedantic | Clean |

---

## Changes Since Last Handoff (G66 → 157g)

### G68 Platform Substrate (Wave 157a)

| Component | Change |
|-----------|--------|
| `platform::fs` | `create_link()` / `remove_link()` — abstracts symlinks (Unix/Windows/Other) |
| `platform::access` | `PlatformAccess` enum, `set_executable()`, `is_executable()` |
| `main.rs` | Replaced direct `std::os::unix::fs::symlink` with `platform::create_link` |
| `cli_signer_tests_integration.rs` | Replaced `PermissionsExt` with `platform::set_executable/is_executable` |

### Vertebrate Evolution Self-Audit (Wave 157a)

| Component | Change |
|-----------|--------|
| RPC audit | 54/54 JSON-RPC methods match `capability_registry.toml` |
| `persist_tip()` | New helper — save tip entry + save spine, applied in 18 call sites |
| `ndjson_rpc_call` | Attestation IPC consolidated to shared helper |
| `capability_registry.toml` | `domains.waypoint` → `domains.slice` naming fix |

### Gossip Injection (Wave 157e)

| Component | Change |
|-----------|--------|
| `gossip.rs` (305 LOC) | `GossipEvent` enum: `CasHave`, `BraidHead`, `SpineSealed`, `AnchorPublished` |
| `GossipEmitter` | Connects to swarmVine via `gossip.inject` JSON-RPC over UDS |
| Service hooks | `persist_tip` → CasHave, `seal_spine` → SpineSealed, `anchor_to_public_chain` → AnchorPublished, `commit_braid` → BraidHead |
| `AnchorTarget::chain_name()` | Human-readable chain names for gossip events |

### G72 Dependency Pandemic (Wave 157g)

| Component | Change |
|-----------|--------|
| `url` crate | Excised — manual port parsing replaces `url::Url::parse().port()` |
| `chacha20poly1305` | 0.10 → 0.11 — unifies `cpufeatures` + `crypto-common` |
| tokio features | Already lean (8 specific features, not `["full"]`) |

### Deep Debt Test Refactoring (Wave 157g)

| File | Before | After |
|------|--------|-------|
| `certificate_tests.rs` | 807L | 409L + `certificate_tests_provenance.rs` (398L) |
| `service_tests.rs` | 827L | 654L + `service_tests_spine_status.rs` (172L) |

---

## Verification

```bash
cargo clippy --workspace --all-targets -- -D warnings  # clean
cargo fmt --all -- --check                              # clean
cargo test --workspace                                  # 1,820 passing
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps  # clean
```

---

## Upstream Attention

- **bearDog P0-A** (depot binary health-only stub, unsigned spine commits): loamSpine's signing path is correct (`JsonRpcCryptoSigner` → `crypto.sign_ed25519` via capability-discovered UDS). Will work when bearDog ships the fix.
- **Remaining transitive duplicates**: `rand` v0.8/v0.9 (tarpc/proptest), `syn` v2/v3 (clap/serde). These require upstream crate updates — beyond loamSpine's control.

---

## What's Next

- Continue G72 Tier 2/3 as upstream deps align
- Monitor bearDog P0-A resolution for signing integration
- Await next ecosystem blurb for wave direction
