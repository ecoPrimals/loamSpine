// SPDX-License-Identifier: AGPL-3.0-only

//! Infant Discovery - Start with Zero Knowledge
//!
//! This module implements the "infant discovery" pattern where LoamSpine
//! starts with ZERO knowledge of external services and discovers everything
//! at runtime based on capabilities.
//!
//! ## Philosophy
//!
//! **"Each primal is born as an infant, knowing only itself."**
//!
//! - No hardcoded primal names
//! - No hardcoded endpoints  
//! - No hardcoded ports
//! - All discovery at runtime via multiple methods
//!
//! ## Discovery Methods (Priority Order)
//!
//! 1. **Environment Variables** - Explicit configuration
//! 2. **mDNS/Bonjour** - Zero-config LAN discovery
//! 3. **DNS SRV** - Production service discovery
//! 4. **Service Registry** - Universal adapter (any RFC 2782 compliant system)
//! 5. **Degraded Mode** - Operate with reduced functionality
//!
//! ## Examples
//!
//! ```rust,no_run
//! use loam_spine_core::infant_discovery::InfantDiscovery;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create with ZERO external knowledge
//! let discovery = InfantDiscovery::new()?;
//!
//! // Discover signing service (NOT "BearDog"!)
//! let signers = discovery
//!     .find_capability("cryptographic-signing")
//!     .await?;
//!
//! if let Some(signer) = signers.first() {
//!     println!("Found signing service at: {}", signer.endpoint);
//! } else {
//!     println!("No signing service available, operating in degraded mode");
//! }
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    lookup::SrvLookup,
    TokioAsyncResolver,
};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::capabilities::{DiscoveredService, LoamSpineCapability, ServiceHealth};
use crate::error::LoamSpineResult;

/// Discovery method types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// Environment variables (highest priority)
    Environment,
    /// mDNS/Bonjour for local network (requires `mdns` feature)
    MDns,
    /// DNS SRV records for production (RFC 2782)
    DnsSrv,
    /// Service registry/universal adapter
    ServiceRegistry(String),
}

/// Discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Discovery methods to try (in order)
    pub methods: Vec<DiscoveryMethod>,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
    /// Retry attempts for failed discoveries
    pub retry_attempts: u32,
    /// Timeout for each discovery attempt
    pub discovery_timeout: Duration,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        let mut methods = vec![DiscoveryMethod::Environment];

        // DNS-SRV is production-grade; always available as fallback
        methods.push(DiscoveryMethod::DnsSrv);

        // mDNS for zero-config LAN discovery when feature-enabled
        #[cfg(feature = "mdns")]
        methods.push(DiscoveryMethod::MDns);

        Self {
            methods,
            cache_ttl_secs: 300,
            retry_attempts: 3,
            discovery_timeout: Duration::from_secs(5),
        }
    }
}

impl DiscoveryConfig {
    /// Create configuration from environment or defaults
    pub fn from_env_or_default() -> Self {
        let mut config = Self::default();

        // Check for service registry URL
        if let Ok(registry_url) = env::var("SERVICE_REGISTRY_URL") {
            config
                .methods
                .push(DiscoveryMethod::ServiceRegistry(registry_url));
        }

        // Allow overriding cache TTL
        if let Ok(ttl_str) = env::var("DISCOVERY_CACHE_TTL") {
            if let Ok(ttl) = ttl_str.parse::<u64>() {
                config.cache_ttl_secs = ttl;
            }
        }

        config
    }
}

/// Infant Discovery - starts with zero knowledge
pub struct InfantDiscovery {
    /// Our own capabilities (self-knowledge only)
    own_capabilities: Vec<LoamSpineCapability>,

    /// Discovered services (learned at runtime)
    discovered: Arc<RwLock<HashMap<String, Vec<DiscoveredService>>>>,

    /// Discovery configuration
    config: DiscoveryConfig,
}

