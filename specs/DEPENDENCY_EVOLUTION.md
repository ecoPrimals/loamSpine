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

## mdns 3.0 → mdns-sd 0.19 — **COMPLETE**

| Field | Detail |
|-------|--------|
| **Status** | **COMPLETE** (April 20, 2026) |
| **Former** | `mdns = "3.0"` + `async-std` + `futures-util` (feature-gated) |
| **Now** | `mdns-sd = "0.19"` — pure Rust, manages its own daemon thread, no async runtime dep |
| **Eliminated** | `async-std`, `net2`, `proc-macro-error`; 3 RUSTSEC advisories (2025-0052, 2020-0016, 2024-0370) removed from `deny.toml` |
| **API change** | `mdns::discover::all` → `ServiceDaemon::new()` + `browse()` + `recv_async().await`. No `block_on`, no `spawn_blocking`, no thread isolation needed. |
| **PG-33 fix** | The startup panic ("block_on inside async runtime") that blocked ludoSpring exp095 is structurally eliminated — `mdns-sd` never enters a tokio runtime context. |

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
