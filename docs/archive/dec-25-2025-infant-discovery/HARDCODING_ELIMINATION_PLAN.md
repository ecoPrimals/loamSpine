# 🦴 LoamSpine — Hardcoding Elimination Plan

**Date**: December 25, 2025  
**Version**: 0.7.0 Target  
**Philosophy**: Infant Discovery — Start with zero knowledge, discover everything

---

## 🎯 VISION: INFANT DISCOVERY

**LoamSpine should start knowing ONLY itself and discover everything else at runtime.**

Like an infant learning about the world:
1. **Self-knowledge**: "I am LoamSpine, I provide persistent-ledger capability"
2. **Universal adapter**: "I use the universal adapter (discovery service) to find others"
3. **Capability-based**: "I need a 'signer', not a 'BearDog'"
4. **No assumptions**: "I don't know what network mesh exists (k8s, consul, etc)"
5. **Graceful degradation**: "If I can't find something, I continue with reduced capabilities"

---

## 📊 CURRENT HARDCODING AUDIT

### 🔴 CRITICAL: Primal Name Hardcoding (235 instances)

#### Production Code (Must Fix)
```
crates/loam-spine-core/src/songbird.rs:      23 instances
crates/loam-spine-core/src/service/lifecycle.rs: 46 instances
crates/loam-spine-core/src/config.rs:         14 instances
crates/loam-spine-core/src/discovery.rs:      21 instances
crates/loam-spine-core/src/traits/cli_signer.rs: 11 instances
crates/loam-spine-api/src/health.rs:          11 instances
crates/loam-spine-core/src/error.rs:           1 instance
```

**Primals Hardcoded**:
- ❌ `Songbird` (149 instances) — Should be "discovery-service"
- ❌ `BearDog` (68 instances) — Should be "signer"
- ❌ `NestGate` (6 instances) — Should be "storage"
- ❌ `ToadStool` (5 instances) — Should be "compute" or "encryption"
- ❌ `Squirrel` (7 instances) — Should be "ai-service"

#### Test Code (Acceptable but Should Improve)
```
crates/loam-spine-core/tests/songbird_integration.rs: 64 instances
crates/loam-spine-core/tests/cli_signer_integration.rs: 42 instances
```

### 🟡 MEDIUM: Vendor/Infrastructure Hardcoding (5 instances)

```
crates/loam-spine-api/src/health.rs:1 - "Kubernetes" in comments
crates/loam-spine-api/src/service.rs:2 - "Kubernetes" in comments
crates/loam-spine-api/src/jsonrpc.rs:2 - "Kubernetes" in comments
```

**Vendors Mentioned**:
- ❌ `Kubernetes` (3 instances) — Should be "container orchestrator" or "service mesh"
- ❌ `k8s` (0 instances) — Good!
- ✅ `consul`, `etcd`, `zookeeper` (0 instances) — Good!

### 🟡 MEDIUM: Storage Backend Hardcoding (118 instances)

```
crates/loam-spine-core/src/storage/sled.rs:36 instances
crates/loam-spine-core/src/storage/mod.rs:8 instances
crates/loam-spine-core/src/storage/tests.rs:16 instances
```

**Status**: ✅ **ACCEPTABLE** — These are in concrete implementations
- `Sled` is abstracted behind `Storage` trait
- Production code uses trait, not concrete type
- This is proper abstraction, not hardcoding

### 🔴 CRITICAL: Port/Endpoint Hardcoding (41 instances)

#### Production Defaults
```rust
// crates/loam-spine-core/src/config.rs:102-106
songbird_endpoint: Some("http://localhost:8082".to_string()),
tarpc_endpoint: "http://localhost:9001".to_string(),
jsonrpc_endpoint: "http://localhost:8080".to_string(),
```

#### Test Code (35+ instances)
```
"http://localhost:8082" - Songbird
"http://localhost:9001" - tarpc
"http://localhost:8080" - JSON-RPC
"http://localhost:9999" - test endpoints
```

---

## 🚀 ELIMINATION STRATEGY

### Phase 1: Rename Types (Non-Breaking) ✅ Can do now
Replace primal-specific names with capability-based names in new code:

```rust
// OLD (hardcoded primal)
pub songbird_client: Option<SongbirdClient>

// NEW (capability-based)
pub discovery_client: Option<DiscoveryClient>
```

