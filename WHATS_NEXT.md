<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Development Roadmap

**Current Version**: 0.8.2  
**Last Updated**: March 14, 2026

---

## Completed (v0.8.0 -- v0.8.2)

- SQLite storage backend (feature-gated)
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
- Smart file refactoring (all files < 422 lines)
- **redb** default storage backend (pure Rust, sled demoted to optional)
- **jsonrpsee removed** -- pure JSON-RPC 2.0 server (no ring/C)
- **reqwest removed** -- ureq for HTTP discovery (no ring/C)
- **ring fully eliminated** -- ecoBin compliant, zero C dependencies

---

## v0.9.0 Targets

- **Broader `Cow<'a, str>` / `Arc<str>` adoption** -- EntryType string fields, metadata keys
- **Waypoint relending chain** -- Depth-limited relend with term inheritance
- **Waypoint expiry sweep** -- Background task for expired anchor auto-return
- **Certificate provenance proof** -- `generate_provenance_proof` per CERTIFICATE_LAYER.md
- **Certificate escrow** -- `TransferConditions`, escrow hold/release
- **SyncProtocol** -- Spine federation per INTEGRATION_SPECIFICATION.md
- **PrimalAdapter** -- Retry + circuit-breaker for inter-primal calls
- **Sled certificate storage** -- Extend `SledStorage` to implement `CertificateStorage` (sled now optional)
- **BearDog authentication middleware** -- Signature verification on RPC layer

---

## v1.0.0 Targets

- **PostgreSQL storage backend** -- Implement per [specs/STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md)
- **RocksDB storage backend** -- Implement per [specs/STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md)
- **Full Universal IPC v3 compliance** -- Complete protocol alignment
- **genomeBin readiness** -- Meet genomeBin integration requirements
- **95%+ test coverage** -- Raise from current ~91% baseline
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
