<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Dependency Evolution Tracking

> **Status**: Active — tracks planned dependency migrations and their rationale.

## Principles

- **ecoBin compliance**: Pure Rust, no C dependencies in the default build
- **Minimal surface**: Only what we actually use
- **Security**: No unmaintained crates, audit with `cargo deny`

---

## bincode v1.3 → v2

| Field | Detail |
|-------|--------|
| **Current** | `bincode = "1.3"` (via serde `Serialize`/`Deserialize`) |
| **Target** | `bincode = "2.x"` with native `Encode`/`Decode` derive macros |
| **Blocker** | All storage backends (redb, sled, sqlite) use `bincode::serialize`/`deserialize`. Migration requires adding `#[derive(bincode::Encode, bincode::Decode)]` to all stored types (`Spine`, `Entry`, `Certificate`, etc.) and updating all call sites. |
| **Risk** | Storage format is not wire-compatible between v1 and v2. Requires a migration strategy for existing databases or a format version header. |
| **Benefit** | ~30% faster serialization, no serde dependency for storage path, smaller binary. bincode v2 also fixes RUSTSEC-2025-0141 (length-prefix confusion). |
| **Priority** | Medium. Current v1 usage is safe for embedded use (trusted data). |

### Migration Plan

1. Add `bincode2 = { package = "bincode", version = "2" }` alongside v1
2. Add `#[derive(bincode::Encode, bincode::Decode)]` to core types
3. Implement format-version header in storage (1 byte prefix)
4. Write migration tool: read v1, write v2
5. Swap default storage format to v2
6. Remove bincode v1 after one release cycle

---

## mdns crate → tokio-native alternative

| Field | Detail |
|-------|--------|
| **Current** | `mdns = "3.0"` + `async-std` + `futures-util` (feature-gated behind `mdns`) |
| **Problem** | Pulls in `async-std` — an entire second async runtime alongside tokio. Violates ecoBin "minimal surface" and adds ~200KB to binary. |
| **Target** | `mdns-sd` crate (pure Rust, tokio-native) or `hickory-resolver` mDNS support |
| **Blocker** | `mdns-sd` API differs significantly. Need to rewrite `try_mdns_discovery()` in `infant_discovery.rs`. |
| **Priority** | Low. Feature is experimental and behind a non-default feature flag. |

### Migration Plan

1. Evaluate `mdns-sd` (tokio-native, actively maintained)
2. Rewrite `try_mdns_discovery()` to use new API
3. Remove `async-std` and `futures-util` optional deps
4. Update feature flag: `mdns = ["dep:mdns-sd"]`

---

## sled → redb (completed)

redb is now the default storage backend (`default = ["redb-storage"]`). sled remains
available behind the `sled-storage` feature flag for backward compatibility. sled's
maintenance status is uncertain; redb is actively maintained and pure Rust.

**Action**: No immediate change needed. sled will be deprecated in a future release
if its maintenance situation does not improve.

---

## Dependency Audit Checklist

Run periodically:

```bash
cargo deny check advisories
cargo deny check licenses
cargo audit
```
