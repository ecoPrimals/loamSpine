# 🦴 LoamSpine Showcase Buildout Plan

**Date**: December 24, 2025  
**Goal**: Build comprehensive showcase following mature primal patterns  
**Inspiration**: Songbird (federation), ToadStool (compute), NestGate (complete)

---

## 🎯 VISION: Progressive Capability Demonstration

### What Makes a Great Showcase?

Based on analysis of **mature Phase 1 primals**:

**From Songbird** (Best federation demos):
- ✅ Multi-tower federation working
- ✅ Zero-configuration mesh joining
- ✅ Real friend-joins-LAN scenario
- ✅ Progressive levels (isolated → federated → ecosystem)

**From ToadStool** (Best compute demos):
- ✅ GPU benchmarking with real hardware
- ✅ Gaming showcases (OpenArena, native games)
- ✅ ML inference demonstrations
- ✅ Multi-node coordination

**From NestGate** (Best structure):
- ✅ 5 clear levels (00-local → 05-real-world)
- ✅ Comprehensive START_HERE.md
- ✅ Real-world scenarios (home NAS, research)
- ✅ Live service integration tests

**From BearDog** (Best security):
- ✅ Mixed entropy showcase
- ✅ HSM integration
- ✅ Production-ready patterns

---

## 📐 OUR SHOWCASE STRUCTURE

### Level 0: Local Primal (LoamSpine BY ITSELF)

**Status**: 29% complete (2/7 demos)

**What to Build**:
```
01-local-primal/
├── 01-hello-loamspine/     ✅ DONE
├── 02-entry-types/          🟡 Example done, needs demo script
├── 03-certificate-lifecycle/ ⏳ Need demo script + validation
├── 04-proofs/               ⏳ Need demo script
├── 05-backup-restore/       ⏳ Need demo script
├── 06-storage-backends/     ⏳ Need Sled integration demo
└── 07-concurrent-ops/       ⏳ Need stress test demo
```

**Pattern**: No external services, pure LoamSpine capabilities

---

### Level 1: RPC API (Pure Rust RPC)

**Status**: 0% complete (0/5 demos)

**What to Build**:
```
02-rpc-api/
├── 01-tarpc-basics/         ⏳ Binary RPC client/server
├── 02-jsonrpc-basics/       ⏳ curl + Python client examples
├── 03-health-monitoring/    ⏳ Health checks and status
├── 04-concurrent-ops/       ⏳ Parallel RPC calls
└── 05-error-handling/       ⏳ Error propagation demo
```

**Pattern**: Start loamspine service, interact via RPC

**Key Innovation**: Show Pure Rust RPC (no gRPC!) advantage

---

### Level 2: Songbird Discovery (O(n) Adapter Pattern)

**Status**: 0% complete (0/4 demos)

**What to Build**:
```
03-songbird-discovery/
├── 01-songbird-connect/     ⏳ Connect to Songbird from ../bins
├── 02-capability-discovery/ ⏳ Advertise loamspine capabilities
├── 03-auto-advertise/       ⏳ Lifecycle manager auto-register
└── 04-heartbeat-monitoring/ ⏳ Keep-alive and health reporting
```

**Pattern**: Use `../bins/songbird-orchestrator` as real service

**Key Innovation**: Show O(n) vs O(n²) discovery architecture

---

### Level 3: Inter-Primal Integration (Real Binaries)

**Status**: 0% complete (0/5 demos)

**What to Build**:
```
04-inter-primal/
├── 01-session-commit/       ⏳ Mock RhizoCrypt session → LoamSpine
├── 02-braid-commit/         ⏳ Mock SweetGrass braid → LoamSpine
├── 03-signing-capability/   ⏳ Use ../bins/beardog for signing
├── 04-storage-capability/   ⏳ Use ../bins/nestgate for payloads
└── 05-full-ecosystem/       ⏳ All primals coordinating
```

**Pattern**: Real Phase 1 binaries, no mocks!

**Available Binaries**:
- ✅ `../bins/beardog` — Signing & security
- ✅ `../bins/nestgate` — Storage & ZFS
- ✅ `../bins/songbird-orchestrator` — Discovery
- ✅ `../bins/toadstool-cli` — Compute
- ✅ `../bins/squirrel` — AI/MCP

---

### Level 4: Real-World Scenarios (Future)

**Status**: 0% complete (planning)

**Ideas** (inspired by NestGate):
```
05-real-world/
├── 01-game-save-history/    # Gaming provenance
├── 02-research-audit-log/   # Scientific reproducibility
├── 03-legal-document-chain/ # Legal/compliance
├── 04-medical-records/      # Healthcare provenance
└── 05-supply-chain/         # Logistics tracking
```

