# Anchoring Architecture

**How DAG-to-linear compression enables a massive public record for gas cost only**

**Status:** ACTIVE — aggregation implemented, chain submission delegated
**Date:** 2026-05-20
**Origin:** WS-3 (River Delta), gen4/architecture/ANCHORING_PIPELINE
**License:** AGPL-3.0-or-later

---

## 1. The Compression Pipeline

The ecoPrimals provenance stack compresses arbitrarily large computational
histories into 32 bytes suitable for on-chain anchoring. Each layer
reduces the data by orders of magnitude while preserving cryptographic
verifiability.

```
Layer 1: rhizoCrypt DAG          (GB of data, thousands of vertices)
    │
    │  Merkle root
    ▼
Layer 2: 32 bytes                (session root — fingerprint of entire computation)
    │
    │  dehydrate (session.commit)
    ▼
Layer 3: loamSpine spine entry   (certificate + merkle root + metadata)
    │
    │  Blake3(tip entry)
    ▼
Layer 4: 32 bytes                (state_hash — fingerprint of entire spine state)
    │
    │  aggregate N state hashes
    ▼
Layer 5: 32 bytes                (aggregate root — fingerprint of N spines)
    │
    │  OP_RETURN / calldata / TSA
    ▼
Layer 6: Public chain            (one transaction on Bitcoin, Ethereum, L2, or TSA)
```

### What each layer does

| Layer | Input | Output | Compression |
|-------|-------|--------|-------------|
| 1. DAG recording | Full computation (parameters, kernels, results, checks) | Content-addressed DAG with Merkle root | GB → 32 bytes |
| 2. Dehydration | rhizoCrypt session DAG | loamSpine `SessionCommit` entry with `merkle_root` | DAG → spine entry |
| 3. State hash | Spine with N entries | Blake3 hash of tip entry | spine → 32 bytes |
| 4. Aggregation | N state hashes from N spines | Single Merkle tree root | N×32 bytes → 32 bytes |
| 5. Anchoring | Aggregate root (32 bytes) | On-chain transaction | 32 bytes → public record |

**The compression ratio is astronomical.** A genomics pipeline processing
590 GB of sequencing data across 264 clones produces thousands of DAG
vertices per clone. All 264 clones aggregate to a single 32-byte root.
That root anchors to Bitcoin for ~$1-3. The chain stores the commitment.
The sovereign infrastructure stores the data.

---

## 2. Crypto as Infrastructure, Not Currency

### The stance

Public blockchains are **immutable append-only ledgers**. We use them as
notary services — the same way you use a post office to timestamp a
letter. The gas cost is postage. That is the extent of our interaction
with cryptocurrency.

| What we use | What we don't use |
|-------------|-------------------|
| Bitcoin as a public timestamp ledger | Bitcoin as a currency |
| Ethereum as a programmable timestamp ledger | Ethereum tokens, DeFi, NFT marketplaces |
| L2 rollups as cheap, fast timestamp ledgers | L2 bridges for financial transactions |
| Gas fees as postage for public record | Wallets, exchanges, token economics |

**Currency is an abstraction of the blockchain's own economics.** Bitcoin
miners accept BTC for including transactions; Ethereum validators accept
ETH for processing calldata. We pay that postage to write 32 bytes to
their ledger. We don't hold positions, trade tokens, or participate in
their economic systems beyond the gas transaction.

### What IS possible (for others)

The anchor surface is a bridge. Someone who holds a Loam Certificate
(a Novel Ferment Transcript) COULD:

- Transfer it via an L2 smart contract (the certificate's `state_hash`
  is already on-chain — a bridge contract could verify ownership)
- Use a Merkle proof to demonstrate inclusion in an aggregate anchor
  to any smart contract on any EVM chain
- Build a verification contract that checks loamSpine anchor proofs
  entirely on-chain

These are **possible** because the infrastructure is chain-compatible.
They are not **required** because the ecosystem is sovereign. The
internal provenance chain works without any external chain. The
external chain is an optional public proof layer.

### The cellMembrane analogy

This mirrors the cellMembrane / projectNUCLEUS pattern:

- **VPS** is available infrastructure — we use it to host while building
  toward self-hosted sovereignty
- **GitHub** is available infrastructure — we use it for code hosting
  while building toward Forgejo sovereignty
- **Bitcoin/Ethereum** is available infrastructure — we use it for public
  timestamps while the internal provenance chain provides sovereign
  verification

Each is a stepping stone. The ecosystem works without any of them. They
make it stronger by adding external verifiability.