impl InfantDiscovery {
    /// Create infant discovery with ZERO external knowledge
    ///
    /// Only knows its own capabilities via introspection.
    /// All external services will be discovered at runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails (currently infallible).
    pub fn new() -> LoamSpineResult<Self> {
        Self::with_config(DiscoveryConfig::from_env_or_default())
    }

    /// Create with custom configuration
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails (currently infallible).
    pub fn with_config(config: DiscoveryConfig) -> LoamSpineResult<Self> {
        info!("Initializing infant discovery (zero external knowledge)");

        let own_capabilities = LoamSpineCapability::introspect();
        info!("Self-knowledge: {} capabilities", own_capabilities.len());

        Ok(Self {
            own_capabilities,
            discovered: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }

    /// Get our own capabilities (self-knowledge)
    pub fn own_capabilities(&self) -> &[LoamSpineCapability] {
        &self.own_capabilities
    }

    /// Discover services that provide a capability
    ///
    /// This is the main entry point for capability-based discovery.
    /// Returns empty vec if no services found (graceful degradation).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use loam_spine_core::infant_discovery::InfantDiscovery;
    /// # async fn example(discovery: &InfantDiscovery) -> Result<(), Box<dyn std::error::Error>> {
    /// // Find ANY service that provides signing
    /// let signers = discovery.find_capability("cryptographic-signing").await?;
    ///
    /// for signer in signers {
    ///     println!("Found: {} at {}", signer.capability, signer.endpoint);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if discovery methods fail critically.
    pub async fn find_capability(
        &self,
        capability: &str,
    ) -> LoamSpineResult<Vec<DiscoveredService>> {
        info!(
            "Infant discovery: searching for capability '{}'",
            capability
        );

        // Check cache first
        {
            let discovered = self.discovered.read().await;
            if let Some(services) = discovered.get(capability) {
                if !services.is_empty() {
                    // Verify services are still fresh
                    let fresh: Vec<_> = services
                        .iter()
                        .filter(|s| Self::is_fresh(s))
                        .cloned()
                        .collect();

                    if !fresh.is_empty() {
                        debug!("Found {} cached services for '{}'", fresh.len(), capability);
                        return Ok(fresh);
                    }
                }
            }
        }

        // Try each discovery method
        let mut all_services = Vec::new();

        for method in &self.config.methods {
            debug!("Trying discovery method: {:?}", method);

            let services = match method {
                DiscoveryMethod::Environment => self.discover_via_environment(capability),
                DiscoveryMethod::MDns => self.discover_via_mdns(capability).await,
                DiscoveryMethod::DnsSrv => self.discover_via_dns_srv(capability).await,
                DiscoveryMethod::ServiceRegistry(url) => {
                    self.discover_via_registry(url, capability).await
                }
            };

            all_services.extend(services);

            // If we found services, we can stop (unless we want to aggregate)
            if !all_services.is_empty() {
                break;
            }
        }

        // Update cache
        if all_services.is_empty() {
            warn!(
                "No services found for capability '{}', operating in degraded mode",
                capability
            );
        } else {
            let mut discovered = self.discovered.write().await;
            discovered.insert(capability.to_string(), all_services.clone());
            info!(
                "Discovered {} services for capability '{}'",
                all_services.len(),
                capability
            );
        }

        Ok(all_services)
    }

    /// Discover services via environment variables
    ///
    /// Looks for patterns:
    /// - `CAPABILITY_<TYPE>_ENDPOINT` - e.g., `CAPABILITY_SIGNING_ENDPOINT`
    /// - `<TYPE>_SERVICE_URL` - e.g., `SIGNING_SERVICE_URL`
    fn discover_via_environment(&self, capability: &str) -> Vec<DiscoveredService> {
        let mut services = Vec::new();

        // Pattern 1: CAPABILITY_<TYPE>_ENDPOINT
        let capability_key = format!(
            "CAPABILITY_{}_ENDPOINT",
            capability.to_uppercase().replace('-', "_")
        );

        if let Ok(endpoint) = env::var(&capability_key) {
            debug!(
                "Found capability endpoint: {} = {}",
                capability_key, endpoint
            );

            services.push(DiscoveredService {
                id: format!("env-{capability}"),
                capability: capability.to_string(),
                endpoint,
                discovered_via: "environment".to_string(),
                metadata: HashMap::new(),
                health: ServiceHealth::Unknown,
                discovered_at: SystemTime::now(),
                ttl_secs: self.config.cache_ttl_secs,
            });

            return services;
        }

        // Pattern 2: <TYPE>_SERVICE_URL
        let service_key = format!(
            "{}_SERVICE_URL",
            capability
                .to_uppercase()
                .replace('-', "_")
                .replace("CRYPTOGRAPHIC_", "")
                .replace("CONTENT_", "")
        );

        if let Ok(endpoint) = env::var(&service_key) {
            debug!("Found service URL: {service_key} = {endpoint}");

            services.push(DiscoveredService {
                id: format!("env-{capability}"),
                capability: capability.to_string(),
                endpoint,
                discovered_via: "environment".to_string(),
                metadata: HashMap::new(),
                health: ServiceHealth::Unknown,
                discovered_at: SystemTime::now(),
                ttl_secs: self.config.cache_ttl_secs,
            });
        }

        services
    }

    /// Discover services via DNS SRV records (RFC 2782)
    ///
    /// Queries for SRV records in the format: `_<capability>._tcp.local`
    /// For example: `_signing._tcp.local` for cryptographic-signing capability.
    ///
    /// This enables production deployments with standard DNS infrastructure.
    async fn discover_via_dns_srv(&self, capability: &str) -> Vec<DiscoveredService> {
        debug!("Attempting DNS SRV discovery for '{}'", capability);

        // Convert capability to DNS SRV service name
        // "cryptographic-signing" -> "_signing._tcp.local"
        let service_name = capability_to_srv_name(capability);

        debug!("Querying DNS SRV for: {}", service_name);

        // Create resolver (TokioAsyncResolver::tokio returns the resolver directly, not a Result)
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        // Query SRV records with timeout
        let lookup: SrvLookup = match tokio::time::timeout(
            Duration::from_secs(2),
            resolver.srv_lookup(service_name.as_str()),
        )
        .await
        {
            Ok(Ok(l)) => l,
            Ok(Err(e)) => {
                debug!("DNS SRV lookup failed for {}: {}", service_name, e);
                return vec![];
            }
            Err(_) => {
                debug!("DNS SRV lookup timeout for {}", service_name);
                return vec![];
            }
        };

        // Collect records with their properties
        let mut records_data: Vec<(u16, u16, String, u16)> = Vec::new();
        for record in lookup.iter() {
            let priority = record.priority();
            let weight = record.weight();
            let port = record.port();
            let target = record.target().to_utf8();
            records_data.push((priority, weight, target, port));
        }

        // Sort by priority (lower is better), then by weight (higher is better)
        records_data.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));

        let mut services = Vec::new();
        for (priority, weight, target, port) in records_data.iter().take(5) {
            // Limit to top 5
            // Construct endpoint (assume https for production)
            let endpoint = if *port == 443 {
                format!("https://{target}")
            } else {
                format!("https://{target}:{port}")
            };

            debug!(
                "Found service via DNS SRV: {} (priority: {}, weight: {})",
                endpoint, priority, weight
            );

            services.push(DiscoveredService {
                id: format!("dns-srv-{target}"),
                capability: capability.to_string(),
                endpoint,
                discovered_via: "dns-srv".to_string(),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("priority".to_string(), priority.to_string());
                    map.insert("weight".to_string(), weight.to_string());
                    map.insert("target".to_string(), target.clone());
                    map.insert("port".to_string(), port.to_string());
                    map
                },
                health: ServiceHealth::Unknown,
                discovered_at: SystemTime::now(),
                ttl_secs: self.config.cache_ttl_secs,
            });
        }

