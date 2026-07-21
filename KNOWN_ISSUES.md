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
| `bin/loamspine-service/main.rs` | Integration tests added (CLI parsing, subcommands, server start/shutdown). Remaining untested: error-recovery paths in `run_server`. | Low — main paths covered; error recovery is thin orchestration. |
| DNS SRV / mDNS-SD discovery | Network-dependent paths have limited testability in CI without real DNS infrastructure. mDNS-SD uses `mdns-sd` 0.19 (pure Rust daemon thread, no async-std; PG-33 structurally eliminated). | Low — core discovery logic tested via mocks; network transport tested with `ConfigurableTransport`. |

---

## Dependencies

| Dependency | Issue | Mitigation |
|------------|-------|------------|
| `opentelemetry_sdk` | RUSTSEC-2026-0007. Hard dep of tarpc 0.37 (not feature-gated). | **RESOLVED** — advisory no longer detected by `cargo deny check advisories`; ignore removed from `deny.toml`. |
| `hickory-net` (via `hickory-resolver` 0.26) | Pulls `async-trait` (stadial ghost; upstream debt). Only present when `dns-srv` feature is enabled (opt-in since Wave 60). Default build has zero `hickory` deps. | Non-blocking; `dns-srv` opt-in eliminates this from default builds. |
| `ring` | Lockfile artifact via `hickory-proto` 0.26 optional `dnssec-ring` feature. Only appears when `dns-srv` feature is enabled (opt-in since Wave 60). Default build has zero `ring` deps. `ring` is explicitly banned in `deny.toml`. | **Eliminated from default build.** Only enters via opt-in `dns-srv` feature. |
| `mdns` 3.0 → `mdns-sd` 0.19 | **RESOLVED** — migrated to `mdns-sd` 0.19 (pure Rust, no async runtime dep). `async-std`, `net2`, `proc-macro-error` all eliminated. 3 RUSTSEC advisories removed from `deny.toml`. | N/A |

---

## Architecture

| Area | Issue | Notes |
|------|-------|-------|
| PostgreSQL / RocksDB backends | Specified in `STORAGE_BACKENDS.md` but not yet implemented. | v1.0.0 target. Memory and redb (default) are implemented; sled and SQLite were removed for stadial compliance. |
| blake3 SIMD performance | Switched to `pure` Rust mode (no C/asm) for ecoBin compliance. Performance impact is ~2-3x slower hashing vs. SIMD, acceptable for LoamSpine's workload. | Can be feature-gated back to SIMD if performance-critical deployment needs it. |
| BTSP Phase 3 — encrypted framing | **IMPLEMENTED (FULL + TRANSPORT VERIFIED)** — `btsp.negotiate` returns `cipher: "chacha20-poly1305"` + base64 server nonce when Tower-provided handshake key is available (Pattern B from `CRYPTO_CONSUMPTION_HIERARCHY.md`). `SessionKeys` derived via HKDF-SHA256, encrypted framing via ChaCha20-Poly1305 AEAD with `[4B len][12B nonce][ciphertext + tag]` wire format. **Transport switch verified**: after negotiate, the UDS accept loop enters `handle_encrypted_stream` using `read_encrypted_frame`/`write_encrypted_frame` for all subsequent messages on that connection. Falls back to `cipher: "null"` for unauthenticated covalent bonds. 4 transport switch integration tests confirm encrypted round-trip, multi-message, null-fallback, and no-key-plaintext paths. |
| BTSP challenge generation | **RESOLVED (v0.9.16)** — `generate_challenge()` removed; the BTSP provider is the sole challenge authority via `btsp.session.create`. LoamSpine sends `family_seed` and receives the challenge in the response. |
| Computation provenance receipts | **IMPLEMENTED** — `CommitSessionResponse` is a self-contained provenance receipt: ledger anchor (`spine_id`, `commit_hash`, `index`, `committed_at`) + session binding (`session_id`, `merkle_root`, `vertex_count`, `committer`) + optional `tower_signature` (base64 Ed25519, present when `TOWER_SIGNER_SOCKET` is set). `get_provenance_chain()` now matches `SessionCommit` entries on `merkle_root` (relationship: `committed-from`). Downstream consumers (guideStone, composition scripts) can trace DAG-to-ledger provenance from the receipt alone without follow-up entry fetches. `TrioCommitReceipt` (`trio_types.rs`) is an orchestration-level type with the same fields. |
| PG-52 UDS trio empty responses | **RESOLVED (v0.9.16)** — `spine.create`, `entry.append`, `spine.seal` all work correctly over UDS JSON-RPC (with and without BTSP config). Root cause: stale plasmidBin binary + double-`BufReader` on post-BTSP path (now cleaned up). 3 UDS transport integration tests added. plasmidBin rebuild required. |
| Tower signing of ledger entries | **IMPLEMENTED (v0.9.16)** — `entry.append` and `session.commit` sign entries via tower signer `crypto.sign_ed25519` when `TOWER_SIGNER_SOCKET` is set (deprecated env fallback preserved). Signature stored in entry metadata (`tower_signature`, `tower_signature_alg`). Standalone mode produces unsigned entries (backward-compatible). |
| BTSP encrypted tunnels | loamSpine now implements Phase 3 encrypted framing (ChaCha20-Poly1305). Persistent BTSP tunnel-mode ledger replication (long-lived encrypted channels beyond per-session negotiate) remains a future evolution target — same as all other primals. |
| Hex string acceptance (Gap 9) | **RESOLVED (v0.9.16)** — All `ContentHash`/`EntryHash` fields (`[u8; 32]`) accept both JSON byte arrays and 64-char hex strings. `AppendEntryRequest.committer` made `Option<Did>` (was required but unused). 14 new tests. |

