# 🔄 Hardcoding Elimination & Infant Discovery Migration

**Date**: December 28, 2025  
**Goal**: Achieve **100% zero knowledge start** — No hardcoded primal names, vendor names, or port numbers  
**Philosophy**: Each primal starts as an **infant** with zero knowledge, discovering the world at runtime

---

## 🎯 Current Status

### Remaining Hardcoding Found

**Primal Names** (9 files):
- `crates/loam-spine-core/src/temporal/moment.rs` - Comments referencing "NestGate"
- `crates/loam-spine-core/src/discovery.rs` - Primal name constants
- `crates/loam-spine-core/src/config.rs` - Service name references
- `crates/loam-spine-core/src/discovery_client.rs` - Test/example hardcoding
- `crates/loam-spine-core/src/service/lifecycle.rs` - Service references
- `crates/loam-spine-core/tests/songbird_integration.rs` - Test hardcoding
- `crates/loam-spine-core/tests/cli_signer_integration.rs` - Test hardcoding

**Port Numbers** (18 occurrences):
- `:8080` - JSON-RPC port
- `:9001` - tarpc port
- `:7070` - Other service ports
- `:5000` - Additional services

**Vendor Names**:
- Kubernetes (k8s) references
- Consul references (if any)
- Docker-specific assumptions

---

## 🌟 Target Architecture: Infant Discovery

### Core Principle
```
Start → Zero Knowledge
↓
Discover Capabilities (not names!)
↓
Build Service Mesh Dynamically
↓
Interact via Universal Adapter (Songbird)
↓
Never hardcode connections
```

### Discovery Layers (Order of Precedence)

1. **Environment Variables**
   ```bash
   CAPABILITY_SIGNING_ENDPOINT="http://localhost:8001"
   CAPABILITY_STORAGE_ENDPOINT="http://10.0.0.5:7070"
   CAPABILITY_ORCHESTRATION_ENDPOINT="http://service-mesh:9000"
   ```

2. **mDNS/Bonjour** (Local Network)
   - Zero-config LAN discovery
   - Friend brings laptop → automatically discovered
   - No configuration needed

3. **DNS SRV Records** (Production)
   ```
   _signing._tcp.example.com
   _storage._tcp.example.com
   _orchestration._tcp.example.com
   ```

4. **Service Registry** (Universal Adapter)
   - Query Songbird for capabilities
   - "Who provides 'signing'?" → Get endpoints
   - Dynamic service mesh formation

5. **Fallback/Degraded Mode**
   - Local-only operation
   - Reduced functionality
   - Clear error messages

---

## 📋 Implementation Plan

### Phase 1: Capability Definitions (2 hours)

**Create `crates/loam-spine-core/src/capabilities.rs`**:

```rust
/// Core LoamSpine capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoamSpineCapability {
    /// Permanent ledger storage
    PermanentLedger {
        entry_types: Vec<String>,
        max_spine_size: Option<usize>,
        supports_sealing: bool,
    },
    
    /// Certificate issuance and verification
    CertificateAuthority {
        cert_types: Vec<String>,
        supports_revocation: bool,
        supports_lending: bool,
    },
    
    /// Cryptographic proof generation
    ProofGeneration {
        proof_types: Vec<String>,
        algorithms: Vec<String>,
    },
    
    /// Temporal moment tracking
    TemporalTracking {
        moment_types: Vec<String>,
        anchor_types: Vec<String>,
    },
}

/// External capabilities we consume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExternalCapability {
    /// Cryptographic signing
    Signing {
        algorithms: Vec<String>,
        key_types: Vec<String>,
    },
    
    /// Content storage
    Storage {
        content_addressable: bool,
        max_size: Option<usize>,
    },
    
    /// Service discovery
    Discovery {
        protocols: Vec<String>,
        supports_federation: bool,
    },
    
    /// Session management
    SessionManagement {
        supports_persistence: bool,
    },
    
    /// Compute orchestration
    Compute {
        resource_types: Vec<String>,
    },
}
```

**Create `crates/loam-spine-core/src/constants/capabilities.rs`**:

```rust
/// LoamSpine capability identifiers (not names!)
pub mod loamspine {
    pub const PERMANENT_LEDGER: &str = "permanent-ledger";
    pub const CERTIFICATE_AUTHORITY: &str = "certificate-authority";
    pub const PROOF_GENERATION: &str = "proof-generation";
    pub const TEMPORAL_TRACKING: &str = "temporal-tracking";
}

/// External capability identifiers (what we need, not who provides!)
pub mod external {
    pub const SIGNING: &str = "cryptographic-signing";
    pub const STORAGE: &str = "content-storage";
    pub const DISCOVERY: &str = "service-discovery";
    pub const SESSION_MANAGEMENT: &str = "session-management";
    pub const COMPUTE: &str = "compute-orchestration";
}
```

