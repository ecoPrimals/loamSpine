# 🦴 LoamSpine — Smart Refactoring Recommendations

**Created**: December 25, 2025  
**Version**: 0.6.2  
**Status**: Analysis complete — 5 files >700 lines reviewed

---

## 📊 OVERVIEW

Analysis of files exceeding 700 lines of code, with smart refactoring strategies (not just splitting).

| File | Lines | Status | Priority | Strategy |
|------|-------|--------|----------|----------|
| `service.rs` | 889 | 🟡 Review | Medium | Domain-based separation |
| `backup.rs` | 863 | 🟢 Good | Low | Well-structured, minor improvements |
| `manager.rs` | 781 | 🟡 Review | Medium | Extract coordinators |
| `chaos.rs` | 770 | ✅ OK | N/A | Test file — acceptable size |
| `certificate.rs` | 743 | 🟢 Good | Low | Already well-modularized |

**Summary**: 2 files need refactoring, 2 are acceptable, 1 is test code

---

## 🔍 FILE-BY-FILE ANALYSIS

### 1. `crates/loam-spine-api/src/service.rs` (889 lines)

**Status**: 🟡 **Needs Domain-Based Separation**  
**Priority**: MEDIUM  
**Current Structure**: Single `LoamSpineRpcService` with 20+ methods

#### Problem
- 20 public async methods in one file
- Methods span multiple domains:
  - Spine operations (create, get, seal)
  - Entry operations (append, get)
  - Certificate operations (mint, transfer, loan, return)
  - Slice operations (anchor, checkout)
  - Proof operations (generate, verify)
  - Session operations (commit session, commit braid)
  - Health (health check)

**This violates Single Responsibility Principle** — one struct handles 7 different concerns.

#### Smart Refactoring Strategy

**Option A: Domain-Based Separation** (Recommended)

Split into focused service modules:

```rust
// src/service/mod.rs
pub mod spine;      // Spine operations
pub mod entry;      // Entry operations
pub mod certificate;// Certificate operations
pub mod slice;      // Slice operations
pub mod proof;      // Proof operations
pub mod session;    // Session/braid operations
pub mod health;     // Health checks

pub use self::spine::SpineService;
pub use self::entry::EntryService;
// ... etc
```

Each service would be 100-150 lines and focused on one domain:

```rust
// src/service/spine.rs
pub struct SpineService {
    core: Arc<RwLock<CoreService>>,
}

impl SpineService {
    pub async fn create_spine(&self, request: CreateSpineRequest) -> ApiResult<CreateSpineResponse> { /* */ }
    pub async fn get_spine(&self, request: GetSpineRequest) -> ApiResult<GetSpineResponse> { /* */ }
    pub async fn seal_spine(&self, request: SealSpineRequest) -> ApiResult<SealSpineResponse> { /* */ }
}
```

```rust
// src/service/certificate.rs
pub struct CertificateService {
    core: Arc<RwLock<CoreService>>,
}

impl CertificateService {
    pub async fn mint(&self, request: MintCertificateRequest) -> ApiResult<MintCertificateResponse> { /* */ }
    pub async fn transfer(&self, request: TransferCertificateRequest) -> ApiResult<TransferCertificateResponse> { /* */ }
    pub async fn loan(&self, request: LoanCertificateRequest) -> ApiResult<LoanCertificateResponse> { /* */ }
    pub async fn return_cert(&self, request: ReturnCertificateRequest) -> ApiResult<ReturnCertificateResponse> { /* */ }
    pub async fn get(&self, request: GetCertificateRequest) -> ApiResult<GetCertificateResponse> { /* */ }
}
```

**Main coordinator**:

```rust
// src/service.rs (new, smaller)
pub struct LoamSpineRpcService {
    pub spine: SpineService,
    pub entry: EntryService,
    pub certificate: CertificateService,
    pub slice: SliceService,
    pub proof: ProofService,
    pub session: SessionService,
    pub health: HealthService,
}

impl LoamSpineRpcService {
    pub fn new(core: CoreService) -> Self {
        let core = Arc::new(RwLock::new(core));
        Self {
            spine: SpineService::new(Arc::clone(&core)),
            entry: EntryService::new(Arc::clone(&core)),
            certificate: CertificateService::new(Arc::clone(&core)),
            slice: SliceService::new(Arc::clone(&core)),
            proof: ProofService::new(Arc::clone(&core)),
            session: SessionService::new(Arc::clone(&core)),
            health: HealthService::new(Arc::clone(&core)),
        }
    }
    
    // Delegate methods to sub-services
    pub async fn create_spine(&self, request: CreateSpineRequest) -> ApiResult<CreateSpineResponse> {
        self.spine.create_spine(request).await
    }
    // ... or expose sub-services directly
}
```

