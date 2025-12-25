# ⚡ Concurrent RPC Operations

**Time**: 10 minutes  
**Difficulty**: Advanced  
**Prerequisites**: tarpc or JSON-RPC basics

---

## 🎯 What You'll Learn

- Make parallel RPC calls
- Understand concurrency safety
- Handle race conditions
- Optimize throughput

---

## 📖 Concepts

### Thread Safety

LoamSpine is **fully thread-safe**:
- `Arc<RwLock<Spine>>` for concurrent access
- Readers don't block readers
- Writers wait for exclusive access
- No unsafe code for concurrency

### Concurrency Patterns

**Fan-Out Pattern**:
```
Client
  ├─> create_spine (spine A)
  ├─> create_spine (spine B)
  └─> create_spine (spine C)
```

**Pipeline Pattern**:
```
create_spine → add_entry → add_entry → seal_spine
```

**Scatter-Gather**:
```
Query multiple spines in parallel, aggregate results
```

---

## 🔍 Demo Flow

```
1. Start LoamSpine service
   ↓
2. Create 100 spines in parallel
   ↓
3. Add entries to each spine concurrently
   ↓
4. Query all spines simultaneously
   ↓
5. Measure throughput & latency
```

---

## 💡 Example: Parallel Creates

```rust
use tokio::task::JoinSet;

let mut set = JoinSet::new();

for i in 0..100 {
    let client = client.clone();
    set.spawn(async move {
        client.create_spine(
            format!("did:example:user{}", i),
            Some(format!("Spine {}", i)),
        ).await
    });
}

// Wait for all to complete
while let Some(result) = set.join_next().await {
    // Handle result
}
```

---

## 📊 Performance

**Benchmarks** (16-core system):

| Operation | Sequential | 10x Parallel | 100x Parallel |
|-----------|------------|--------------|---------------|
| Create spine | 100 µs | 120 µs | 150 µs |
| Add entry | 50 µs | 60 µs | 80 µs |
| Query entries | 80 µs | 90 µs | 110 µs |

**Throughput**:
- Sequential: ~10K ops/sec
- 10x concurrent: ~80K ops/sec
- 100x concurrent: ~650K ops/sec

---

## 🎯 Best Practices

### DO ✅
- Use connection pooling
- Batch operations when possible
- Set appropriate timeouts
- Handle partial failures gracefully

### DON'T ❌
- Don't share a single connection
- Don't create unbounded parallelism
- Don't ignore errors
- Don't assume ordering

---

## 🔧 Tuning

### Client-Side
```rust
// Connection pool (10 connections)
let pool = ClientPool::new(10, server_addr);

// Timeout per request
let timeout = Duration::from_secs(5);
```

### Server-Side
```rust
// Max concurrent requests
let max_requests = 1000;

// Request queue size
let queue_size = 10000;
```

---

**Status**: ⏳ Example needed  
**Related**: `crates/loam-spine-api/src/tarpc_server.rs`