### Phase 2: Port Constants Evolution (1 hour)

**Replace**: Hardcoded ports  
**With**: Named constants + environment override

```rust
// crates/loam-spine-core/src/constants/network.rs

use std::env;

/// Get JSON-RPC port from environment or default
pub fn jsonrpc_port() -> u16 {
    env::var("LOAMSPINE_JSONRPC_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_JSONRPC_PORT)
}

/// Get tarpc port from environment or default
pub fn tarpc_port() -> u16 {
    env::var("LOAMSPINE_TARPC_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_TARPC_PORT)
}

// Private defaults (never exposed directly)
const DEFAULT_JSONRPC_PORT: u16 = 8080;
const DEFAULT_TARPC_PORT: u16 = 9001;
```

### Phase 3: Discovery Implementation (3 hours)

**Evolve `crates/loam-spine-core/src/discovery.rs`**:

```rust
/// Infant discovery - starts with ZERO knowledge
pub struct InfantDiscovery {
    /// Our own capabilities (self-knowledge only)
    own_capabilities: Vec<LoamSpineCapability>,
    
    /// Discovered services (learned at runtime)
    discovered: Arc<RwLock<HashMap<String, DiscoveredService>>>,
    
    /// Discovery config (methods to try)
    config: DiscoveryConfig,
}

impl InfantDiscovery {
    /// Create with ZERO external knowledge
    pub fn new() -> Self {
        Self {
            own_capabilities: Self::introspect_capabilities(),
            discovered: Arc::new(RwLock::new(HashMap::new())),
            config: DiscoveryConfig::from_env_or_default(),
        }
    }
    
    /// Discover services that provide a capability
    pub async fn find_capability(&self, capability: &str) -> Result<Vec<DiscoveredService>> {
        info!("Infant discovery: searching for capability '{}'", capability);
        
        // Try discovery methods in order
        for method in &self.config.methods {
            match method {
                DiscoveryMethod::Environment => {
                    if let Some(service) = self.discover_via_env(capability).await? {
                        return Ok(vec![service]);
                    }
                }
                DiscoveryMethod::MDns => {
                    if let Ok(services) = self.discover_via_mdns(capability).await {
                        if !services.is_empty() {
                            return Ok(services);
                        }
                    }
                }
                DiscoveryMethod::DnsSrv => {
                    if let Ok(services) = self.discover_via_dns_srv(capability).await {
                        if !services.is_empty() {
                            return Ok(services);
                        }
                    }
                }
                DiscoveryMethod::ServiceRegistry(registry_url) => {
                    // Query universal adapter (Songbird or similar)
                    if let Ok(services) = self.query_registry(registry_url, capability).await {
                        if !services.is_empty() {
                            return Ok(services);
                        }
                    }
                }
            }
        }
        
        // No services found - return empty, don't fail
        warn!("No services found for capability '{}', operating in degraded mode", capability);
        Ok(vec![])
    }
    
    /// Discover via environment variables
    async fn discover_via_env(&self, capability: &str) -> Result<Option<DiscoveredService>> {
        let env_key = format!("CAPABILITY_{}_ENDPOINT", 
            capability.to_uppercase().replace('-', "_"));
        
        if let Ok(endpoint) = env::var(&env_key) {
            info!("Found capability '{}' via environment: {}", capability, endpoint);
            
            return Ok(Some(DiscoveredService {
                capability: capability.to_string(),
                endpoint,
                discovered_via: "environment".to_string(),
                metadata: HashMap::new(),
            }));
        }
        
        Ok(None)
    }
}
```

### Phase 4: Primal Name Removal (2 hours)

**Replace ALL instances of**:
- "Songbird" → "service-discovery" or "universal-adapter"
- "NestGate" → "content-storage"
- "BearDog" → "cryptographic-signing"
- "Squirrel" → "session-management"
- "ToadStool" → "compute-orchestration"

**Example transformation**:

```rust
// BEFORE (hardcoded primal name)
async fn get_nestgate_endpoint(&self) -> Result<String> {
    self.config.nestgate_url.clone()
}

// AFTER (capability-based)
async fn find_storage_service(&self) -> Result<DiscoveredService> {
    self.discovery
        .find_capability("content-storage")
        .await?
        .first()
        .cloned()
        .ok_or(Error::CapabilityNotFound("content-storage"))
}
```

### Phase 5: Vendor Name Removal (1 hour)

**Replace**:
- "Kubernetes" / "k8s" → "container-orchestrator"
- "Consul" → "service-registry"
- "Docker" → "container-runtime"
- "Prometheus" → "metrics-exporter"
- "Grafana" → "metrics-visualizer"

