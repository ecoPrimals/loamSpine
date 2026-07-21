<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Known Issues

**Last Updated**: July 21, 2026

---

## Testing

The full workspace test suite runs **fully concurrent** (no `#[serial]`; no dependency on serialized execution or process-wide env mutexes for correctness).

---

## Coverage

| Area | Issue | Impact |
|------|-------|--------|
| `bin/loamspine-service/main.rs` | Integration tests added (CLI parsing, subcommands). Remaining untested: error-recovery paths in `run_server`. | Low — main paths covered; error recovery is thin orchestration. |
| DNS SRV / mDNS-SD discovery | Network-dependent paths have limited testability in CI without real DNS infrastructure. `mdns-sd` 0.19 (pure Rust, no async-std). | Low — core logic tested via mocks; network transport tested with `ConfigurableTransport`. |

---

## Dependencies

| Dependency | Issue | Mitigation |
|------------|-------|------------|
| `hickory-net` (via `hickory-resolver` 0.26) | Pulls `async-trait`. Only present when `dns-srv` feature is enabled (opt-in). Default build has zero `hickory` deps. | Non-blocking; `dns-srv` opt-in eliminates this from default builds. |

---

## Architecture

| Area | Issue | Notes |
|------|-------|-------|
| PostgreSQL / RocksDB backends | Specified in `STORAGE_BACKENDS.md` but not yet implemented. | v1.0.0 target. Memory and redb (default) are implemented; sled and SQLite were removed for stadial compliance. |
| blake3 SIMD performance | Switched to `pure` Rust mode (no C/asm) for ecoBin compliance. ~2-3x slower hashing vs SIMD, acceptable for LoamSpine's workload. | Can be feature-gated back to SIMD if needed. |
| BTSP encrypted tunnels | Per-session Phase 3 encrypted framing is implemented (ChaCha20-Poly1305). Persistent tunnel-mode ledger replication (long-lived encrypted channels) remains future work. | Same status across all primals. |

---

## Platform

| Area | Issue | Notes |
|------|-------|-------|
| `/proc/self/status` UID | 5-tier socket discovery reads UID from `/proc/self/status` — Linux-only. Falls through to `temp_dir()` on non-Linux. | Graceful degradation; only applies when `XDG_RUNTIME_DIR` is unset. |
| Windows GNU | UDS JSON-RPC server, BTSP `ProviderConn`, NeuralAPI registration, `crypto_provider_call`, PID file, capability symlinks are `#[cfg(unix)]`-gated. Non-Unix builds return stub errors for IPC-dependent paths. | `cargo check --target x86_64-pc-windows-gnu` clean. Named Pipe adoption is future work. |

---

*See [STATUS.md](STATUS.md) for implementation progress and [WHATS_NEXT.md](WHATS_NEXT.md) for the roadmap.*
