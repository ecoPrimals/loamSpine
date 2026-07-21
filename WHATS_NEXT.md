<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Development Roadmap

**Current Version**: 0.9.16  
**Last Updated**: July 21, 2026

---

## Recent Changes

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
