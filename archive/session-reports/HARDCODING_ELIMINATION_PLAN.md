# 🔥 Hardcoding Elimination Plan — LoamSpine

**Date**: December 26, 2025  
**Target**: 100% Zero Hardcoding (BearDog Standard)  
**Philosophy**: Infant Discovery — Start with Zero Knowledge

---

## 🎯 EXECUTIVE SUMMARY

**Current Status**: ~70% Hardcoding-Free (Good but not excellent)

| Category | Current | Target | Status |
|----------|---------|--------|--------|
| **Primal Names** | Partial | 100% | ⚠️ "Songbird" hardcoded |
| **Service Names** | Excellent | 100% | ✅ Capability-based |
| **Port Numbers** | Good | 100% | ⚠️ Some defaults hardcoded |
| **Vendor Names** | Partial | 100% | ⚠️ K8s/Consul in docs only |
| **Binary Paths** | Good | 100% | ⚠️ Test-only hardcoding |

**Gap to Close**: ~30% — Primarily vendor name "Songbird" throughout codebase

**Learning from Phase1**: BearDog achieved 100% zero hardcoding with `beardog-discovery` crate

---

## 🔍 HARDCODING AUDIT RESULTS

### 1. ❌ CRITICAL: "Songbird" Vendor Hardcoding (162 instances!)

**Impact**: HIGH — Violates infant discovery principle

**Locations**:
- `crates/loam-spine-core/src/songbird.rs` — **25 instances** (module name!)
- `crates/loam-spine-core/src/service/infant_discovery.rs` — 6 instances
- `crates/loam-spine-core/tests/songbird_integration.rs` — **72 instances** (test file!)
- `crates/loam-spine-api/src/health.rs` — 2 instances
- `crates/loam-spine-core/src/service/lifecycle.rs` — 25 instances
- `crates/loam-spine-core/src/config.rs` — 9 instances
- `crates/loam-spine-core/src/discovery.rs` — 21 instances
- `crates/loam-spine-core/src/error.rs` — 1 instance
- `crates/loam-spine-core/src/lib.rs` — 1 instance

**Examples**:
```rust
// ❌ HARDCODED - Vendor name "Songbird"
pub struct SongbirdClient { ... }
pub songbird_endpoint: Option<String>
use crate::songbird::SongbirdClient;
```

**Evolution Required**:
```rust
// ✅ GENERIC - "Discovery" not vendor-specific
pub struct DiscoveryClient { ... }
pub discovery_endpoint: Option<String>
use crate::discovery::DiscoveryClient;
```

---

### 2. ⚠️ MODERATE: Primal Name References (55 instances)

**Impact**: MEDIUM — Test-only, but violates pure abstraction

**Locations**:
- `crates/loam-spine-core/src/traits/cli_signer.rs` — Multiple "beardog" references
- `crates/loam-spine-core/tests/cli_signer_integration.rs` — **Extensive** beardog hardcoding

**Examples**:
```rust
// ❌ Test-only but still hardcoded
const BEARDOG_BIN: &str = "../bins/beardog";
let bins_dir = PathBuf::from("../bins/beardog");

// Comments and error messages
eprintln!("⚠️  Skipping test: BearDog binary not found");
```

**Analysis**:
- ✅ **API is generic** (CliSigner, not BearDogSigner)
- ✅ **Type names are generic** (no vendor lock-in)
- ⚠️ **Tests hardcode paths** (acceptable for integration tests)
- ⚠️ **Environment variable hints** mention specific primals

**Evolution Strategy**:
- Tests can remain hardcoded (they test specific integrations)
- Add generic discovery: `LOAMSPINE_SIGNER_PATH` → discover via capability
- Document: "Any Ed25519 CLI binary works, not just BearDog"

---

### 3. ⚠️ MODERATE: Port Number Hardcoding (52 instances)

**Impact**: LOW — All have environment variable overrides

**Locations**:
- `crates/loam-spine-core/src/songbird.rs` — 30 instances
- `crates/loam-spine-core/src/service/infant_discovery.rs` — 8 instances
- `crates/loam-spine-core/tests/songbird_integration.rs` — 8 instances
- `crates/loam-spine-core/src/config.rs` — 6 instances

**Examples**:
```rust
// ⚠️ Hardcoded defaults (but overridable)
.unwrap_or(9001);  // tarpc default
.unwrap_or(8080);  // jsonrpc default
"http://localhost:8082"  // Development fallback

// Test constants
const SONGBIRD_ENDPOINT: &str = "http://localhost:8082";
```

