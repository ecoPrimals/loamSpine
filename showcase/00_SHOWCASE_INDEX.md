# 🦴 LoamSpine Showcase Index

**Version**: 0.8.3  
**Last Updated**: March 14, 2026  
**Status**: ✅ Production Ready (matches Squirrel's excellence!)

---

## 🎯 Start Here

**New to LoamSpine?** Choose your entry point:

| Entry Point | Time | Audience | Command |
|-------------|------|----------|---------|
| **Quick Demo** | 5 min | Everyone | `./QUICK_DEMO.sh` |
| **Complete Walkthrough** | 2.5 hours | Beginners | `./RUN_ME_FIRST.sh` |
| **Full Navigation** | - | All | Read this file |
| **Level 1 Only** | 60 min | Developers | `cd 01-local-primal && ./RUN_ALL.sh` |

**Pro Tip**: Start with `00_START_HERE.md` for orientation!

---

## 📁 Structure

```
showcase/
├── 00_START_HERE.md ✨ NEW       # 5-minute orientation (all personas)
├── QUICK_DEMO.sh ✨ NEW           # 5-minute highlight reel (executable)
├── RUN_ME_FIRST.sh ✨ NEW         # Progressive walkthrough (executable)
├── 00_SHOWCASE_INDEX.md           # This file (navigation)
├── README.md                      # Main showcase overview
├── SHOWCASE_PRINCIPLES.md         # No mocks philosophy
├── SHOWCASE_ANALYSIS_DEC_27_2025.md ✨ NEW  # Gap analysis
├── SHOWCASE_EVOLUTION_COMPLETE_DEC_27_2025.md ✨ NEW  # Completion report
│
├── 01-local-primal/               # Level 1: LoamSpine BY ITSELF (60 min)
│   ├── 01-hello-loamspine/        ✅ Complete
│   ├── 02-entry-types/            ✅ Complete
│   ├── 03-certificate-lifecycle/  ✅ Complete
│   ├── 04-proofs/                 ✅ Complete
│   ├── 05-backup-restore/         ✅ Complete
│   ├── 06-storage-backends/       ✅ Complete
│   ├── 07-concurrent-ops/         ✅ Complete
│   └── RUN_ALL.sh
│
├── 02-rpc-api/                    # Level 2: Pure Rust RPC (30 min)
│   ├── 01-tarpc-basics/           ✅ Ready (service exists)
│   ├── 02-jsonrpc-basics/         ✅ Ready
│   ├── 03-health-monitoring/      ✅ Ready
│   ├── 04-concurrent-ops/         ✅ Ready
│   ├── 05-error-handling/         ✅ Ready
│   └── RUN_ALL.sh
│
├── 03-songbird-discovery/         # Level 3: Service Discovery (20 min)
│   ├── 01-songbird-connect/       ✅ Ready (needs Songbird)
│   ├── 02-capability-discovery/   ✅ Ready
│   ├── 03-auto-advertise/         ✅ Ready
│   ├── 04-heartbeat-monitoring/   ✅ Ready
│   └── RUN_ALL.sh
│
└── 04-inter-primal/               # Level 4: Ecosystem Integration (45 min)
    ├── 01-beardog-signing/        ✅ Ready (NO MOCKS, uses ../bins/)
    ├── 02-nestgate-storage/       ✅ Ready (NO MOCKS)
    ├── 03-squirrel-sessions/      ✅ Ready (NO MOCKS)
    ├── 04-toadstool-compute/      ✅ Ready (NO MOCKS)
    ├── 05-full-ecosystem/         ✅ Ready (NO MOCKS)
    └── RUN_ALL.sh
```

---

## 🚀 Quick Start

### Option A: 5-Minute Quick Demo
```bash
cd showcase
./QUICK_DEMO.sh
```

Shows: Spine creation → Certificates → Proofs → RPC API

### Option B: Complete Progressive Walkthrough
```bash
cd showcase
./RUN_ME_FIRST.sh
```

Interactive menu with all 4 levels

### Option C: Individual Level
```bash
# Level 1: Local primal capabilities
cd showcase/01-local-primal
./RUN_ALL.sh

# Level 2: RPC API
cd showcase/02-rpc-api
./RUN_ALL.sh

# Level 3: Songbird discovery
cd showcase/03-songbird-discovery
./RUN_ALL.sh

# Level 4: Inter-primal integration
cd showcase/04-inter-primal
./RUN_ALL.sh
```

### Option D: Specific Demo
```bash
# Navigate to any demo directory
cd showcase/01-local-primal/01-hello-loamspine
./demo.sh
```

Or run examples directly:

```bash
cargo run --example demo_hello_loamspine
cargo run --example entry_types
cargo run --example certificate_lifecycle
cargo run --example proofs
cargo run --example backup_restore
```

---

## 🎓 Progressive Learning Paths

### For Complete Beginners (120 min)
Perfect if you've never used LoamSpine:
```
Level 1 (60 min) → Level 2 (30 min) → Level 3 (20 min) → Level 4 (10 min)
```
**Start**: `00_START_HERE.md`

### For Developers (70 min)
You want to understand the architecture:
```
Level 1 highlights (30 min) → Level 2 (30 min) → Level 4 (10 min)
```
**Start**: `01-local-primal/01-hello-loamspine/`

### For Architects (60 min)
You want to see integration patterns:
```
Certificates (10 min) → RPC (10 min) → Discovery (20 min) → Ecosystem (20 min)
```
**Start**: `01-local-primal/03-certificate-lifecycle/`

### For Ecosystem Contributors (180+ min)
Complete mastery of all patterns:
```
All levels complete + code review + contribution
```
**Start**: This file + all READMEs

---

## 📊 Available Examples

| Example | Description | Phase | Crate |
|---------|-------------|-------|-------|
| `demo_hello_loamspine` | First spine creation | 1 | loam-spine-core |
| `demo_entry_types` | Entry type variants | 1 | loam-spine-core |
| `demo_certificate_lifecycle` | Mint → Transfer → Loan → Return | 1 | loam-spine-core |
| `demo_backup_restore` | Export/import with verification | 1 | loam-spine-core |
| `demo_rpc_service` | Pure Rust RPC API | 2 | loam-spine-api |
| `demo_inter_primal` | Integration traits | 3 | loam-spine-core |

---

## 📋 Level Details

### Level 1: Local Primal (60 min) ✅ 7/7 COMPLETE
**Goal**: Understand LoamSpine standalone

| Demo | Description | Time | Status |
|------|-------------|------|--------|
| 01-hello-loamspine | First spine creation | 5 min | ✅ Complete |
| 02-entry-types | All 15+ entry types | 15 min | ✅ Complete |
| 03-certificate-lifecycle | Mint → Transfer → Loan | 20 min | ✅ Complete |
| 04-proofs | Inclusion & provenance | 15 min | ✅ Complete |
| 05-backup-restore | Export & import | 10 min | ✅ Complete |
| 06-storage-backends | InMemory vs redb vs Sled | 10 min | ✅ Complete |
| 07-concurrent-ops | Thread-safe operations | 10 min | ✅ Complete |

**What you'll learn**:
- Sovereign spine creation
- Entry types and chaining
- Certificate ownership
- Cryptographic proofs
- Data persistence

---

### Level 2: RPC API (30 min) ✅ 5/5 READY
**Goal**: See Pure Rust RPC (no gRPC!)

| Demo | Description | Time | Status |
|------|-------------|------|--------|
| 01-tarpc-basics | Binary RPC basics | 10 min | ✅ Ready |
| 02-jsonrpc-basics | JSON-RPC 2.0 API | 10 min | ✅ Ready |
| 03-health-monitoring | Service health | 5 min | ✅ Ready |
| 04-concurrent-ops | Parallel RPC calls | 10 min | ✅ Ready |
| 05-error-handling | Graceful degradation | 5 min | ✅ Ready |

**What you'll learn**:
- tarpc for primal-to-primal
- JSON-RPC for external clients
- Service health monitoring
- Concurrent operations

**Prerequisites**: `loamspine` binary

---

### Level 3: Songbird Discovery (20 min) ✅ 4/4 READY
**Goal**: See runtime service discovery

| Demo | Description | Time | Status |
|------|-------------|------|--------|
| 01-songbird-connect | Service registration | 5 min | ✅ Ready |
| 02-capability-discovery | Runtime discovery | 5 min | ✅ Ready |
| 03-auto-advertise | Capability advertisement | 5 min | ✅ Ready |
| 04-heartbeat-monitoring | Health & failover | 5 min | ✅ Ready |

**What you'll learn**:
- Capability registration
- Runtime discovery
- Zero hardcoding
- Automatic failover

**Prerequisites**: Songbird (`../bins/songbird-orchestrator`)

---

### Level 4: Inter-Primal (45 min) ✅ 5/5 READY (NO MOCKS!)
**Goal**: Complete ecosystem integration

| Demo | Description | Time | Status |
|------|-------------|------|--------|
| 01-beardog-signing | Cryptographic trust | 10 min | ✅ Ready |
| 02-nestgate-storage | Sovereign storage | 10 min | ✅ Ready |
| 03-squirrel-sessions | AI session anchoring | 10 min | ✅ Ready |
| 04-toadstool-compute | Verifiable compute | 10 min | ✅ Ready |
| 05-full-ecosystem | ALL primals together | 15 min | ✅ Ready |

**What you'll learn**:
- BearDog signing integration
- NestGate storage integration
- Squirrel session commits
- ToadStool compute verification
- Complete ecosystem coordination

**Prerequisites**: All binaries in `../bins/` (beardog, nestgate, squirrel, toadstool-byob-server, songbird-orchestrator)

**Philosophy**: NO MOCKS! Uses real binaries from `../bins/`

---

## 🏆 Success Criteria

### After Level 1
- [ ] I can create sovereign spines
- [ ] I understand all entry types
- [ ] I can mint and transfer certificates
- [ ] I can generate cryptographic proofs
- [ ] I can backup and restore spines

### After Level 2
- [ ] I understand Pure Rust RPC benefits
- [ ] I can use tarpc for primal-to-primal
- [ ] I can use JSON-RPC for external clients
- [ ] I can monitor service health

### After Level 3
- [ ] I understand runtime service discovery
- [ ] I see zero-hardcoding in action
- [ ] I can register capabilities
- [ ] I understand automatic failover

### After Level 4
- [ ] I can integrate with BearDog
- [ ] I can integrate with NestGate
- [ ] I can integrate with Squirrel
- [ ] I can integrate with ToadStool
- [ ] I understand complete ecosystem coordination

---

## 🔧 Prerequisites

### Minimum (Level 1)
- Rust 1.75+
- `cargo build --release`
- That's it!

### Recommended (Level 2)
- `cargo build --release --bin loamspine`

### Optional (Level 3)
- Songbird binary in `../bins/`

### Optional (Level 4)
- All Phase 1 primal binaries in `../bins/`
- See `../bins/README.md` for details

---

## 🔗 Capability-Based Integration

LoamSpine uses capability discovery for all external services:

| Capability | Discovery Method | Description |
|------------|------------------|-------------|
| Signing | Songbird discovery | Ed25519 signing service |
| Storage | Songbird discovery | Content-addressable storage |
| Discovery | DNS SRV + mDNS | Service mesh integration |

**Philosophy**: *No primal names are hardcoded - all services discovered at runtime.*

---

## 📚 References

### In This Showcase
- **[00_START_HERE.md](./00_START_HERE.md)** ✨ NEW - Main entry point
- **[QUICK_DEMO.sh](./QUICK_DEMO.sh)** ✨ NEW - 5-minute demo
- **[RUN_ME_FIRST.sh](./RUN_ME_FIRST.sh)** ✨ NEW - Progressive walkthrough
- **[SHOWCASE_ANALYSIS_DEC_27_2025.md](./SHOWCASE_ANALYSIS_DEC_27_2025.md)** ✨ NEW - Gap analysis
- **[SHOWCASE_EVOLUTION_COMPLETE_DEC_27_2025.md](./SHOWCASE_EVOLUTION_COMPLETE_DEC_27_2025.md)** ✨ NEW - Completion report
- [README.md](./README.md) - Main overview
- [SHOWCASE_PRINCIPLES.md](./SHOWCASE_PRINCIPLES.md) - No mocks philosophy
- [QUICK_REFERENCE.md](./QUICK_REFERENCE.md) - Quick reference card

### In Repository Root
- [../README.md](../README.md) - LoamSpine overview
- [../STATUS.md](../STATUS.md) - Project status (A+ 98/100)
- [../ROOT_DOCS_INDEX.md](../ROOT_DOCS_INDEX.md) - Documentation index
- [../specs/](../specs/) - Complete specifications (8,400+ lines)

### Ecosystem Showcases
- `../../phase1/squirrel/showcase/` - AI orchestration (EXCELLENT model)
- `../../phase1/songbird/showcase/` - Service mesh (if exists)
- `../../phase1/toadstool/showcase/` - Universal compute (if exists)
- `../../phase1/nestgate/showcase/` - Storage infrastructure (if exists)
- `../../phase1/beardog/showcase/` - Sovereign security (if exists)

---

## 🎉 Showcase Evolution

**Before** (Dec 24, 2025):
- ✅ Level 1 complete (7/7)
- ❌ No entry points
- ❌ No quick demo
- ⚠️ Some mocks in Level 4

**After** (Dec 27, 2025):
- ✅ All 4 levels ready (21/21)
- ✅ 3 entry points (START_HERE, QUICK_DEMO, RUN_ME_FIRST)
- ✅ Progressive learning paths
- ✅ NO MOCKS policy enforced
- ✅ Real binaries from `../bins/`
- ✅ Matches Squirrel's excellence!

**Status**: PRODUCTION READY 🚀

---

## 🌟 The LoamSpine Promise

> **"See sovereign permanence in action — no mocks, just real capabilities anchoring ephemeral operations into eternal truth."**

**Following the ecoPrimals showcase pattern**:
- 🎵 Songbird: Multi-tower federation
- 🍄 ToadStool: GPU compute benchmarks
- 🐻 BearDog: Interactive security demos
- 🏰 NestGate: Progressive storage levels
- 🐿️ Squirrel: Universal AI orchestration (EXCELLENT model)
- 🦴 **LoamSpine: Sovereign permanence** (NOW MATCHES EXCELLENCE!)

---

🦴 **LoamSpine: Where memories become permanent.** 🚀

*Last updated: March 14, 2026 — Showcase Evolution Complete*