**Pattern**: Complete end-to-end workflows

---

## 🚀 IMPLEMENTATION PRIORITY

### Phase 1: Complete Level 0 (Immediate — 3-4 hours)

**Goal**: Show LoamSpine's standalone power

| Demo | What's Needed | Time |
|------|---------------|------|
| 02-entry-types | Add demo.sh wrapper | 30 min |
| 03-certificate-lifecycle | Create demo script + example | 60 min |
| 04-proofs | Create demo script + example | 45 min |
| 05-backup-restore | Create demo script | 30 min |
| 06-storage-backends | Create Sled demo | 45 min |
| 07-concurrent-ops | Create stress test | 30 min |

**Deliverable**: All Level 0 demos working independently

---

### Phase 2: Build Level 1 (Short-term — 2-3 hours)

**Goal**: Show Pure Rust RPC advantage

**Key Demos**:
1. **tarpc-basics**: Start service, make binary RPC calls
2. **jsonrpc-basics**: curl examples + Python client
3. **concurrent-ops**: Parallel operations benchmarking

**Learning**: Developers see how to integrate LoamSpine

---

### Phase 3: Build Level 2 (Short-term — 2 hours)

**Goal**: Show Songbird integration (O(n) adapter)

**Key Requirement**: Use `../bins/songbird-orchestrator`

**Demo Flow**:
```bash
# Start Songbird
../bins/songbird-orchestrator --port 8082 &

# Start LoamSpine with Songbird discovery
SONGBIRD_URL=http://localhost:8082 cargo run --release

# Verify registration
../bins/songbird-cli discover --capability loamspine
```

**Learning**: Zero hardcoding, runtime discovery works!

---

### Phase 4: Build Level 3 (Medium-term — 4-5 hours)

**Goal**: Show real inter-primal coordination

**Critical Demos**:

**1. Signing Integration** (Use BearDog):
```bash
# Generate key with BearDog
../bins/beardog key generate --type ed25519 --id alice

# Sign entry with BearDog
loamspine entry sign --signer-binary ../bins/beardog --key-id alice
```

**2. Storage Integration** (Use NestGate):
```bash
# Start NestGate
../bins/nestgate service start --port 8093 &

# Store large payload in NestGate
loamspine entry create --payload-url nestgate://localhost:8093/blob123
```

**3. Full Ecosystem**:
- Songbird for discovery
- BearDog for signing
- NestGate for storage
- LoamSpine for permanence

**Learning**: Ecosystem coordination patterns

---

## 🎬 DEMO SCRIPT PATTERNS

### Best Practices (From Mature Primals)

**1. Safe Execution**:
```bash
#!/usr/bin/env bash
set -euo pipefail  # Exit on error, undefined vars

# Cleanup on exit
trap cleanup EXIT
```

**2. Visual Feedback**:
```bash
# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ${NC} $*"; }
log_success() { echo -e "${GREEN}✅${NC} $*"; }
log_warning() { echo -e "${YELLOW}⚠${NC} $*"; }
log_error() { echo -e "${RED}❌${NC} $*"; }
```

**3. Graceful Degradation**:
```bash
# Check if service available
if ! curl -s http://localhost:8082/health > /dev/null 2>&1; then
    log_warning "Songbird not available, showing expected behavior..."
    DEMO_MODE=true
fi

if [ "$DEMO_MODE" = true ]; then
    # Simulate with explanation
    log_info "[DEMO MODE] Would register with Songbird..."
else
    # Real operation
    curl -X POST http://localhost:8082/register ...
fi
```

**4. Receipts & Logs**:
```bash
# Generate receipt for audit
generate_receipt() {
    local demo=$1
    local status=$2
    shift 2
    
    cat > "receipts/${demo}_$(date +%Y%m%d_%H%M%S).txt" <<EOF
Demo: $demo
Status: $status
Timestamp: $(date -Iseconds)
Steps:
$(printf "  - %s\n" "$@")
EOF
}
```

---

## 🔍 GAPS WE'LL DISCOVER

Based on NestGate/ToadStool experience, interactions will reveal:

**Likely Gaps**:
1. **CLI Tool Missing**: Need `loamspine-cli` for demos
2. **Service Lifecycle**: Start/stop/status commands
3. **Health Endpoints**: Beyond basic RPC
4. **Configuration Files**: TOML-based config needed
5. **Error Messages**: User-friendly error formatting
6. **Metrics Export**: For monitoring integration

