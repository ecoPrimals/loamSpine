<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Known Issues

**Last Updated**: March 15, 2026

---

## Coverage

| Area | Issue | Impact |
|------|-------|--------|
| `bin/loamspine-service/main.rs` | 0% coverage (150 lines). Binary entry point with Tokio runtime setup, signal handling, and server orchestration. Inherently difficult to unit test. | Low — thin orchestration layer; all called components have >85% coverage. |
| DNS SRV / mDNS discovery | Network-dependent paths have limited testability in CI without real DNS infrastructure. | Low — core discovery logic tested via mocks; network transport tested with `ConfigurableTransport`. |

---

## Dependencies

| Dependency | Issue | Mitigation |
|------------|-------|------------|
| `libsqlite3-sys` | C dependency, but only compiled when `sqlite` feature is enabled. Not part of default build. | Feature-gated. Default storage is pure-Rust `redb`. |
| `async-channel` | Minor duplicate versions pulled in via `mdns` feature dependency tree. | Cosmetic — no functional impact. Will resolve when upstream updates. |

---

## Architecture

| Area | Issue | Notes |
|------|-------|-------|
| Attestation runtime wiring | `AttestationRequirement` types defined and integrated into `WaypointConfig`, but actual runtime enforcement (checking attestation before waypoint operations) is not yet wired into the operation flow. | Types and framework ready; enforcement is a v0.9.0 target. |
| PostgreSQL / RocksDB backends | Specified in `STORAGE_BACKENDS.md` but not yet implemented. | v1.0.0 target. Memory, redb (default), sled, and SQLite backends are complete. |

---

## Edition 2024

| Area | Issue | Notes |
|------|-------|-------|
| `unsafe_code` lint | Changed from `forbid` to `deny` to allow `#[allow(unsafe_code)]` in test modules. Edition 2024 makes `env::set_var`/`remove_var` `unsafe`. | Production code remains protected — `deny` still errors on any `unsafe` in non-test code. Most tests migrated to `temp-env` crate (v0.8.9); remaining `unsafe` env mutations are in async tests with mock servers (tokio runtime incompatible with temp-env closures). |
| Dockerfile MSRV | Updated to `rust:1.85`. Edition 2024 requires Rust 1.85+. | CI MSRV job also updated. |
| `/proc/self/status` UID | 5-tier socket discovery reads UID from `/proc/self/status` — Linux-only. Falls through to `temp_dir()` on non-Linux. | Graceful degradation; only applies when XDG_RUNTIME_DIR is unset. |

---

*See [STATUS.md](STATUS.md) for implementation progress and [WHATS_NEXT.md](WHATS_NEXT.md) for the roadmap.*