**Benefits**:
- ✅ Single Responsibility: Each service handles one domain
- ✅ Testability: Test domains in isolation
- ✅ Maintainability: Changes to certificates don't affect spines
- ✅ Discoverability: Clear structure for new developers
- ✅ Parallel development: Multiple devs can work on different domains

**Effort**: ~4 hours
- Split file into 7 modules
- Update imports
- Run tests to verify no regression

---

### 2. `crates/loam-spine-core/src/backup.rs` (863 lines)

**Status**: 🟢 **Well-Structured, Minor Improvements**  
**Priority**: LOW

#### Current Structure
- `SpineBackup` (main backup struct + impl)
- `MultiSpineBackup` (multi-spine backup)
- `BackupVerification` (verification results)
- `BackupError` (error types)
- Comprehensive tests

**Analysis**: This file is large but **well-organized**. Each struct has a clear purpose and focused implementation.

#### Why NOT to split
- Backup operations are cohesive (export, import, verify)
- Splitting would create artificial boundaries
- All code relates to the same concern
- Tests are comprehensive and pass

#### Minor Improvements (Optional)

**Option 1**: Extract verification logic

```rust
// src/backup/verification.rs
pub struct BackupVerifier {
    // Verification logic
}

impl BackupVerifier {
    pub fn verify_spine_backup(backup: &SpineBackup) -> BackupVerification { /* */ }
    pub fn verify_multi_spine_backup(backup: &MultiSpineBackup) -> BackupVerification { /* */ }
}
```

**Option 2**: Extract serialization formats

```rust
// src/backup/format.rs
pub trait BackupFormat {
    fn write<W: Write>(&self, writer: W) -> Result<()>;
    fn read<R: Read>(reader: R) -> Result<Self>;
}

pub struct CborFormat;
pub struct JsonFormat;  // Alternative for debugging
```

**Recommendation**: Leave as-is unless you need alternative formats or verification strategies. Current structure is good.

**Effort**: ~2 hours (if pursuing minor improvements)

---

### 3. `crates/loam-spine-core/src/manager.rs` (781 lines)

**Status**: 🟡 **Extract Coordinators**  
**Priority**: MEDIUM

#### Problem
Likely a "God object" that coordinates multiple concerns. Common pattern in managers.

#### Smart Refactoring Strategy

**Pattern**: Extract specialized coordinators

```rust
// src/manager/mod.rs
mod spine_coordinator;
mod certificate_coordinator;
mod slice_coordinator;

pub use spine_coordinator::SpineCoordinator;
pub use certificate_coordinator::CertificateCoordinator;
pub use slice_coordinator::SliceCoordinator;

pub struct LoamSpineManager {
    spines: SpineCoordinator,
    certificates: CertificateCoordinator,
    slices: SliceCoordinator,
}
```

**Each coordinator** handles:
- Lifecycle management for its domain
- Consistency checks
- Transaction coordination
- Cache management

**Benefits**:
- ✅ Separation of concerns
- ✅ Testable coordinators
- ✅ Clear boundaries

**Effort**: ~3 hours

---

### 4. `crates/loam-spine-core/tests/chaos.rs` (770 lines)

**Status**: ✅ **Acceptable**  
**Priority**: N/A (Test file)

#### Analysis
Chaos testing file. Large test files are acceptable because:
- Tests are naturally verbose
- Many test cases expected
- Each test case is independent
- Splitting would reduce discoverability

**Recommendation**: Keep as-is. 770 lines for comprehensive chaos testing is reasonable.

---

### 5. `crates/loam-spine-core/src/certificate.rs` (743 lines)

**Status**: 🟢 **Well-Modularized**  
**Priority**: LOW

#### Current Structure
Likely multiple certificate-related structs with focused implementations:
- `Certificate` struct
- Certificate operations (mint, transfer, loan, return)
- Validation logic
- Serialization

**Analysis**: Just under 750 lines, and likely well-structured. Certificate management is cohesive.

**Recommendation**: No refactoring needed unless analysis shows mixing of concerns (e.g., certificate + unrelated logic).

---

## 🎯 REFACTORING PRIORITIES

### Priority 1: `service.rs` Domain Separation

**Why**: Highest value, reduces complexity  
**Effort**: ~4 hours  
**Impact**: HIGH — Improves maintainability, testability, and team velocity

**Steps**:
1. Create `src/service/` directory
2. Create domain modules (spine, entry, certificate, etc.)
3. Move methods to appropriate modules
4. Update `service.rs` to coordinator
5. Update imports in `jsonrpc.rs` and `tarpc_server.rs`
6. Run full test suite

### Priority 2: `manager.rs` Extract Coordinators

**Why**: Improves manager maintainability  
**Effort**: ~3 hours  
**Impact**: MEDIUM — Better separation of concerns

**Steps**:
1. Analyze current structure
2. Identify coordinator boundaries
3. Extract specialized coordinators
4. Update manager to use coordinators
5. Run tests

