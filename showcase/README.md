# 🦴 LoamSpine Showcase — Progressive Capability Demonstrations

**Purpose**: Demonstrate LoamSpine's permanence layer capabilities  
**Philosophy**: Progressive complexity — isolated → API → inter-primal  
**Goal**: Show sovereign ledgers, certificates, and primal integration

---

## 🎯 Showcase Philosophy

This showcase demonstrates LoamSpine's evolution:

1. **Local Primal**: Spine, entries, certificates by themselves
2. **RPC API**: tarpc + JSON-RPC for external access
3. **Inter-Primal**: Integration with ephemeral, attribution, and signing services

**Real-World Scenario**: *"Your game saves get committed to permanent, sovereign history with full provenance"*

---

## 📁 Structure

```
showcase/
├── 00_SHOWCASE_INDEX.md        # This index
├── SHOWCASE_PRINCIPLES.md      # No mocks, real capabilities only
│
├── 01-local-primal/            # LoamSpine BY ITSELF
│   ├── 01-hello-loamspine/     # Basic spine creation
│   ├── 02-entry-types/         # All 15+ entry types
│   ├── 03-certificate-lifecycle/# Mint → Transfer → Loan → Return
│   ├── 04-proofs/              # Inclusion and provenance proofs
│   ├── 05-backup-restore/      # Binary + JSON backup
│   ├── 06-storage-backends/    # InMemory + redb + Sled
│   ├── 07-concurrent-ops/      # Thread-safe operations
│   ├── 08-temporal-moments/    # Temporal moment recording
│   ├── 09-waypoint-anchoring/  # Waypoint slice anchoring
│   └── 10-recursive-spines/    # Recursive spine operations
│
├── 02-rpc-api/                 # Pure Rust RPC Demos
│   ├── 01-tarpc-basics/        # Binary RPC (primal-to-primal)
│   ├── 02-jsonrpc-basics/      # JSON-RPC 2.0 (external clients)
│   ├── 03-health-monitoring/   # Health checks and metrics
│   ├── 04-concurrent-ops/      # Parallel operations
│   └── 05-error-handling/      # Graceful degradation
│
├── 03-inter-primal/            # Cross-Primal Integration
│   ├── 01-beardog-signing/     # BearDog signing → LoamSpine certs
│   ├── 02-nestgate-storage/    # NestGate braids → LoamSpine anchoring
│   ├── 03-squirrel-sessions/   # Squirrel sessions → LoamSpine persistence
│   ├── 04-toadstool-compute/   # Toadstool uses LoamSpine storage
│   └── 05-full-ecosystem/      # Complete integration demo
│
└── scripts/                    # External Service Integration
    ├── start_primals.sh        # Start external services
    ├── stop_primals.sh         # Stop external services
    ├── demo_signing_service.sh # Signing service integration (agnostic)
    └── demo_storage_service.sh # Storage service integration (agnostic)
```

---

## 🚀 Quick Start

```bash
# 1. Build LoamSpine
cd /path/to/loamSpine
cargo build --release

# 2. Run tests to verify everything works
cargo test

# 3. Start the showcase
cd showcase
./QUICK_START.sh              # Core demos only
./QUICK_START.sh --with-phase1  # Include Phase 1 primal integration
./QUICK_START.sh --fast         # Skip pauses between demos
```

---

## 📋 Progressive Levels

### Phase 1: Local Primal Capabilities (60-90 min)

**LoamSpine BY ITSELF is powerful:**

| Demo | Description | Time |
|------|-------------|------|
| Hello LoamSpine | Create your first spine | 5 min |
| Entry Types | All 15+ entry type variants | 15 min |
| Certificate Lifecycle | Mint, transfer, loan, return | 20 min |
| Proofs | Inclusion and provenance proofs | 15 min |
| Backup/Restore | Export and import spines | 10 min |
| Storage Backends | InMemory vs redb vs Sled | 10 min |

### Phase 2: RPC API (30-45 min)

**Pure Rust RPC — No gRPC, no protobuf:**

| Demo | Description | Time |
|------|-------------|------|
| tarpc Basics | High-performance binary RPC | 10 min |
| JSON-RPC 2.0 | Language-agnostic external API | 10 min |
| Health Monitoring | Service health and metrics | 5 min |
| Concurrent Ops | Parallel spine operations | 10 min |

### Phase 3: Inter-Primal Integration (45-60 min)

**LoamSpine in the ecosystem:**

| Demo | Description | Time |
|------|-------------|------|
| Session Commit | Session dehydration to permanence | 15 min |
| Braid Commit | Semantic attribution commits | 15 min |
| Signing Capability | Capability-based cryptography | 15 min |
| Full Ecosystem | Complete primal coordination | 20 min |

