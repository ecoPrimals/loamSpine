# 🦴 LoamSpine Local Primal Capabilities

**Phase 1: LoamSpine BY ITSELF is Amazing**

---

## 🎯 Purpose

This showcase demonstrates LoamSpine's value **independently**.

Before showing how LoamSpine integrates with the ecosystem, let's show what makes it special:

- ✅ Sovereign spines (you own your history)
- ✅ 15+ entry type variants
- ✅ Full certificate lifecycle (mint, transfer, loan, return)
- ✅ Cryptographic proofs (inclusion, provenance)
- ✅ Backup/restore with verification
- ✅ Pure Rust storage (InMemory + Sled)
- ✅ Zero hardcoded dependencies

---

## 🚀 Quick Start

```bash
# From loamSpine root
cargo build --release

# Run all local demos
cd showcase/01-local-primal
./RUN_ALL.sh
```

Or run individual demos:
```bash
cargo run --example demo_hello_loamspine
cargo run --example demo_entry_types
cargo run --example demo_certificate_lifecycle
cargo run --example demo_proofs
cargo run --example demo_backup_restore
```

---

## 📋 Progressive Levels

### Level 1: Hello LoamSpine (5 min)
**Directory**: `01-hello-loamspine/`

Your first contact with sovereign spines.

**Demos**:
- `demo_hello_loamspine.rs` - Create your first spine

**Learn**: Spine creation, DIDs, SpineBuilder

---

### Level 2: Entry Types (15 min)
**Directory**: `02-entry-types/`

All 15+ entry type variants in action.

**Demos**:
- `demo_entry_types.rs` - Comprehensive entry types

**Learn**: EntryType variants, entry chaining, validation

---

### Level 3: Certificate Lifecycle (20 min)
**Directory**: `03-certificate-lifecycle/`

Full ownership model: mint → transfer → loan → return.

**Demos**:
- `demo_certificate_lifecycle.rs` - Complete lifecycle

**Learn**: Certificates, loans, provenance tracking

---

### Level 4: Proofs (15 min)
**Directory**: `04-proofs/`

Cryptographic verification of history.

**Demos**:
- `demo_proofs.rs` - Inclusion and provenance proofs

**Learn**: InclusionProof, ProvenanceProof, verification

---

### Level 5: Backup/Restore (10 min)
**Directory**: `05-backup-restore/`

Export and import spines with verification.

**Demos**:
- `demo_backup_restore.rs` - Full backup cycle

**Learn**: SpineBackup, binary vs JSON, verification

---

### Level 6: Storage Backends (10 min)
**Directory**: `06-storage-backends/`

InMemory for testing, Sled for production.

**Demos**:
- `demo_storage.rs` - Both storage backends

**Learn**: SpineStorage, EntryStorage, Sled persistence

---

## 🎓 Learning Path

**Recommended order**:
1. Start with Hello LoamSpine
2. Explore entry types
3. Master certificates
4. Understand proofs
5. Learn backup/restore
6. Choose storage backend

**Time**: 60-90 minutes for complete local showcase

---

## 🏆 Success Criteria

After completing this showcase, you should understand:

- ✅ How to create and manage spines
- ✅ The different entry types and when to use them
- ✅ How certificate ownership works
- ✅ How to generate and verify proofs
- ✅ How to backup and restore spines
- ✅ The difference between storage backends
- ✅ Why LoamSpine is valuable BY ITSELF

---

## 🎯 Next Phase

Once you've seen LoamSpine's local capabilities:

**Phase 2**: `../02-rpc-api/` - See the Pure Rust RPC API

**Phase 3**: `../03-inter-primal/` - Complete ecosystem integration

---

**Following ecosystem showcase patterns:**
- 🎵 Songbird: Multi-tower federation
- 🍄 ToadStool: GPU compute benchmarks
- 🐻 BearDog: Interactive demos
- 🏰 NestGate: Progressive levels
- 🐿️ Squirrel: Universal AI orchestration

🦴 **LoamSpine: Sovereign Permanence**