---

## 3. The Economics: Gas vs Everything Else

### Single anchor costs

| Target | Cost | Latency | Permanence |
|--------|------|---------|------------|
| Bitcoin OP_RETURN | $0.10–$2.00 | 10–60 min | 15+ years, proof-of-work |
| Ethereum L1 calldata | $0.10–$2.00 | ~15 sec | 10+ years, proof-of-stake |
| Ethereum L2 (Arbitrum, Base) | $0.001–$0.01 | near-instant | L1 security inheritance |
| RFC 3161 TSA | Free | <1 sec | TSA CA chain lifetime |
| Data Commons (IPFS) | Free | <5 sec | Pinning-dependent |

### Aggregation amortization

The aggregation Merkle tree is the key economic insight. Because the
on-chain cost is fixed per transaction (not per result), batching N
results into one anchor divides the cost by N:

| Results batched | BTC cost per result | ETH L2 cost per result |
|-----------------|--------------------|-----------------------|
| 1 | $1.00 | $0.005 |
| 10 | $0.10 | $0.0005 |
| 100 | $0.01 | $0.00005 |
| 1,000 | $0.001 | $0.000005 |
| 10,000 | $0.0001 | $0.0000005 |

At 1,000 results per batch, Bitcoin anchoring costs $0.001 per result.
Ethereum L2 anchoring costs $0.000005 per result — effectively free.

### Comparison to alternatives

| Method | Cost per "publication" | Speed | Verification |
|--------|----------------------|-------|-------------|
| Journal APC (Nature Comms) | $5,500 | 6–36 months | Trust the journal |
| Journal APC (PLOS ONE) | $1,800 | 3–12 months | Trust the journal |
| arXiv preprint | $0 | Days | No formal verification |
| Cloud provenance export | $0.09/GB egress + storage | Minutes | Trust the cloud provider |
| **Sovereign publication (BTC)** | **$0.001** (aggregated) | **~40 min** | **Cryptographic proof** |
| **Sovereign publication (L2)** | **$0.000005** (aggregated) | **~15 sec** | **Cryptographic proof** |

### Community pooling

A community of researchers can pool their anchors:

```
100 researchers × 10 results/month = 1,000 results
1 BTC aggregate anchor = ~$1.50
1 ETH L2 aggregate anchor = ~$0.005
Dual-chain (BTC + L2) = ~$1.51 total

Per researcher: $0.015/month for dual-chain public record
Per result: $0.0015 for Bitcoin + $0.000005 for L2
```

For comparison, a single AWS S3 GET request costs $0.0004. Publishing
1,000 provenance-verified scientific results to Bitcoin + Ethereum L2
costs less than 4 S3 GET requests.

A community could publish on **every major chain** (Bitcoin, Ethereum L1,
Arbitrum, Base, Optimism) for less than the cost of cloud data egress
for a single export of the underlying data.

---

## 4. Multi-Chain Strategy

### Why multiple chains

Redundancy. Each chain has different security properties:

| Chain | Security model | Failure mode |
|-------|---------------|-------------|
| Bitcoin | Proof-of-work (hashrate dominance) | 51% attack (astronomically expensive) |
| Ethereum L1 | Proof-of-stake (32 ETH per validator) | Validator collusion |
| L2 (Arbitrum) | Optimistic rollup + L1 settlement | Sequencer failure (L1 fallback) |
| L2 (Base) | Optimistic rollup + L1 settlement | Same |
| RFC 3161 TSA | CA certificate chain | TSA compromise (detectable) |

Anchoring to multiple chains means the proof survives the failure of
any single chain. An anchor on Bitcoin + Ethereum + RFC 3161 TSA gives:

- Decentralized immutability (Bitcoin)
- Programmable verification (Ethereum)
- Legal recognition (TSA, ISO 18014-2)

### The `AnchorTarget` enum

loamSpine models this via a chain-agnostic enum (already implemented):

```rust
pub enum AnchorTarget {
    Bitcoin,
    Ethereum,
    Rfc3161Tsa { tsa_url: String },
    FederatedSpine { peer_id: String },
    DataCommons { commons_id: String },
    Other { name: String },
}
```

Each anchor is a separate `PublicChainAnchor` entry on the spine. A
single spine can have anchors on multiple chains — each entry records
the receipt from that chain.

---

## 5. Aggregation Architecture

### The Merkle tree

N spine state hashes form the leaves of a binary Merkle tree:

