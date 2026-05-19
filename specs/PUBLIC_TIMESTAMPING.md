# Public Timestamping — Exploration Spec (WS-3)

**Status:** EXPLORATION — spec-level analysis, no implementation commitment
**Date:** 2026-05-19
**Origin:** WS-3 (Upstream Gaps — River Delta, May 19 2026)
**License:** AGPL-3.0-or-later

---

## Problem

The provenance trio produces DAG sessions (rhizoCrypt), ledger commits
(loamSpine), and semantic braids (sweetGrass). Provenance is verifiable
only within the ecosystem trust boundary. An independent third party
cannot verify *when* a spine state was established without trusting the
loamSpine clock.

## Philosophy: Anchor Publish as Public Record

The ecoPrimals ecosystem is sovereign by design — loamSpine and
rhizoCrypt layer and evolve internally. But "publishing" a study, a
provenance milestone, or a significant spine state means **stamping it
to an external immutable ledger that is linked to our internal ledger**.

Established public blockchains (Bitcoin, Ethereum) are exactly that:
immutable public ledgers. We use them as **anchor targets**, paying only
the gas cost to record a hash. There is explicitly **no other interaction
with cryptocurrency** — no tokens, no DeFi, no wallets beyond what's
needed for a single OP_RETURN or event log. The blockchain is a public
notary, nothing more.

This mirrors the cellMembrane / projectNUCLEUS pattern: use available
infrastructure (VPS, GitHub) as stepping stones while building toward
full sovereignty. Bitcoin's OP_RETURN is today's most established
"external notary." RFC 3161 TSAs and data commons (IPFS) serve
complementary roles — legal-grade timestamps and content-addressed
persistence respectively.

The anchor surface is intentionally **multi-target**: different use cases
call for different anchors. A genomics study might anchor to Bitcoin for
maximum public verifiability; a rapid iteration might anchor to a TSA for
sub-second legal-grade proof; internal federation might anchor to another
loamSpine instance via `FederatedSpine`.

## Current State

loamSpine already has a chain-agnostic anchor system (§2.5 of
LOAMSPINE_SPECIFICATION.md):

- `EntryType::PublicChainAnchor` records an anchor receipt on the spine
- `AnchorTarget` enum: `Bitcoin`, `Ethereum`, `FederatedSpine`,
  `DataCommons`, `Other`
- `anchor.publish` / `anchor.verify` JSON-RPC methods (implemented, tested)
- loamSpine only **records** the receipt — actual chain submission is
  delegated to a capability-discovered `"chain-anchor"` primal

**Gap:** No `chain-anchor` primal exists. No RFC 3161 TSA option. No
integration with any external verifiable system.

---

## Anchor Targets

### Target A: Bitcoin OP_RETURN (Primary — Public Immutability)

Embed a 32-byte state hash in a Bitcoin `OP_RETURN` output. This is the
strongest form of "publishing" to a public record — once confirmed, the
hash is part of the most established immutable ledger on Earth.

**Workflow:**
1. loamSpine computes `state_hash` = Blake3 of the current spine tip
2. `chain-anchor` primal broadcasts a Bitcoin transaction with
   `OP_RETURN <state_hash>`
3. Once confirmed, store `tx_id`, `block_height`, `block_timestamp`
4. The spine now has a publicly verifiable link: anyone with the `tx_id`
   can confirm the state hash existed at `block_timestamp`

**Use case:** Publishing a completed study, major provenance milestone,
or any state that needs maximum external verifiability.

