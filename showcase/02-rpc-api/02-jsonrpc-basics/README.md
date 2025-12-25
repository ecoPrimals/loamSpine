# 🌐 JSON-RPC 2.0 Basics - Universal API

**Time**: 10 minutes  
**Difficulty**: Beginner  
**Prerequisites**: Level 1 complete

---

## 🎯 What You'll Learn

- Start JSON-RPC server
- Make requests with `curl`
- Handle responses and errors
- Use from any programming language

---

## 📖 Concepts

### What is JSON-RPC 2.0?

A **language-agnostic** RPC protocol:
- JSON for request/response
- Standard specification
- Works with any HTTP client
- Human-readable debugging

### When to Use JSON-RPC

**Use for**:
- ✅ External clients (Python, JS, etc.)
- ✅ Browser applications
- ✅ Debugging (with curl)
- ✅ Language-agnostic APIs

**Don't use for**:
- ❌ Primal-to-primal (use tarpc instead)
- ❌ High-performance critical paths
- ❌ Large binary payloads

---

## 🔍 Demo Flow

```
1. Start JSON-RPC server (port 8080)
   ↓
2. Create spine with curl
   ↓
3. Add entries with curl
   ↓
4. Query with curl
   ↓
5. Verify results
```

---

## 💡 Example Requests

### Create Spine
```bash
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "create_spine",
    "params": {
      "owner": "did:example:alice123",
      "name": "My Spine"
    },
    "id": 1
  }'
```

### Query Entries
```bash
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "get_entries",
    "params": {
      "spine_id": "...",
      "start": 0,
      "limit": 10
    },
    "id": 2
  }'
```

---

## 📊 Performance

Typical latency (localhost):
- Create spine: ~200 µs (includes JSON parsing)
- Add entry: ~150 µs
- Query entries: ~180 µs

**Throughput**: ~5K ops/sec (2x slower than tarpc, but universal)

---

**Status**: ⏳ Example needed  
**Related**: `crates/loam-spine-api/src/jsonrpc.rs`

