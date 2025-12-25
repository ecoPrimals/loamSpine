# 🦴 LoamSpine Showcase Index

**Version**: 0.4.1  
**Last Updated**: December 24, 2025

---

## 📁 Structure

```
showcase/
├── 00_SHOWCASE_INDEX.md        # This file
├── README.md                   # Main showcase overview
├── SHOWCASE_PRINCIPLES.md      # No mocks philosophy
├── QUICK_START.sh              # Run all demos
│
├── 01-local-primal/            # Phase 1: LoamSpine BY ITSELF
│   └── README.md
│
├── 02-rpc-api/                 # Phase 2: Pure Rust RPC
│   └── README.md
│
└── 03-inter-primal/            # Phase 3: Ecosystem Integration
    └── README.md
```

---

## 🚀 Quick Start

```bash
# From loamSpine root
cd showcase
./QUICK_START.sh
```

Or run individual examples:

```bash
cargo run --example demo_hello_loamspine
cargo run --example demo_entry_types
cargo run --example demo_certificate_lifecycle
cargo run --example demo_backup_restore
```

---

## 📊 Available Examples

| Example | Description | Phase | Crate |
|---------|-------------|-------|-------|
| `demo_hello_loamspine` | First spine creation | 1 | loam-spine-core |
| `demo_entry_types` | Entry type variants | 1 | loam-spine-core |
| `demo_certificate_lifecycle` | Mint → Transfer → Loan → Return | 1 | loam-spine-core |
| `demo_backup_restore` | Export/import with verification | 1 | loam-spine-core |
| `demo_rpc_service` | Pure Rust RPC API | 2 | loam-spine-api |
| `demo_inter_primal` | Integration traits | 3 | loam-spine-core |

---

## 🎯 Progressive Phases

### Phase 1: Local Primal (60-90 min)
LoamSpine BY ITSELF is powerful:
- Sovereign spine creation
- Entry types and chaining
- Certificate lifecycle
- Backup and restore

### Phase 2: RPC API (30-45 min)
Pure Rust RPC — No gRPC, no protobuf:
- tarpc for primal-to-primal
- JSON-RPC 2.0 for external clients
- Health monitoring

### Phase 3: Inter-Primal (45-60 min)
Ecosystem integration:
- Ephemeral session commits
- Semantic attribution braid commits
- Signing capabilities

---

## 🔗 Capability-Based Integration

LoamSpine uses capability discovery for all external services:

| Capability | Environment Variable | Description |
|------------|---------------------|-------------|
| Signing | `LOAMSPINE_SIGNER_PATH` | Ed25519 signing service |
| Storage | `LOAMSPINE_STORAGE_PATH` | Content-addressable storage |
| Discovery | (via capability registry) | Service mesh integration |

*No primal names are hardcoded - all services are discovered at runtime.*

---

## 📚 References

- [README.md](./README.md) - Main overview
- [SHOWCASE_PRINCIPLES.md](./SHOWCASE_PRINCIPLES.md) - Philosophy
- [../STATUS.md](../STATUS.md) - Project status
- [../specs/](../specs/) - Specifications

---

🦴 **LoamSpine: Where memories become permanent.**

