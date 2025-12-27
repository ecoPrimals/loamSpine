# 🔥 Immediate Hardcoding Fixes — Quick Wins

**Priority**: CRITICAL  
**Time**: 2-3 hours  
**Impact**: Eliminate 162 "Songbird" vendor hardcodings

---

## 🎯 QUICK WIN #1: Rename Module (30 min)

### Step 1: Rename File
```bash
cd /path/to/ecoPrimals/phase2/loamSpine

# Rename the module file
git mv crates/loam-spine-core/src/songbird.rs \
       crates/loam-spine-core/src/discovery_client.rs
```

### Step 2: Update Module Declaration
```rust
// crates/loam-spine-core/src/lib.rs

// Before
pub mod songbird;

// After
pub mod discovery_client;

// Re-export for backward compatibility (v0.7.0 only)
#[deprecated(since = "0.7.0", note = "Use discovery_client module instead")]
pub use discovery_client as songbird;
```

---

## 🎯 QUICK WIN #2: Rename Type (1 hour)

### Find and Replace (162 instances)

```bash
# Use ripgrep to find all instances
rg "SongbirdClient" --type rust

# Automated replacement (careful!)
find crates -name "*.rs" -type f -exec sed -i 's/SongbirdClient/DiscoveryClient/g' {} +

# Manual verification recommended for critical files
```

### Key Files to Update Manually

1. **`discovery_client.rs`** (formerly songbird.rs)
```rust
// Before
pub struct SongbirdClient {
    endpoint: String,
    client: reqwest::Client,
}

// After
pub struct DiscoveryClient {
    endpoint: String,
    client: reqwest::Client,
}
```

2. **`infant_discovery.rs`**
```rust
// Before
use crate::songbird::SongbirdClient;

// After
use crate::discovery_client::DiscoveryClient;
```

3. **`lifecycle.rs`**
```rust
// Before
songbird_client: Option<Arc<SongbirdClient>>,

// After
discovery_client: Option<Arc<DiscoveryClient>>,
```

4. **`discovery.rs`**
```rust
// Before
pub use crate::songbird::{DiscoveredService, SongbirdClient};

// After
pub use crate::discovery_client::{DiscoveredService, DiscoveryClient};
```

---

## 🎯 QUICK WIN #3: Update Documentation (30 min)

### Module-Level Documentation

```rust
// crates/loam-spine-core/src/discovery_client.rs

//! Discovery client for universal service discovery.
//!
//! This module provides integration with discovery services (such as
//! Songbird, Consul, etcd, or custom implementations) for discovering
//! other primals' capabilities at runtime without hardcoding.
//!
//! ## Philosophy
//!
//! - **Zero hardcoding**: No primal or vendor names in code
//! - **Runtime discovery**: Find capabilities when needed
//! - **O(n) complexity**: Each primal connects to discovery service, not to each other
//! - **Infant learning**: Start with zero knowledge, discover everything
//!
//! ## Supported Discovery Services
//!
//! This client works with any discovery service that implements the
//! capability-based discovery protocol:
//!
//! - **Songbird** (reference implementation)
//! - **Consul** (with adapter)
//! - **etcd** (with adapter)
//! - **Kubernetes DNS-SD**
//! - **Custom implementations**
//!
//! ## Example
//!
//! ```rust,no_run
//! use loam_spine_core::discovery_client::DiscoveryClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to discovery service (auto-discovered or explicit)
//! let client = DiscoveryClient::connect("http://discovery.local:8082").await?;
//!
//! // Discover signing capability (works with any provider!)
//! let services = client.discover_capability("signing").await?;
//! for service in services {
//!     println!("Found signing service: {} at {}", service.name, service.endpoint);
//! }
//!
//! // Advertise our capabilities
//! client.advertise_loamspine("http://localhost:9001", "http://localhost:8080").await?;
//! # Ok(())
//! # }
//! ```
```

### Type Documentation

```rust
/// Discovery client for capability-based service discovery.
///
/// This client connects to a discovery service instance to discover other
/// primals' capabilities and advertise LoamSpine's own capabilities.
///
/// ## Discovery Service Agnostic
///
/// This client works with **any** discovery service that implements the
/// capability-based discovery protocol. Common options include:
///
/// - Songbird (reference implementation)
/// - Consul service mesh
/// - etcd key-value store
/// - Kubernetes DNS-based service discovery
/// - Custom implementations
///
/// ## Auto-Discovery
///
/// The client can auto-discover the discovery service endpoint via:
/// 1. `DISCOVERY_ENDPOINT` environment variable
/// 2. DNS SRV records (`_discovery._tcp.local`)
/// 3. mDNS (local network)
/// 4. Development fallback (localhost:8082 with warning)
#[derive(Clone, Debug)]
pub struct DiscoveryClient {
    /// Discovery service endpoint.
    endpoint: String,
    /// HTTP client.
    client: reqwest::Client,
}
```

---

## 🎯 QUICK WIN #4: Named Port Constants (15 min)

### Create Constants Module

```rust
// crates/loam-spine-core/src/constants.rs (NEW FILE)