**Analysis**:
- ✅ All are **fallback defaults** with warnings
- ✅ All are **overridable** via environment variables
- ✅ Production deployments use env vars
- ⚠️ Magic numbers could be named constants

**Evolution Strategy**:
```rust
// Better: Named constants with documentation
pub const DEFAULT_TARPC_PORT: u16 = 9001;
pub const DEFAULT_JSONRPC_PORT: u16 = 8080;
pub const DEFAULT_DISCOVERY_PORT: u16 = 8082;  // Or: 0 for OS assignment

// Best: Randomize or let OS choose
.unwrap_or(0);  // Let OS assign available port
```

---

### 4. ✅ EXCELLENT: External Vendor References (1 instance only!)

**Impact**: NONE — Documentation only

**Location**:
- `crates/loam-spine-api/src/health.rs`

**Example**:
```rust
// ✅ Documentation mentions platforms, not hardcoded
//! (Kubernetes, Nomad, Docker Swarm) and service meshes (Consul, etcd, etc.):
```

**Analysis**: Perfect! Just documentation examples, no code coupling.

---

### 5. ✅ EXCELLENT: Service Hardcoding

**Impact**: NONE — Fully capability-based

**Evidence**:
```rust
// ✅ No service names hardcoded
client.discover_capability("signing").await?;
client.discover_capability("storage").await?;
client.discover_capability("orchestration").await?;

// ✅ Generic discovery
let services = discovery.find_by_capability("compute").await?;
```

**Result**: Perfect abstraction! We don't care WHO provides the capability.

---

## 📊 COMPARISON WITH BEARDOG

### BearDog's Approach (100% Zero Hardcoding)

**Structure**:
```
bearDog/
├── crates/
│   ├── beardog-discovery/  ✅ Generic discovery crate
│   │   ├── src/
│   │   │   ├── discovery.rs       # CapabilityDiscovery (generic!)
│   │   │   ├── announcement.rs    # Announcer (self-knowledge)
│   │   │   ├── mdns.rs            # mDNS discovery
│   │   │   ├── service_registry.rs # Generic registry
│   │   │   └── types.rs           # DiscoveredService (no vendor names)
```

**Key Patterns**:
1. ✅ **No vendor names** — "CapabilityDiscovery" not "SongbirdDiscovery"
2. ✅ **Self-knowledge only** — BearDog knows what IT provides
3. ✅ **Capability queries** — `find_by_capability("orchestration")`
4. ✅ **Environment discovery** — `CAPABILITY_ORCHESTRATION_ENDPOINT`
5. ✅ **mDNS/DNS-SD** — Zero-config local discovery
6. ✅ **Service registry** — Generic, works with any registry

### LoamSpine's Current Approach (70% Zero Hardcoding)

**Structure**:
```
loamSpine/
├── crates/
│   ├── loam-spine-core/
│   │   ├── src/
│   │   │   ├── songbird.rs       ❌ Vendor name in module!
│   │   │   ├── discovery.rs      ✅ Generic (but uses SongbirdClient)
│   │   │   ├── infant_discovery.rs ✅ Excellent pattern
│   │   │   └── config.rs         ⚠️ "songbird_endpoint" field
```

**Issues**:
1. ❌ **Module named after vendor** — `songbird.rs` → should be `discovery_client.rs`
2. ❌ **Type named after vendor** — `SongbirdClient` → should be `DiscoveryClient`
3. ❌ **Config fields named after vendor** — `songbird_enabled` → `discovery_enabled`
4. ⚠️ **Test files named after vendor** — `songbird_integration.rs`

---

## 🚀 EVOLUTION PLAN

### Phase 1: Rename Vendor-Specific Code (2-3 hours)

**Goal**: Eliminate "Songbird" from all production code

#### Step 1.1: Rename Module
```bash
# Rename file
mv crates/loam-spine-core/src/songbird.rs \
   crates/loam-spine-core/src/discovery_client.rs
```

#### Step 1.2: Rename Types
```rust
// Before (162 instances to change!)
pub struct SongbirdClient { ... }

// After
pub struct DiscoveryClient { ... }
```

