# 🚀 tarpc Basics - High-Performance Binary RPC

**Time**: 10 minutes  
**Difficulty**: Intermediate  
**Prerequisites**: Level 1 complete

---

## 🎯 What You'll Learn

- Start tarpc RPC server
- Connect Rust client to server
- Make RPC calls (create spine, add entries)
- Understand tarpc performance benefits

---

## 📖 Concepts

### What is tarpc?

**tarpc** is a pure Rust RPC framework:
- Native Rust types (no protobuf)
- Compile-time type safety
- High performance (binary serialization)
- Procedural macros (no code generation)

### When to Use tarpc

**Use for**:
- ✅ Primal-to-primal communication
- ✅ High-performance requirements
- ✅ Type-safe APIs
- ✅ Rust-to-Rust services

**Don't use for**:
- ❌ External clients (use JSON-RPC)
- ❌ Non-Rust languages
- ❌ Browser JavaScript

---

## 🔍 Demo Flow

```
1. Start tarpc server (port 9001)
   ↓
2. Connect Rust client
   ↓
3. Create spine via RPC
   ↓
4. Add entries via RPC
   ↓
5. Query spine via RPC
   ↓
6. Verify results
```

---

## 💡 Key Features

- ✅ **Fast**: Binary serialization
- ✅ **Type-safe**: Compile-time checks
- ✅ **Simple**: No code generation
- ✅ **Pure Rust**: No C++ dependencies

---

## 📊 Performance

Typical latency (localhost):
- Create spine: ~100 µs
- Add entry: ~50 µs
- Query entries: ~80 µs

**Throughput**: ~10K ops/sec single-threaded

---

**Status**: ⏳ Example needed  
**Related**: `crates/loam-spine-api/src/tarpc_server.rs`

