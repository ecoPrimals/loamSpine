# 🗄️ Storage Backends - InMemory vs Sled

**Time**: 10 minutes  
**Difficulty**: Beginner  
**Prerequisites**: 01-hello-loamspine

---

## 🎯 What You'll Learn

- InMemory storage for testing
- Sled storage for production
- Performance comparison
- When to use each backend

---

## 📖 Storage Options

### 1. InMemory
- **Use**: Testing, demos, temporary data
- **Pros**: Fast, no I/O, easy setup
- **Cons**: Lost on restart, memory limited

### 2. Sled
- **Use**: Production, persistence required
- **Pros**: Durable, pure Rust, ACID
- **Cons**: Disk I/O, needs flush

---

## 🔍 Comparison

| Feature | InMemory | Sled |
|---------|----------|------|
| **Speed** | ⚡ Fastest | 🚀 Fast |
| **Persistence** | ❌ No | ✅ Yes |
| **Setup** | 🟢 Trivial | 🟢 Easy |
| **Use Case** | Testing | Production |

---

## 💡 Key Features

- ✅ Same API for both backends
- ✅ Easy migration (export/import)
- ✅ Pure Rust (no C dependencies)
- ✅ Thread-safe operations

---

**Status**: ⏳ Rust example needed  
**Related**: `crates/loam-spine-core/src/storage/`

