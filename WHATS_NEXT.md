<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Development Roadmap

**Current Version**: 0.8.0  
**Last Updated**: March 13, 2026

---

## v0.9.0 Targets

- **SQLite storage backend** — Implement per [specs/STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md)
- **Real mDNS implementation** — Replace stub with full discovery
- **Remove deprecated songbird fields** — Clean up Phase 1 integration
- **`Cow<'a, str>` zero-copy evolution** — Reduce allocations in hot paths
- **Reduce `#[allow]` in production** — Address lints at source
- **Enable `must_use_candidate` lint** — Improve API ergonomics

---

## v1.0.0 Targets

- **PostgreSQL storage backend** — Implement per [specs/STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md)
- **RocksDB storage backend** — Implement per [specs/STORAGE_BACKENDS.md](specs/STORAGE_BACKENDS.md)
- **Full Universal IPC v3 compliance** — Complete protocol alignment
- **genomeBin readiness** — Meet genomeBin integration requirements
- **95%+ test coverage** — Raise from current 90%+ baseline

---

## Long-term

- **Cross-primal integration testing** — With rhizoCrypt and sweetGrass
- **Service mesh patterns** — From [specs/SERVICE_LIFECYCLE.md](specs/SERVICE_LIFECYCLE.md)

---

*See [STATUS.md](STATUS.md) for current implementation progress.*
