<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Known Issues

**Last Updated**: April 15, 2026

---

## Testing

The full workspace test suite runs **fully concurrent** (no `#[serial]`; no dependency on serialized execution or process-wide env mutexes for correctness).

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
| `libsqlite3-sys` | C dependency, compiled when `sqlite` feature enabled. Not part of default build. | Feature-gated. `deny.toml` allows only via `rusqlite` wrapper. Default storage is pure-Rust `redb`. |
| `bincode` v1 | RUSTSEC-2025-0141. Direct dep for storage/backup serialization. | tarpc tokio-serde path eliminated via feature trimming (v0.9.7). Direct usage deep in storage layer — migration to v2 tracked in `specs/DEPENDENCY_EVOLUTION.md` (storage format breaking change). |
| `opentelemetry_sdk` | RUSTSEC-2026-0007. Hard dep of tarpc 0.37 (not feature-gated). | Tracked in `deny.toml`; awaiting upstream tarpc resolution. |
| `sled` | Pulls `fxhash` (RUSTSEC-2025-0057), `instant` (RUSTSEC-2024-0384) via old `parking_lot`. | Optional feature only (`sled-storage`). Default is `redb`. |
| `mdns` 3.0 | Pulls discontinued `async-std`, deprecated `net2`, unmaintained `proc-macro-error`. | Optional feature only. All three advisories tracked in `deny.toml`. Evolution to `mdns-sd` or `hickory-resolver` mDNS tracked in `specs/DEPENDENCY_EVOLUTION.md`. |
| `async-channel` | Minor duplicate versions via `mdns` → `async-std`. | Cosmetic — no functional impact. |

---

## Architecture

| Area | Issue | Notes |
|------|-------|-------|
| PostgreSQL / RocksDB backends | Specified in `STORAGE_BACKENDS.md` but not yet implemented. | v1.0.0 target. Memory, redb (default), sled, and SQLite backends are complete. |
| blake3 SIMD performance | Switched to `pure` Rust mode (no C/asm) for ecoBin compliance. Performance impact is ~2-3x slower hashing vs. SIMD, acceptable for LoamSpine's workload. | Can be feature-gated back to SIMD if performance-critical deployment needs it. |
| BTSP Phase 2 — `BTSP_NULL` cipher only | BTSP handshake authenticates connections but `BTSP_NULL` is the only functional cipher. Encrypted framing (ChaCha20-Poly1305, HMAC-Plain) requires the BTSP provider's session key propagation, which is Phase 3. | Authentication is complete. Encryption is the BTSP provider's responsibility — LoamSpine will adopt encrypted framing when the provider exposes `btsp.encrypt`/`btsp.decrypt` over session keys. |
| BTSP challenge generation | **RESOLVED (v0.9.16)** — `generate_challenge()` now uses `blake3(uuid_v7_a \|\| uuid_v7_b)` with 148+ bits of OS-sourced entropy. The BTSP provider's `btsp.session.create` remains the authority for session key material. |

---

## Platform

| Area | Issue | Notes |
|------|-------|-------|
| `/proc/self/status` UID | 5-tier socket discovery reads UID from `/proc/self/status` — Linux-only. Falls through to `temp_dir()` on non-Linux. | Graceful degradation; only applies when XDG_RUNTIME_DIR is unset. |

---

*See [STATUS.md](STATUS.md) for implementation progress and [WHATS_NEXT.md](WHATS_NEXT.md) for the roadmap.*
