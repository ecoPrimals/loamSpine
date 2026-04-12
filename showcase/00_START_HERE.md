# 🦴 LoamSpine Showcase - Start Here

**Sovereign Permanence for the ecoPrimals Ecosystem**

**Version**: 0.9.16  
**Status**: Production Ready  
**Date**: April 11, 2026  
**Latest**: BTSP Phase 2 handshake, deep debt overhaul, dependency evolution — 1,507 tests, 92% line coverage, `#![forbid(unsafe_code)]` in v0.9.16

---

## 🎯 What Is This?

This showcase demonstrates **LoamSpine** - the permanent ledger primal that provides:

✅ **Sovereign Spines** - You own your history forever  
✅ **15+ Entry Types** - Sessions, certificates, proofs, braids, and more  
✅ **Temporal Primitives** - NEW! Moments, Epochs, Eras for universal time tracking  
✅ **Cryptographic Proofs** - Inclusion and provenance guarantees  
✅ **Pure Rust RPC** - tarpc + JSON-RPC, no gRPC/protobuf  
✅ **Ecosystem Integration** - Anchors ephemeral operations permanently  
✅ **Zero Hardcoding** - Dynamic capability discovery

---

## 🚀 Quick Start (5 Minutes)

### Option A: Just Show Me Something!
```bash
# Run the quickest demo (works without any setup)
cd showcase/01-local-primal/01-hello-loamspine
./demo.sh

# Or use the quick demo script
cd showcase
./QUICK_DEMO.sh
```