### Phase 4: Phase 1 Primal Integration (--with-phase1)

**Real binaries from `../bins/`:**

| Demo | Description | Time |
|------|-------------|------|
| Signing Service | Ed25519 key generation, HSM discovery | 5 min |
| Storage Service | Content-addressable storage patterns | 5 min |
| Cross-Primal Messaging | Secure messaging | 5 min |

**Total**: ~3.5 hours for complete showcase with Phase 1 integration

---

## 🎓 Learning Path

### For New Users
1. Start with Phase 1 to understand LoamSpine basics
2. Progress to Phase 2 to see RPC capabilities
3. Explore Phase 3 for primal integration patterns

### For Operators
1. Review Phase 2 for deployment patterns
2. Study storage backends for production setup
3. Use backup/restore for data management

### For Developers
1. Study Rust examples in each phase
2. Review `crates/` source code
3. Extend demos with custom entry types

---

## 🏆 Success Criteria

### Phase 1 Complete When:
- [x] Spine created with owner DID
- [x] Entries appended and verified
- [x] Certificates minted and transferred
- [x] Loans created with terms
- [x] Proofs generated and verified
- [x] Backup exported and restored

### Phase 2 Complete When:
- [x] tarpc server responds to RPCs
- [x] JSON-RPC 2.0 accessible via curl
- [x] Health checks return healthy
- [x] Concurrent operations succeed

### Phase 3 Complete When:
- [ ] Ephemeral sessions commit to spine
- [ ] Attribution braids commit to spine
- [ ] Signing via capability registry
- [ ] Full ecosystem integration works

---

## 💡 Featured Demo: Game Save Permanence

**Scenario**: A game session needs to be committed to permanent history

**What Happens**:
1. Ephemeral primal dehydrates session to merkle root
2. LoamSpine receives commit via `CommitAcceptor`
3. Entry created with `SessionCommit` type
4. Entry signed via signing capability (if available)
5. Entry appended to player's spine
6. Certificate minted for game achievement
7. Full provenance available forever

**Demo**: `03-inter-primal/03-squirrel-sessions/demo.sh`

---

## 🛠️ Why Pure Rust RPC?

| ❌ gRPC Problems | ✅ Our Solution |
|-----------------|-----------------|
| Requires `protoc` (C++ compiler) | Pure Rust with tarpc macros |
| Requires protobuf (Google tooling) | Native serde serialization |
| Non-Rust code generation | Rust procedural macros |
| Vendor lock-in | Community-driven development |
| Complex build process | Standard `cargo build` |

**Dual Protocol Strategy:**
- **tarpc** — High-performance binary RPC for primal-to-primal
- **JSON-RPC 2.0** — Universal, language-agnostic for external clients

---

## 📊 Current Metrics

| Metric | Value |
|--------|-------|
| **Tests** | 1,486 passing (all concurrent, ~3s, zero flaky) |
| **Coverage** | ~91% line (llvm-cov) |
| **JSON-RPC Methods** | 37 (semantic naming) |
| **Entry Types** | 15+ |
| **Clippy** | pedantic + nursery (0 warnings) |
| **Unsafe** | 0 (`#![forbid(unsafe_code)]`) |

---

## 🎯 Showcase Principles

**No mocks, real capabilities only!**

See [SHOWCASE_PRINCIPLES.md](./SHOWCASE_PRINCIPLES.md) for details.

- ✅ Demonstrate features that are actually implemented
- ✅ Use real APIs and real data
- ✅ Check for capability availability before running
- ✅ Exit gracefully when capabilities are missing
- ❌ No mock or simulated features
- ❌ No fake API responses
- ❌ No placeholder data

---

## 🔗 Integration Points

### Capability-Based Integration
| Capability | Relationship |
|------------|--------------|
| **Ephemeral Storage** | Sends session commits via `CommitAcceptor` |
| **Semantic Attribution** | Sends braids via `BraidAcceptor` |
| **Signing Service** | Entry signing via capability registry |
| **Discovery Service** | Service discovery |
| **Storage Service** | Payload storage |

*Note: LoamSpine discovers these capabilities at runtime via environment variables and the capability registry. No primal names are hardcoded.*

---

## 📚 References

- **Specifications**: `../specs/`
- **Architecture**: `../specs/ARCHITECTURE.md`
- **API Reference**: `cargo doc --open`
- **Status**: `../STATUS.md`
- **Contributing**: `../CONTRIBUTING.md`

---

**Ready to explore?** Start with `01-local-primal/01-hello-loamspine/`

🦴 **LoamSpine: Where memories become permanent.**