### Phase 2: Configuration Abstraction ✅ Can do now
Replace hardcoded defaults with environment-driven discovery:

```rust
// OLD
songbird_endpoint: Some("http://localhost:8082".to_string())

// NEW
discovery_endpoint: std::env::var("DISCOVERY_ENDPOINT").ok()
    .or_else(|| self.discover_from_dns("_discovery._tcp.local"))
    .or_else(|| self.discover_from_mdns())
```

### Phase 3: Universal Adapter Pattern ✅ Can do now
Create generic adapter that wraps primal-specific clients:

```rust
pub trait DiscoveryAdapter: Send + Sync {
    async fn register(&self, capabilities: Vec<Capability>) -> Result<()>;
    async fn discover(&self, capability: &str) -> Result<Vec<Service>>;
    async fn heartbeat(&self) -> Result<()>;
}

// Concrete adapters
pub struct SongbirdAdapter { /* ... */ }
pub struct ConsulAdapter { /* ... */ }
pub struct EtcdAdapter { /* ... */ }
```

### Phase 4: Breaking Changes (v1.0.0)
Rename public APIs to be primal-agnostic:

```rust
// OLD (breaking change)
impl SongbirdClient { /* ... */ }

// NEW
impl DiscoveryClient { /* ... */ }
```

---

## 📋 DETAILED ACTION ITEMS

### 1. Replace "Songbird" with "Discovery Service" (3 hours)

**Files to modify**:
- `crates/loam-spine-core/src/config.rs`
- `crates/loam-spine-core/src/service/lifecycle.rs`
- `crates/loam-spine-api/src/health.rs`

**Changes**:
```rust
// Config field names (non-breaking, add deprecation)
pub struct DiscoveryConfig {
    pub discovery_enabled: bool,
    pub discovery_endpoint: Option<String>,
    
    #[deprecated(note = "Use discovery_enabled")]
    pub songbird_enabled: bool,
    
    #[deprecated(note = "Use discovery_endpoint")]
    pub songbird_endpoint: Option<String>,
}

// Struct field names
pub struct LifecycleManager {
    discovery_client: Option<DiscoveryClient>,
    #[deprecated] songbird_client: Option<SongbirdClient>,
}

// Health check fields
pub struct DependencyHealth {
    pub storage: bool,
    pub discovery: Option<bool>,  // NEW
    #[deprecated] pub songbird: Option<bool>,  // OLD
}
```

### 2. Replace "BearDog" with "Signer" (2 hours)

**Files to modify**:
- `crates/loam-spine-core/src/traits/cli_signer.rs`
- `crates/loam-spine-core/src/error.rs`

**Changes**:
```rust
// Error messages
// OLD: "Failed to execute BearDog CLI"
// NEW: "Failed to execute signer CLI"

// Comments and documentation
// OLD: "Call BearDog CLI binary for signing"
// NEW: "Call signer CLI binary (discovered via universal adapter)"

// Binary path discovery
// OLD: "../bins/beardog-signer" (hardcoded)
// NEW: discover_signer_binary() -> Result<PathBuf>
```

### 3. Abstract Infrastructure Names (1 hour)

**Files to modify**:
- `crates/loam-spine-api/src/health.rs`
- `crates/loam-spine-api/src/service.rs`
- `crates/loam-spine-api/src/jsonrpc.rs`

**Changes**:
```rust
// OLD: "Kubernetes liveness probe"
// NEW: "Container orchestrator liveness probe"

// OLD: "Kubernetes-style health checks"
// NEW: "Standard health check endpoints (compatible with k8s, consul, etc)"

// Function names stay the same (liveness/readiness are standard)
// Just update comments to be vendor-agnostic
```

### 4. Environment-Based Discovery (4 hours)

**New file**: `crates/loam-spine-core/src/service/infant_discovery.rs`