#### Step 1.3: Update Config Fields
```rust
// Before
pub struct DiscoveryConfig {
    pub songbird_enabled: bool,
    pub songbird_endpoint: Option<String>,
}

// After (already partially done!)
pub struct DiscoveryConfig {
    pub discovery_enabled: bool,
    pub discovery_endpoint: Option<String>,
    
    // Deprecated for backward compatibility
    #[deprecated(since = "0.7.0", note = "Use discovery_enabled")]
    pub songbird_enabled: bool,  // Remove in v1.0
}
```

#### Step 1.4: Update Imports
```rust
// Before
use crate::songbird::SongbirdClient;

// After
use crate::discovery_client::DiscoveryClient;
```

#### Step 1.5: Update Documentation
```rust
// Before
//! Songbird integration for universal service discovery.

// After
//! Discovery client for universal service adapter.
//!
//! This module provides integration with any discovery service (such as
//! Songbird, Consul, etcd, or custom implementations) for discovering
//! other primals' capabilities at runtime without hardcoding.
```

---

### Phase 2: Port Number Abstraction (1 hour)

**Goal**: Named constants, OS assignment option

#### Step 2.1: Define Constants
```rust
// crates/loam-spine-core/src/ports.rs (new file)

/// Default tarpc port for primal-to-primal communication.
///
/// Set to 0 to let the OS assign an available port automatically.
pub const DEFAULT_TARPC_PORT: u16 = 9001;

/// Default JSON-RPC port for external clients.
///
/// Set to 0 to let the OS assign an available port automatically.
pub const DEFAULT_JSONRPC_PORT: u16 = 8080;

/// Default discovery service port.
///
/// Only used as fallback when no discovery service is configured.
/// Production deployments should use environment variables.
pub const DEFAULT_DISCOVERY_PORT: u16 = 8082;

/// Development fallback: Let OS choose available ports.
pub const OS_ASSIGNED_PORT: u16 = 0;
```

#### Step 2.2: Use Constants
```rust
// Before
.unwrap_or(9001);

// After
.unwrap_or(DEFAULT_TARPC_PORT);
```

#### Step 2.3: Document OS Assignment
```rust
// config.toml example
[discovery]
tarpc_port = 0        # Let OS choose
jsonrpc_port = 0      # Let OS choose
discovery_port = 0    # Auto-discover
```

---

### Phase 3: Test Hardcoding Mitigation (30 min)

**Goal**: Document test patterns, reduce brittleness

#### Step 3.1: Environment-Based Test Config
```rust
// Before
const BEARDOG_BIN: &str = "../bins/beardog";

// After
fn get_signer_binary() -> Option<PathBuf> {
    // 1. Check environment variable (highest priority)
    if let Ok(path) = env::var("LOAMSPINE_TEST_SIGNER") {
        return Some(PathBuf::from(path));
    }
    
    // 2. Check standard locations
    for candidate in &["../bins/beardog", "../../phase1/bearDog/target/release/beardog"] {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    
    // 3. Check PATH (any Ed25519 CLI will work!)
    if let Ok(path) = which::which("beardog") {
        return Some(path);
    }
    
    None
}
```

#### Step 3.2: Document Test Patterns
```rust
//! # Integration Test Patterns
//!
//! These tests require an Ed25519 CLI signing binary. By default, we look for
//! BearDog, but **any Ed25519 CLI that follows the protocol will work**.
//!
//! ## Setup Options
//!
//! 1. **Use BearDog**: Place binary at `../bins/beardog`
//! 2. **Use Custom Signer**: Set `LOAMSPINE_TEST_SIGNER=/path/to/signer`
//! 3. **Skip Tests**: Tests auto-skip if no signer found
```

---

### Phase 4: Create `loam-spine-discovery` Crate (4-6 hours)

**Goal**: Match BearDog's architecture with separate discovery crate

**Structure**:
```
loamSpine/
├── crates/
│   ├── loam-spine-discovery/  ✅ NEW: Generic discovery crate
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs            # Public API
│   │   │   ├── client.rs         # DiscoveryClient (renamed from SongbirdClient)
│   │   │   ├── announcement.rs   # Self-advertisement
│   │   │   ├── infant.rs         # Infant discovery (env, DNS, mDNS)
│   │   │   ├── registry.rs       # Generic service registry
│   │   │   ├── mdns.rs           # mDNS discovery
│   │   │   ├── dns_sd.rs         # DNS-SD discovery
│   │   │   └── types.rs          # Generic types (no vendor names)
```

**Benefits**:
1. ✅ Clear separation of concerns
2. ✅ Can be used by other primals
3. ✅ Easier to test in isolation
4. ✅ Matches BearDog's excellent architecture

