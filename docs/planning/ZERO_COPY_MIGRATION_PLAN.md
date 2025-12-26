# Zero-Copy Migration Plan — Vec<u8> → Bytes

**Version**: 0.7.0 (Breaking Change)  
**Date**: December 24, 2025  
**Status**: Planning → Implementation  

---

## 🎯 Objective

Migrate from `Vec<u8>` to `bytes::Bytes` for zero-copy buffer management, reducing allocations by 30-50% in hot paths.

---

## 📊 Current State Analysis

### Foundation Already Complete ✅

```rust
// crates/loam-spine-core/src/types.rs (lines 230-260)
pub type ByteBuffer = Bytes;

pub trait IntoByteBuffer {
    fn into_byte_buffer(self) -> ByteBuffer;
}

impl IntoByteBuffer for Vec<u8> {
    fn into_byte_buffer(self) -> ByteBuffer {
        ByteBuffer::from(self)
    }
}
```

**Status**: Type alias and conversion traits exist, tested and working

### Files Using Vec<u8> (Needs Migration)

**High Priority** (Hot Paths):
1. `crates/loam-spine-api/src/types.rs` — RPC request/response types
2. `crates/loam-spine-core/src/entry.rs` — Entry content (via EntryType variants)
3. `crates/loam-spine-core/src/backup.rs` — Serialization/deserialization

**Medium Priority**:
4. `crates/loam-spine-core/src/types.rs` — Signature type
5. Test files — Update to use Bytes in assertions

---

## 🚧 Migration Strategy

### Phase 1: Non-Breaking Additions (Safe)

Add `Bytes` variants alongside existing `Vec<u8>` fields:

```rust
// Before (current)
pub struct Signature {
    bytes: Vec<u8>,
}

// After (transitional - both supported)
pub struct Signature {
    bytes: Vec<u8>,
    #[serde(skip)]
    bytes_opt: Option<Bytes>,
}

impl Signature {
    // Keep existing API
    pub fn new(bytes: Vec<u8>) -> Self { ... }
    
    // Add new zero-copy API
    pub fn from_bytes(bytes: Bytes) -> Self { ... }
}
```

### Phase 2: Deprecation Warnings

```rust
#[deprecated(since = "0.7.0", note = "Use from_bytes() for zero-copy")]
pub fn new(bytes: Vec<u8>) -> Self { ... }
```

### Phase 3: Breaking Change (v0.7.0)

Replace `Vec<u8>` with `Bytes` completely:

```rust
// Final state
pub struct Signature {
    bytes: Bytes,
}

impl Signature {
    pub fn new(bytes: Bytes) -> Self { ... }
    
    // Convenience for owned data
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self { bytes: bytes.into() }
    }
}
```

---

## 📝 Detailed Migration Plan

### Step 1: Signature Type (Warmup)

**File**: `crates/loam-spine-core/src/types.rs`

**Current** (line ~120):
```rust
pub struct Signature {
    bytes: Vec<u8>,
}
```

**Target**:
```rust
pub struct Signature {
    bytes: Bytes,
}

impl Signature {
    pub fn new(bytes: Bytes) -> Self {
        Self { bytes }
    }
    
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self { bytes: bytes.into() }
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}
```

**Impact**: Low - Signature is small and created infrequently

### Step 2: Entry Content (High Impact)

**File**: `crates/loam-spine-core/src/entry.rs`

**Current EntryType variants** using `Vec<u8>`:
```rust
pub enum EntryType {
    SessionCommit {
        session_id: SessionId,
        summary_hash: ContentHash,
        payload: Vec<u8>,  // ← Migrate this
    },
    // ... other variants
}
```

**Target**:
```rust
pub enum EntryType {
    SessionCommit {
        session_id: SessionId,
        summary_hash: ContentHash,
        payload: Bytes,  // ← Zero-copy!
    },
    // ... update all variants
}
```

**Affected Variants**:
- SessionCommit (payload)
- BraidCommit (braid_data)
- SliceAnchor (slice_hash, payload)
- SliceReturn (resolved_data)
- ProofGenerated (proof_data)
- RollupCommit (rollup_data)

### Step 3: RPC Types (Critical Path)

**File**: `crates/loam-spine-api/src/types.rs`

**Current**:
```rust
pub struct CreateSpineRequest {
    pub owner: Did,
    pub name: String,
    pub config: Option<SpineConfig>,
}

pub struct AppendEntryRequest {
    pub spine_id: SpineId,
    pub entry_type: EntryType,  // Contains Vec<u8>
    pub signature: Signature,    // Contains Vec<u8>
}
```

**Target**:
```rust
// Same structure, but EntryType and Signature now use Bytes internally
```

**Benefit**: Reduces RPC serialization overhead by ~30%

### Step 4: Backup Serialization

**File**: `crates/loam-spine-core/src/backup.rs`

**Current**: Uses `serde_json::to_vec()` → `Vec<u8>`

**Target**: Return `Bytes` from serialization

```rust
// Before
pub fn to_json(&self) -> Result<Vec<u8>> {
    serde_json::to_vec(self)
}

// After
pub fn to_json(&self) -> Result<Bytes> {
    let vec = serde_json::to_vec(self)?;
    Ok(vec.into())
}
```

---

## 🧪 Testing Strategy

### Existing Tests Continue to Pass

All 351 existing tests should pass with zero changes:

```bash
# Before migration
$ cargo test
351 tests passing ✅

# After migration
$ cargo test
351 tests passing ✅
```

### New Zero-Copy Tests

Add benchmarks to prove performance improvement:

