# 🌐 Level 4: Inter-Primal Integration Demos

**Purpose**: Demonstrate full ecosystem integration  
**Philosophy**: Capability composition, not tight coupling  
**Time**: 40-50 minutes total

---

## 🎯 Overview

**Inter-Primal Integration** is where the ecoPrimals ecosystem shines:
- **Session Commit**: User sessions → LoamSpine persistence
- **Braid Commit**: Distributed braids → LoamSpine anchoring
- **Signing Capability**: BearDog signing → LoamSpine certificates
- **Storage Capability**: LoamSpine as backend for other primals
- **Full Ecosystem**: All primals working together

**Why Inter-Primal?**
- ✅ Composable capabilities
- ✅ No tight coupling
- ✅ Horizontal scalability
- ✅ Fault tolerance

---

## 📁 Demos

| # | Demo | Description | Time |
|---|------|-------------|------|
| 01 | session-commit | Squirrel sessions → LoamSpine | 10 min |
| 02 | braid-commit | NestGate braids → LoamSpine | 10 min |
| 03 | signing-capability | BearDog signs → LoamSpine certs | 10 min |
| 04 | storage-capability | Toadstool uses LoamSpine | 10 min |
| 05 | full-ecosystem | All primals together | 15 min |

---

## 🚀 Quick Start

```bash
# Prerequisites: All primals must be running
cd ../bins

# Start core services
./songbird &
./loamspine &

# Start integration primals
./squirrel &
./beardog &
./nestgate &
./toadstool &

# Run Level 4 demos
cd ../loamSpine/showcase/04-inter-primal
./RUN_ALL.sh
```

---

## 🎓 Learning Path

### For Architects
1. Study capability composition patterns
2. Review inter-primal contracts
3. Understand failure modes
4. Design new primal integrations

### For Developers
1. Start with `01-session-commit` (simple)
2. Progress to `02-braid-commit` (complex)
3. Study `03-signing-capability` (security)
4. Understand `05-full-ecosystem` (production)

### For Operators
1. Deploy primal ecosystem
2. Monitor inter-primal health
3. Handle cascading failures
4. Scale individual primals

---

## 🔧 Prerequisites

### Required Binaries
- `../bins/songbird` (discovery)
- `../bins/loamspine` (this primal)
- `../bins/squirrel` (session management)
- `../bins/beardog` (signing)
- `../bins/nestgate` (gateway/braid)
- `../bins/toadstool` (compute)

### Network Ports
- Songbird: 3000 (HTTP)
- LoamSpine: 9001 (tarpc), 8080 (JSON-RPC)
- Squirrel: 9002 (tarpc)
- BearDog: 9003 (tarpc)
- NestGate: 9004 (tarpc)
- Toadstool: 9005 (tarpc)

---

## 📊 Integration Patterns

### 1. Session Commit (Squirrel → LoamSpine)
```
User action → Squirrel session
               ↓
         Session complete
               ↓
         commit_session RPC
               ↓
         LoamSpine creates spine
               ↓
         Returns spine_id & proof
```

**Use Case**: Permanent session history

### 2. Braid Commit (NestGate → LoamSpine)
```
Multi-party braid
       ↓
Braid sealed
       ↓
commit_braid RPC
       ↓
LoamSpine anchors braid
       ↓
Returns anchor proof
```

**Use Case**: Distributed consensus anchoring

### 3. Signing Capability (BearDog ↔ LoamSpine)
```
LoamSpine needs signature
         ↓
Discover BearDog via Songbird
         ↓
Request signature
         ↓
BearDog signs + returns
         ↓
LoamSpine embeds in certificate
```

**Use Case**: Certificate issuance with HSM

### 4. Storage Capability (Toadstool → LoamSpine)
```
Toadstool compute result
          ↓
Needs persistent storage
          ↓
Discover LoamSpine via Songbird
          ↓
Store as spine entry
          ↓
Return tamper-proof receipt
```

**Use Case**: Verifiable compute results