//! Well-known constants for LoamSpine.
//!
//! These provide sensible defaults but can always be overridden via
//! environment variables or configuration files.

/// Default tarpc port for primal-to-primal communication.
///
/// This is a sensible default, but production deployments should:
/// - Set port to `0` for OS assignment
/// - Or configure via `TARPC_PORT` environment variable
pub const DEFAULT_TARPC_PORT: u16 = 9001;

/// Default JSON-RPC port for external clients.
///
/// This is a sensible default, but production deployments should:
/// - Set port to `0` for OS assignment
/// - Or configure via `JSONRPC_PORT` environment variable
pub const DEFAULT_JSONRPC_PORT: u16 = 8080;

/// Default discovery service port (fallback only).
///
/// This is **only** used as a last-resort development fallback when:
/// - No `DISCOVERY_ENDPOINT` environment variable is set
/// - DNS SRV lookup fails
/// - mDNS discovery fails
///
/// Production deployments should **never** rely on this fallback.
pub const DEFAULT_DISCOVERY_PORT: u16 = 8082;

/// OS-assigned port (let kernel choose available port).
///
/// Recommended for production:
/// ```rust
/// config.tarpc_port = OS_ASSIGNED_PORT;  // Let OS choose
/// ```
pub const OS_ASSIGNED_PORT: u16 = 0;
```

### Update Usage

```rust
// crates/loam-spine-core/src/config.rs

use crate::constants::{DEFAULT_TARPC_PORT, DEFAULT_JSONRPC_PORT};

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            discovery_enabled: true,
            discovery_endpoint: None,
            tarpc_endpoint: env::var("TARPC_ENDPOINT")
                .unwrap_or_else(|_| format!("http://0.0.0.0:{}", DEFAULT_TARPC_PORT)),
            jsonrpc_endpoint: env::var("JSONRPC_ENDPOINT")
                .unwrap_or_else(|_| format!("http://0.0.0.0:{}", DEFAULT_JSONRPC_PORT)),
            // ...
        }
    }
}
```

---

## 🎯 QUICK WIN #5: Update Test Names (15 min)

### Rename Test File (Optional but Recommended)

```bash
# Make it clear these test ANY discovery service (Songbird is reference)
git mv crates/loam-spine-core/tests/songbird_integration.rs \
       crates/loam-spine-core/tests/discovery_integration.rs
```

### Update Test Documentation

```rust
// crates/loam-spine-core/tests/discovery_integration.rs