**Characteristics:**
- Strongest decentralized ordering guarantee
- Tamper-proof once confirmed (Bitcoin's proof-of-work)
- Well-understood by the security community
- Transaction fees: ~$0.10–$2.00 per anchor (gas cost only)
- Confirmation latency: 10–60 minutes (1–6 confirmations)
- Requires minimal wallet: one UTXO per anchor, no ongoing balance

**Rust ecosystem:** `bitcoin` crate (MIT, well-maintained) for
transaction construction. Broadcast via Esplora/Electrum HTTP API —
no full node required. `bdk` for wallet management if needed.

### Target B: Ethereum Event Log

Emit an Ethereum event log containing the state hash via a minimal
anchor contract. Similar public record guarantee with faster
confirmation (~15s) and lower cost via L2 rollups (Arbitrum, Optimism).

**Use case:** Higher-frequency anchoring where Bitcoin's 10-minute
blocks are too slow.

**Characteristics:**
- Fast confirmation (~15s L1, near-instant L2)
- Lower cost via L2 ($0.001–$0.01 per anchor)
- Same public immutability guarantee
- Requires a deployed anchor contract (one-time)

**Rust ecosystem:** `alloy` (MIT/Apache-2.0, successor to `ethers-rs`,
pure Rust, actively maintained).

### Target C: RFC 3161 TSA (Legal-Grade Timestamp)

An RFC 3161 Time-Stamp Authority returns a cryptographically signed
timestamp token (TST) over a message digest. The TST proves that the
digest existed at the time of issuance.

**Workflow:**
1. loamSpine computes `state_hash` = Blake3 of the current spine tip
2. Client sends `TimeStampReq` containing `SHA-256(state_hash)` to a TSA
3. TSA returns signed `TimeStampResp` with embedded `TstInfo`
4. Receipt stored as `PublicChainAnchor` with
   `anchor_target: Rfc3161Tsa { tsa_url }`,
   `tx_ref` = base64-encoded DER `TimeStampResp`

**Use case:** Legal/regulatory contexts where ISO/IEC 18014-2 compliance
matters. Sub-second anchoring during active development.

**Characteristics:**
- ISO/IEC 18014-2 compliant — recognized by courts and auditors
- Zero cost (FreeTSA.org, Sigstore TSA are free)
- Sub-second latency (HTTP round-trip)
- Offline verification (TST is self-contained)
- Trust anchored to TSA's signing certificate (CA chain)
- No global ordering guarantee (point-in-time only)

**Rust ecosystem:** `sigstore-tsa` (v0.6.6, Apache-2.0, actively
maintained, 105k recent downloads).

### Target D: Data Commons (Content Persistence)

Anchor to a federated data commons (IPFS CID, Arweave, or another
loamSpine instance in a different trust domain).

**Workflow:**
1. Compute state hash
2. Publish to IPFS; record the CID
3. Store as `AnchorTarget::DataCommons { commons_id: "ipfs:<cid>" }`

**Use case:** Content-addressed persistence, federated replication,
cross-ecosystem data sharing.

**Characteristics:**
- Content-addressed — the CID *is* the proof of content
- No transaction fees (IPFS pinning is optional)
- Does not prove *when* data was published (no timestamp alone)
- Useful in combination with TSA or blockchain anchor

### Target E: Federated Spine (Internal Cross-Trust)

Anchor to another loamSpine instance in a different trust domain.
Already modeled as `AnchorTarget::FederatedSpine { peer_id }`.

**Use case:** Cross-spring verification, multi-institution collaboration.

---

## Comparison Matrix

| Criterion | Bitcoin | Ethereum/L2 | RFC 3161 | Data Commons | Federated |
|-----------|---------|-------------|----------|--------------|-----------|
| Public immutability | Strongest | Strong | No (CA trust) | No (content only) | Trust-scoped |
| Legal recognition | Partial | Partial | Yes (ISO) | No | No |
| Cost per anchor | $0.10–$2 | $0.001–$0.01 | Free | Free | Free |
| Latency | 10–60min | 15s–instant | <1s | <5s | <1s |
| Global ordering | Yes | Yes | No | No | No |
| Offline verification | Yes (header) | Yes (proof) | Yes (TST) | No (need node) | No (need peer) |
| Rust ecosystem | `bitcoin` | `alloy` | `sigstore-tsa` | `iroh` | built-in |

---

## Recommended Approach: Multi-Target Anchoring

Rather than choosing one anchor target, loamSpine should support
**all targets** through the existing `AnchorTarget` enum. Different
anchoring strategies serve different purposes:

### Tier 1: Bitcoin OP_RETURN — "Publish to Public Record"

The primary anchor for significant milestones. When a researcher
completes a study, when a provenance chain reaches a stable state, when
a braid is finalized — stamp it to Bitcoin. Gas cost is the price of
public immutability. This is how we tie into the public record.

### Tier 2: RFC 3161 TSA — "Legal Timestamp"

For contexts requiring legal-grade proof. Free, sub-second, ISO-
compliant. Useful during active development and for regulatory
submissions.

### Tier 3: Data Commons / Federated — "Persistence + Replication"

For content-addressed persistence and cross-ecosystem sharing. Pair
with Tier 1 or 2 for timestamping.

### Combined "publish" flow

A single `anchor.publish` call could chain multiple targets:

```
anchor.publish(spine_id, targets: ["bitcoin", "rfc3161"]) → {
    anchors: [
        { target: "bitcoin", tx_ref: "abc123...", block_height: 890000 },
        { target: "rfc3161", tx_ref: "base64(tst)...", tsa_url: "..." }
    ],
    state_hash,
    entry_hash
}
```

---

## Implementation Sketch

### Phase 1: `chain-anchor` Capability Primal

A standalone primal that provides the `"chain-anchor"` capability:

- Accepts `{ state_hash, targets: [...] }` via NeuralAPI
- Manages minimal wallets (Bitcoin UTXO, Ethereum EOA)
- Broadcasts transactions and returns receipts
- loamSpine discovers it via capability and delegates submission

This keeps loamSpine clean — it only records receipts. The chain-anchor
primal handles wallet management, fee estimation, and broadcast.

### Phase 2: RFC 3161 TSA (loamSpine built-in)

TSA support can live directly in loamSpine behind a `tsa` feature flag,
since it requires no wallet management — just an HTTP request.

```
anchor.publish(spine_id, anchor_target: "rfc3161", tsa_url?) → {
    state_hash, entry_hash, tst_token, tsa_url, timestamp_utc
}
```

**Dependency:** `sigstore-tsa` (Apache-2.0, pure Rust)

### AnchorTarget extension

```rust
pub enum AnchorTarget {
    Bitcoin,
    Ethereum,
    FederatedSpine { peer_id: String },
    DataCommons { commons_id: String },
    Rfc3161Tsa { tsa_url: String },     // NEW
    Other { name: String },
}
```

---

## The Sovereignty Gradient

```
Full internal          Available infra           Public record
(loamSpine)            (VPS, GitHub)             (Bitcoin, TSA)
    │                       │                         │
    ▼                       ▼                         ▼
┌──────────┐         ┌──────────────┐          ┌─────────────┐
│ Internal │         │ cellMembrane │          │   anchor    │
│ spine    │────────►│ NUCLEUS      │─────────►│   .publish  │
│ evolves  │         │ (builds to   │          │   (stamps   │
│ freely   │         │  sovereignty)│          │    to public│
└──────────┘         └──────────────┘          │    record)  │
                                               └─────────────┘
```

loamSpine and rhizoCrypt layer and evolve internally with no external
dependency. When a state is ready for public record — a completed study,
a provenance milestone, a reproducibility proof — `anchor.publish`
stamps it to the established immutable ledger(s) of choice. The gas cost
is the price of public verifiability. Nothing more.

---

## References

- RFC 3161: Internet X.509 PKI Time-Stamp Protocol (TSP)
- ISO/IEC 18014-2: Time-stamping mechanisms
- `sigstore-tsa` crate: https://crates.io/crates/sigstore-tsa
- `bitcoin` crate: https://crates.io/crates/bitcoin
- `alloy` crate: https://crates.io/crates/alloy
- FreeTSA: https://freetsa.org/
- Sigstore TSA: https://docs.sigstore.dev/timestamp-authority/overview/
- LOAMSPINE_SPECIFICATION.md §2.5 (External Anchors)
