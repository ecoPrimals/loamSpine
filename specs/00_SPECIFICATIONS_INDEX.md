# 🦴 LoamSpine — Specifications Index

**Last Updated**: December 22, 2025  
**Version**: 0.2.0  
**Status**: Active Development

---

## Overview

This directory contains the complete specification suite for LoamSpine, the permanent ledger of the ecoPrimals Phase 2 infrastructure. LoamSpine provides the "fossil record"—the slow, anaerobic layer where ephemeral DAG state (from RhizoCrypt) compresses into permanent, immutable history.

---

## 📚 Specification Documents

### Core Specifications

| Document | Purpose | Status |
|----------|---------|--------|
| [LOAMSPINE_SPECIFICATION.md](./LOAMSPINE_SPECIFICATION.md) | Master specification document | ✅ Complete |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | High-level architecture & component overview | ✅ Complete |
| [DATA_MODEL.md](./DATA_MODEL.md) | Entry, Spine, Chain structures | ✅ Complete |

### Protocol Specifications

| Document | Purpose | Status |
|----------|---------|--------|
| [WAYPOINT_SEMANTICS.md](./WAYPOINT_SEMANTICS.md) | Waypoint spines & slice anchoring | ✅ Complete |
| [CERTIFICATE_LAYER.md](./CERTIFICATE_LAYER.md) | Loam Certificate Layer (memory-bound objects) | ✅ Complete |
| [API_SPECIFICATION.md](./API_SPECIFICATION.md) | gRPC & REST API definitions | ✅ Complete |

### Integration Specifications

| Document | Purpose | Status |
|----------|---------|--------|
| [INTEGRATION_SPECIFICATION.md](./INTEGRATION_SPECIFICATION.md) | RhizoCrypt, BearDog, SweetGrass | ✅ Complete |
| [STORAGE_BACKENDS.md](./STORAGE_BACKENDS.md) | SQLite, PostgreSQL, RocksDB | ✅ Complete |

---

## 🧬 Biological Model

LoamSpine is named after **loam**—the slow, anaerobic soil layer:

```
    RhizoCrypt (Rhizome Layer)
    ══════════════════════════
         ○──○──○    ○──○
        /        \  /    \
       ○    ○──○──○──○    ○     ← Ephemeral branching
        \  /          \  /
         ○─────────────○
              │
              │ Dehydration (selective commitment)
              ▼
    ══════════════════════════
    LoamSpine (Anaerobic Layer)
    
    [Genesis]──[Entry]──[Entry]──[Entry]──[Tip]
                                     ↑
                            Linear, permanent
```

---

## 🔗 Related Specifications

### Phase 2 Siblings
- [RhizoCrypt Specification](../../rhizoCrypt/specs/)
- [SweetGrass Specification](../../sweetGrass/specs/)

### Gen 1 Dependencies
- [BearDog Specification](../../../beardog/specs/)
- [Songbird Specification](../../../songbird/specs/)
- [NestGate Specification](../../../nestgate/specs/)

### Foundational
- [sourDough Core](../../sourDough/)
- [Phase 2 Architecture](../../ARCHITECTURE.md)

---

## 📖 Reading Order

For new developers, we recommend this reading order:

1. **[ARCHITECTURE.md](./ARCHITECTURE.md)** — Understand the big picture
2. **[DATA_MODEL.md](./DATA_MODEL.md)** — Learn the core data structures
3. **[WAYPOINT_SEMANTICS.md](./WAYPOINT_SEMANTICS.md)** — Understand slice anchoring
4. **[CERTIFICATE_LAYER.md](./CERTIFICATE_LAYER.md)** — Learn about memory-bound objects
5. **[API_SPECIFICATION.md](./API_SPECIFICATION.md)** — See the external interfaces
6. **[INTEGRATION_SPECIFICATION.md](./INTEGRATION_SPECIFICATION.md)** — Understand primal interactions
7. **[LOAMSPINE_SPECIFICATION.md](./LOAMSPINE_SPECIFICATION.md)** — Full reference (read as needed)

---

## 🎯 Key Concepts Quick Reference

| Concept | Definition |
|---------|------------|
| **Entry** | A single, immutable record in a LoamSpine |
| **Spine** | A linear chain of entries with common ownership |
| **Certificate** | A memory-bound object with ownership and history |
| **Waypoint** | A local spine that anchors borrowed slices |
| **Slice Anchor** | Entry recording a slice's arrival at a waypoint |
| **Inclusion Proof** | Cryptographic proof an entry exists in a spine |
| **Rollup** | Compression of multiple entries into a single hash |

---

## 🔄 Relationship to RhizoCrypt

LoamSpine is the **permanence layer** that receives commits from RhizoCrypt:

```
RhizoCrypt Session              LoamSpine Spine
┌─────────────────┐            ┌─────────────────┐
│                 │            │                 │
│  DAG branches   │  Dehydrate │  Linear chain   │
│  and explores   │ ─────────► │  with proofs    │
│                 │            │                 │
│  Ephemeral      │            │  Permanent      │
│                 │            │                 │
└─────────────────┘            └─────────────────┘
```

Key integration points:
- **SessionCommit**: Dehydrated DAG summary → LoamSpine entry
- **SliceAnchor**: Borrowed state → Waypoint spine
- **SliceReturn**: Resolved slice → Origin spine update

---

## 🏗️ Implementation Status

See [../STATUS.md](../STATUS.md) for current implementation progress.

See [../WHATS_NEXT.md](../WHATS_NEXT.md) for the development roadmap.

---

*LoamSpine: The permanent record that gives memory its meaning.*