        if services.is_empty() {
            debug!("No DNS SRV records found for {}", service_name);
        } else {
            info!(
                "Found {} services via DNS SRV for '{}'",
                services.len(),
                capability
            );
        }

        services
    }

    /// Discover services via a service registry (universal adapter).
    ///
    /// Queries the registry's HTTP API for services providing the given capability.
    /// Compatible with any registry exposing a `/discover?capability=...` endpoint
    /// (Consul adapter, etcd adapter, or any compatible registry).
    async fn discover_via_registry(
        &self,
        registry_url: &str,
        capability: &str,
    ) -> Vec<DiscoveredService> {
        debug!(
            "Querying service registry at {} for '{}'",
            registry_url, capability
        );

        let client = match crate::discovery_client::DiscoveryClient::connect(registry_url).await {
            Ok(c) => c,
            Err(e) => {
                warn!("Cannot reach service registry at {registry_url}: {e}");
                return vec![];
            }
        };

        let services = match client.discover_capability(capability).await {
            Ok(s) => s,
            Err(e) => {
                warn!("Registry query failed for '{capability}': {e}");
                return vec![];
            }
        };

        services
            .into_iter()
            .map(|svc| DiscoveredService {
                id: format!("registry-{}", svc.name),
                capability: capability.to_string(),
                endpoint: svc.endpoint,
                discovered_via: format!("service-registry:{registry_url}"),
                metadata: svc.metadata,
                health: if svc.healthy {
                    ServiceHealth::Healthy
                } else {
                    ServiceHealth::Unknown
                },
                discovered_at: SystemTime::now(),
                ttl_secs: self.config.cache_ttl_secs,
            })
            .collect()
    }

    /// Discover services via mDNS/Bonjour (RFC 6762)
    ///
    /// Broadcasts query for services on the local network.
    /// This enables zero-configuration discovery on LANs.
    ///
    /// Note: mDNS is optional and requires the `mdns` feature to be enabled.
    #[allow(clippy::unused_async)] // async required for uniform dispatch in find_capability
    async fn discover_via_mdns(&self, capability: &str) -> Vec<DiscoveredService> {
        debug!("Attempting mDNS discovery for '{}'", capability);

        // mDNS discovery is optional (experimental feature)
        #[cfg(feature = "mdns")]
        {
            let service_name = capability_to_srv_name(capability);
            debug!("Broadcasting mDNS query for: {}", service_name);
            Self::mdns_query_stub(&service_name, capability)
        }

        #[cfg(not(feature = "mdns"))]
        {
            debug!("mDNS discovery not available (feature not enabled)");
            vec![]
        }
    }

    #[cfg(feature = "mdns")]
    fn mdns_query_stub(_service_name: &str, _capability: &str) -> Vec<DiscoveredService> {
        // Note: mDNS discovery is currently experimental
        // The mdns crate API may need adjustment based on version
        debug!("mDNS discovery is experimental and may need API adjustments");

        // For now, return empty to allow compilation
        // Full implementation would require:
        // 1. Proper async iteration over mDNS responses
        // 2. Parsing SRV records from responses
        // 3. Handling different mDNS service types

        warn!("mDNS feature is experimental - returning empty results");

        Vec::new()
    }

    /// Check if a service is still fresh (within TTL)
    fn is_fresh(service: &DiscoveredService) -> bool {
        let age = SystemTime::now()
            .duration_since(service.discovered_at)
            .unwrap_or(Duration::from_secs(u64::MAX));

        age.as_secs() < service.ttl_secs
    }

    /// Clear cached discoveries (force rediscovery)
    pub async fn clear_cache(&self) {
        let mut discovered = self.discovered.write().await;
        discovered.clear();
        info!("Discovery cache cleared");
    }

    /// Get all currently discovered services
    pub async fn all_discovered(&self) -> HashMap<String, Vec<DiscoveredService>> {
        let discovered = self.discovered.read().await;
        discovered.clone()
    }
}

