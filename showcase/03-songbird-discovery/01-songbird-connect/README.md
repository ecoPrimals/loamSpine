# 🔌 Songbird Connect - Service Registration

**Time**: 10 minutes  
**Difficulty**: Intermediate  
**Prerequisites**: Level 1 & 2 complete, Songbird running

---

## 🎯 What You'll Learn

- Start Songbird discovery service
- Register LoamSpine with Songbird
- Advertise capabilities
- Verify registration

---

## 📖 Concepts

### Service Registration

**What gets registered**:
- Service name ("loamspine")
- Service version ("0.9.16")
- Capabilities (spine_management, certificate_management, etc.)
- Endpoints (tarpc: 9001, jsonrpc: 8080)
- Metadata (storage backend, uptime, etc.)

**Why register**:
- Enable discovery by other primals
- Participate in ecosystem
- Get automatic health monitoring
- Enable load balancing

---

## 🔍 Demo Flow

```
1. Start Songbird (port 3000)
   ↓
2. Start LoamSpine
   ↓
3. LoamSpine auto-registers with Songbird
   ↓
4. Query Songbird for LoamSpine
   ↓
5. Verify endpoints & capabilities
```

---

## 💡 Registration Payload

```json
{
  "name": "loamspine",
  "version": "0.9.16",
  "endpoints": {
    "tarpc": "http://localhost:9001",
    "jsonrpc": "http://localhost:8080"
  },
  "capabilities": [
    "spine_management",
    "certificate_management",
    "proof_generation",
    "session_commit",
    "braid_commit"
  ],
  "metadata": {
    "storage": "redb",
    "forbid_unsafe": true,
    "phase": 2
  }
}
```

---

## 🔧 Configuration

### LoamSpine Config
```toml
[discovery]
enabled = true
songbird_endpoint = "http://localhost:3000"
tarpc_endpoint = "http://localhost:9001"
jsonrpc_endpoint = "http://localhost:8080"
heartbeat_interval = 30  # seconds
```

### Songbird Config
```toml
[server]
host = "0.0.0.0"
port = 3000

[federation]
enabled = false  # Single tower for demo
```

---

## 📊 Verification

### Query Songbird
```bash
curl http://localhost:3000/services/loamspine
```

**Expected Response**:
```json
{
  "name": "loamspine",
  "status": "healthy",
  "registered_at": "2025-12-24T12:00:00Z",
  "last_heartbeat": "2025-12-24T12:05:00Z",
  "endpoints": {
    "tarpc": "http://localhost:9001",
    "jsonrpc": "http://localhost:8080"
  },
  "capabilities": [...]
}
```

---

## 🎯 Success Criteria

- ✅ Songbird running on port 3000
- ✅ LoamSpine successfully registered
- ✅ Endpoints queryable via Songbird API
- ✅ Capabilities correctly advertised

---

## 🛡️ Error Handling

**Songbird Unavailable**:
- ⚠️ Warning logged (non-fatal)
- ✅ LoamSpine continues in local mode
- ✅ Retry registration every 60s

**Registration Failed**:
- ⚠️ Log error details
- ✅ Continue operating locally
- ✅ Exponential backoff on retries

---

**Status**: ⏳ Example needed  
**Related**: `crates/loam-spine-core/src/discovery.rs:advertise_to_registry`

