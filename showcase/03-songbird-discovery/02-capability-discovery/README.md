# 🦴 Demo: Capability-Based Discovery

**Goal**: Demonstrate runtime service discovery using Songbird with real binaries  
**Time**: 5 minutes  
**Prerequisites**: Songbird binary at `../bins/songbird-orchestrator`

---

## 🎯 What You'll Learn

- How LoamSpine registers with Songbird using real binary
- Capability-based service discovery (not hardcoded endpoints)
- Querying services by capability at runtime
- O(n) discovery complexity (efficient)

---

## 🚀 Quick Start

```bash
# Run the demo
./demo.sh
```

---

## 📋 What This Demo Shows

### 1. Service Registration
LoamSpine registers with Songbird, advertising its capabilities:
- `persistent-ledger` - Permanent immutable storage
- `waypoint-anchoring` - Slice anchor points
- `certificate-manager` - Loam certificate lifecycle
- `proof-generation` - Inclusion and provenance proofs

### 2. Capability-Based Discovery
Query Songbird for services that can do specific things:
```bash
# Find who can provide persistent storage
curl "http://localhost:8082/api/v1/discover?capability=persistent-ledger"

# Find who can anchor waypoints
curl "http://localhost:8082/api/v1/discover?capability=waypoint-anchoring"
```

### 3. Runtime Discovery
**No hardcoding!** Services discover each other at runtime:
- Primal advertises WHAT it can do
- Consumer discovers WHO can do it
- No knowledge of specific primal names or endpoints

---

## 💡 Key Concepts

### Capability-Based Architecture
```
Traditional (hardcoded):
  App → knows about "LoamSpine" → connects to localhost:9001

Capability-based (discovery):
  App → needs "persistent-ledger" → queries Songbird → finds LoamSpine
```

### Why This Matters
1. **Flexibility**: Swap implementations without code changes
2. **Scalability**: Multiple primals can provide same capability
3. **Sovereignty**: Primals don't hardcode dependencies
4. **Evolution**: New capabilities can be added dynamically

---

## 🔍 What You'll See

```
================================================================
  🦴 LoamSpine + 🎵 Songbird: Capability Discovery
================================================================

Step 1: Checking prerequisites...
✓ Songbird binary found
✓ LoamSpine compiles

Step 2: Starting Songbird orchestrator...
✓ Songbird running (PID: 12345)

Step 3: Testing Songbird health...
✓ Songbird health check passed

Step 4: Registering LoamSpine with Songbird...
✓ LoamSpine registered with Songbird
   Capabilities: persistent-ledger, waypoint-anchoring, ...

Step 5: Discovering services by capability...
   Query 1: Find 'persistent-ledger' → Found: loamspine
   Query 2: Find 'waypoint-anchoring' → Found: loamspine
   Query 3: Find 'certificate-manager' → Found: loamspine
   Query 4: Find 'nonexistent' → Found: (empty)

Step 6: Discovering all registered services...
   All services: [loamspine]

================================================================
  Demo Complete!
================================================================
```

---

## 🎓 Learning Points

### Pattern 1: Self-Knowledge
Primals know WHAT they can do, not WHO else exists:
```rust
// LoamSpine knows its own capabilities
let capabilities = vec![
    "persistent-ledger",
    "waypoint-anchoring",
    "certificate-manager",
    "proof-generation",
];

// But doesn't hardcode other primals!
```

### Pattern 2: Runtime Discovery
Consumers discover providers at runtime:
```rust
// Instead of: connect to "loamspine" at "localhost:9001"
// Do: find service with capability "persistent-ledger"
let services = songbird.discover_capability("persistent-ledger").await?;
let loamspine = services.first().ok_or("No persistent ledger available")?;
connect_to(loamspine.endpoint).await?;
```

### Pattern 3: Graceful Degradation
If capability not available, handle gracefully:
```rust
match songbird.discover_capability("optional-feature").await {
    Ok(services) if !services.is_empty() => use_feature(),
    _ => continue_without_feature(),
}
```

---

## 📊 Performance

**Discovery Complexity**: O(n) where n = number of registered services
- Efficient even with many services
- Songbird maintains indexed registry
- Query returns in milliseconds

**Comparison**:
- Traditional DNS lookup: O(1) but hardcoded
- Service mesh discovery: O(n) with capability filtering
- **This approach**: O(n) with capability semantics ✅

---

## 🔧 Troubleshooting

### Error: "Songbird binary not found"
```bash
# Build Songbird from Phase 1
cd ../../phase1/songBird
cargo build --release
cp target/release/songbird-orchestrator ../phase2/bins/
```

### Error: "Failed to start Songbird"
```bash
# Check if port 8082 is in use
lsof -i :8082

# Check Songbird logs
cat /tmp/songbird.log
```

### Error: "Songbird not responding"
```bash
# Test Songbird manually
curl http://localhost:8082/health

# Or restart demo
./demo.sh
```

---

## ➡️ Next Steps

After completing this demo:
- **Next Demo**: `03-auto-advertise` - Automatic lifecycle management
- **Related**: `04-heartbeat-monitoring` - Health checks and recovery
- **Deep Dive**: `../../specs/INTEGRATION_SPECIFICATION.md`

---

## 🎯 Success Criteria

You'll know this demo worked if you see:
- ✅ Songbird starts successfully
- ✅ LoamSpine registers with capabilities
- ✅ Capability queries return LoamSpine
- ✅ Non-existent capability returns empty list

---

**Updated**: December 25, 2025  
**Demo uses**: Real Songbird binary (NO MOCKS!)  
**Principle**: Capability-based discovery, not hardcoded dependencies

🦴 **LoamSpine: Discovering capabilities, not primals**