/// Convert capability to DNS SRV service name (RFC 2782)
///
/// Examples:
/// - "cryptographic-signing" -> "_signing._tcp.local"
/// - "content-storage" -> "_storage._tcp.local"
/// - "service-discovery" -> "_discovery._tcp.local"
fn capability_to_srv_name(capability: &str) -> String {
    let service_part = match capability {
        "cryptographic-signing" => "signing",
        "content-storage" => "storage",
        "service-discovery" => "discovery",
        "session-management" => "session",
        "compute-orchestration" => "compute",
        other => {
            // Extract last part of capability name
            other.split('-').next_back().unwrap_or("service")
        }
    };

    format!("_{service_part}._tcp.local")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Tests use unwrap for clarity
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    async fn test_infant_starts_with_zero_knowledge() {
        let discovery = InfantDiscovery::new().unwrap();

        // Should know only its own capabilities
        assert!(!discovery.own_capabilities().is_empty());

        // Should have no discovered services initially
        let all = discovery.all_discovered().await;
        assert!(all.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_discover_via_environment() {
        // Clean environment first
        env::remove_var("CAPABILITY_SIGNING_ENDPOINT");
        env::remove_var("CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT");
        env::remove_var("SIGNING_SERVICE_URL");

        let discovery = InfantDiscovery::new().unwrap();

        // Should NOT find anything initially
        let services = discovery
            .find_capability("cryptographic-signing")
            .await
            .unwrap();
        assert!(services.is_empty());

        // Now set the environment variable
        env::set_var(
            "CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT",
            "http://localhost:8001",
        );

        // Clear cache to force rediscovery
        discovery.clear_cache().await;

        let services = discovery
            .find_capability("cryptographic-signing")
            .await
            .unwrap();

        assert_eq!(services.len(), 1);
        assert_eq!(services[0].endpoint, "http://localhost:8001");
        assert_eq!(services[0].discovered_via, "environment");

        // Cleanup
        env::remove_var("CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT");
    }

    #[tokio::test]
    #[serial]
    async fn test_degraded_mode_when_no_services() {
        // Don't set any environment variables
        env::remove_var("CAPABILITY_STORAGE_ENDPOINT");

        let discovery = InfantDiscovery::new().unwrap();
        let services = discovery.find_capability("content-storage").await.unwrap();

        // Should return empty, not error (graceful degradation)
        assert!(services.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_cache_functionality() {
        // Clean up any existing env vars first
        env::remove_var("CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT");
        env::remove_var("CAPABILITY_SIGNING_ENDPOINT");
        env::remove_var("SIGNING_SERVICE_URL");

        env::set_var(
            "CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT",
            "http://localhost:8001",
        );

        let discovery = InfantDiscovery::new().unwrap();

        // First discovery
        let services1 = discovery
            .find_capability("cryptographic-signing")
            .await
            .unwrap();
        assert_eq!(services1.len(), 1);

        // Second discovery (should hit cache)
        let services2 = discovery
            .find_capability("cryptographic-signing")
            .await
            .unwrap();
        assert_eq!(services2.len(), 1);

        // Clear cache
        discovery.clear_cache().await;

        // Third discovery (cache cleared, should rediscover)
        let services3 = discovery
            .find_capability("cryptographic-signing")
            .await
            .unwrap();
        assert_eq!(services3.len(), 1);

        // Clean up
        env::remove_var("CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT");
    }

    #[tokio::test]
    #[serial]
    async fn test_discover_via_signing_service_url() {
        env::remove_var("CAPABILITY_SIGNING_ENDPOINT");
        env::remove_var("CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT");
        env::remove_var("SIGNING_SERVICE_URL");

        let discovery = InfantDiscovery::new().unwrap();
        let services = discovery
            .find_capability("cryptographic-signing")
            .await
            .unwrap();
        assert!(services.is_empty());

        env::set_var("SIGNING_SERVICE_URL", "http://localhost:8002");
        discovery.clear_cache().await;

        let services = discovery
            .find_capability("cryptographic-signing")
            .await
            .unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].endpoint, "http://localhost:8002");

        env::remove_var("SIGNING_SERVICE_URL");
    }

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert!(!config.methods.is_empty());
        assert!(config.cache_ttl_secs > 0);
        assert!(config.retry_attempts > 0);
    }

    #[test]
    #[serial]
    fn test_discovery_config_from_env() {
        env::set_var("SERVICE_REGISTRY_URL", "http://registry.example.com");
        env::set_var("DISCOVERY_CACHE_TTL", "600");

        let config = DiscoveryConfig::from_env_or_default();
        assert!(config.methods.iter().any(|m| matches!(
            m,
            DiscoveryMethod::ServiceRegistry(url) if url == "http://registry.example.com"
        )));
        assert_eq!(config.cache_ttl_secs, 600);

        env::remove_var("SERVICE_REGISTRY_URL");
        env::remove_var("DISCOVERY_CACHE_TTL");
    }

    #[test]
    fn test_capability_to_srv_name_indirect() {
        let discovery = InfantDiscovery::new().unwrap();
        let capabilities = discovery.own_capabilities();
        assert!(!capabilities.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_service_registry_discovery_returns_empty() {
        env::remove_var("CAPABILITY_SIGNING_ENDPOINT");
        env::remove_var("SIGNING_SERVICE_URL");
        env::set_var("SERVICE_REGISTRY_URL", "http://registry.test");

        let config = DiscoveryConfig::from_env_or_default();
        let discovery = InfantDiscovery::with_config(config).unwrap();
        discovery.clear_cache().await;

        let services = discovery.find_capability("signing").await.unwrap();
        assert!(services.is_empty());

        env::remove_var("SERVICE_REGISTRY_URL");
    }

    #[tokio::test]
    async fn test_dns_srv_discovery_no_records() {
        let discovery = InfantDiscovery::new().unwrap();
        let services = discovery
            .discover_via_dns_srv("nonexistent-capability")
            .await;
        assert!(services.is_empty());
    }

    #[tokio::test]
    async fn test_registry_discovery_unreachable() {
        let discovery = InfantDiscovery::new().unwrap();
        let services = discovery
            .discover_via_registry("http://127.0.0.1:1", "signing")
            .await;
        assert!(services.is_empty());
    }

    #[tokio::test]
    async fn test_is_fresh_with_recent_service() {
        let service = DiscoveredService {
            id: "test".to_string(),
            capability: "signing".to_string(),
            endpoint: "http://localhost:8001".to_string(),
            discovered_via: "test".to_string(),
            metadata: HashMap::new(),
            health: ServiceHealth::Healthy,
            discovered_at: SystemTime::now(),
            ttl_secs: 300,
        };
        assert!(InfantDiscovery::is_fresh(&service));
    }

    #[tokio::test]
    async fn test_is_fresh_with_expired_service() {
        let service = DiscoveredService {
            id: "test".to_string(),
            capability: "signing".to_string(),
            endpoint: "http://localhost:8001".to_string(),
            discovered_via: "test".to_string(),
            metadata: HashMap::new(),
            health: ServiceHealth::Healthy,
            discovered_at: SystemTime::now() - std::time::Duration::from_secs(600),
            ttl_secs: 300,
        };
        assert!(!InfantDiscovery::is_fresh(&service));
    }

    #[test]
    fn test_capability_to_srv_name() {
        assert_eq!(
            capability_to_srv_name("cryptographic-signing"),
            "_signing._tcp.local"
        );
        assert_eq!(
            capability_to_srv_name("content-storage"),
            "_storage._tcp.local"
        );
        assert_eq!(capability_to_srv_name("simple"), "_simple._tcp.local");
    }

    #[test]
    fn test_own_capabilities_are_loamspine_specific() {
        let discovery = InfantDiscovery::new().unwrap();
        let caps = discovery.own_capabilities();
        let identifiers: Vec<&str> = caps.iter().map(LoamSpineCapability::identifier).collect();
        assert!(identifiers
            .iter()
            .any(|id| id.contains("ledger") || id.contains("permanence")));
    }
}
