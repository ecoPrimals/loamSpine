<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# loamSpine — Evolution Record

The changelog is sorted by capability surface, not by date.
Each section tells the arc: what emerged, what it replaced, what it became.
The date-sorted history lives in `CHANGELOG_ARCHIVE.md`.

`H(data|epitope) < H(data|date) < H(data)`

---

## Spine & Entry Model — append-only distributed ledger

The core evolved from basic spine ops to rich entry types with canonical hashing, batch append, and semantic verification.

- **Spine lifecycle** (v0.4→0.9): `spine.create`, `entry.append`, `spine.seal` — Merkle-linked append-only chains.
- **Entry type expansion** (v0.6→0.9): 21+ variants — `DataAnchor`, `SessionCommit`, `BraidCommit`, trust events, bond records.
- **Canonical serialization** (v0.8): `BTreeMap` metadata + deterministic hashing via `rmp-serde`.
- **Batch append** (Wave 155n): `entry.append_batch` — ~30 ms/object → ~3 ms/object amortized for bulk ingestion.
- **Hex hash acceptance** (May 2026): All 32-byte fields accept byte arrays or 64-char hex strings.
- **`spine.status`** (Wave 156e): Entry count, tip/genesis hashes, spine state, and associated sessions.

---

## Certificate Layer — ownership, loans, and lifecycle proofs

Certificates evolved from mint/verify to full loan/sublend/escrow lifecycle with semantic integrity and structured history.

- **Mint & verify** (v0.4→0.9): Progressive checks — `MintEntryValid`, `OwnerConsistent`, `ChainValid`.
- **Loan lifecycle** (v0.6→0.9): Loan, sublend, return, auto-return; `CertificateManager` superseded by `LoamSpineService`.
- **Structured history** (Wave 155f): `certificate.history` returns typed ownership and loan records.
- **Batch mint** (Wave 155n): `certificate.mint_batch` with same amortization as batch append.
- **Tower-signed entries** (April 2026): `entry.append`/`session.commit` sign via `crypto.sign_ed25519` when configured.

---

## Session & Provenance Trio — rhizoCrypt → loamSpine → sweetGrass

Permanence evolved from compat stubs to full trio integration with dehydration, braid anchoring, and self-contained receipts.

- **Trio compat layer** (v0.8): `permanent-storage.*` bridges rhizoCrypt wire format with auto-spine creation.
- **Session commit** (v0.8→0.9): `session.commit` with Merkle root; aliases for orchestrator routing.
- **Session dehydration** (Wave 60): `session.dehydrate` — blake3 summary without state mutation for rootPulse pipeline.
- **Self-contained receipts** (April 2026): `CommitSessionResponse` echoes session binding + ledger anchor + optional tower signature.
- **rootPulse handlers** (Wave 157k): `rootpulse.ledger_commit`/`query_commit` — graph step in dehydrate-sign-commit pipeline.

---

## Public Chain Anchoring — external provenance verification

Anchoring evolved from single-spine receipts to aggregate batch proofs with multi-chain strategy.

- **Public chain anchor** (April 2026): `EntryType::PublicChainAnchor` + `AnchorTarget` for Bitcoin, Ethereum, federated spines.
- **Publish/verify** (April 2026): `anchor.publish`/`anchor.verify`; chain submission is capability-discovered.
- **Aggregate batch** (May 2026): `anchor.publish_batch` — one on-chain transaction covers N spine state hashes.
- **Inclusion proofs** (June 2026): Merkle path validation with cycle detection — evolved from stub.

---

## JSON-RPC & tarpc IPC Surface — 55 methods, dual protocol

RPC grew from 18 methods to 55 JSON-RPC with full tarpc binary parity for performance-critical ops.

- **Semantic naming** (v0.8): `{domain}.{operation}` — `spine.create`, `certificate.mint`, `health.check`.
- **Method gate** (May 2026): JH-0 Public vs Protected classification with `auth.check`/`auth.mode`.
- **tarpc convergence** (Wave 156h): 37 tarpc methods; JSON-RPC retains diagnostic/conjugation surface.
- **G65 negotiation** (Wave 156p): Single-socket tarpc/JSON-RPC selection replaces C2 dual-socket for new connections.
- **Capability Wire Standard L3** (April 2026): Flat methods array, provided/consumed capabilities, operation DAG.

---

## BTSP & Transport Security — encrypted wire sessions

