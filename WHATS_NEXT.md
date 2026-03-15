<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Development Roadmap

**Current Version**: 0.8.4  
**Last Updated**: March 15, 2026

---

## Completed (v0.8.0 -- v0.8.4)

- SQLite storage backend (feature-gated) with full test coverage
- SQLite smart refactoring: modular `sqlite/` directory (`mod.rs`, `common.rs`, `spine.rs`, `entry.rs`, `certificate.rs`, `tests.rs`)
- Real mDNS implementation (feature-gated)
- Deprecated songbird fields removed
- `Cow<'static, str>` for config/bind paths
- `Did` evolved to `Arc<str>` for O(1) cloning
- `must_use_candidate` lint enabled crate-wide
- `#[allow]` attributes audited (remaining are justified)
- Certificate storage trait (`CertificateStorage` + in-memory impl)
- `ServiceState` enum with `watch` channel
- Waypoint types (`WaypointConfig`, `PropagationPolicy`, `SliceTerms`)
- `verify_certificate`, `certificate_lifecycle`, `record_operation`, `depart_slice`
- **redb** default storage backend (pure Rust, sled demoted to optional)
- **jsonrpsee removed** -- pure JSON-RPC 2.0 server (no ring/C)
- **reqwest removed** -- ureq for HTTP discovery (no ring/C)
- **ring fully eliminated** -- ecoBin compliant, zero C dependencies
- **Clippy pedantic + nursery**: 0 errors across all 3 workspace crates
- **Zero-copy JSON-RPC dispatch**: `params.clone()` eliminated, by-value ownership
- **MockTransport cfg-gated**: No mock code in production binary
- **Smart file splits**: All production files under 1000 lines (max: 915)
- **15 const fn promotions**, `let...else` modernization, lock scope tightening
- **Scyborg license schema**: `CertificateType::scyborg_license()`, metadata builders, constants
- **Protocol escalation**: `IpcProtocol` negotiation (prefers tarpc Unix socket, fallback JSON-RPC)
- **SyncProtocol evolved**: JSON-RPC/TCP sync engine with `push_to_peer`/`pull_from_peer`, graceful fallback
- **Zero-copy storage keys**: `[u8; 24]` stack allocation for redb/sled index keys
- **CI cross-compilation**: musl targets (`x86_64`, `aarch64`, `armv7`) via `cross-rs/cross`
- **Coverage boost**: 84.52% → 86.47% line coverage (+61 tests)

---

## v0.9.0 Targets

- **90%+ test coverage** -- Raise from 86.47% (remaining gaps in jsonrpc server loop, main.rs, DNS SRV/mDNS network paths)
- **Split `storage/tests.rs`** -- Currently 1122 lines; split into per-backend test modules (redb, sled, sqlite, memory)
- **Broader `Cow<'a, str>` / `Arc<str>` adoption** -- EntryType string fields, metadata keys
- **Waypoint relending chain** -- Depth-limited relend with term inheritance
- **Waypoint expiry sweep** -- Background task for expired anchor auto-return
- **Certificate provenance proof** -- `generate_provenance_proof` per CERTIFICATE_LAYER.md
- **Certificate escrow** -- `TransferConditions`, escrow hold/release
- **PrimalAdapter** -- Retry + circuit-breaker for inter-primal calls
- **Signing capability middleware** -- Signature verification on RPC layer (capability-discovered)

---

## v1.0.0 Targets

- **PostgreSQL storage backend** -- Implement per [specs/STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md)
- **RocksDB storage backend** -- Implement per [specs/STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md)
- **Full Universal IPC v3 compliance** -- Complete protocol alignment
- **genomeBin readiness** -- Meet genomeBin integration requirements
- **95%+ test coverage**
- **HTTP health endpoints** -- `/health/liveness`, `/health/readiness`
- **Prometheus metrics** -- Request counts, latencies, queue depths
- **Rate limiting** -- Per-capability and per-client limits

---

## Long-term

- **Cross-primal integration testing** -- With rhizoCrypt and sweetGrass
- **Service mesh patterns** -- From [specs/SERVICE_LIFECYCLE.md](specs/SERVICE_LIFECYCLE.md)
- **Showcase demos** -- Expand from ~10% to full coverage

---

*See [STATUS.md](STATUS.md) for current implementation progress.*