### 5. Full Ecosystem
```
User request → NestGate
               ↓
        Routes to Toadstool
               ↓
        Compute + sign (BearDog)
               ↓
        Store result (LoamSpine)
               ↓
        Track session (Squirrel)
               ↓
        Return to user
```

**Use Case**: Production workflows

---

## 💡 Key Concepts

### Capability Composition
**Instead of**:
```rust
// ❌ Hardcoded dependencies
let signer = BearDogClient::new("localhost:9003");
let storage = LoamSpineClient::new("localhost:9001");
```

**Use**:
```rust
// ✅ Capability discovery
let signer = songbird.discover("signing").await?;
let storage = songbird.discover("storage").await?;
```

### Contract-Based Integration
Each primal defines **contracts** (RPC methods):
- **LoamSpine**: `commit_session`, `commit_braid`
- **BearDog**: `sign_data`, `verify_signature`
- **Squirrel**: `create_session`, `end_session`

**No shared code, only contracts!**

### Fault Isolation
```
Toadstool crash
    ↓
Does NOT affect LoamSpine
    ↓
Songbird marks Toadstool unhealthy
    ↓
Clients failover to alternate Toadstool
    ↓
LoamSpine continues normally
```

---

## 🎯 Success Criteria

By the end of Level 4, you should:
- ✅ Commit sessions from Squirrel to LoamSpine
- ✅ Anchor braids from NestGate to LoamSpine
- ✅ Use BearDog signing in LoamSpine certificates
- ✅ Provide storage capability to other primals
- ✅ Run full ecosystem with graceful failures
- ✅ Understand capability composition

---

## 🛡️ Failure Modes

### Single Primal Failure
**Scenario**: LoamSpine crashes

**Impact**:
- ❌ Can't commit new sessions/braids
- ✅ Other primals continue operating
- ✅ Existing data unaffected

**Recovery**:
- Restart LoamSpine
- Auto re-register with Songbird
- Clients discover new instance
- Resume operations

### Cascading Prevention
**Design**:
- Circuit breakers on RPC calls
- Timeouts (5s default)
- Fallback behaviors
- Async operations

**Example**:
```rust
match storage_client.commit_session(session).await {
    Ok(spine_id) => {
        // Success path
    },
    Err(e) => {
        // Fallback: Store session locally
        // Retry commit in background
    }
}
```

---

## 📈 Production Patterns

### Service Mesh
```
┌─────────────┐
│  Songbird   │ (Discovery)
│  (Port 3000)│
└──────┬──────┘
       │
   ┌───┴───┬───────┬───────┬────────┐
   │       │       │       │        │
┌──▼──┐ ┌──▼──┐ ┌──▼──┐ ┌──▼──┐  ┌──▼──────┐
│Loam │ │Bear │ │Nest │ │Toad │  │Squirrel │
│Spine│ │Dog  │ │Gate │ │stool│  │         │
└─────┘ └─────┘ └─────┘ └─────┘  └─────────┘
```

### Horizontal Scaling
```
┌─────────────┐
│  Songbird   │
└──────┬──────┘
       │
   ┌───┴───────────────┐
   │                   │
┌──▼───────┐    ┌──────▼──┐
│LoamSpine │    │LoamSpine│
│Instance 1│    │Instance 2│
└──────────┘    └─────────┘

Clients load-balance via Songbird
```

### Geographic Distribution
```
US West              US East
┌─────────────┐     ┌─────────────┐
│ Songbird-US │     │Songbird-East│
└──────┬──────┘     └──────┬──────┘
       │                   │
   ┌───┴───┐           ┌───┴───┐
   │Loam   │           │Loam   │
   │Spine  │           │Spine  │
   └───────┘           └───────┘

Federation between towers
```

---

## 🔗 Next Steps

- **Production**: Deploy to Kubernetes
- **Monitoring**: Add Prometheus + Grafana
- **Scaling**: Add more instances
- **Federation**: Multi-region deployment

---

**Status**: ⏳ Documentation complete, examples pending  
**Related**: All `crates/loam-spine-core/src/` integration points

🦴 **LoamSpine: Where memories become permanent.**