---

## Resolved Upstream Audit Items

| Audit Item | Resolution |
|------------|------------|
| **Tokio runtime-in-runtime panic on health probe** | **NOT REPRODUCIBLE / ALREADY FIXED.** Health handlers (`liveness`, `readiness`, `check`) are simple async methods — zero `Runtime::new()`, zero `block_on()`, zero infant discovery on the request path. The original nested-runtime bug was LS-03 (v0.9.15): `block_on()` in infant discovery at *startup*, not health probes. Hardened in v0.9.16 via `mdns-sd` migration (PG-33). Wave 47 documented a reported "Tokio double-runtime crash" as misdiagnosed — actual cause was `serve` vs `server` CLI mismatch in plasmidBin launcher. Production code has **zero** `Runtime::new()` or `block_on()` calls. See `LOAMSPINE_WAVE47_BEHAVIORAL_CONVERGENCE_HANDOFF_MAY24_2026.md`. |
| **benchScale uses source-build + PATH** | **RESOLVED (Wave 49).** `validate_roundtrip.sh` now checks `LOAMSPINE_BINARY` env first, then `target/release`, `target/debug`, then `command -v loamspine` (PATH / plasmidBin). Build step skippable via `SKIP_BUILD=1`. |
| **wateringHole flat handoffs** | **RESOLVED (Wave 49).** `archive/` subdir created, superseded handoffs moved. Active handoffs remain in `infra/wateringHole/handoffs/`. |

---

## Platform

| Area | Issue | Notes |
|------|-------|-------|
| `/proc/self/status` UID | 5-tier socket discovery reads UID from `/proc/self/status` — Linux-only. Falls through to `temp_dir()` on non-Linux. | Graceful degradation; only applies when XDG_RUNTIME_DIR is unset. |
| Windows GNU | UDS JSON-RPC server, BTSP `ProviderConn`, NeuralAPI registration, `crypto_provider_call`, PID file, capability symlinks are `#[cfg(unix)]`-gated. Non-Unix builds return stub errors for IPC-dependent paths. | `cargo check --target x86_64-pc-windows-gnu` clean. Named Pipe adoption is future work. |

---

*See [STATUS.md](STATUS.md) for implementation progress and [WHATS_NEXT.md](WHATS_NEXT.md) for the roadmap.*
