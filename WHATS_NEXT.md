<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Development Roadmap

**Current Version**: 0.9.16  
**Last Updated**: July 27, 2026

---

## Recent Changes

- **August 3, 2026** — **Wave 155n: G31 Batch Provenance Pipeline**: `entry.append_batch` and `certificate.mint_batch` JSON-RPC methods for 10× faster bulk ingestion. Amortizes spine I/O (1 read + 1 write vs N+N). CLI `--bind` alias. 1,747 tests, 50 JSON-RPC methods.

- **July 29, 2026** — **Wave 155i: CLI Standardization + Deep Audit**: `--bind` alias for `--bind-address` (biomeOS P2). Deep audit: zero production unwrap/expect, zero TODOs, zero `#[allow]` in non-test code. 1,740 tests, 211 source files.

- **July 28, 2026** — **Wave 155f: Structural Extraction + Schema Evolution + BTSP Dedup**: Entry module extraction (648L → 227L + 437L). Schema orphans populated: `CertificateHistory::from_certificate_and_entries()`. New `certificate.history` RPC (48 methods). BTSP handshake dedup: `verify_and_negotiate()` + `AsyncErrorSender` trait consolidates both framing modes. Deep audit confirmed zero production unwrap/expect, zero TODOs. 1,739 tests, 211 source files.

- **July 27, 2026** — **Wave 155b+: G3 Verification Path + RPC Surface**: `verify_certificate` evolved from storage-existence to semantic checks: `MintEntryValid` (entry type + cert_id match) and `OwnerConsistent` (initial_owner matches minter). New JSON-RPC methods: `certificate.verify`, `certificate.lifecycle`. `MintInfo::with_authority` builder for delegated minting path. `CertificateVerification` + `VerificationCheck` now `Serialize`/`Deserialize` for wire transport. `certificate` module promoted to `pub` for cross-crate verification types. mDNS `let _ = daemon.shutdown()` evolved to traced errors. 4 new tests. 1,736 tests, 210 source files.

- **July 27, 2026** — **Wave 155b: G3 Readiness + Doc Accuracy**: Full certificate lifecycle E2E test with seal (mint → loan → return → transfer → seal → reject-on-sealed). Dead_code reasons updated from "strandGate deploy" to "Nest Atomic Phase 0 (G3)". Inline `Duration::from_secs(5)` extracted to `HEALTH_PROBE_TIMEOUT` and `DEFAULT_IPC_TIMEOUT` constants. Doc metrics corrected: "52-validation" → "20-phase, 44-method", `#[allow]` claim corrected, dead_code count aligned. 1,732 tests, 210 source files.

- **July 26, 2026** — **Wave 151c: TransportEndpoint Compliance + Deep Debt**: Evolved `sync/mod.rs` and `discovery/mod.rs` from raw `TcpStream::connect` to `connect_transport(&TransportEndpoint)` — all outbound IPC now routes through platform-abstracted transport. `endpoint_from_addr()` parser added. Error swallowing fixed: discovery capability failures now log at debug level, progress channel drops traced. `set_nodelay` failures traced instead of silently ignored. Dead `let _ = &request_line` borrow removed from HTTP server. 11 new tests: `generate_aggregate_proof` coverage (5), `read_ndjson_stream_bounded` (2), `endpoint_from_addr` parsing (4). 1,731 tests, 210 source files.

- **July 26, 2026** — **Wave 151b: BTSP ClientHello Handshake**: Implemented BTSP client-side 4-step handshake following songBird reference. Local HMAC-SHA256 challenge-response with family seed. Wired into `crypto_provider_call` (Tower signer) and `ProviderConn` (BTSP provider relay). `hmac 0.13` added. 5 new integration tests (mock bearDog server). Seed resolution aligned to songBird standard (`FAMILY_SEED` first). 1,723 tests, 210 source files.

- **July 21, 2026** — **Wave 150t: Health Probe Honesty + Entry Path Coverage**: `readiness()` and `health_check()` evolved from hardcoded `ready: true`/`Healthy` to honest storage probes with 5-second timeouts — returns `ready: false` / `Unhealthy` on storage lock timeout. 5 new tests for `prepare_entry`/`append_prepared_entry` error paths (tower-signing delegation). 4 new health probe tests. Stale `ring` comment in Cargo.toml corrected (`hickory-resolver 0.26` is pure Rust). 1,711 tests, 208 source files.

- **July 18, 2026** — **Wave 149b: Dimensional Self-Audit + Test File Splits**: Self-audit at Wave 149b standard (all 10 dimensions). GAP-036 socket naming PASS, GAP-038 stale UDS cleanup PASS, 0 prod unwrap/expect, 0 debt, 0 unsafe, 0 `#[allow]`. `chaos.rs` split (783L → 2 modules). `lifecycle_tests.rs` split (779L → 2 modules). `#![forbid(unsafe_code)]` on all 3 fuzz targets. 1,702 tests, 208 source files.

- **July 16, 2026** — **Wave 143b: Transport Endpoint Functional Wiring**: `TRANSPORT_ENDPOINT` env wired to functional dispatch in `main.rs`. `service_tests.rs` split (789L → 3 modules). 7 new framing edge-case tests. 1,702 tests.

- **July 16, 2026** — **Wave 142b: Silicon Atheism Phase 2**: `TransportStream` enum + `connect_transport()` dispatch. All outbound IPC clients migrated from raw `#[cfg(unix)] UnixStream` to `TransportEndpoint`. `base64` crate migration. `spawn_blocking` async fs hygiene. 1,697 tests.

- **July 15, 2026** — **Wave 141a: Cross-Architecture Adoption**: All Unix-specific IPC gated behind `#[cfg(unix)]`. `cargo check --target x86_64-pc-windows-gnu` clean. `integration_tests.rs` split (1002L → 3 modules). 1,684 tests.

*For complete historical changelog, see [CHANGELOG.md](CHANGELOG.md).*

---

## v0.10.0 Targets

- **Signing capability middleware** — Signature verification on RPC layer (capability-discovered)
- **Collision layer validation** — neuralSpring experiments (Python baseline). See `specs/COLLISION_LAYER_ARCHITECTURE.md`

---

## v1.0.0 Targets

- **PostgreSQL storage backend** — Roadmap item. Current: redb (default, pure Rust) + in-memory (testing). See [specs/STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md).
- **RocksDB storage backend** — Demand-driven — implement when a composition requires it
- **Full Universal IPC v3 compliance** — Complete protocol alignment
- **genomeBin readiness** — musl-static resolved; remaining: checksums.toml musl triple + PIE verification
- **95%+ test coverage** — Currently 92.26% line / 92.56% region
- **HTTP health endpoints** — `/health/liveness`, `/health/readiness` (JSON-RPC health triad already complete)
- **Prometheus metrics** — Request counts, latencies, queue depths
- **Rate limiting** — Per-capability and per-client limits

---

## Long-term

- **Provenance trio E2E** — Live compositions with rhizoCrypt + sweetGrass
- **Cross-gate capability.call** — Smoke test with mesh overlay
- **Service mesh patterns** — From [specs/SERVICE_LIFECYCLE.md](specs/SERVICE_LIFECYCLE.md)
- **rootPulse integration** — Sovereign version control over nestGate CAS + Provenance Trio

---

*See [STATUS.md](STATUS.md) for current implementation progress.*
