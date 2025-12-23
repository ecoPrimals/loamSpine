# 🦴 LoamSpine — Start Here

Welcome to LoamSpine! This guide will get you up to speed quickly.

---

## 📖 What is LoamSpine?

LoamSpine is the **Permanence Layer** for Phase 2 of ecoPrimals. It's the "fossil record" — the canonical source of truth for everything that matters. Unlike RhizoCrypt's ephemeral working memory, committing to LoamSpine is a deliberate act that makes data permanent.

**Key insight**: LoamSpine embodies selective remembering — the complement to RhizoCrypt's philosophy of forgetting.

---

## 🚀 Quick Start

### 1. Build the Project

```bash
cd /path/to/ecoPrimals/phase2/loamSpine
cargo build
```

### 2. Run Tests

```bash
cargo test
```

### 3. Explore the Code

```bash
# Main entry point
cat crates/loam-spine-core/src/lib.rs

# Configuration
cat crates/loam-spine-core/src/config.rs

# Error types
cat crates/loam-spine-core/src/error.rs
```

---

## 🏗️ Architecture Overview

```
LoamSpine
    │
    ├── Spines (sovereign ledgers)
    │   ├── Entries (sequential chain)
    │   │   ├── Session commits (from RhizoCrypt)
    │   │   ├── Certificate operations
    │   │   └── Spine anchors (recursive)
    │   │
    │   ├── Owner (BearDog DID)
    │   └── Head (latest entry hash)
    │
    ├── Certificates (ownership model)
    │   ├── Mint (creation)
    │   ├── Transfer (ownership change)
    │   ├── Loan (temporary lending)
    │   └── Return (loan completion)
    │
    └── Federation (replication)
        ├── Spine export
        └── Peer sync
```

---

## 📚 Key Concepts

### 1. Spines
A sovereign, append-only ledger:
- **Owned** — Each spine has a DID owner
- **Sequential** — Entries form a chain
- **Signed** — Every entry is signed
- **Sovereign** — You control your history

### 2. Entries
The fundamental record type:
- **Indexed** — Sequential position in spine
- **Chained** — References previous entry hash
- **Typed** — Session commit, certificate, anchor
- **Signed** — Non-repudiable

### 3. Certificates
Digital ownership with lending:
- **Mint** — Create new certificate
- **Transfer** — Permanent ownership change
- **Loan** — Temporary access grant
- **Return** — Loan completion

### 4. Recursive Stacking
Spines can reference other spines:
```
Global Commons Spine
    ↑
Community Spines (hash anchors)
    ↑
Personal Spines
```

---

## 📂 Project Structure

```
loamSpine/
├── Cargo.toml           # Workspace manifest
├── README.md            # Overview
├── STATUS.md            # Current status
├── WHATS_NEXT.md        # Roadmap
├── START_HERE.md        # This file
│
├── crates/
│   └── loam-spine-core/     # Core library
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs       # Entry + traits
│           ├── config.rs    # Configuration
│           └── error.rs     # Error types
│
├── specs/
│   └── LOAMSPINE_SPECIFICATION.md  # Full spec (~900 lines)
│
└── showcase/            # Demo applications (coming soon)
```

---

## 🔗 Integration Points

### Depends On (Gen 1)
| Primal | Purpose |
|--------|---------|
| **BearDog** | Entry signing, DID ownership |
| **Songbird** | Service discovery |
| **NestGate** | Content storage for payloads |

### Phase 2 Siblings
| Primal | Relationship |
|--------|--------------|
| **RhizoCrypt** | Sends dehydrated commits |
| **SweetGrass** | Receives commit events for Braids |

---

## 🎯 Current Status

| Aspect | Status |
|--------|--------|
| **Scaffolding** | ✅ Complete |
| **Build** | ✅ Passing |
| **Entry Types** | ⬜ Not started |
| **Certificate Model** | ⬜ Not started |
| **Spine Storage** | ⬜ Not started |

See [STATUS.md](./STATUS.md) for detailed status.

---

## 📝 Next Steps for Contributors

### Immediate (Week 3)
1. Implement `EntryHash` type
2. Implement `LoamEntry` struct
3. Add entry chaining
4. BearDog signing integration

### Short Term (Weeks 4-6)
1. Implement `Spine` struct
2. Implement certificate operations
3. Implement commit acceptance from RhizoCrypt

See [WHATS_NEXT.md](./WHATS_NEXT.md) for full roadmap.

---

## 📖 Further Reading

| Document | Description |
|----------|-------------|
| [specs/LOAMSPINE_SPECIFICATION.md](./specs/LOAMSPINE_SPECIFICATION.md) | Complete technical specification |
| [../ARCHITECTURE.md](../ARCHITECTURE.md) | Unified Phase 2 architecture |
| [../INTEGRATION_OVERVIEW.md](../INTEGRATION_OVERVIEW.md) | Cross-primal data flows |
| [../sourDough/CONVENTIONS.md](../sourDough/CONVENTIONS.md) | Coding conventions |

---

## 💡 The Museum Analogy

From the specification:

> If RhizoCrypt is the workshop where creative chaos happens, LoamSpine is the museum where finished works are preserved:
>
> | RhizoCrypt | LoamSpine |
> |------------|-----------|
> | Every sketch, draft, iteration | The final masterpiece |
> | Every shot fired in a raid | The validated extraction |
> | Every experimental result | The published finding |
> | Working memory | Permanent record |

---

## 💡 Loam Certificates

Powerful ownership model examples:

### Digital Game Keys
```
Publisher mints "hl3-key-001"
    ↓
Publisher sells to Retailer
    ↓
Retailer sells to Player
    ↓
Player loans to Friend (48h, auto-return)
    ↓
Loan expires → Player owns again
```

### Collectibles with Provenance
```
"This deck won the championship"
    ↓
Tournament victory recorded in spine
    ↓
Deck certificate references tournament entry
    ↓
Future owners can verify provenance
```

---

## ❓ Questions?

- Check [STATUS.md](./STATUS.md) for current state
- Check [WHATS_NEXT.md](./WHATS_NEXT.md) for roadmap
- Read the [specification](./specs/LOAMSPINE_SPECIFICATION.md) for deep details

---

*LoamSpine: Where memories become permanent.*

