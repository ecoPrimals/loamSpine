# 🔌 Level 2: RPC API Demos

**Purpose**: Demonstrate LoamSpine's Pure Rust RPC capabilities  
**Philosophy**: No gRPC, no protobuf, no C++ tooling  
**Time**: 30-45 minutes total

---

## 🎯 Overview

LoamSpine uses **Pure Rust RPC** with dual protocol support:

1. **tarpc** - High-performance binary RPC for primal-to-primal communication
2. **JSON-RPC 2.0** - Language-agnostic API for external clients

**Why Pure Rust?**
- ✅ No `protoc` compiler required
- ✅ No C++ dependencies
- ✅ Standard `cargo build`
- ✅ Native Rust types
- ✅ Procedural macros (no code generation)

---

## 📁 Demos

| # | Demo | Description | Time |
|---|------|-------------|------|
| 01 | tarpc-basics | Binary RPC fundamentals | 10 min |
| 02 | jsonrpc-basics | JSON-RPC 2.0 with curl | 10 min |
| 03 | health-monitoring | Service health checks | 5 min |
| 04 | concurrent-ops | Parallel RPC operations | 10 min |
| 05 | error-handling | Error scenarios & recovery | 10 min |

---

## 🚀 Quick Start

```bash
# Run all Level 2 demos
cd showcase/02-rpc-api
./RUN_ALL.sh

# Or individual demos
./01-tarpc-basics/demo.sh
./02-jsonrpc-basics/demo.sh
```

---

## 🎓 Learning Path

### For Developers
1. Start with `01-tarpc-basics` to understand binary RPC
2. Try `02-jsonrpc-basics` for external API usage
3. Study `04-concurrent-ops` for production patterns

### For Operators
1. Review `03-health-monitoring` for service checks
2. Study `05-error-handling` for failure modes
3. Check production deployment patterns

---

## 🔧 Prerequisites

- LoamSpine built (`cargo build --release`)
- Basic understanding of Level 1 concepts
- Optional: `curl` for JSON-RPC demos

---

## 📊 RPC Methods (18 total)

**Spine Management** (6 methods):
- `get_spine`, `create_spine`, `list_spines`, `get_tip`, `get_entries`, `seal_spine`

**Certificate Operations** (4 methods):
- `mint_certificate`, `transfer_certificate`, `loan_certificate`, `return_certificate`

**Proofs & Queries** (4 methods):
- `get_inclusion_proof`, `get_certificate_proof`, `get_certificate_history`, `query_entries`

**Integration** (2 methods):
- `commit_session`, `commit_braid`

**Service** (2 methods):
- `health_check`, `get_metrics`

---

## 💡 Key Concepts

### tarpc (Binary RPC)
- **Fast**: Native Rust serialization
- **Type-safe**: Compile-time checks
- **Use**: Primal-to-primal communication

### JSON-RPC 2.0 (Universal API)
- **Standard**: JSON-RPC 2.0 spec compliant
- **Language-agnostic**: curl, Python, JS, etc.
- **Use**: External clients, debugging

---

## 🎯 Success Criteria

By the end of Level 2, you should:
- ✅ Understand tarpc vs JSON-RPC tradeoffs
- ✅ Make RPC calls from Rust and curl
- ✅ Handle errors gracefully
- ✅ Monitor service health
- ✅ Use RPC in production

---

## 🔗 Next Steps

- **Level 3**: Songbird Discovery (capability-based service discovery)
- **Level 4**: Inter-Primal (full ecosystem integration)

---

**Status**: ⏳ Documentation complete, examples pending  
**Related**: `crates/loam-spine-api/`

🦴 **LoamSpine: Where memories become permanent.**