BTSP evolved from null-cipher stubs to ChaCha20-Poly1305 encrypted framing with client handshake.

- **BTSP Phase 3** (May 2026): ChaCha20-Poly1305 AEAD; HKDF-SHA256 key derivation; `btsp.negotiate`.
- **Client handshake** (Wave 151b): 4-step ClientHello sequence following songBird reference.
- **NDJSON alignment** (April 2026): Auto-detection of primalSpring-style BTSP in UDS accept loop.
- **Persistent ProviderConn** (April 2026): Single connection per handshake — fixes BearDog verify race.
- **`btsp.capabilities`** (May 2026): Public cipher/HKDF/frame discovery for stadial security checklist.

---

## MCP, Neural API & Gossip — discovery and mesh observation

Discovery evolved from songbird coupling to capability-based self-registration with gossip mesh injection.

- **NeuralAPI** (v0.8): Capability registration via biomeOS UDS; `primal.announce` Wave 43 schema.
- **MCP tools** (v0.9.9): `tools.list`/`tools.call` with completeness test enforcing registry parity.
- **Niche self-knowledge** (Wave 155u): 52 semantic mappings and cost estimates for orchestrator routing.
- **Gossip injection** (Wave 157e): `CasHave`, `BraidHead`, `SpineSealed`, `AnchorPublished`, `RootpulseCommit` events via swarmVine.

---

## Trust & Bond Ledger — cross-gate persistence

Trust and bond state evolved from entry types to dedicated ledger RPC with ionic bond wire contract.

- **Trust ledger** (June 2026): `trust.anchor`/`trust.query`/`trust.event_count` — cross-gate events as permanent entries.
- **Bond ledger** (April 2026): `bonding.ledger.store`/`retrieve`/`list` per `STORAGE_WIRE_CONTRACT.md`.
- **`JsonRpcCryptoSigner`** (April 2026): Production signing via `crypto.sign_ed25519` wire contract.

---

## Storage, Sync & Federation — redb default, resilient replication

Storage consolidated to pure Rust defaults with resilient federation and streaming sync.

- **Stadial parity** (April 2026): sled/SQLite removed; production is redb (default) + memory only.
- **ResilientSyncEngine** (v0.9.9): Circuit-breaker + retry for outbound federation IPC.
- **Streaming sync** (v0.9.5): `push/pull_entries_streaming` emit `StreamItem` variants for pipeline coordination.
- **Backup/restore** (v0.6): rmp-serde serialization; bincode v1 excised.

---

## Transport Abstraction — silicon atheism, cross-platform IPC

Transport evolved from Unix-only to platform-abstracted UDS/TCP/mesh_relay with genetics-layer prefix.

- **G66 abstraction** (Wave 156s): Generic protocol negotiation over any `AsyncRead + AsyncWrite` pipe.
- **TransportEndpoint** (Wave 101): Wire-compatible sourDough standard — `uds`, `tcp`, `mesh_relay`.
- **G68 platform** (Wave 157a): Symlinks, permissions, executable bits — zero raw OS imports outside platform layer.
- **riboCipher signals** (Wave 113→114): All three genetics signals (`0xEC`/`0xED`/`0xEE`) stripped as 2-byte prefix.
- **Socket naming** (April 2026): `loamspine.sock` primary; `ledger.sock` capability symlink; `permanence.sock` legacy.

---

## Discovery, Waypoint & Health — service location, attestation, operational readiness

Discovery, waypoint, and health surfaces evolved to multi-tier fallback with honest async probes.

- **DNS-SRV + mDNS** (v0.7.1→April 2026): RFC 2782 via hickory-resolver; mdns-sd 0.19 replaces async-std mdns.
- **Waypoint enforcement** (v0.9.0): Attestation wired into anchor/record/depart; per-spine `WaypointConfig`.
- **Health probe honesty** (Wave 150t): Readiness wraps storage in timeout; LS-03 nested-runtime fix (v0.9.15).
- **Zero unsafe + deep debt** (v0.9.12→Wave 157g): `#![forbid(unsafe_code)]`; all files under 800L; G72 dependency excision.
- **Test scale** (Wave 157k): 1,865 tests, 55 JSON-RPC methods, 20 domains, musl-static ecoBin deployment.

---

*Full date-sorted history: `CHANGELOG_ARCHIVE.md`*
*Last compressed: Wave 172*