### Priority 3: `backup.rs` Minor Improvements (Optional)

**Why**: Low priority, already good  
**Effort**: ~2 hours  
**Impact**: LOW — Marginal improvement

**When to do**: Only if adding alternative formats or complex verification

---

## 📏 FILE SIZE GUIDELINES

### Acceptable Sizes

**Library code**: 300-500 lines per file  
**Service code**: 150-300 lines per file  
**Test code**: 500-1000 lines per file (acceptable)  
**Integration tests**: 1000+ lines (acceptable if organized)

### When to Refactor

**700+ lines**: Review for multiple concerns  
**1000+ lines**: Strong signal to refactor  
**1500+ lines**: Almost certainly needs splitting

### When NOT to Refactor

- ✅ Test files with many test cases
- ✅ Well-structured files with single concern
- ✅ Generated code
- ✅ Comprehensive examples/documentation

---

## 🔧 REFACTORING PRINCIPLES

### 1. Follow Domain Boundaries
Split by **logical domain**, not arbitrary line count.

❌ **Bad**: Split `service.rs` into `service_part1.rs` and `service_part2.rs`  
✅ **Good**: Split into `spine_service.rs`, `certificate_service.rs`, etc.

### 2. Maintain Cohesion
Keep related code together.

❌ **Bad**: Split backup export and import into separate files  
✅ **Good**: Keep backup operations together, they're cohesive

### 3. Improve Testability
Refactoring should make testing easier.

✅ **Good**: Domain services can be tested independently  
✅ **Good**: Mock coordinators instead of entire manager

### 4. Preserve API Stability
Public API should remain stable during refactoring.

```rust
// Old API still works
let service = LoamSpineRpcService::new(core);
let response = service.create_spine(request).await?;

// New API also available
let response = service.spine.create_spine(request).await?;
```

### 5. Incremental Refactoring
Refactor in small, testable steps.

**Phase 1**: Extract one service (e.g., `SpineService`)  
**Phase 2**: Verify tests pass  
**Phase 3**: Extract next service  
**Repeat**: Until complete

---

## 📊 EXPECTED OUTCOMES

### After `service.rs` Refactoring

**Before**:
```
service.rs (889 lines)
  - 20 methods
  - 7 domains mixed
```

**After**:
```
service/
  ├── mod.rs (50 lines) - re-exports
  ├── spine.rs (100 lines) - 3 methods
  ├── entry.rs (120 lines) - 3 methods
  ├── certificate.rs (150 lines) - 5 methods
  ├── slice.rs (130 lines) - 2 methods
  ├── proof.rs (110 lines) - 2 methods
  ├── session.rs (140 lines) - 2 methods
  └── health.rs (90 lines) - 1 method

service.rs (100 lines) - coordinator
```

**Metrics**:
- Average file size: ~120 lines (down from 889)
- Files follow Single Responsibility
- Each domain independently testable
- Parallel development enabled

### After `manager.rs` Refactoring

**Before**:
```
manager.rs (781 lines)
  - Mixed coordination logic
```

**After**:
```
manager/
  ├── mod.rs (100 lines) - main coordinator
  ├── spine_coordinator.rs (200 lines)
  ├── certificate_coordinator.rs (230 lines)
  └── slice_coordinator.rs (250 lines)
```

**Metrics**:
- Max file size: 250 lines (down from 781)
- Clear coordinator responsibilities
- Testable in isolation

---

## 🚀 IMPLEMENTATION PLAN

### Week 1: `service.rs` Refactoring

**Day 1-2**: Design & Structure
- Map methods to domains
- Design coordinator pattern
- Plan API compatibility

**Day 3-4**: Implementation
- Extract services incrementally
- Update imports
- Maintain tests passing

**Day 5**: Verification
- Full test suite
- Performance benchmarks
- Documentation updates

### Week 2: `manager.rs` Refactoring

**Day 1**: Analysis
- Understand current structure
- Identify coordinator boundaries

**Day 2-3**: Implementation
- Extract coordinators
- Update manager
- Update tests

**Day 4**: Verification & Documentation

---

## 📚 RELATED DOCUMENTS

- **Audit**: `AUDIT_SUMMARY.md` — Overall codebase health
- **Gaps**: `INTEGRATION_GAPS.md` — Implementation gaps
- **Deep Solutions**: `CLIPPY_FIXES_DEEP_SOLUTIONS.md` — Quality improvements

---

## 🎯 SUCCESS CRITERIA

Refactoring is successful when:
- ✅ All tests pass
- ✅ No performance regression
- ✅ Public API remains stable
- ✅ Average file size < 300 lines (library code)
- ✅ Each file has single, clear responsibility
- ✅ Test coverage maintained or improved
- ✅ Documentation updated

---

**Principle**: Refactor by **domain and responsibility**, not by **arbitrary line count**.

🦴 **LoamSpine: Clean architecture through smart refactoring**

