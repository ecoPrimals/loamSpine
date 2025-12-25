# 🦴 LoamSpine Inter-Primal Integration

**Phase 3: LoamSpine in the Ecosystem**

---

## 🎯 Purpose

This phase demonstrates how LoamSpine integrates with other primals using
capability-based discovery. No primal names are hardcoded - all services
are discovered at runtime.

- **Ephemeral Storage** → LoamSpine: Session commits
- **Semantic Attribution** → LoamSpine: Braid commits
- **Signing Service** → LoamSpine: Cryptographic signing via capability registry

---

## 🔗 Integration Architecture

```
╔═══════════════════════════════════════════════════════════════╗
║                      ECOPRIMALS PHASE 2                        ║
╠═══════════════════════════════════════════════════════════════╣
║                                                                 ║
║  ┌─────────────┐    CommitAcceptor    ┌─────────────────────┐ ║
║  │  Ephemeral  │ ──────────────────▶ │                     │ ║
║  │  Storage    │    SessionCommit     │                     │ ║
║  └─────────────┘                      │                     │ ║
║                                       │    LoamSpine        │ ║
║  ┌─────────────┐    BraidAcceptor     │    (Permanence)     │ ║
║  │  Semantic   │ ──────────────────▶ │                     │ ║
║  │  Attribution│    BraidCommit       │                     │ ║
║  └─────────────┘                      └──────────┬──────────┘ ║
║                                                   │            ║
║                                                   │ Signer     ║
║                                                   ▼ Verifier   ║
║                                       ┌─────────────────────┐ ║
║                                       │    Signing          │ ║
║                                       │    Service          │ ║
║                                       └─────────────────────┘ ║
║                                                                 ║
║  ┌─────────────────────────────────────────────────────────┐  ║
║  │              CapabilityRegistry (Runtime Discovery)       │  ║
║  └─────────────────────────────────────────────────────────┘  ║
║                                                                 ║
╚═══════════════════════════════════════════════════════════════╝
```

---

## 📋 Integration Traits

### CommitAcceptor (Ephemeral Storage → LoamSpine)

```rust
use loam_spine_core::traits::{CommitAcceptor, DehydrationSummary};

// Ephemeral primal calls this when dehydrating a session
let summary = DehydrationSummary::new(session_id, "game", merkle_root)
    .with_vertex_count(42);

let commit_ref = service.commit_session(spine_id, owner, summary).await?;
```

### BraidAcceptor (Semantic Attribution → LoamSpine)

```rust
use loam_spine_core::traits::{BraidAcceptor, BraidSummary};

// Attribution primal calls this when committing a braid
let braid = BraidSummary::new(braid_id, "attribution", subject_hash, braid_hash);

let entry_hash = service.commit_braid(spine_id, owner, braid).await?;
```

### Signer/Verifier (via Capability Registry)

```rust
use loam_spine_core::discovery::CapabilityRegistry;
use loam_spine_core::traits::Signer;

// Signing service registers its capabilities
registry.register_signer(signer).await;

// LoamSpine discovers and uses them
if let Some(signer) = registry.get_signer().await {
    let sig = signer.sign_boxed(data).await?;
}
```

---

## 📋 Demos

### Level 1: Session Commits (15 min)
**Directory**: `01-session-commit/`

Session dehydration to permanence.

**Scenario**: A game session with 100 vertices is dehydrated and committed.

### Level 2: Braid Commits (15 min)
**Directory**: `02-braid-commit/`

Semantic attribution commits.

**Scenario**: An attribution braid for authorship is committed.

### Level 3: Signing Capability (15 min)
**Directory**: `03-signing-capability/`

Capability-based cryptography.

**Scenario**: Entries are signed with discovered signing capability.

### Level 4: Full Ecosystem (20 min)
**Directory**: `04-full-ecosystem/`

Complete primal coordination.

**Scenario**: All capabilities working together.

---

## 🏆 Success Criteria

After completing this phase:

- [ ] Understand `CommitAcceptor` trait
- [ ] Understand `BraidAcceptor` trait
- [ ] Understand capability-based signing
- [ ] See full ecosystem in action

---

## ⚠️ Current Status

The integration traits are **fully implemented** and tested.

| Capability | Status |
|------------|--------|
| Ephemeral Storage | ✅ Traits ready (`CommitAcceptor`) |
| Semantic Attribution | ✅ Traits ready (`BraidAcceptor`) |
| Signing | ✅ Traits ready (`CliSigner`, `CliVerifier`) |

**Key Principle**: LoamSpine discovers capabilities at runtime. No primal
names are hardcoded in the code.

---

## 🎓 Learning Path

1. Study trait definitions in `crates/loam-spine-core/src/traits/`
2. Review capability registry in `crates/loam-spine-core/src/discovery.rs`
3. Run demo examples
4. Understand the ecosystem integration patterns

---

## 🔗 Related Documentation

- [INTEGRATION_SPECIFICATION.md](../../specs/INTEGRATION_SPECIFICATION.md)
- [ARCHITECTURE.md](../../specs/ARCHITECTURE.md)
- [PURE_RUST_RPC.md](../../specs/PURE_RUST_RPC.md)

---

🦴 **LoamSpine: The Permanence Layer for ecoPrimals**