```rust
/// Infant discovery - start with zero knowledge, discover everything.
pub struct InfantDiscovery {
    /// Our own capabilities (the only thing we know at start)
    self_capabilities: Vec<Capability>,
}

impl InfantDiscovery {
    /// Create new infant discovery with self-knowledge only.
    pub fn new(self_capabilities: Vec<Capability>) -> Self {
        Self { self_capabilities }
    }
    
    /// Discover discovery service (the universal adapter).
    /// 
    /// Tries in order:
    /// 1. Environment variable (DISCOVERY_ENDPOINT)
    /// 2. DNS SRV records (_discovery._tcp.local)
    /// 3. mDNS (local network discovery)
    /// 4. Well-known local paths (development only)
    pub async fn discover_discovery_service(&self) -> Result<DiscoveryClient> {
        // Try environment first
        if let Ok(endpoint) = std::env::var("DISCOVERY_ENDPOINT") {
            return DiscoveryClient::connect(&endpoint).await;
        }
        
        // Try DNS SRV
        if let Some(endpoint) = self.discover_from_dns("_discovery._tcp").await {
            return DiscoveryClient::connect(&endpoint).await;
        }
        
        // Try mDNS (local network)
        if let Some(endpoint) = self.discover_from_mdns().await {
            return DiscoveryClient::connect(&endpoint).await;
        }
        
        // Development fallback (logged as warning)
        self.try_development_fallback().await
    }
    
    /// Discover capability by asking discovery service.
    pub async fn discover_capability(
        &self,
        discovery: &DiscoveryClient,
        capability: &str,
    ) -> Result<Vec<Service>> {
        discovery.discover(capability).await
    }
}
```

### 5. Remove Hardcoded Ports (2 hours)

**Changes to `config.rs`**:
```rust
impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            discovery_enabled: true,
            // No default endpoint - must be discovered or configured
            discovery_endpoint: None,
            
            // Our own endpoints - from environment or defaults
            tarpc_endpoint: std::env::var("TARPC_ENDPOINT")
                .unwrap_or_else(|_| "http://0.0.0.0:0".to_string()), // OS-assigned port
            jsonrpc_endpoint: std::env::var("JSONRPC_ENDPOINT")
                .unwrap_or_else(|_| "http://0.0.0.0:0".to_string()), // OS-assigned port
            
            auto_advertise: true,
            heartbeat_interval_seconds: 30,
        }
    }
}
```

**Test changes**:
```rust
// Tests can still hardcode for deterministic behavior
// But should document this is test-only
#[cfg(test)]
const TEST_DISCOVERY_ENDPOINT: &str = "http://localhost:8082";
```

### 6. Create Universal Adapter Trait (3 hours)

**New file**: `crates/loam-spine-core/src/adapters/mod.rs`

```rust
/// Universal adapter for service discovery.
/// 
/// This trait abstracts over different discovery mechanisms:
/// - Songbird (ecoPrimals native)
/// - Consul
/// - Etcd
/// - Kubernetes service discovery
/// - mDNS
#[async_trait::async_trait]
pub trait DiscoveryAdapter: Send + Sync {
    /// Register self with discovery service.
    async fn register(&self, info: ServiceInfo) -> Result<()>;
    
    /// Discover services by capability.
    async fn discover(&self, capability: &str) -> Result<Vec<Service>>;
    
    /// Send heartbeat to maintain registration.
    async fn heartbeat(&self) -> Result<()>;
    
    /// Deregister from discovery service.
    async fn deregister(&self) -> Result<()>;
    
    /// Get adapter name (for logging/debugging).
    fn adapter_name(&self) -> &'static str;
}

/// Songbird-specific adapter implementation.
pub struct SongbirdAdapter {
    client: SongbirdClient,
}

#[async_trait::async_trait]
impl DiscoveryAdapter for SongbirdAdapter {
    async fn register(&self, info: ServiceInfo) -> Result<()> {
        self.client.advertise_loamspine(&info.tarpc_endpoint, &info.jsonrpc_endpoint).await
    }
    
    async fn discover(&self, capability: &str) -> Result<Vec<Service>> {
        self.client.discover_capability(capability).await
    }
    
    async fn heartbeat(&self) -> Result<()> {
        self.client.heartbeat().await
    }
    
    async fn deregister(&self) -> Result<()> {
        self.client.deregister().await
    }
    
    fn adapter_name(&self) -> &'static str {
        "songbird"
    }
}

/// Factory for creating adapters based on discovery.
pub struct AdapterFactory;

impl AdapterFactory {
    /// Create adapter by trying to connect to discovery services.
    pub async fn create() -> Result<Box<dyn DiscoveryAdapter>> {
        // Try Songbird first (ecoPrimals native)
        if let Ok(adapter) = Self::try_songbird().await {
            return Ok(Box::new(adapter));
        }
        
        // Try Consul
        if let Ok(adapter) = Self::try_consul().await {
            return Ok(Box::new(adapter));
        }
        
        // Try mDNS
        if let Ok(adapter) = Self::try_mdns().await {
            return Ok(Box::new(adapter));
        }
        
        Err(LoamSpineError::Internal("No discovery service found".to_string()))
    }
}
```

