# 🗄️ Storage Backends - InMemory vs redb

**Time**: 10 minutes  
**Difficulty**: Beginner  
**Prerequisites**: 01-hello-loamspine

---

## 🎯 What You'll Learn

- InMemory storage for testing
- redb storage for production (default)
- Performance comparison
- When to use each backend

---

## 📖 Storage Options

### 1. InMemory
- **Use**: Testing, demos, temporary data
- **Pros**: Fast, no I/O, easy setup
- **Cons**: Lost on restart, memory limited

### 2. redb (Default Production)
- **Use**: Production, persistence required
- **Pros**: Durable, pure Rust, ACID, default backend
- **Cons**: Disk I/O

---

## 🔍 Comparison

| Feature | InMemory | redb |
|---------|----------|------|
| **Speed** | ⚡ Fastest | 🚀 Fast |
| **Persistence** | ❌ No | ✅ Yes |
| **Setup** | 🟢 Trivial | 🟢 Easy |
| **Use Case** | Testing | Production (default) |

---

## 💡 Key Features

- ✅ Same API for all backends
- ✅ Easy migration (export/import)
- ✅ Pure Rust (no C dependencies)
- ✅ Thread-safe operations

---

**Status**: ✅ Complete (see `storage_backends` example)  
**Related**: `crates/loam-spine-core/src/storage/`