---

### Phase 5: Capability-Based Binary Discovery (2-3 hours)

**Goal**: Discover signers by capability, not by primal name

#### Step 5.1: Capability Discovery for Binaries
```rust
// Instead of hardcoding BearDog path
pub async fn discover_signer() -> Option<PathBuf> {
    // 1. Environment variable (explicit override)
    if let Ok(path) = env::var("LOAMSPINE_SIGNER_PATH") {
        return Some(PathBuf::from(path));
    }
    
    // 2. Query discovery service for "signing" capability
    if let Ok(client) = DiscoveryClient::connect_auto().await {
        if let Ok(services) = client.discover_capability("signing").await {
            if let Some(service) = services.first() {
                // Service advertises its CLI path in metadata
                if let Some(cli_path) = service.metadata.get("cli_path") {
                    return Some(PathBuf::from(cli_path));
                }
            }
        }
    }
    
    // 3. Check PATH for any Ed25519 binary
    for candidate in &["beardog", "ed25519-cli", "signer"] {
        if let Ok(path) = which::which(candidate) {
            return Some(path);
        }
    }
    
    // 4. Check local bins
    for dir in &["../bins", "../../phase1/bearDog/target/release"] {
        let path = PathBuf::from(dir);
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                // Any executable that responds to `--help` with "ed25519" or "sign"
                // (Smart discovery!)
            }
        }
    }
    
    None
}
```

---

## 📋 FILE-BY-FILE CHANGES

### Critical Files (Must Change)

1. **`songbird.rs` → `discovery_client.rs`**
   - Rename file
   - `SongbirdClient` → `DiscoveryClient`
   - Update all documentation
   - **Impact**: 162 instances to update

2. **`config.rs`**
   - Already has deprecation warnings ✅
   - Remove deprecated fields in v1.0
   - **Impact**: 9 instances

3. **`infant_discovery.rs`**
   - Already mostly generic ✅
   - Update "Songbird" → "discovery service"
   - **Impact**: 6 instances

4. **`lifecycle.rs`**
   - Update client instantiation
   - Generic error messages
   - **Impact**: 25 instances

5. **`discovery.rs`**
   - Update imports
   - **Impact**: 21 instances

### Test Files (Can Remain More Specific)

1. **`songbird_integration.rs`**
   - Can keep specific tests
   - Update to test "any discovery service"
   - Add comment: "Uses Songbird as reference implementation"
   - **Impact**: 72 instances (acceptable)

2. **`cli_signer_integration.rs`**
   - Document: "Tests with BearDog as reference Ed25519 provider"
   - Add capability-based discovery tests
   - **Impact**: 55 instances (test-only, acceptable)

---

## 🎯 SUCCESS CRITERIA

### Phase 1: Zero Production Hardcoding ✅
- [ ] Zero vendor names in `src/` (except deprecated fields)
- [ ] All types generic (DiscoveryClient, not SongbirdClient)
- [ ] All modules generic (discovery_client.rs, not songbird.rs)
- [ ] Documentation vendor-agnostic

### Phase 2: Named Constants ✅
- [ ] All port numbers as named constants
- [ ] OS assignment option (port = 0)
- [ ] Clear documentation

### Phase 3: Test Patterns ✅
- [ ] Environment-based test configuration
- [ ] Multiple binary discovery paths
- [ ] Documentation explains patterns

### Phase 4: Separate Crate ✅
- [ ] `loam-spine-discovery` crate created
- [ ] Matches BearDog architecture
- [ ] Can be used by other primals

### Phase 5: Capability Discovery ✅
- [ ] Binaries discovered by capability
- [ ] No primal names in discovery code
- [ ] Smart fallback chain

---

## 📊 EFFORT ESTIMATE

| Phase | Effort | Priority | Blockers |
|-------|--------|----------|----------|
| **Phase 1: Rename** | 2-3 hours | CRITICAL | None |
| **Phase 2: Ports** | 1 hour | HIGH | None |
| **Phase 3: Tests** | 30 min | MEDIUM | None |
| **Phase 4: Crate** | 4-6 hours | HIGH | Phase 1 |
| **Phase 5: Capability** | 2-3 hours | MEDIUM | Phase 4 |

**Total**: 10-14 hours to 100% zero hardcoding

---

## 🏆 EXPECTED OUTCOME