//! Integration tests for discovery client.
//!
//! These tests verify integration with discovery services. We use Songbird
//! as the **reference implementation**, but any discovery service that
//! implements the capability-based protocol should work.
//!
//! ## Test Setup
//!
//! Tests require a discovery service binary (Songbird by default):
//! - Place at `../bins/songbird`
//! - Or set `DISCOVERY_SERVICE_BIN=/path/to/discovery/service`
//! - Or use `DISCOVERY_ENDPOINT=http://existing-service:port`
```

---

## 🎯 VERIFICATION CHECKLIST

### After Quick Wins
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy errors: `cargo clippy --workspace --all-features -- -D warnings`
- [ ] Documentation builds: `cargo doc --no-deps`
- [ ] Grep verification: `rg "Songbird" --type rust` (should only find deprecated code and comments)
- [ ] Examples work: `cargo run --example hello_loamspine`

### Grep Checks
```bash
# Should find ZERO instances in src/ (excluding deprecated/docs)
rg "SongbirdClient" crates/*/src --type rust

# Should find named constants
rg "DEFAULT_.*_PORT" crates/loam-spine-core/src --type rust

# Should find generic module name
rg "mod discovery_client" crates/loam-spine-core/src --type rust
```

---

## 📊 EXPECTED IMPACT

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| **Vendor References** | 162 | 0 (in src/) | ✅ 100% |
| **Magic Numbers** | 15+ | 0 | ✅ 100% |
| **Generic Types** | 30% | 100% | ✅ +70% |
| **Hardcoding Score** | 70% | 95% | ✅ +25% |

---

## 🚀 RUN SCRIPT

Save as `scripts/eliminate_songbird_hardcoding.sh`:

```bash
#!/bin/bash
set -e

echo "🔥 Eliminating Songbird vendor hardcoding..."

# Step 1: Rename module file
echo "📁 Renaming songbird.rs → discovery_client.rs"
git mv crates/loam-spine-core/src/songbird.rs \
       crates/loam-spine-core/src/discovery_client.rs

# Step 2: Create constants file
echo "📝 Creating constants.rs"
cat > crates/loam-spine-core/src/constants.rs << 'EOF'
//! Well-known constants for LoamSpine.

/// Default tarpc port for primal-to-primal communication.
pub const DEFAULT_TARPC_PORT: u16 = 9001;

/// Default JSON-RPC port for external clients.
pub const DEFAULT_JSONRPC_PORT: u16 = 8080;

/// Default discovery service port (fallback only).
pub const DEFAULT_DISCOVERY_PORT: u16 = 8082;

/// OS-assigned port (let kernel choose available port).
pub const OS_ASSIGNED_PORT: u16 = 0;
EOF

# Step 3: Update lib.rs
echo "🔧 Updating lib.rs module declarations"
sed -i 's/pub mod songbird;/pub mod discovery_client;\n\n#[deprecated(since = "0.7.0", note = "Use discovery_client instead")]\npub use discovery_client as songbird;/' \
    crates/loam-spine-core/src/lib.rs

# Step 4: Global replace SongbirdClient → DiscoveryClient
echo "🔍 Replacing SongbirdClient → DiscoveryClient (162 instances)"
find crates -name "*.rs" -type f -exec sed -i 's/SongbirdClient/DiscoveryClient/g' {} +

# Step 5: Update imports
echo "📦 Updating imports"
find crates -name "*.rs" -type f -exec sed -i 's/use crate::songbird::/use crate::discovery_client::/g' {} +
find crates -name "*.rs" -type f -exec sed -i 's/crate::songbird::/crate::discovery_client::/g' {} +

# Step 6: Verify
echo "✅ Running tests to verify changes"
cargo test --workspace --quiet

echo "✅ Running clippy to verify no new warnings"
cargo clippy --workspace --all-features -- -D warnings

echo ""
echo "🎉 Hardcoding elimination complete!"
echo ""
echo "📊 Summary:"
echo "  - Renamed module: songbird.rs → discovery_client.rs"
echo "  - Renamed type: SongbirdClient → DiscoveryClient (~162 instances)"
echo "  - Created constants: DEFAULT_*_PORT"
echo "  - All tests passing ✅"
echo ""
echo "Next steps:"
echo "  1. Review git diff"
echo "  2. Update documentation (see IMMEDIATE_HARDCODING_FIXES.md)"
echo "  3. Commit changes"
echo "  4. Move to Phase 2 (separate discovery crate)"
```

Make executable:
```bash
chmod +x scripts/eliminate_songbird_hardcoding.sh
```

---

## ⚠️ BREAKING CHANGES (v0.7.0 → v0.8.0)

### For Users

**v0.7.0** (backward compatible):
```rust
// Old code still works (deprecated warnings)
use loam_spine_core::songbird::SongbirdClient;
let client = SongbirdClient::connect(...).await?;

// New code (recommended)
use loam_spine_core::discovery_client::DiscoveryClient;
let client = DiscoveryClient::connect(...).await?;
```

**v0.8.0** (migration required):
```rust
// Only new names supported
use loam_spine_core::discovery_client::DiscoveryClient;
let client = DiscoveryClient::connect(...).await?;
```

### Migration Guide

Include in CHANGELOG.md:
```markdown
## [0.8.0] - BREAKING CHANGES

### Renamed: Songbird → Discovery Client

To eliminate vendor hardcoding and achieve 100% zero hardcoding:

- Module: `songbird.rs` → `discovery_client.rs`
- Type: `SongbirdClient` → `DiscoveryClient`
- Config: `songbird_endpoint` → `discovery_endpoint` (deprecated in v0.7.0)

### Migration

```rust
// Before
use loam_spine_core::songbird::SongbirdClient;
let client = SongbirdClient::connect("http://discovery:8082").await?;

// After
use loam_spine_core::discovery_client::DiscoveryClient;
let client = DiscoveryClient::connect("http://discovery:8082").await?;
```

### Why?

This change eliminates hardcoded vendor names ("Songbird") and makes LoamSpine
work with **any** discovery service (Consul, etcd, Kubernetes DNS, custom, etc.).

LoamSpine now achieves **100% zero hardcoding**, matching BearDog's world-class standard.
```

---

**🔥 Ready to eliminate hardcoding! Run the script or follow steps manually.**

