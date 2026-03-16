# LoamSpine — Collision Layer Architecture

**Version**: 0.1.0  
**Status**: Proposal  
**Last Updated**: March 16, 2026

> **Experimental**: This spec describes a research direction for hash-based
> collision layers that bridge LoamSpine's linear model with rhizoCrypt's
> DAG model. Validation experiments are tracked in neuralSpring.

---

## 1. Motivation

LoamSpine maintains a strict linear chain. rhizoCrypt maintains an ephemeral DAG.
The bridge between them (dehydration) is always **DAG → linear** — branching state
compresses into a single entry. There is no mechanism for **linear → branch** where
the spine itself discovers structural relationships between entries.

A collision layer introduces a third lens: **similarity grouping via intentional
hash collisions**. By applying progressively weaker hash functions to the same
content-addressed entries, we create collision classes that reveal hidden structural
relationships — entries that are "near" each other under various notions of closeness.

### 1.1 Historical Precedent

Cross-writing (19th century): letters were written normally, then the page was
rotated 90 degrees and overwritten with a second message. Paper was the limiting
resource, but both layers of information persisted because they used orthogonal
encoding. A human reader applied the appropriate "lens" (reading direction) to
recover each layer.

The collision layer applies the same principle to data: **multiple information
layers coexist on the same substrate, recoverable through different hash lenses**.

### 1.2 Biological Model

In mycelial networks, linear hyphae and branched networks are not alternatives —
they are coexistent phases that evolve into each other:

```
Linear → Branch point → Linear → Anastomosis → Linear
```

The collision layer is the chemical gradient that tells distant hyphae they are
in similar environments — discovering relatedness without explicit connection.

---

## 2. Architecture

### 2.1 Hash Resolution Hierarchy

```
Level 0: Blake3-256 (32 bytes)  — Unique identity (current EntryHash)
Level 1: Blake3-128 (16 bytes)  — Neighborhood grouping
Level 2: Blake3-64  (8 bytes)   — Coarse similarity buckets
Level 3: Custom     (4 bytes)   — Domain-specific collision classes
```

Each level is a truncation or projection of the full hash. Level 0 is the current
`ContentHash = [u8; 32]`. Higher levels intentionally increase collision probability
to create meaningful groupings.

### 2.2 Sub-Hash Resolution

When entries collide at Level N, a sub-hash at Level N-1 disambiguates within the
collision class. This creates a tree of resolution:

```
Level 2 bucket: [Entry A, Entry B, Entry C, Entry D]
                    |
                    ├── Level 1 group: [A, B]  (similar neighborhood)
                    |       ├── Level 0: A (unique)
                    |       └── Level 0: B (unique)
                    |
                    └── Level 1 group: [C, D]  (similar neighborhood)
                            ├── Level 0: C (unique)
                            └── Level 0: D (unique)
```

The collision class at each level is itself a data structure — it tells us which
entries are structurally related under that hash lens.

### 2.3 Collision Layer Store

```
CollisionLayerIndex {
    level: u8,
    hash_fn: HashProjection,
    buckets: HashMap<TruncatedHash, Vec<EntryRef>>,
    sub_index: Option<Box<CollisionLayerIndex>>,
}
```

An entry participates in the linear chain (horizontal script) **and** in collision-
group clusters (vertical script). Neither view is canonical — both coexist.

### 2.4 Integration with Existing Types

| Existing Type | Collision Layer Role |
|---------------|---------------------|
| `EntryHash` (`[u8; 32]`) | Level 0 — unique identity (unchanged) |
| `ContentHash` | Level 0 — content addressing (unchanged) |
| `Spine` (linear chain) | One "lens" — horizontal reading direction |
| Collision group | Second "lens" — vertical reading direction |
| `Waypoint` | Already models temporary coexistence; collision layer extends this to structural coexistence |

---

## 3. Relationship to Existing Architecture

### 3.1 LoamSpine (Linear)

The spine remains the authoritative permanence layer. The collision layer is an
**index**, not a replacement. Entries are stored once in the spine; the collision
index provides alternative access patterns.

### 3.2 rhizoCrypt (DAG)

The collision layer provides a bridge from linear → branching that doesn't require
explicit DAG construction. If a collision group grows beyond a threshold, it can
be promoted to a rhizoCrypt session for active exploration.

```
LoamSpine (linear)  ←→  Collision Layer (similarity)  ←→  rhizoCrypt (DAG)
```

### 3.3 Dehydration Path Extended

Current: `rhizoCrypt session → dehydrate → LoamSpine entry`

Extended: `LoamSpine entries → collision grouping → promote → rhizoCrypt session`

This creates a bidirectional bridge where linear and branching structures evolve
into each other based on the data itself.

---

## 4. Data Science Applications

### 4.1 Collision Topology as Embedding

Different hash table sizes and hash functions produce different collision topologies.
Each topology reveals a different similarity structure in the data. This is related
to locality-sensitive hashing (LSH) but adds hierarchical resolution via sub-hashing.

### 4.2 Experiment Design

1. Select a dataset with known cluster structure
2. Hash entries through the resolution hierarchy (Level 0 → Level 3)
3. Analyze collision groups at each level for cluster correspondence
4. Measure: adjusted Rand index, normalized mutual information, silhouette score
5. Compare against LSH, k-means, spectral clustering baselines
6. Vary hash table sizes to find optimal collision density

### 4.3 Cross-Writing Information Recovery

The cross-writing hypothesis: when two information layers overlay the same address
space, the overlay itself encodes information not present in either layer alone.

Test: given two spine histories that share a collision group, does the collision
structure predict future convergence (merge) or divergence (fork)?

---

## 5. Implementation Phases

### Phase 0: Validation (neuralSpring)

- Python prototype of collision hierarchy on synthetic datasets
- Measure clustering quality vs. hash level
- Compare to LSH baselines
- Determine optimal level count and bucket sizes

### Phase 1: Index Prototype (loamSpine)

- `CollisionLayerIndex` type in `loam-spine-core`
- Truncated hash computation (`Blake3 → [u8; 16]`, `[u8; 8]`, `[u8; 4]`)
- Read-only index built from existing spine entries
- Query: "which entries are in the same collision group as entry X at level N?"

### Phase 2: Bidirectional Bridge (loamSpine + rhizoCrypt)

- Collision group promotion to rhizoCrypt session
- rhizoCrypt session dehydration back to spine with collision metadata
- Cross-spine collision discovery (entries from different spines in same group)

### Phase 3: Data Science Layer (neuralSpring + healthSpring)

- Collision topology as feature for ML models
- Domain-specific hash projections (biosignal similarity, attribution similarity)
- Cross-writing information recovery experiments

---

## 6. Constraints

- Collision layer is always an **index**, never authoritative storage
- Blake3-256 remains the canonical identity hash — collision layers use projections
- No collision layer computation on the critical append path
- Collision indices are rebuiltable from spine entries (no durability requirement)
- Level 0 (full Blake3) collisions remain treated as impossible (2^-256)

---

## 7. Open Questions

1. What is the optimal number of hierarchy levels?
2. Should collision groups have TTLs or be permanent indices?
3. Can domain-specific hash projections (not just truncation) improve grouping?
4. How does collision density scale with spine size?
5. What is the computational cost of maintaining collision indices on append?

---

## 8. References

- Sub-hash collision resolution technique (data science literature)
- Locality-sensitive hashing (Indyk & Motwani, 1998)
- Cross-writing / crossed letters (19th century postal history)
- Mycelial network anastomosis and chemical gradient sensing

---

*The collision layer: discovering relatedness without explicit connection.*
