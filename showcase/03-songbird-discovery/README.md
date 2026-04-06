# 🎵 Level 3: Songbird Discovery Demos

**Purpose**: Demonstrate capability-based service discovery  
**Philosophy**: No hardcoded endpoints, runtime discovery only  
**Time**: 30-40 minutes total

---

## 🎯 Overview

**Songbird** is the ecoPrimals discovery service:
- **Capability-based**: Services advertise what they can do
- **Dynamic**: No hardcoded endpoints
- **Federated**: Multi-tower support
- **Resilient**: Automatic heartbeats & re-registration

**Why Songbird?**
- ✅ Primal sovereignty (self-knowledge only)
- ✅ Runtime discovery (no compile-time dependencies)
- ✅ Horizontal scaling (federated towers)
- ✅ Automatic health monitoring

---

## 📁 Demos

| # | Demo | Description | Time |
|---|------|-------------|------|
| 01 | songbird-connect | Connect & register with Songbird | 10 min |
| 02 | capability-discovery | Query capabilities | 10 min |
| 03 | auto-advertise | Automatic service advertisement | 5 min |
| 04 | heartbeat-monitoring | Health checks & re-registration | 10 min |

---

## 🚀 Quick Start

```bash
# Prerequisites: Songbird must be running
cd ../bins
./songbird &
SONGBIRD_PID=$!

# Run Level 3 demos
cd ../loamSpine/showcase/03-songbird-discovery
./RUN_ALL.sh

# Cleanup
kill $SONGBIRD_PID
```

---

## 🎓 Learning Path

### For Architects
1. Understand capability-based discovery
2. Study federation patterns
3. Review primal sovereignty principles

### For Developers
1. Start with `01-songbird-connect`
2. Learn capability queries in `02-capability-discovery`
3. Study auto-registration in `03-auto-advertise`

### For Operators
1. Deploy Songbird towers
2. Monitor heartbeats in `04-heartbeat-monitoring`
3. Handle failover scenarios

---

## 🔧 Prerequisites

- **Songbird binary**: `../bins/songbird`
- Level 1 & 2 understanding
- Basic networking knowledge

**Songbird Endpoints**:
- HTTP API: `http://localhost:3000`
- Discovery: `/discover`
- Register: `/register`
- Heartbeat: `/heartbeat`

---

## 📊 Capability Schema

```json
{
  "name": "loamspine",
  "version": "0.9.16",
  "capabilities": [
    {
      "type": "spine_management",
      "methods": ["create", "seal", "query"],
      "protocol": "tarpc",
      "endpoint": "http://localhost:9001"
    },
    {
      "type": "spine_management",
      "methods": ["create", "seal", "query"],
      "protocol": "jsonrpc",
      "endpoint": "http://localhost:8080"
    },
    {
      "type": "certificate_management",
      "methods": ["mint", "transfer", "verify"],
      "protocol": "tarpc",
      "endpoint": "http://localhost:9001"
    }
  ],
  "metadata": {
    "storage": "redb",
    "uptime": 3600,
    "spines": 42
  }
}
```

---

## 💡 Key Concepts

### Primal Sovereignty
**Rules**:
- ✅ Know only yourself (name, version, capabilities)
- ✅ Discover others at runtime (via Songbird)
- ❌ Never hardcode other primal endpoints
- ❌ Never hardcode dependency versions

### Capability-Based Discovery
**Query by capability**, not by name:
```
"Find me a primal that can sign transactions"
  → Returns: [BearDog, LocalSigner, HSM]
  
"Find me a primal that can store spines"
  → Returns: [LoamSpine, ArchivalNode]
```

### Federation
- Multiple Songbird towers
- Geographic distribution
- Load balancing
- Failover support

---

## 🎯 Success Criteria

By the end of Level 3, you should:
- ✅ Register LoamSpine with Songbird
- ✅ Query capabilities dynamically
- ✅ Understand heartbeat mechanisms
- ✅ Handle Songbird unavailability gracefully
- ✅ Respect primal sovereignty principles

---

## 🔗 Integration Points

### With LoamSpine
- Auto-register on startup
- Advertise tarpc + JSON-RPC endpoints
- Send heartbeats every 30s
- Deregister on graceful shutdown

### With Other Primals
- **Toadstool**: Discover compute capabilities
- **BearDog**: Discover signing capabilities
- **NestGate**: Discover gateway capabilities
- **Squirrel**: Discover caching capabilities

---

## 🛡️ Fault Tolerance

### Songbird Unavailable
- ✅ Continue operating (local mode)
- ✅ Retry registration with backoff
- ✅ Log warnings (non-fatal)

### Heartbeat Missed
- ✅ Songbird marks service degraded
- ✅ Service attempts re-registration
- ✅ Clients query alternate instances

### Network Partition
- ✅ Federated towers provide redundancy
- ✅ Clients fail over to other towers
- ✅ Services re-advertise when partition heals

---

## 📈 Production Patterns

### Startup Sequence
```
1. Start service (local mode)
2. Attempt Songbird registration (async)
3. If success: advertise capabilities
4. If failure: log warning, continue local
5. Retry registration every 60s
```

### Graceful Shutdown
```
1. Stop accepting new requests
2. Finish in-flight requests
3. Deregister from Songbird
4. Close connections
5. Exit
```

---

## 🔗 Next Steps

- **Level 4**: Inter-Primal Integration (use discovered capabilities)
- **Advanced**: Multi-tower federation
- **Production**: Kubernetes integration

---

**Status**: ⏳ Documentation complete, examples pending  
**Related**: `crates/loam-spine-core/src/discovery.rs`

🦴 **LoamSpine: Where memories become permanent.**