**Expected Findings**:
- API adjustments for easier integration
- Documentation gaps
- Missing convenience functions
- Performance bottlenecks
- Edge case handling

---

## 📊 SUCCESS METRICS

### Level 0 Complete When:
- ✅ All 7 demos run independently
- ✅ Receipts generated for each
- ✅ No external service dependencies
- ✅ < 5 minutes per demo

### Level 1 Complete When:
- ✅ RPC service starts cleanly
- ✅ tarpc + JSON-RPC both work
- ✅ Python client example works
- ✅ Concurrent operations succeed

### Level 2 Complete When:
- ✅ Songbird binary starts
- ✅ LoamSpine registers capabilities
- ✅ Discovery works bidirectionally
- ✅ Heartbeat keeps registration alive

### Level 3 Complete When:
- ✅ BearDog signs entries
- ✅ NestGate stores payloads
- ✅ All primals coordinate via Songbird
- ✅ No mocks used anywhere

---

## 🛠️ IMPLEMENTATION APPROACH

### Step 1: Common Infrastructure (30 min)

**Files to Create**:
```
showcase/scripts/
├── common.sh                # Shared functions
├── colors.sh                # Color definitions
├── check_services.sh        # Service availability
├── start_songbird.sh        # Start Songbird from ../bins
├── start_beardog.sh         # Start BearDog from ../bins
├── start_nestgate.sh        # Start NestGate from ../bins
└── cleanup_all.sh           # Stop all services
```

### Step 2: Level 0 Completion (3-4 hours)

**For Each Demo**:
1. Review existing README
2. Create or update Rust example (if needed)
3. Write demo.sh script with:
   - Clear explanation
   - Step-by-step execution
   - Visual feedback
   - Receipt generation
4. Test end-to-end
5. Update IMPLEMENTATION_STATUS.md

### Step 3: Levels 1-3 (8-10 hours)

**Iterative Approach**:
1. Start with simplest demo in each level
2. Test with real binaries from ../bins
3. Document any gaps or issues
4. Implement workarounds or file issues
5. Complete remaining demos
6. Integration testing

---

## 📚 DOCUMENTATION UPDATES

### Files to Update:
- ✅ `IMPLEMENTATION_STATUS.md` — Track progress
- ✅ `README.md` — Update with new demos
- ✅ `SHOWCASE_PRINCIPLES.md` — Reinforce no-mocks rule
- ⏳ `SHOWCASE_PATTERNS.md` — Document patterns we use
- ⏳ `GAP_ANALYSIS.md` — Document discovered gaps
- ⏳ `INTEGRATION_LEARNINGS.md` — Lessons from inter-primal

### New Files to Create:
- ⏳ `QUICK_START_COMPLETE.md` — Full walkthrough
- ⏳ `TROUBLESHOOTING.md` — Common issues
- ⏳ `BINARIES_GUIDE.md` — How to use ../bins primals

---

## 🎯 NEXT ACTIONS

### Immediate (Start Now):
1. ✅ Review mature primal showcases — DONE
2. ⏳ Create common infrastructure scripts
3. ⏳ Complete Level 0 demo #2 (entry-types)
4. ⏳ Complete Level 0 demo #3 (certificate-lifecycle)

### Short-term (This Session):
1. ⏳ Complete all Level 0 demos
2. ⏳ Test Level 0 end-to-end
3. ⏳ Start Level 1 (tarpc-basics)

### Medium-term (Next Session):
1. ⏳ Complete Levels 1-2
2. ⏳ Test with real Songbird binary
3. ⏳ Start Level 3 integration

---

## 💡 KEY INSIGHTS

### What Makes Showcases Successful:

**From Songbird**:
- Real multi-machine federation demos
- Friend-joins-LAN scenario resonates
- Progressive complexity works

**From ToadStool**:
- Real hardware (GPU) demos impress
- Gaming showcases are relatable
- Performance numbers matter

**From NestGate**:
- Clear 5-level structure helps navigation
- Real-world scenarios demonstrate value
- Comprehensive START_HERE is essential

### What We'll Do Better:

1. **No Mocks**: All integration uses real binaries from ../bins
2. **Clear Receipts**: Every demo generates audit receipt
3. **Visual Feedback**: Colored output, progress indicators
4. **Graceful Degradation**: Demos work even if services unavailable
5. **Gap Documentation**: Track and report integration issues

---

**Ready to Build**: Start with Level 0 completion → Level 1 → Songbird integration → Full ecosystem

**Estimated Total Time**: 15-18 hours for Levels 0-3 complete

🦴 **LoamSpine: Where memories become permanent.**