**Pattern**:
```rust
// BEFORE
#[cfg(feature = "k8s")]
mod kubernetes_integration;

// AFTER
#[cfg(feature = "container-orchestration")]
mod container_orchestration;

// Runtime detection
pub fn detect_orchestrator() -> Option<OrchestratorType> {
    if env::var("KUBERNETES_SERVICE_HOST").is_ok() {
        Some(OrchestratorType::Kubernetes)
    } else if env::var("DOCKER_HOST").is_ok() {
        Some(OrchestratorType::Docker)
    } else {
        None
    }
}
```

### Phase 6: Test Evolution (2 hours)

**Update tests to use capability patterns**:

```rust
#[tokio::test]
async fn test_infant_discovery() {
    // Start with ZERO knowledge
    let discovery = InfantDiscovery::new();
    
    // Set environment for test
    env::set_var("CAPABILITY_SIGNING_ENDPOINT", "http://localhost:8001");
    
    // Discover signing service
    let services = discovery.find_capability("cryptographic-signing").await.unwrap();
    assert!(!services.is_empty());
    
    // Verify we discovered it (not hardcoded it!)
    assert_eq!(services[0].endpoint, "http://localhost:8001");
}

#[tokio::test]
async fn test_degraded_mode_when_no_services() {
    let discovery = InfantDiscovery::new();
    
    // Don't set any environment variables
    env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
    
    // Should return empty, not error
    let services = discovery.find_capability("content-storage").await.unwrap();
    assert!(services.is_empty());
    
    // Application should handle gracefully
}
```

---

## 🔍 Migration Checklist

### Code Changes
- [ ] Create `capabilities.rs` module
- [ ] Create `constants/capabilities.rs`
- [ ] Evolve `constants/network.rs` (port functions)
- [ ] Implement `InfantDiscovery` in `discovery.rs`
- [ ] Remove all primal names from comments
- [ ] Remove all primal names from code
- [ ] Remove all vendor names
- [ ] Update all tests to use capabilities
- [ ] Update examples to show infant discovery

### Documentation
- [ ] Update README to emphasize zero hardcoding
- [ ] Document capability identifiers
- [ ] Document environment variables
- [ ] Create infant discovery guide
- [ ] Update integration examples

### Testing
- [ ] Test environment-based discovery
- [ ] Test mDNS discovery (local)
- [ ] Test DNS SRV discovery (production)
- [ ] Test service registry discovery
- [ ] Test degraded mode (no services found)
- [ ] Test zero knowledge startup

---

## 📊 Success Criteria

### Zero Hardcoding
✅ No primal names in production code  
✅ No vendor names in production code  
✅ No hardcoded ports (all via env or discovery)  
✅ No hardcoded endpoints  
✅ Tests can use test-specific config

### Infant Discovery
✅ Start with zero knowledge  
✅ Discover via environment first  
✅ Discover via mDNS for LAN  
✅ Discover via DNS SRV for production  
✅ Query service registry (Songbird) as universal adapter  
✅ Operate in degraded mode if services unavailable

### O(n) Not O(n²)
✅ Each primal knows only itself  
✅ All discovery via universal adapter  
✅ No direct primal-to-primal hardcoding  
✅ Network effects through service mesh

---

## 🎓 Learning from Mature Primals

### BearDog Pattern
- Capability-based discovery
- Environment variables first
- Multiple discovery methods with fallback
- Cache discovered services
- Clear error messages for degraded mode

### Key Files to Review
- `beardog-discovery/src/discovery.rs` - Capability discovery
- `beardog-types/src/capabilities.rs` - Capability definitions
- `beardog-utils/src/network/port_discovery.rs` - Port discovery
- `beardog-discovery/src/mdns.rs` - mDNS implementation

---

## 🚀 Implementation Order

1. **Capabilities Module** (foundation) - 2 hours
2. **Port Constants** (easy wins) - 1 hour
3. **Discovery Implementation** (core) - 3 hours
4. **Primal Name Removal** (systematic) - 2 hours
5. **Vendor Name Removal** (cleanup) - 1 hour
6. **Test Evolution** (validation) - 2 hours

**Total Estimated Time**: 11 hours

---

## 💡 Philosophy

**"Each primal is born as an infant with zero knowledge of the world."**

- **Self-Knowledge Only**: Each primal introspects its own capabilities
- **Runtime Discovery**: All external services discovered dynamically
- **Universal Adapter**: Songbird (or similar) connects the ecosystem
- **O(n) Scaling**: Add new primals without changing existing ones
- **Graceful Degradation**: Operate with reduced functionality if services unavailable
- **Friend's Laptop**: Bring device to LAN → automatically discovered via mDNS

**Result**: Truly sovereign, decentralized ecosystem with zero hardcoded dependencies!

---

**Status**: Plan Complete, Ready for Implementation  
**Next**: Begin Phase 1 (Capabilities Module)