### Option B: I Want the Full Picture
**Read this file** (you're already here!) then follow the [Progressive Learning Path](#-progressive-learning-path) below.

### Option C: I'm Ready to Deep Dive
```bash
# Start with Level 1: Local Primal Capabilities
cd showcase/01-local-primal
cat README.md
```

---

## 📚 Showcase Structure

This showcase is organized into **4 progressive levels**:

### Level 1: Local Primal (60-90 min) ⭐ START HERE
**Goal**: Understand LoamSpine as a standalone permanence layer

**What you'll see**:
- Sovereign spine creation
- All 15+ entry types in action
- Certificate lifecycle (mint, transfer, loan, return)
- Cryptographic proofs (inclusion, provenance)
- Backup and restore with verification
- Storage backends (InMemory + redb + Sled)
- Concurrent operations

**Path**: `01-local-primal/`  
**Status**: ✅ 7/7 demos complete

---

### Level 2: RPC API (30-45 min)
**Goal**: See Pure Rust RPC (no gRPC, no protobuf!)

**What you'll see**:
- tarpc basics (primal-to-primal binary RPC)
- JSON-RPC 2.0 (external client API)
- Health monitoring and metrics
- Concurrent RPC operations
- Error handling patterns

**Path**: `02-rpc-api/`  
**Status**: ✅ 5/5 demos complete

---

### Level 3: Inter-Primal Integration (45-60 min) 🌟
**Goal**: Complete ecosystem working together

**What you'll see**:
- BearDog signing → LoamSpine certificates
- NestGate storage → LoamSpine anchoring
- Squirrel sessions → LoamSpine commits
- ToadStool compute → LoamSpine proofs
- **Full Ecosystem** - All primals coordinated!

**Path**: `03-inter-primal/`  
**Status**: ✅ 5/5 demos complete (using real binaries from `../bins/`)

---

## 🎓 Progressive Learning Path

### For Complete Beginners (120 min)
Perfect if you've never used LoamSpine or the ecoPrimals

```
Step 1: Level 1 (60 min) → Understand LoamSpine standalone
Step 2: Level 2 (30 min) → See RPC API
Step 3: Level 3 (45 min) → Experience ecosystem integration
```

**Start**: `01-local-primal/README.md`

---

### For Developers (70 min)
You want to understand the architecture

```
Step 1: Level 1 - Demos 1-3 (30 min)
Step 2: Level 2 - RPC API (30 min)
Step 3: Level 3 - Inter-primal (10 min)
```

**Start**: `01-local-primal/01-hello-loamspine/`

---

### For Architects (60 min)
You want to see integration patterns

```
Step 1: Level 1 - Demo 3 (certificates) (10 min)
Step 2: Level 2 - RPC basics (10 min)
Step 3: Level 3 - Complete ecosystem (20 min)
```

**Start**: `01-local-primal/03-certificate-lifecycle/`

---

### For Ecosystem Contributors (180+ min)
Complete mastery of all patterns

```
Step 1-4: All levels complete
Step 5: Review code and architecture
Step 6: Contribute improvements
```

**Start**: `00_SHOWCASE_INDEX.md`

---

## 🎯 What You'll Learn

### After Level 1: Local Primal
- ✅ How to create and manage sovereign spines
- ✅ The different entry types and when to use them
- ✅ How certificate ownership works
- ✅ How to generate and verify cryptographic proofs
- ✅ How to backup and restore spines
- ✅ The difference between storage backends
- ✅ Why LoamSpine is valuable BY ITSELF

### After Level 2: RPC API
- ✅ How to use tarpc for primal-to-primal communication
- ✅ How to expose JSON-RPC 2.0 for external clients
- ✅ How to monitor service health
- ✅ How to handle concurrent RPC operations
- ✅ Why Pure Rust RPC beats gRPC/protobuf

### After Level 3: Inter-Primal
- ✅ How to anchor Squirrel sessions permanently
- ✅ How to store NestGate data with provenance
- ✅ How to use BearDog signing for certificates
- ✅ How to anchor ToadStool compute results
- ✅ How the complete ecosystem works together
- ✅ How to build production workflows

---

## 🌟 Featured Demos

### 1. Hello LoamSpine (Level 1) 🏠
**Why it matters**: Your first sovereign spine

```bash
cd 01-local-primal/01-hello-loamspine
./demo.sh
```

**What you'll see**:
- Create a spine with owner DID
- Add entries (Text, Metadata)
- Verify integrity
- View spine metadata

---

### 2. Certificate Lifecycle (Level 1) 🎫
**Why it matters**: NFT-like ownership without blockchain

```bash
cd 01-local-primal/03-certificate-lifecycle
./demo.sh
```

**What you'll see**:
- Mint a certificate
- Transfer ownership
- Loan with terms
- Return to owner
- Complete provenance

---

### 3. Pure Rust RPC (Level 2) 🔌
**Why it matters**: No gRPC, no protobuf, just Rust

```bash
cd 02-rpc-api/01-tarpc-basics
./demo.sh
```

**What you'll see**:
- Start LoamSpine service
- Make tarpc calls (binary RPC)
- Make JSON-RPC calls (curl)
- See performance benefits

---

### 4. Full Ecosystem (Level 4) 🤝
**Why it matters**: All primals working together

```bash
cd 03-inter-primal/05-full-ecosystem
./demo.sh
```

**What you'll see**:
- Capability-based discovery
- BearDog signing
- Squirrel sessions
- ToadStool compute
- LoamSpine anchoring everything

**This is the hero demo!** 🌟

---

## 🛠️ Prerequisites

### Minimum (For Level 1)
- ✅ Rust 1.85+ installed (edition 2024)
- ✅ LoamSpine built (`cargo build --release`)
- ✅ That's it! Level 1 works standalone

### Recommended (For Level 2+)
- ✅ LoamSpine binary (`loamspine server`)
- ✅ Songbird orchestrator for discovery (Level 3)
- ✅ Phase 1 primal binaries in `../bins/` (Level 4)

### Optional (For Level 4)
- ✅ BearDog (`../bins/beardog`)
- ✅ NestGate (`../bins/nestgate`)
- ✅ Squirrel (`../bins/squirrel`)
- ✅ ToadStool (`../bins/toadstool-byob-server`)
- ✅ Songbird (`../bins/songbird-orchestrator`)

### Verification
```bash
# Check Rust version
rustc --version  # Should be 1.85+ (edition 2024)

# Build LoamSpine
cargo build --release

# Run tests
cargo test --workspace

# Verify showcase scripts (manual check: ensure demos run)
cd showcase
# ./scripts/verify-prerequisites.sh  # (script not present; run demos to verify)
```

---

## 📖 Documentation Structure

### Entry Points
- **This File** (`00_START_HERE.md`) - Main entry point
- `00_SHOWCASE_INDEX.md` - Complete navigation
- `README.md` - Overview and capabilities
- `SHOWCASE_QUICK_REFERENCE_CARD.md` - Quick reference card

### Level Guides
- `01-local-primal/README.md` - Level 1 guide
- `02-rpc-api/README.md` - Level 2 guide
- `03-inter-primal/README.md` - Level 3 guide

### Project Documentation
- `../README.md` - Project overview and documentation
- `../STATUS.md` - Current status
- `../specs/` - Complete specifications

---

## 🎬 Quick Demo Scripts

### 5-Minute Overview
```bash
./QUICK_DEMO.sh
```

Shows: Spine creation → Certificates → RPC API → Ecosystem

### Complete Walkthrough
```bash
# Level 1 (60 min)
cd 01-local-primal && ./RUN_ALL.sh

# Level 2 (30 min)
cd ../02-rpc-api && ./RUN_ALL.sh

# Level 3 (45 min)
cd ../03-inter-primal && ./RUN_ALL.sh
```

### Specific Demo
```bash
# Navigate to any demo directory
cd 01-local-primal/01-hello-loamspine
./demo.sh
```

---

## 💡 Tips for Success

### 1. Start Simple
Begin with Level 1 even if you're experienced. It establishes the foundation.

### 2. Run Demos in Order
Each level builds on the previous. Don't skip ahead until you understand the current level.

### 3. No Mocks!
All demos use REAL capabilities. If a service isn't available, the demo will explain what's needed.

### 4. Read the READMEs
Each demo directory has a comprehensive README explaining what it demonstrates.

### 5. Experiment
All demos are safe. Feel free to modify and explore!

---

## 🆘 Troubleshooting

### "Build failed"
```bash
# Check Rust version
rustc --version  # Need 1.85+ (edition 2024)

# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### "Demo script failed"
1. Read the error message carefully
2. Check prerequisites for that demo
3. Verify services are running (if needed)
4. Check terminal output for details

### "Service not found"
```bash
# For Level 4 demos, verify binaries exist
ls -lh ../bins/

# If missing, they need to be built from phase1 primals
# See ../bins/README.md for instructions
```

### "I'm confused about the structure"
Start here: `00_SHOWCASE_INDEX.md` - it's a complete navigation guide.

---

## 🎯 Success Criteria

### After Level 1
- [ ] I understand what LoamSpine does
- [ ] I can create sovereign spines
- [ ] I understand entry types
- [ ] I can generate cryptographic proofs
- [ ] I understand certificate ownership

### After Level 2
- [ ] I see how Pure Rust RPC works
- [ ] I can use tarpc for primal-to-primal
- [ ] I can use JSON-RPC for external clients
- [ ] I understand RPC performance benefits

### After Level 3
- [ ] I understand complete ecosystem integration
- [ ] I can anchor ephemeral operations permanently
- [ ] I see sovereign permanence value
- [ ] I can build production workflows

---

## 🚀 Ready? Choose Your Path:

### Fastest: 5-Minute Demo
```bash
./QUICK_DEMO.sh
```

### Recommended: Level 1 Complete
```bash
cd 01-local-primal
cat README.md
```

### Comprehensive: Full Showcase
```bash
cat 00_SHOWCASE_INDEX.md
```

---

## 🌟 Why LoamSpine?

**Sovereign Permanence. Cryptographic Provenance. Zero Hardcoding.**

- 🦴 **Permanent**: Your history never disappears
- 🔒 **Sovereign**: You own and control your spines
- 🔐 **Provable**: Cryptographic proofs of everything
- 🚀 **Fast**: Pure Rust, zero-copy, optimized
- 🤝 **Composable**: Seamlessly integrates with ecosystem
- 🏆 **World-Class**: A+ grade, 1,507 tests passing, 92% line coverage
- 🌍 **Universal**: tarpc + JSON-RPC for any client

---

## 📚 Additional Resources

### In This Showcase
- `00_SHOWCASE_INDEX.md` - Navigation
- `SHOWCASE_PRINCIPLES.md` - No mocks philosophy
- `SHOWCASE_QUICK_REFERENCE_CARD.md` - Quick reference
- `QUICK_REFERENCE.md` - Quick reference card

### In Repository Root
- `../README.md` - Project overview and documentation
- `../STATUS.md` - Project status
- `../specs/` - Complete specifications

### Ecosystem Showcases
- `../../squirrel/showcase/` - AI orchestration (if exists)
- `../../songBird/showcase/` - Service mesh (if exists)
- `../../toadStool/showcase/` - Universal compute (if exists)
- `../../nestGate/showcase/` - Storage infrastructure (if exists)
- `../../bearDog/showcase/` - Sovereign security (if exists)

---

## 🎉 Let's Begin!

**Choose your starting point**:

1. **Quick Demo** → `./QUICK_DEMO.sh`
2. **Level 1** → `cd 01-local-primal`
3. **Full Navigation** → `cat 00_SHOWCASE_INDEX.md`
4. **Automated Walkthrough** → `./RUN_ME_FIRST.sh`

---

**Status**: ✅ Ready to demonstrate sovereign permanence  
**Next**: Choose your learning path above  
**Support**: See troubleshooting section or review `SHOWCASE_PRINCIPLES.md`

🦴 **Welcome to the permanent layer of the ecoPrimals!** 🚀

---

*LoamSpine v0.9.16 - ecoPrimals Ecosystem*  
*April 11, 2026*

