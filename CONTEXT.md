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

LoamSpine is the permanence layer of the **provenance trio**: the ephemeral
DAG primal handles session storage, LoamSpine commits selected data into
permanent history, and the attribution primal records provenance braids.
Other primals interact with LoamSpine through capability-discovered JSON-RPC
when they need to commit, verify, or query permanent records.

## Technical Facts

- **Language:** 100% Rust, zero C dependencies (pure-Rust ecoBin)
- **Architecture:** Single binary (UniBin), multiple operational modes
- **Deployment:** musl-static (x86_64 + aarch64), 4.3M stripped — plasmidBin / benchScale ready
- **Communication:** JSON-RPC 2.0 over platform-agnostic IPC (Unix sockets)
- **License:** AGPL-3.0-or-later + ORC + CC-BY-SA-4.0 (scyBorg triple)
- **Tests:** 1,528 (all concurrent, ~3s, zero flaky)
- **Coverage:** 90.92% line / 89.09% branch / 92.92% region
- **Unsafe:** 0 (`#![forbid(unsafe_code)]`)
- **MSRV:** Rust 2024 edition (1.85+)
- **Version:** 0.9.16
- **Source files:** 189 `.rs` files across 3 workspace crates (`loam-spine-core`, `loam-spine-api`, `loamspine-service`)
- **Production crypto adapters:** `JsonRpcCryptoSigner` and `JsonRpcCryptoVerifier` implement the signing capability via JSON-RPC `crypto.sign_ed25519` / `crypto.verify_ed25519` per `CRYPTO_WIRE_CONTRACT.md` (see `crates/loam-spine-core/src/traits/crypto_provider.rs`). `CliSigner` remains the development fallback.

## Key Capabilities (JSON-RPC methods)

- `spine.create`, `spine.get`, `spine.list`, `spine.seal` — Spine lifecycle
- `entry.append`, `entry.get`, `entry.get_tip`, `entry.list` — Entry management
- `certificate.mint`, `certificate.transfer`, `certificate.loan`, `certificate.return`, `certificate.get` — Loam Certificates
- `session.commit`, `braid.commit` — Provenance trio coordination
- `slice.anchor`, `slice.checkout` — Waypoint anchoring
- `anchor.publish`, `anchor.publish_batch`, `anchor.verify` — Public chain anchoring (single + aggregate Merkle batch)
- `proof.generate_inclusion`, `proof.verify_inclusion` — Merkle inclusion proofs
- `bonding.ledger.store`, `bonding.ledger.retrieve`, `bonding.ledger.list` — Ionic bond persistence
- `btsp.negotiate`, `btsp.capabilities` — BTSP cipher negotiation
- `auth.check`, `auth.mode`, `auth.peer_info` — JH-0 access control
- `lifecycle.status`, `primal.announce` — Lifecycle + self-registration
- `health.check`, `health.liveness`, `health.readiness` — Health probes
- `capabilities.list`, `identity.get` — Capability discovery (Wire Standard L3)
- `tools.list`, `tools.call` — MCP tool discovery and invocation
- `permanence.*` (4) — Legacy naming compat
- **43 methods total** (37 stable, 2 evolving, 4 compat). Storage backends: redb (default) and in-memory.

## What This Does NOT Do

- Does not handle ephemeral/DAG storage (ephemeral DAG capability primal)
- Does not manage attribution braids (attribution capability primal)
- Does not provide cryptographic primitives (delegates to the signing capability provider)
- Does not discover hardware (hardware discovery capability primal)
- Does not manage networking or TLS (capability-discovered provider)
- Does not orchestrate processes (that's biomeOS)

## Related Repositories

- [wateringHole](https://github.com/ecoPrimals/wateringHole) — ecosystem standards and registry
- Ephemeral DAG primal — ephemeral session storage (provenance trio)
- Attribution primal — semantic attribution (provenance trio)
- [biomeOS](https://github.com/ecoPrimals/biomeOS) — process orchestration and NeuralAPI

## Design Philosophy

These binaries are built using AI-assisted constrained evolution. Rust's
compiler constraints (ownership, lifetimes, type system) reshape the fitness
landscape and drive specialization. Primals are self-contained — they know
what they can do, never what others can do. Complexity emerges from runtime
coordination, not compile-time coupling.
