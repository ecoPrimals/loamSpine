# Showcase 07: Songbird Discovery Integration

**Status**: 🌟 Universal Adapter Integration  
**Complexity**: Advanced  
**Prerequisites**: Songbird running, Phase 1 primals available

---

## Overview

This showcase demonstrates LoamSpine's integration with **Songbird** (the universal adapter) for runtime service discovery. Instead of hardcoding primal names or endpoints, LoamSpine discovers capabilities dynamically through Songbird.

### Philosophy

> "Each primal knows only itself. Everything else is discovered at runtime."

- **Zero hardcoding**: No primal names in code
- **O(n) complexity**: Each primal connects to Songbird, not to each other
- **Infant learning model**: Start with zero knowledge, discover everything
- **Graceful degradation**: Handle missing capabilities elegantly

---

## Architecture

```
┌─────────────┐
│  LoamSpine  │────┐
└─────────────┘    │
                   │    ┌──────────────┐
┌─────────────┐    ├───▶│   Songbird   │◀────┐
│   Beardog   │────┘    │   (Adapter)  │     │
└─────────────┘         └──────────────┘     │
                                             │
┌─────────────┐                              │
│  NestGate   │──────────────────────────────┘
└─────────────┘

Instead of O(n²) connections, we have O(n) through Songbird.
```

---

## What This Demonstrates

1. **Service Advertisement**: LoamSpine advertises its capabilities to Songbird
2. **Capability Discovery**: LoamSpine discovers other primals by capability
3. **Runtime Registration**: Capabilities are registered in `CapabilityRegistry`
4. **Heartbeat Mechanism**: Keep advertisement alive with periodic heartbeats
5. **Graceful Fallback**: Handle Songbird unavailability

---

## Prerequisites

### 1. Start Songbird Tower

```bash
# From ../bins/
cd ../../bins
./songbird-cli tower start
```

Expected output:
```
🏰 Starting Songbird tower...
✅ Tower started at http://localhost:8082
✅ Discovery endpoint: /discover
✅ Registry endpoint: /register
```

### 2. Verify Songbird is Running

```bash
curl http://localhost:8082/health
# Should return: {"status":"ok"}
```

### 3. (Optional) Start Phase 1 Primals

```bash
# BearDog (signing)
cd ../../phase1/bearDog
cargo run --release -- service start --port 8081

# NestGate (storage)
cd ../../phase1/nestGate
cargo run --release -- service start --port 8093
```

---

## Demos

### Demo 1: Basic Discovery

**File**: `01-basic-discovery.rs`

Demonstrates:
- Connecting to Songbird
- Discovering all available services
- Filtering by capability

```bash
cargo run --example 07-01-basic-discovery
```

**Expected Output**:
```
🔍 Connecting to Songbird at http://localhost:8082...
✅ Connected to Songbird

📡 Discovering all services...
Found 3 services:
  • beardog (security) at http://localhost:8081
    Capabilities: signing, verification, key-management
  • nestgate (storage) at http://localhost:8093
    Capabilities: storage, backup, replication
  • loamspine (permanence) at http://localhost:9001
    Capabilities: spine-management, certificate-management

🔎 Discovering signing capability...
Found 1 service with 'signing':
  • beardog at http://localhost:8081
```

---

### Demo 2: Service Advertisement

**File**: `02-service-advertisement.rs`

Demonstrates:
- Advertising LoamSpine's capabilities
- Heartbeat mechanism
- Re-advertisement after timeout

```bash
cargo run --example 07-02-service-advertisement
```

**Expected Output**:
```
📢 Advertising LoamSpine to Songbird...
✅ Advertisement successful

Capabilities advertised:
  • permanence
  • selective-memory
  • spine-management
  • certificate-management
  • inclusion-proofs
  • backup/restore

❤️ Sending heartbeat...
✅ Heartbeat acknowledged

Waiting 60 seconds for next heartbeat...
```

---

### Demo 3: Capability Registry Integration

**File**: `03-capability-registry.rs`

Demonstrates:
- Creating `CapabilityRegistry` with Songbird
- Discovering and registering capabilities
- Using discovered capabilities

```bash
cargo run --example 07-03-capability-registry
```