```
                    aggregate_root (32 bytes)
                   /                         \
              H(A,B)                        H(C,D)
             /      \                      /      \
        state_A   state_B            state_C   state_D
        (spine1)  (spine2)           (spine3)  (spine4)
```

Properties:
- Root is always 32 bytes regardless of N
- Each leaf has an inclusion proof of log2(N) hashes
- Any leaf can be verified without knowing the other leaves
- Uses Blake3 for consistency with loamSpine's existing hash function

### The `0x6563` namespace

On-chain data is prefixed with `0x6563` ("ec" — ecoPrimals namespace)
to distinguish ecoPrimals anchors from other OP_RETURN / calldata uses:

```
OP_RETURN <0x6563> <32-byte aggregate root>
```

This allows scanners and indexers to identify ecoPrimals anchors in
the blockchain without false positives.

### The batch anchor flow

```
1. Collect N spine state hashes (from anchor.publish_batch request)
2. Build Merkle tree → aggregate_root
3. Submit aggregate_root to chain-anchor primal (or external)
4. Receive tx_ref, block_height, anchor_timestamp
5. Record PublicChainAnchor entry on EACH spine with:
   - state_hash: that spine's state hash
   - aggregate_root: the batch root
   - inclusion_proof: Merkle path from state_hash to aggregate_root
   - tx_ref, block_height, anchor_timestamp: from the chain receipt
6. Each spine now has an independently verifiable anchor
```

### Verification

To verify any single result from a batch:

1. Recompute the spine's `state_hash` from its tip entry
2. Verify the `inclusion_proof` against the `aggregate_root`
3. Verify the `aggregate_root` matches the on-chain `tx_ref` data
4. Confirm the `block_height` and `anchor_timestamp`

Steps 1-2 are local (no chain access needed). Steps 3-4 require
read access to a chain node (public, no authentication).

---

## 6. RPC Surface

### Existing (implemented)

| Method | Description |
|--------|-------------|
| `anchor.publish` | Record a single anchor receipt on one spine |
| `anchor.verify` | Verify anchor against spine state |

### New (this document)

| Method | Description |
|--------|-------------|
| `anchor.publish_batch` | Aggregate N spines, record anchors with inclusion proofs |

### Wire format

```json
// anchor.publish_batch request
{
  "spine_ids": ["uuid-1", "uuid-2", "uuid-3"],
  "anchor_target": "Bitcoin",
  "tx_ref": "a1b2c3d4...e5f6",
  "block_height": 892547,
  "anchor_timestamp": 1716235200
}

// anchor.publish_batch response
{
  "aggregate_root": "8c3d...1f4e",
  "anchors": [
    {
      "spine_id": "uuid-1",
      "entry_hash": "...",
      "state_hash": "...",
      "inclusion_proof": ["hash1", "hash2"]
    },
    ...
  ]
}
```

---

## 7. The Sovereignty Gradient

```
Full internal                Available infra              Public record
(loamSpine + rhizoCrypt)     (VPS, GitHub, Forgejo)       (Bitcoin, Ethereum, TSA)
        │                            │                           │
        ▼                            ▼                           ▼
┌──────────────┐            ┌────────────────┐          ┌───────────────┐
│ Internal     │            │ cellMembrane   │          │ anchor        │
│ spine        │───────────►│ NUCLEUS        │─────────►│ .publish      │
│ evolves      │            │ (sovereignty   │          │ .publish_batch│
│ freely       │            │  in progress)  │          │ (32 bytes →   │
│              │            │                │          │  public chain)│
└──────────────┘            └────────────────┘          └───────────────┘
```

loamSpine and rhizoCrypt layer and evolve internally with no external
dependency. When a state is ready for public record — a completed
study, a provenance milestone, a reproducibility proof —
`anchor.publish` or `anchor.publish_batch` stamps it to the
established immutable ledger(s) of choice.

The gas cost is the price of public verifiability. Nothing more.

---

## References

- gen4/architecture/ANCHORING_PIPELINE.md — the 5-layer pipeline design
- gen4/economics/NOVEL_FERMENT_TRANSCRIPTS.md — NFT economics
- gen4/thesis/THE_SOVEREIGN_PUBLICATION.md — the case for sovereign science
- specs/PUBLIC_TIMESTAMPING.md — anchor target comparison and philosophy
- wateringHole/ANCHORING_STANDARD.md — ecosystem-wide guidance
- `bitcoin` crate: https://crates.io/crates/bitcoin
- `alloy` crate: https://crates.io/crates/alloy
- `sigstore-tsa` crate: https://crates.io/crates/sigstore-tsa
