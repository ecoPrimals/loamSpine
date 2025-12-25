# 🛡️ RPC Error Handling - Graceful Failure

**Time**: 10 minutes  
**Difficulty**: Intermediate  
**Prerequisites**: Basic RPC understanding

---

## 🎯 What You'll Learn

- Handle RPC errors gracefully
- Distinguish transient vs permanent failures
- Implement retry strategies
- Design fault-tolerant clients

---

## 📖 Concepts

### Error Categories

**Network Errors** (transient):
- Connection timeout
- Connection refused
- Network unreachable
- **Action**: Retry with backoff

**Application Errors** (permanent):
- Invalid spine ID
- Permission denied
- Invalid input
- **Action**: Don't retry, fix input

**Server Errors** (mixed):
- Internal server error
- Storage failure
- Out of memory
- **Action**: Retry once, then fail

---

## 🔍 Demo Flow

```
1. Start LoamSpine service
   ↓
2. Test network failures (port closed)
   ↓
3. Test application errors (invalid IDs)
   ↓
4. Test server errors (storage issues)
   ↓
5. Demonstrate recovery strategies
```

---

## 💡 Error Response Format

### tarpc Error
```rust
match client.get_spine(spine_id).await {
    Ok(Ok(spine)) => { /* success */ },
    Ok(Err(e)) => {
        // Application error
        match e {
            LoamSpineError::NotFound(_) => { /* retry won't help */ },
            LoamSpineError::Storage(_) => { /* retry might work */ },
            _ => { /* handle other cases */ },
        }
    },
    Err(e) => {
        // Network/RPC error - retry!
    }
}
```

### JSON-RPC Error
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid params: spine_id required",
    "data": {
      "field": "spine_id",
      "reason": "missing"
    }
  },
  "id": 1
}
```

---

## 🔧 Retry Strategies

### Exponential Backoff
```rust
let mut delay = Duration::from_millis(100);
for attempt in 1..=3 {
    match client.operation().await {
        Ok(result) => return Ok(result),
        Err(e) if is_transient(&e) => {
            tokio::time::sleep(delay).await;
            delay *= 2; // 100ms, 200ms, 400ms
        },
        Err(e) => return Err(e), // Don't retry
    }
}
```

### Circuit Breaker
```rust
if circuit_breaker.is_open() {
    return Err("Service unavailable");
}

match client.operation().await {
    Ok(result) => {
        circuit_breaker.record_success();
        Ok(result)
    },
    Err(e) => {
        circuit_breaker.record_failure();
        Err(e)
    }
}
```

---

## 📊 Error Codes (JSON-RPC 2.0)

| Code | Meaning | Action |
|------|---------|--------|
| -32700 | Parse error | Check JSON syntax |
| -32600 | Invalid request | Check request format |
| -32601 | Method not found | Check method name |
| -32602 | Invalid params | Check parameters |
| -32603 | Internal error | Retry |
| -32000 | Server error | Retry once |
| -32001 | Not found | Don't retry |
| -32002 | Permission denied | Don't retry |

---

## 🎯 Best Practices

### DO ✅
- Log errors with context
- Retry transient failures (max 3 times)
- Use exponential backoff
- Implement circuit breakers
- Set reasonable timeouts
- Return structured errors

### DON'T ❌
- Don't retry forever
- Don't retry non-transient errors
- Don't hide error details
- Don't use generic error messages
- Don't ignore partial failures

---

## 🔍 Debugging

### Client-Side Logging
```
ERROR: RPC call failed
  method: get_spine
  spine_id: abc123...
  error: Connection refused (ECONNREFUSED)
  attempt: 2/3
  next_retry: 200ms
```

### Server-Side Logging
```
ERROR: Request failed
  method: get_spine
  spine_id: abc123...
  error: Storage backend unavailable
  latency: 5.2s (timeout: 5s)
  client: 127.0.0.1:54321
```

---

## 💡 Testing Failures

**Simulate**:
- Network partition (iptables)
- Port closed (kill server)
- Slow responses (add delay)
- Invalid data (corrupt storage)

**Verify**:
- Retries work
- Timeouts trigger
- Errors propagate
- Logs are helpful

---

**Status**: ⏳ Example needed  
**Related**: `crates/loam-spine-core/src/error.rs`

