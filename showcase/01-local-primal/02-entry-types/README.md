# 📋 Entry Types - Understanding All 15+ Variants

**Time**: 10 minutes  
**Difficulty**: Beginner  
**Prerequisites**: Hello LoamSpine complete

---

## 🎯 What You'll Learn

- All 15+ `EntryType` variants
- When to use each entry type
- How different operations map to entries
- Entry type organization by category

---

## 📖 Entry Type Categories

### Session Management (3 types)
- **`SessionStart`**: Begin a new session
- **`SessionEvent`**: Record session activity
- **`SessionEnd`**: Finalize session

### Certificate Operations (4 types)
- **`CertificateMint`**: Issue new certificate
- **`CertificateTransfer`**: Transfer ownership
- **`CertificateLoan`**: Temporary loan
- **`CertificateReturn`**: Return from loan

### Slice Operations (4 types)
- **`SliceCreate`**: Create new slice
- **`SliceJoin`**: Join existing slice
- **`SliceDeparture`**: Leave slice
- **`SliceSeal`**: Seal slice permanently

### Braid Operations (3 types)
- **`BraidJoin`**: Join distributed braid
- **`BraidContribution`**: Add to braid
- **`BraidSeal`**: Seal braid (consensus)

### Data Operations (2 types)
- **`MetadataUpdate`**: Update spine metadata
- **`DataAnchor`**: Anchor external data

---

## 🔍 Demo Flow

```
1. Initialize spine
   ↓
2. Demonstrate session types
   ↓
3. Demonstrate certificate types
   ↓
4. Demonstrate slice types
   ↓
5. Demonstrate braid types
   ↓
6. Demonstrate data types
   ↓
7. Review all entries
```

---

## 💡 When to Use Each Type

### Session Types
**Use when**: Tracking user activity, audit logging
```rust
EntryType::SessionStart { session_id, user_did }
EntryType::SessionEvent { session_id, event_type, data }
EntryType::SessionEnd { session_id, duration }
```

### Certificate Types
**Use when**: Managing credentials, permissions, assets
```rust
EntryType::CertificateMint { certificate_id, holder, capabilities }
EntryType::CertificateTransfer { certificate_id, from, to }
```

### Slice Types
**Use when**: Managing group memberships, collaborations
```rust
EntryType::SliceCreate { slice_id, name, participants }
EntryType::SliceJoin { slice_id, participant }
```

### Braid Types
**Use when**: Distributed consensus, multi-party workflows
```rust
EntryType::BraidJoin { braid_id, participant }
EntryType::BraidContribution { braid_id, data }
```

### Data Types
**Use when**: Anchoring external content, metadata updates
```rust
EntryType::DataAnchor { content_type, hash, size }
EntryType::MetadataUpdate { key, value }
```

---

## 📊 Entry Type Matrix

| Category | Count | Use Case | Typical Spine |
|----------|-------|----------|---------------|
| Session | 3 | User activity | User audit log |
| Certificate | 4 | Credentials | Asset ledger |
| Slice | 4 | Group membership | Collaboration spine |
| Braid | 3 | Distributed work | Consensus spine |
| Data | 2 | Content anchoring | Data registry |

---

## 🎯 Success Criteria

- ✅ All 15+ entry types demonstrated
- ✅ Understand category organization
- ✅ Know when to use each type
- ✅ See entry type in spine context

---

## 💡 Design Patterns

### Pattern 1: Session Spine
```
Height 0: SessionStart
Height 1: SessionEvent (login)
Height 2: SessionEvent (action)
Height 3: SessionEvent (action)
Height 4: SessionEnd
```

### Pattern 2: Certificate Spine
```
Height 0: CertificateMint
Height 1: CertificateTransfer
Height 2: CertificateLoan
Height 3: CertificateReturn
Height 4: CertificateTransfer
```

### Pattern 3: Braid Spine
```
Height 0: BraidJoin (Alice)
Height 1: BraidJoin (Bob)
Height 2: BraidContribution (Alice)
Height 3: BraidContribution (Bob)
Height 4: BraidSeal
```

---

## 🔧 Extending Entry Types

**Adding new types** (future):
1. Define in `EntryType` enum
2. Implement serialization
3. Add validation logic
4. Update documentation
5. Add tests

**Guidelines**:
- Keep types semantic
- Include all required context
- Support querying
- Enable proof generation

---

**Status**: ✅ Example complete  
**Related**: `crates/loam-spine-core/src/entry.rs`

**Next**: `03-certificate-lifecycle` - Certificate operations

