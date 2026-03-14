# 🗄️ Storage Backends - InMemory vs redb vs Sled

**Time**: 10 minutes  
**Difficulty**: Beginner  
**Prerequisites**: 01-hello-loamspine

---

## 🎯 What You'll Learn

- InMemory storage for testing
- redb storage for production (default)
- Sled storage (optional, legacy)
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

### 3. Sled (Optional)
- **Use**: Legacy, alternative persistence
- **Pros**: Durable, pure Rust, ACID
- **Cons**: Disk I/O, needs flush

---

## 🔍 Comparison

| Feature | InMemory | redb | Sled |
|---------|----------|------|------|
| **Speed** | ⚡ Fastest | 🚀 Fast | 🚀 Fast |
| **Persistence** | ❌ No | ✅ Yes | ✅ Yes |
| **Setup** | 🟢 Trivial | 🟢 Easy | 🟢 Easy |
| **Use Case** | Testing | Production (default) | Optional |

---

## 💡 Key Features

- ✅ Same API for all backends
- ✅ Easy migration (export/import)
- ✅ Pure Rust (no C dependencies)
- ✅ Thread-safe operations

---

**Status**: ⏳ Rust example needed  
**Related**: `crates/loam-spine-core/src/storage/`

