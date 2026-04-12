# Context — LoamSpine

## What This Is

LoamSpine is a pure Rust binary that provides an immutable, permanent ledger
for the ecoPrimals sovereign computing ecosystem. It is part of a collection
of self-contained binaries that coordinate via JSON-RPC 2.0 over Unix sockets,
with zero compile-time coupling between components.

Named after loam — the slow, anaerobic soil layer where organic matter
compresses into permanent geological record — LoamSpine serves as the
canonical source of truth for events, discoveries, and artifacts that matter.

## Role in the Ecosystem

LoamSpine is the permanence layer of the **provenance trio**: rhizoCrypt
handles ephemeral DAG storage, LoamSpine commits selected data into permanent
history, and sweetGrass records attribution. Other primals (biomeOS, BearDog,
toadStool) interact with LoamSpine through JSON-RPC when they need
to commit, verify, or query permanent records.

## Technical Facts

- **Language:** 100% Rust, zero C dependencies (pure-Rust ecoBin)
- **Architecture:** Single binary (UniBin), multiple operational modes
- **Deployment:** musl-static (x86_64 + aarch64), 4.3M stripped — plasmidBin / benchScale ready
- **Communication:** JSON-RPC 2.0 over platform-agnostic IPC (Unix sockets)
- **License:** AGPL-3.0-or-later + ORC + CC-BY-SA-4.0 (scyBorg triple)
- **Tests:** 1,383 (all concurrent, ~3s, zero flaky)
- **Coverage:** 90.92% line / 89.09% branch / 92.92% region
- **Unsafe:** 0 (`#![forbid(unsafe_code)]`)
- **MSRV:** Rust 2024 edition (1.85+)
- **Version:** 0.9.16
- **Source files:** 175 `.rs` files across 3 workspace crates (`loam-spine-core`, `loam-spine-api`, `loamspine-service`)

## Key Capabilities (JSON-RPC methods)

- `spine.create`, `spine.get`, `spine.seal` — Spine lifecycle
- `entry.append`, `entry.get`, `entry.get_tip` — Entry management
- `certificate.mint`, `certificate.transfer`, `certificate.loan`, `certificate.return`, `certificate.verify`, `certificate.lifecycle` — Loam Certificates
- `session.commit`, `braid.commit` — Provenance trio coordination
- `slice.anchor`, `slice.record_operation`, `slice.depart` — Waypoint anchoring
- `anchor.publish`, `anchor.verify` — Public chain anchoring for external provenance
- `proof.generate_inclusion` — Merkle inclusion proofs
- `permanent-storage.commitSession`, `permanent-storage.verifyCommit`, `permanent-storage.getCommit` — Compat layer
- `health.check`, `health.liveness`, `health.readiness` — Health probes
- `capabilities.list` — Capability-based discovery (Wire Standard L3)
- `identity.get` — Primal identity
- `tools.list`, `tools.call` — MCP tool discovery and invocation

## What This Does NOT Do

- Does not handle ephemeral/DAG storage (that's rhizoCrypt)
- Does not manage attribution braids (that's sweetGrass)
- Does not provide cryptographic primitives (that's BearDog)
- Does not discover hardware (that's toadStool)
- Does not manage networking or TLS (capability-discovered provider via Tower Atomic)
- Does not orchestrate processes (that's biomeOS)

## Related Repositories

- [wateringHole](https://github.com/ecoPrimals/wateringHole) — ecosystem standards and registry
- [rhizoCrypt](https://github.com/ecoPrimals/rhizoCrypt) — ephemeral DAG (provenance trio)
- [sweetGrass](https://github.com/ecoPrimals/sweetGrass) — attribution (provenance trio)
- [biomeOS](https://github.com/ecoPrimals/biomeOS) — process orchestration and NeuralAPI

## Design Philosophy

These binaries are built using AI-assisted constrained evolution. Rust's
compiler constraints (ownership, lifetimes, type system) reshape the fitness
landscape and drive specialization. Primals are self-contained — they know
what they can do, never what others can do. Complexity emerges from runtime
coordination, not compile-time coupling.
