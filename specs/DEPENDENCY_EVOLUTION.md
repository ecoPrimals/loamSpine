<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Dependency Evolution Tracking

> **Status**: Active — tracks planned dependency migrations and their rationale.

## Principles

- **ecoBin compliance**: Pure Rust, no C dependencies in the default build
- **Minimal surface**: Only what we actually use
- **Security**: No unmaintained crates, audit with `cargo deny`

---

## bincode v1 → MessagePack (`rmp-serde`) — **COMPLETE**

| Field | Detail |
|-------|--------|
| **Status** | **COMPLETE** (April 16, 2026) |
| **Former plan** | Migrate to **bincode v2** (`Encode`/`Decode`) to address **RUSTSEC-2025-0141** |
| **Actual path** | Replaced **`bincode` v1** with **`rmp-serde`** (serde over **MessagePack**) for storage and backup serialization — **RUSTSEC-2025-0141** eliminated because **`bincode` is no longer in the affected usage path** |
| **Rationale** | MessagePack via serde keeps a single derive surface (`Serialize`/`Deserialize`) on stored types while avoiding bincode v1’s advisory; bincode v2 was not required to close the issue |
| **Format note** | On-disk bytes are **not** bincode v1-compatible after migration — existing DBs need a one-time migration or restore-from-backup if upgrading from pre-MsgPack snapshots |

Historical planning text for a bincode v2 migration is superseded by this completion record.

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

## sled → redb — **COMPLETE** (removed)

redb is the sole persistent storage backend (`default = ["redb-storage"]`). The `sled`
dependency and `sled-storage` feature were **fully removed** from `loam-spine-core` during
the Stadial Parity Gate (April 2026). `sled`, `instant`, and `fxhash` are no longer in
`Cargo.lock`. SQLite (`rusqlite`/`libsqlite3-sys`) was removed at the same time.

---

## tarpc 0.37 / opentelemetry advisory

| Field | Detail |
|-------|--------|
| **Current** | `tarpc = "0.37"` → transitive `tracing-opentelemetry` → `opentelemetry_sdk` |
| **Advisory** | RUSTSEC-2026-0007 (`opentelemetry_sdk`). tarpc 0.37 unconditionally pulls `tracing-opentelemetry` which depends on the affected `opentelemetry_sdk` version. |
| **Blocker** | Upstream tarpc must bump `tracing-opentelemetry` to a version that depends on a patched `opentelemetry_sdk`. No tarpc release yet addresses this. |
| **Risk** | Low for loamSpine — we do not enable OpenTelemetry export. The SDK is a transitive dependency, not actively invoked. Advisory is ignored in `deny.toml`. |
| **Mitigation** | Monitor tarpc releases. When tarpc 0.38+ ships with an updated `opentelemetry` stack, bump and remove the `deny.toml` ignore. |
| **Priority** | Low — waiting on upstream. |

---

## Dependency Audit Checklist

Run periodically:

```bash
cargo deny check advisories
cargo deny check licenses
cargo audit
```