---

## 📊 IMPACT ASSESSMENT

### Breaking Changes
- ❌ None in Phase 1-3 (all backward compatible with deprecation warnings)
- ✅ Breaking changes deferred to v1.0.0

### Performance Impact
- ✅ Negligible (discovery is O(1) once cached)
- ✅ Lazy discovery on first use

### Testing Impact
- ✅ Tests can still use hardcoded values (documented as test-only)
- ✅ Add new tests for discovery mechanisms

### Migration Path
```rust
// v0.6.3 (current) - Works but hardcoded
config.songbird_endpoint = Some("http://localhost:8082");

// v0.7.0 (transition) - Both work with deprecation warnings
config.songbird_endpoint = Some("http://localhost:8082");  // DEPRECATED
config.discovery_endpoint = None;  // Discovered at runtime

// v1.0.0 (clean) - Only new API
config.discovery_endpoint = None;  // Discovered at runtime
// songbird_endpoint removed
```

---

## 🎯 IMPLEMENTATION PRIORITY

### High Priority (v0.7.0 - Next 2 Weeks)
1. ✅ Replace "Songbird" with "Discovery" in config (non-breaking)
2. ✅ Abstract infrastructure names in comments
3. ✅ Environment-based endpoint discovery
4. ✅ Remove hardcoded default ports

**Estimated Time**: 8 hours

### Medium Priority (v0.8.0 - Next Month)
1. ✅ Create universal adapter trait
2. ✅ Implement infant discovery module
3. ✅ DNS SRV and mDNS support
4. ✅ Adapter factory pattern

**Estimated Time**: 10 hours

### Low Priority (v1.0.0 - Next Quarter)
1. ✅ Complete API renames (breaking changes)
2. ✅ Remove deprecated fields
3. ✅ Multi-adapter support (Consul, etcd, etc)
4. ✅ Advanced discovery strategies

**Estimated Time**: 12 hours

---

## 📚 DESIGN PRINCIPLES

### 1. Infant Discovery Pattern
```rust
// At startup, we only know ourselves
let self_knowledge = ServiceInfo {
    name: "loamspine-instance-123",
    capabilities: vec!["persistent-ledger", "waypoint-anchoring"],
    endpoints: vec![/* dynamically assigned */],
};

// Discover the universal adapter (discovery service)
let discovery = InfantDiscovery::new(self_knowledge.capabilities.clone())
    .discover_discovery_service()
    .await?;

// Register ourselves
discovery.register(self_knowledge).await?;

// Discover capabilities as needed
let signers = discovery.discover("signer").await?;
let storage = discovery.discover("object-storage").await?;
```

### 2. Graceful Degradation
```rust
// If we can't find a capability, we continue with reduced functionality
match discovery.discover("signer").await {
    Ok(signers) if !signers.is_empty() => {
        self.enable_signing(signers[0].clone());
    }
    _ => {
        tracing::warn!("No signer found, certificate operations will be limited");
        // Continue without signing capability
    }
}
```

### 3. Capability Vocabulary
Replace primal names with capability descriptions:

| Old (Primal) | New (Capability) |
|-------------|-----------------|
| Songbird | discovery-service |
| BearDog | signer, verifier, identity-provider |
| NestGate | object-storage, blob-storage |
| ToadStool | encryption-service, compute-provider |
| Squirrel | ai-service, inference-engine |
| LoamSpine | persistent-ledger, waypoint-anchoring |

---

## ✅ SUCCESS CRITERIA

1. **Zero primal names in production config** (tests can keep them)
2. **Zero hardcoded endpoints** (all discovered or env-configured)
3. **Zero infrastructure vendor names** (k8s, consul, etc)
4. **Backward compatible** until v1.0.0
5. **Infant discovery working** - starts with zero knowledge

---

**Next Steps**: Implement Phase 1 in v0.7.0 (8 hours)

🦴 **LoamSpine: Self-discovering permanent ledger**

