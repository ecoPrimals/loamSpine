<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Known Issues

**Last Updated**: August 15, 2026

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
| `rand` v0.8 + v0.9 | Duplicate versions pulled by `tarpc` (v0.8) and `proptest` (v0.9). | Transitive — requires upstream crate updates. No impact on binary size or API. |
| `syn` v2 + v3 | Duplicate versions pulled by `clap` (v2) and `serde` (v3). | Compile-time only (proc-macro). No runtime impact. |

---

## Architecture

| Area | Issue | Notes |
|------|-------|-------|
| PostgreSQL / RocksDB backends | Specified in `STORAGE_BACKENDS.md` but not yet implemented. | v1.0.0 target. Memory and redb (default) are implemented; sled and SQLite were removed for stadial compliance. |
| blake3 SIMD performance | Switched to `pure` Rust mode (no C/asm) for ecoBin compliance. ~2-3x slower hashing vs SIMD, acceptable for LoamSpine's workload. | Can be feature-gated back to SIMD if needed. |
| BTSP encrypted tunnels | Per-session Phase 3 encrypted framing is implemented (ChaCha20-Poly1305). Persistent tunnel-mode ledger replication (long-lived encrypted channels) remains future work. | Same status across all primals. |
| bearDog P0-A | Depot binary is health-only stub; spine commits unsigned. | loamSpine's signing path is correct (`JsonRpcCryptoSigner` → capability-discovered UDS). Will work when bearDog ships the fix. |

---

## Platform

| Area | Issue | Notes |
|------|-------|-------|
| `/proc/self/status` UID | 5-tier socket discovery reads UID from `/proc/self/status` — Linux-only. Falls through to `temp_dir()` on non-Linux. | Graceful degradation; only applies when `XDG_RUNTIME_DIR` is unset. |
| Windows GNU | UDS JSON-RPC server, BTSP `ProviderConn`, NeuralAPI registration, `crypto_provider_call`, PID file, capability symlinks are `#[cfg(unix)]`-gated. Non-Unix builds return stub errors for IPC-dependent paths. | `cargo check --target x86_64-pc-windows-gnu` clean. Named Pipe adoption is future work. |

---

*See [STATUS.md](STATUS.md) for implementation progress and [WHATS_NEXT.md](WHATS_NEXT.md) for the roadmap.*
