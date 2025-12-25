# 🌍 Full Ecosystem - All Primals Together

**Time**: 15 minutes  
**Difficulty**: Advanced  
**Prerequisites**: All previous demos complete

---

## 🎯 What You'll Learn

- Orchestrate all primals in production workflow
- End-to-end request handling
- Cascading capability composition
- Fault tolerance across primals
- Production patterns

---

## 📖 Concepts

### The Full Stack

**7 Primals Working Together**:
1. **Songbird**: Discovery & health monitoring
2. **NestGate**: Gateway & routing
3. **Toadstool**: Compute execution
4. **BearDog**: Signing & cryptography
5. **LoamSpine**: Tamper-proof storage
6. **Squirrel**: Session management
7. *(Future)*: Additional primals

**No hardcoded dependencies!** Pure capability discovery.

---

## 🔍 Demo Flow

### End-to-End Request
```
User request
    ↓
1. NestGate (Gateway)
    ├─ Create session (Squirrel)
    ├─ Route to compute (Toadstool)
    └─ Track in session
    ↓
2. Toadstool (Compute)
    ├─ Execute task
    ├─ Store result (LoamSpine)
    └─ Sign result (BearDog)
    ↓
3. LoamSpine (Storage)
    ├─ Create spine entry
    ├─ Generate proof
    └─ Return receipt
    ↓
4. BearDog (Signing)
    ├─ Sign result hash
    ├─ Return signature
    └─ Log signature request
    ↓
5. NestGate (Gateway)
    ├─ Aggregate results
    ├─ Update session (Squirrel)
    └─ Commit session to LoamSpine
    ↓
6. Squirrel (Session)
    ├─ Finalize session
    ├─ Commit to LoamSpine
    └─ Return session proof
    ↓
User receives:
  - Compute result
  - Signed result (BearDog)
  - Storage proof (LoamSpine)
  - Session proof (Squirrel)
```

**Total**: 6 primal interactions, 0 hardcoded endpoints!

---

## 💡 Implementation

### NestGate: Orchestrator

```rust
async fn handle_compute_request(
    request: ComputeRequest,
    songbird: &SongbirdClient,
) -> Result<ComputeResponse> {
    // 1. Create session (Squirrel)
    let session_services = songbird.discover("session_management").await?;
    let squirrel = session_services.first().ok_or("No session service")?;
    let session_client = SquirrelClient::connect(&squirrel.endpoint).await?;
    
    let session_id = session_client.create_session(CreateSessionRequest {
        user_did: request.user_did.clone(),
        context: "compute_request".into(),
    }).await?;
    
    // 2. Discover compute capability (Toadstool)
    let compute_services = songbird.discover("compute").await?;
    let toadstool = compute_services
        .iter()
        .min_by_key(|s| s.load)
        .ok_or("No compute service")?;
    let compute_client = ToadstoolClient::connect(&toadstool.endpoint).await?;
    
    // 3. Execute computation
    let result = compute_client.execute(ComputeTask {
        task_id: request.task_id,
        code: request.code,
        input: request.input,
        session_id,
    }).await?;
    
    // 4. Toadstool already stored to LoamSpine and got BearDog signature
    // (from previous demos)
    
    // 5. Update session
    session_client.add_event(session_id, SessionEvent {
        event_type: "compute_complete".into(),
        data: serde_json::json!({
            "task_id": request.task_id,
            "result_hash": result.hash,
        }),
    }).await?;
    
    // 6. End & commit session
    let session_proof = session_client.end_session(session_id).await?;
    
    // 7. Return comprehensive response
    Ok(ComputeResponse {
        result: result.data,
        result_hash: result.hash,
        
        // From Toadstool → LoamSpine
        storage_proof: result.storage_proof,
        spine_id: result.spine_id,
        
        // From Toadstool → BearDog
        signature: result.signature,
        signer_did: result.signer_did,
        
        // From NestGate → Squirrel → LoamSpine
        session_proof: session_proof.proof,
        session_spine_id: session_proof.spine_id,
    })
}
```

---

## 📊 Architecture Diagram

```
┌─────────────────────────────────────────────────────┐
│                    Songbird                         │
│              (Discovery Service)                    │
│                  Port 3000                          │
└────────────────┬────────────────────────────────────┘
                 │
        ┌────────┴────────┬────────────┬──────────────┬─────────┐
        │                 │            │              │         │
┌───────▼────────┐ ┌──────▼──────┐ ┌──▼──────────┐ ┌─▼────────┐ ┌──▼────────┐
│   NestGate     │ │  Toadstool  │ │   BearDog   │ │  Loam    │ │ Squirrel  │
│   (Gateway)    │ │  (Compute)  │ │  (Signing)  │ │  Spine   │ │(Session)  │
│   Port 9004    │ │  Port 9005  │ │  Port 9003  │ │Port 9001 │ │Port 9002  │
└───────┬────────┘ └──────┬──────┘ └──┬──────────┘ └─┬────────┘ └──┬────────┘
        │                 │            │              │             │
        │   ┌─────────────▼────────────▼──────────────▼─────────────▼──┐
        │   │           All primals interact via RPC                    │
        │   │     (Discovered endpoints, not hardcoded)                 │
        │   └──────────────────────────────────────────────────────────┘
        │
┌───────▼────────┐
│     User       │
│   (curl/UI)    │
└────────────────┘
```

