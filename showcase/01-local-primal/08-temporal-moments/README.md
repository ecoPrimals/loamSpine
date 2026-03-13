# ⏰ Temporal Moments Demo

**NEW in v0.7.0**: Universal time tracking primitives

---

## 🎯 Purpose

Demonstrate LoamSpine's unique temporal primitives that allow tracking time across ANY domain.

**Philosophy**: "Time is the primitive, not version control."

---

## 🚀 Quick Run

```bash
./demo.sh
```

Or directly:
```bash
cargo run --example temporal_moments
```

---

## 🎓 What This Demo Shows

### 1. Moment Structure
Every moment in LoamSpine has:
- **ID**: Unique identifier (content hash)
- **Timestamp**: When it occurred
- **Agent**: Who/what created it (DID)
- **State Hash**: Cryptographic state fingerprint
- **Context**: Domain-specific data
- **Parents**: Previous moments (causal chain)
- **Anchor**: Time anchoring mechanism (optional)
- **Signature**: Cryptographic proof

### 2. Moment Types

**Code Commit**:
```rust
MomentContext::CodeChange {
    message: "Initial commit of temporal module",
    tree_hash: ContentHash,
}
```

**Art Creation**:
```rust
MomentContext::ArtCreation {
    title: "Starry Night in Digital Space",
    medium: "Digital Painting",
    content_hash: ContentHash,
}
```

**Life Event**:
```rust
MomentContext::LifeEvent {
    event_type: "graduation",
    participants: vec![alice_did, university_did],
    description: "Graduated with honors",
}
```

**Scientific Experiment**:
```rust
MomentContext::Experiment {
    hypothesis: "Rust is the best language",
    result: "Success!",
    data_hash: ContentHash,
}
```

**Business Milestone**:
```rust
MomentContext::Milestone {
    achievement: "Reached 1M users",
    metrics: HashMap<String, f64>,
}
```

### 3. Anchor Types

**Atomic Time** (local system clock):
```rust
Anchor::Atomic(AtomicAnchor {
    timestamp: SystemTime::now(),
    precision: TimePrecision::Nanosecond,
    source: "local_system_clock",
})
```

**Crypto Anchor** (blockchain timestamp):
```rust
Anchor::Crypto(CryptoAnchor {
    chain: "ethereum",
    block_number: 12345678,
    transaction_hash: "0xabc...",
})
```

**Causal Anchor** (Lamport/vector clocks):
```rust
Anchor::Causal(CausalAnchor {
    lamport_time: 42,
    vector_clock: HashMap<NodeId, u64>,
})
```

**Consensus Anchor** (distributed agreement):
```rust
Anchor::Consensus(ConsensusAnchor {
    protocol: "raft",
    epoch: 7,
    leader: NodeId,
})
```

### 4. Spine Integration

Moments are stored as entries in a spine:
```rust
let moment = Moment {
    id: ContentHash::generate(...),
    timestamp: SystemTime::now(),
    agent: user_did,
    context: MomentContext::CodeChange { ... },
    // ...
};

let entry = spine.create_temporal_moment_entry(moment);
spine.append(entry)?;
```

---

## 🎯 Use Cases

### Version Control (Code)
Track every code change with full context:
- Commit messages
- Tree hashes
- Author DIDs
- Timestamps
- Causal relationships

### Art Provenance
Track creative works:
- Creation moment
- Artist DID
- Medium and technique
- Content hash
- Ownership history

### Research Logging
Document scientific process:
- Hypothesis
- Experimental data
- Results
- Methodology
- Peer review

### Life Events
Personal timeline:
- Graduations
- Achievements
- Relationships
- Locations
- Participants

### Business Milestones
Organizational history:
- Product launches
- Revenue milestones
- Team growth
- Strategic decisions

---

## 🔍 Key Concepts

### Universal Time Layer

**Problem**: Every domain reinvents time tracking
- Git: commits with timestamps
- Photo apps: EXIF data
- Journals: dated entries
- Ledgers: transaction times

**Solution**: LoamSpine provides universal temporal primitives
- One time model for all domains
- Composable moments
- Cryptographic proofs
- Causal relationships

### Moment Categories

```rust
impl MomentContext {
    pub fn category(&self) -> &str {
        match self {
            Self::CodeChange { .. } => "code",
            Self::ArtCreation { .. } => "art",
            Self::LifeEvent { .. } => "life",
            Self::Experiment { .. } => "experiment",
            Self::Milestone { .. } => "milestone",
            Self::Generic { category, .. } => category,
        }
    }
}
```

### Temporal Chaining

Moments can reference previous moments:
```rust
let moment2 = Moment {
    parents: vec![moment1.id.clone()],
    // Creates causal chain
    ...
};
```

### Anchor Flexibility

Choose anchoring based on needs:
- **Atomic**: Fast, local, nanosecond precision
- **Crypto**: Blockchain-anchored, immutable
- **Causal**: Distributed systems, causality
- **Consensus**: Multi-party agreement

---

## 💡 Why This Matters

### 1. Universal Applicability
One time model works for:
- Software development
- Creative arts
- Personal history
- Scientific research
- Business records

### 2. Cryptographic Proof
Every moment is:
- Content-addressed
- Cryptographically signed
- Causally linked
- Verifiable

### 3. Sovereign History
Your moments, your control:
- No vendor lock-in
- Export/import freely
- Verify independently
- Own your timeline

### 4. Interoperability
Moments can reference:
- NestGate content hashes
- BearDog signatures
- ToadStool compute tasks
- Squirrel session IDs

---

## 🎯 Next Steps

After this demo, explore:

**Waypoint Anchoring** (`09-waypoint-anchoring/`)
- Spine slices
- Waypoint proofs
- Temporal ranges

**Recursive Spines** (`10-recursive-spines/`)
- Spine-to-spine references
- Composition patterns
- Cross-spine queries

**Service API** (`../02-rpc-service/`)
- Create moments via RPC
- Query temporal history
- Subscribe to moment streams

**Ecosystem Integration** (`../04-inter-primal/`)
- Sign moments with BearDog
- Store moment data in NestGate
- Discover moments via Songbird

---

## 📊 Performance

**Moment Creation**: < 1ms  
**Signature**: < 10ms (Ed25519)  
**Verification**: < 5ms  
**Storage**: ~200 bytes per moment  

**Scales to**:
- 1M+ moments per spine
- 100K+ moments/second append rate
- Sub-millisecond queries

---

## 🏆 Success Criteria

After this demo, you should understand:

- ✅ What temporal moments are
- ✅ The different moment types
- ✅ How anchoring works
- ✅ Why time is a primitive
- ✅ Universal applicability
- ✅ Integration with spines

---

**🦴 LoamSpine: Where time is universal, and memories are permanent.**

