<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# Known Issues

**Last Updated**: March 16, 2026

---

## Coverage

| Area | Issue | Impact |
|------|-------|--------|
| `bin/loamspine-service/main.rs` | Integration tests added (CLI parsing, subcommands, server start/shutdown). Remaining untested: error-recovery paths in `run_server`. | Low — main paths covered; error recovery is thin orchestration. |
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
| Attestation runtime wiring | **RESOLVED in v0.9.1** — `check_attestation_requirement()` wired into all waypoint operations. `DiscoveredAttestationProvider` sends JSON-RPC `attestation.request` to capability-discovered endpoints; degrades gracefully when unreachable. | |
| PostgreSQL / RocksDB backends | Specified in `STORAGE_BACKENDS.md` but not yet implemented. | v1.0.0 target. Memory, redb (default), sled, and SQLite backends are complete. |
| blake3 SIMD performance | Switched to `pure` Rust mode (no C/asm) for ecoBin compliance. Performance impact is ~2-3x slower hashing vs. SIMD, acceptable for LoamSpine's workload. | Can be feature-gated back to SIMD if performance-critical deployment needs it. |

---

## Edition 2024

| Area | Issue | Notes |
|------|-------|-------|
| `unsafe_code` lint | Changed from `forbid` to `deny` to allow `#[expect(unsafe_code)]` in test modules. Edition 2024 makes `env::set_var`/`remove_var` `unsafe`. | Production code remains protected — `deny` still errors on any `unsafe` in non-test code. Most tests migrated to `temp-env` crate; `lifecycle.rs` evolved to `temp_env::with_var_unset` + manual runtime in v0.9.2. Remaining `unsafe` env mutations in `infant_discovery/tests*.rs` use `#[expect(unsafe_code)]` — these require multiple sequential env changes with awaits between them (temp-env cannot wrap per-await mutation). |
| Dockerfile MSRV | Updated to `rust:1.85`. Edition 2024 requires Rust 1.85+. | CI MSRV job also updated. |
| `/proc/self/status` UID | 5-tier socket discovery reads UID from `/proc/self/status` — Linux-only. Falls through to `temp_dir()` on non-Linux. | Graceful degradation; only applies when XDG_RUNTIME_DIR is unset. |

---

*See [STATUS.md](STATUS.md) for implementation progress and [WHATS_NEXT.md](WHATS_NEXT.md) for the roadmap.*