---

## 🎯 Example Request

### Submit Compute Task
```bash
curl -X POST http://localhost:9004/compute \
  -H "Content-Type: application/json" \
  -d '{
    "user_did": "did:example:alice123",
    "task_id": "task-001",
    "code": "def process(x): return x * 2",
    "input": {"x": 21}
  }'
```

### Response (Comprehensive Receipt)
```json
{
  "result": {"x": 42},
  "result_hash": "ba7816bf...",
  
  "storage": {
    "spine_id": "550e8400...",
    "entry_height": 42,
    "inclusion_proof": {
      "path": [...],
      "root": "e3b0c442..."
    }
  },
  
  "signature": {
    "bytes": "304402...",
    "algorithm": "Ed25519",
    "signer_did": "did:example:beardog",
    "public_key": "302a300..."
  },
  
  "session": {
    "session_id": "session-abc123",
    "spine_id": "660e8400...",
    "proof": {
      "path": [...],
      "root": "f4c1c564..."
    }
  }
}
```

**What you get**:
- ✅ Compute result
- ✅ Tamper-proof storage receipt (LoamSpine)
- ✅ Cryptographic signature (BearDog)
- ✅ Session audit trail (Squirrel → LoamSpine)

---

## 🎯 Success Criteria

- ✅ All primals registered with Songbird
- ✅ End-to-end request succeeds
- ✅ All proofs validate
- ✅ Signature verifies
- ✅ Session committed to LoamSpine
- ✅ Zero hardcoded endpoints

---

## 🛡️ Fault Tolerance

### Scenario: LoamSpine Crashes

**Impact**:
- ❌ Can't store new results/sessions
- ✅ Toadstool continues computing
- ✅ BearDog continues signing
- ✅ Squirrel tracks sessions in-memory

**Recovery**:
1. Restart LoamSpine
2. Auto re-register with Songbird
3. Clients discover new instance
4. Background retry of failed storage
5. Resume normal operations

**Downtime**: ~5 seconds (for detection + failover)

### Scenario: Multiple Primal Failures

**Cascading Prevention**:
```rust
// Circuit breaker pattern
let storage_result = with_circuit_breaker(
    "loamspine",
    || storage.add_entry(spine_id, entry)
).await;

match storage_result {
    Ok(entry_id) => { /* success */ },
    Err(CircuitOpen) => {
        // Don't even try - circuit is open
        fallback_local_storage(entry);
    },
    Err(e) => {
        // Try once, might work
        fallback_local_storage(entry);
    }
}
```

---

## 📈 Performance

**End-to-End Latency** (typical):
- NestGate receive: 0 ms
- Create session (Squirrel): 5 ms
- Discover + route: 10 ms
- Compute (Toadstool): 50 ms
  - Execute: 40 ms
  - Store (LoamSpine): 5 ms
  - Sign (BearDog): 5 ms
- Update session: 5 ms
- End session + commit: 10 ms
- NestGate response: 0 ms
- **Total**: ~80 ms

**Throughput**: ~12 requests/sec (single-threaded)  
**Parallel**: ~100+ requests/sec (10 workers)

---

## 💡 Best Practices

### DO ✅
- Implement timeouts (5s default)
- Use circuit breakers
- Cache discovery results (60s TTL)
- Log all inter-primal calls
- Monitor latency per primal
- Retry transient failures
- Return comprehensive receipts

### DON'T ❌
- Don't hardcode any endpoints
- Don't skip error handling
- Don't block on non-critical services
- Don't ignore partial failures
- Don't cascade failures

---

## 🔍 Monitoring

### Key Metrics

**Per Primal**:
- Request rate
- Error rate
- Latency (p50, p95, p99)
- Circuit breaker state

**Inter-Primal**:
- Discovery cache hit rate
- RPC call latency
- Failed discovery attempts
- Timeout rate

**Dashboard**:
```
NestGate    → Squirrel:   98.5% success, 5ms p95
NestGate    → Toadstool:  99.1% success, 45ms p95
Toadstool   → LoamSpine:  99.8% success, 6ms p95
Toadstool   → BearDog:    99.9% success, 7ms p95
NestGate    → Squirrel:   99.2% success, 10ms p95
```

---

## 🎓 Production Deployment

### Kubernetes
```yaml
# Each primal is a Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: loamspine
spec:
  replicas: 3  # Horizontal scaling
  selector:
    matchLabels:
      app: loamspine
  template:
    spec:
      containers:
      - name: loamspine
        image: ecoprimals/loamspine:0.6.1
        env:
        - name: SONGBIRD_ENDPOINT
          value: "http://songbird-service:3000"
        ports:
        - containerPort: 9001  # tarpc
        - containerPort: 8080  # JSON-RPC
```

### Service Mesh (Istio)
- Automatic retries
- Load balancing
- Circuit breaking
- mTLS between primals

---

## 🏆 You Made It!

**Congratulations!** You now understand:
- ✅ Full ecoPrimals ecosystem
- ✅ Capability-based composition
- ✅ Fault-tolerant architecture
- ✅ Production patterns
- ✅ Zero hardcoded dependencies

**Next**: Build your own primal!

---

**Status**: ⏳ Example needed  
**Related**: All crates

🦴 **LoamSpine: Where memories become permanent.**