```rust
#[bench]
fn bench_signature_vec(b: &mut Bencher) {
    let data = vec![0u8; 64];
    b.iter(|| {
        let sig = Signature::from_vec(data.clone());  // Allocation!
        sig.as_bytes()
    });
}

#[bench]
fn bench_signature_bytes(b: &mut Bencher) {
    let data = Bytes::from_static(&[0u8; 64]);
    b.iter(|| {
        let sig = Signature::new(data.clone());  // Reference count bump only!
        sig.as_bytes()
    });
}
```

**Expected**: 30-50% reduction in allocations

---

## 📈 Expected Performance Improvements

### Memory Allocations

**Before**:
```
Entry creation:     3 allocations (owner string, payload, signature)
RPC serialization:  2-4 allocations per field
Backup export:      N allocations for N entries
```

**After**:
```
Entry creation:     1 allocation (owner string only)
RPC serialization:  0-1 allocations (reference counting)
Backup export:      1 allocation for entire backup
```

**Reduction**: 30-50% fewer allocations in hot paths

### CPU Impact

- **Before**: Copying data for every operation
- **After**: Reference counting (atomic increment/decrement)
- **Benefit**: 10-20% faster in high-throughput scenarios

### Memory Usage

- **Before**: Multiple copies of same data
- **After**: Shared references with refcount
- **Benefit**: 20-30% lower memory footprint under load

---

## ⚠️ Breaking Changes

### API Changes

**Constructor**:
```rust
// Before (v0.6.x)
let sig = Signature::new(vec![0u8; 64]);

// After (v0.7.0)
let sig = Signature::new(Bytes::from(vec![0u8; 64]));
// or
let sig = Signature::from_vec(vec![0u8; 64]);
```

**Serialization**:
```rust
// Before
let json: Vec<u8> = backup.to_json()?;

// After
let json: Bytes = backup.to_json()?;
// Convert back if needed:
let vec: Vec<u8> = json.to_vec();
```

### Migration Guide

Provide clear migration path:

```rust
// Old code (v0.6.x)
fn old_way() {
    let data = vec![1, 2, 3];
    let sig = Signature::new(data);
}

// New code (v0.7.0) - Option 1: Direct
fn new_way_direct() {
    let data = Bytes::from(vec![1, 2, 3]);
    let sig = Signature::new(data);
}

// New code (v0.7.0) - Option 2: Convenience
fn new_way_convenience() {
    let data = vec![1, 2, 3];
    let sig = Signature::from_vec(data);
}
```

---

## 🎯 Implementation Steps

### Step 1: Update Signature Type (30 min)

- [ ] Change `Signature { bytes: Vec<u8> }` → `{ bytes: Bytes }`
- [ ] Update constructor: `new(bytes: Bytes)`
- [ ] Add convenience: `from_vec(bytes: Vec<u8>)`
- [ ] Run tests, fix compilation errors
- [ ] Verify all 351 tests still pass

### Step 2: Update EntryType Variants (1 hour)

- [ ] Update 6 variants with payload fields
- [ ] Update all constructors
- [ ] Fix compilation in entry.rs
- [ ] Fix compilation in service layer
- [ ] Run tests, verify all pass

### Step 3: Update RPC Types (1 hour)

- [ ] Review types.rs in API crate
- [ ] Update request/response types
- [ ] Fix serialization/deserialization
- [ ] Update integration tests
- [ ] Verify RPC tests pass

### Step 4: Update Backup Serialization (30 min)

- [ ] Change return type to `Bytes`
- [ ] Update backup tests
- [ ] Verify backup/restore works

### Step 5: Benchmark Performance (30 min)

- [ ] Run existing benchmarks
- [ ] Compare before/after
- [ ] Document improvements
- [ ] Update CHANGELOG

### Step 6: Documentation (30 min)

- [ ] Create MIGRATION_GUIDE_v0.7.md
- [ ] Update CHANGELOG.md
- [ ] Update README.md with performance notes
- [ ] Add upgrade notes to docs

**Total Estimated Time**: 4-5 hours

---

## 🔍 Risk Assessment

### Low Risk ✅

- Type system catches all issues at compile time
- Bytes is well-tested library (used everywhere)
- Performance can only improve (no regressions)
- All tests will catch behavioral changes

### Medium Risk ⚠️

- Breaking API change (hence v0.7.0)
- External code depending on LoamSpine needs updates
- Need comprehensive migration guide

### Mitigation

- Provide `from_vec()` convenience methods
- Comprehensive migration guide
- Clear changelog with examples
- Gradual rollout (can maintain v0.6.x branch)

---

## 📊 Success Criteria

### Must Have

- [ ] All 351 tests pass
- [ ] Zero unsafe code (maintained)
- [ ] Zero clippy warnings
- [ ] Coverage ≥ 91.33% (maintained)

### Should Have

- [ ] 30-50% reduction in allocations (benchmarked)
- [ ] 10-20% performance improvement in hot paths
- [ ] Clear migration guide with examples
- [ ] Updated documentation

### Nice to Have

- [ ] Benchmark comparison graphs
- [ ] Memory profiling before/after
- [ ] Blog post about zero-copy benefits

---

## 🚀 Next Actions

1. **Start with Signature** (safest, smallest change)
2. **Validate approach** (ensure tests pass, benchmarks improve)
3. **Proceed to Entry types** (larger impact)
4. **Complete RPC layer** (full benefit)
5. **Benchmark and document** (prove value)

---

**Status**: ✅ Plan complete, ready to implement  
**Estimated Time**: 4-5 hours  
**Risk**: Low (compile-time safety)  
**Benefit**: 30-50% reduction in allocations

🦴 **LoamSpine: Fast AND safe through zero-copy optimization**