### Before (Current)
```rust
// ❌ Vendor lock-in
use crate::songbird::SongbirdClient;
let client = SongbirdClient::connect("http://localhost:8082").await?;
let services = client.discover_capability("signing").await?;
```

### After (Evolution Complete)
```rust
// ✅ Vendor-agnostic
use loam_spine_discovery::DiscoveryClient;
let client = DiscoveryClient::connect_auto().await?;  // Auto-discovers!
let services = client.discover_capability("signing").await?;
// Works with Songbird, Consul, etcd, or custom discovery service!
```

### Documentation
```rust
//! ## Supported Discovery Services
//!
//! LoamSpine works with any discovery service that implements the
//! capability-based discovery protocol:
//!
//! - **Songbird** (reference implementation)
//! - **Consul** (via adapter)
//! - **etcd** (via adapter)
//! - **Kubernetes** (DNS-based service discovery)
//! - **Custom implementations**
//!
//! The discovery service is auto-detected via:
//! 1. Environment variables (DISCOVERY_ENDPOINT)
//! 2. DNS SRV records (_discovery._tcp.local)
//! 3. mDNS (local network)
//! 4. Development fallback (localhost with warning)
```

---

## 🎖️ COMPARISON AFTER EVOLUTION

| Metric | BearDog (Current) | LoamSpine (Current) | LoamSpine (After) |
|--------|-------------------|---------------------|-------------------|
| **Vendor Names in Code** | 0 | 162 | **0** ✅ |
| **Named Constants** | Yes | Partial | **Yes** ✅ |
| **Separate Crate** | Yes | No | **Yes** ✅ |
| **Capability Discovery** | Yes | Partial | **Yes** ✅ |
| **Test Hardcoding** | Minimal | Moderate | **Minimal** ✅ |
| **Overall Score** | 100% | 70% | **100%** ✅ |

---

## 📝 MIGRATION GUIDE FOR USERS

### Backward Compatibility

**v0.7.0** (Current):
```rust
// Old names still work (deprecated warnings)
config.songbird_enabled = true;
config.songbird_endpoint = Some("...".into());
let client = SongbirdClient::connect(...).await?;
```

**v0.8.0** (Evolution):
```rust
// New names recommended
config.discovery_enabled = true;
config.discovery_endpoint = Some("...".into());
let client = DiscoveryClient::connect(...).await?;

// Old names still work but will be removed in v1.0
```

**v1.0.0** (Complete):
```rust
// Only new names supported
config.discovery_enabled = true;
config.discovery_endpoint = Some("...".into());
let client = DiscoveryClient::connect_auto().await?;
```

---

## 🚀 IMMEDIATE NEXT STEPS

### Today (30 minutes)
1. ✅ Create this plan document
2. Create GitHub issues for each phase
3. Update ROADMAP.md with hardcoding elimination

### This Week (Phase 1 + 2)
1. Rename `songbird.rs` → `discovery_client.rs`
2. Rename `SongbirdClient` → `DiscoveryClient`
3. Update all 162 references
4. Add named port constants
5. Run all tests to verify

### Next Week (Phase 3 + 4)
1. Improve test patterns
2. Create `loam-spine-discovery` crate
3. Port code from core to discovery crate
4. Update dependencies

### Week 3 (Phase 5)
1. Implement capability-based binary discovery
2. Update documentation
3. Create migration guide
4. Release v0.8.0 with evolution complete

---

## 🎯 ALIGNMENT WITH PRIMAL PRINCIPLES

### Infant Discovery ✅
> "Primals start with zero knowledge and discover like an infant"

- ✅ After evolution: Zero hardcoded knowledge
- ✅ Environment-based discovery
- ✅ DNS SRV / mDNS fallbacks
- ✅ Graceful degradation

### Universal Adapter ✅
> "Each primal connects to universal adapter, not to each other (O(n) not O(n²))"

- ✅ Already implemented
- ✅ Capability-based queries
- ✅ Works with any discovery service after evolution

### Sovereignty ✅
> "No vendor lock-in, no hardcoded dependencies"

- ⚠️ Currently: Songbird-specific naming
- ✅ After evolution: Completely vendor-agnostic
- ✅ Works with Songbird, Consul, etcd, custom, or none

---

**🦴 LoamSpine: Evolving to 100% Zero Hardcoding**  
**Target**: v0.8.0 (2-3 weeks)  
**Effort**: 10-14 hours focused work  
**Outcome**: Match BearDog's world-class zero hardcoding standard