**Expected Output**:
```
🔧 Creating CapabilityRegistry with Songbird...
✅ Registry created

🔍 Discovering capabilities from Songbird...
✅ Discovered signing service: beardog at http://localhost:8081
✅ Discovered storage service: nestgate at http://localhost:8093

📊 Capability Status:
  • Signer: Available (beardog)
  • Verifier: Available (beardog)
  • Storage: Available (nestgate)

✅ All capabilities discovered and registered!
```

---

### Demo 4: Full Integration

**File**: `04-full-integration.rs`

Demonstrates:
- Complete LoamSpine startup with Songbird
- Automatic capability discovery
- Service advertisement
- Creating a spine with discovered capabilities

```bash
cargo run --example 07-04-full-integration
```

**Expected Output**:
```
🦴 Starting LoamSpine with Songbird integration...

📡 Connecting to Songbird at http://localhost:8082...
✅ Connected

📢 Advertising LoamSpine capabilities...
✅ Advertised to Songbird

🔍 Discovering capabilities...
✅ Found signing: beardog
✅ Found storage: nestgate

🦴 Creating spine with discovered capabilities...
✅ Spine created: spine_01J9X...
✅ Entry appended with discovered signer

📊 Final Status:
  • Songbird: Connected
  • Signer: Available (beardog)
  • Storage: Available (nestgate)
  • Spines: 1 active
  • Entries: 1 committed

✅ Full integration successful!
```

---

## Configuration

### Environment Variables

```bash
# Songbird endpoint
export LOAMSPINE_SONGBIRD_ENDPOINT=http://localhost:8082

# Enable auto-advertisement
export LOAMSPINE_AUTO_ADVERTISE=true

# Heartbeat interval (seconds)
export LOAMSPINE_HEARTBEAT_INTERVAL=60

# Discovery methods (comma-separated)
export LOAMSPINE_DISCOVERY_METHODS=environment,songbird,local-binaries
```

### Config File

See `../../primal-capabilities.toml` for complete configuration.

---

## Testing

### Unit Tests

```bash
cargo test --package loam-spine-core songbird
cargo test --package loam-spine-core discovery
```

### Integration Tests

```bash
# Requires Songbird running
cargo test --test integration -- --ignored songbird
```

---

## Troubleshooting

### Songbird Not Found

**Error**: `Songbird unavailable at http://localhost:8082`

**Solution**:
```bash
# Start Songbird
cd ../../bins
./songbird-cli tower start
```

### No Services Discovered

**Error**: `Discovered 0 services`

**Solution**:
1. Verify Songbird is running: `curl http://localhost:8082/health`
2. Check if other primals are advertising to Songbird
3. Start Phase 1 primals (BearDog, NestGate)

### Advertisement Failed

**Error**: `Advertisement failed with status: 500`

**Solution**:
1. Check Songbird logs for errors
2. Verify payload format matches Songbird's API
3. Ensure LoamSpine endpoints are accessible

---

## Key Concepts

### Universal Adapter Pattern

Instead of each primal connecting to every other primal (O(n²) connections), all primals connect to Songbird (O(n) connections). Songbird acts as the universal adapter, routing requests and facilitating discovery.

### Capability-Based Discovery

Services are discovered by **what they do** (capabilities), not **who they are** (primal names). This allows:
- Multiple providers for the same capability
- Automatic failover
- Load balancing
- Graceful degradation

### Infant Learning Model

LoamSpine starts with **zero knowledge** of other primals and discovers everything at runtime:
1. Connect to Songbird (universal adapter)
2. Advertise own capabilities
3. Discover other capabilities
4. Register discovered services
5. Use capabilities as needed

---

## Next Steps

- **Showcase 08**: Multi-Primal Coordination (coming soon)
- **Showcase 09**: Network Federation (coming soon)
- **Showcase 10**: Zero-Copy Optimization (coming soon)

---

## References

- `../../primal-capabilities.toml` — LoamSpine capability registry
- `../../phase1/toadStool/primal-capabilities.toml` — Example from ToadStool
- `../../bins/README.md` — Phase 1 primal binaries
- `../../specs/ARCHITECTURE.md` — System architecture
- `../../CONTRIBUTING.md` — Primal sovereignty principles

---

**Status**: ✅ Songbird Integration Complete  
**Grade**: A+ (Universal Adapter Pattern)  
**Next**: Multi-Primal Coordination

