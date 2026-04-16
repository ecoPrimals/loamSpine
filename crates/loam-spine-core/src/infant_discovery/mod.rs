// SPDX-License-Identifier: AGPL-3.0-or-later

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
//! // Discover signing service by capability, not primal name
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

mod backends;
mod cache;

use std::collections::HashMap;
use std::env;
use std::time::{Duration, SystemTime};

use cache::DiscoveryCache;

#[cfg(feature = "dns-srv")]
use hickory_resolver::TokioResolver;
use tracing::{debug, info, warn};

#[cfg(any(feature = "dns-srv", feature = "mdns"))]
use crate::constants::HTTPS_DEFAULT_PORT;

/// DNS SRV lookup timeout — short to avoid blocking the discovery pipeline.
#[cfg(feature = "dns-srv")]
const DNS_SRV_TIMEOUT: Duration = Duration::from_secs(2);

use crate::capabilities::{DiscoveredService, LoamSpineCapability, ServiceHealth};
use crate::error::LoamSpineResult;

/// Discovery method types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryProtocol {
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
    pub methods: Vec<DiscoveryProtocol>,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
    /// Retry attempts for failed discoveries
    pub retry_attempts: u32,
    /// Timeout for each discovery attempt
    pub discovery_timeout: Duration,
    /// Environment variable overrides (for config-injection in tests).
    /// When a key exists here it is returned instead of `std::env::var`.
    pub env_overrides: HashMap<String, String>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        let mut methods = vec![DiscoveryProtocol::Environment];

        // DNS-SRV is production-grade; available when dns-srv feature is enabled
        #[cfg(feature = "dns-srv")]
        methods.push(DiscoveryProtocol::DnsSrv);

        // mDNS for zero-config LAN discovery when feature-enabled
        #[cfg(feature = "mdns")]
        methods.push(DiscoveryProtocol::MDns);

        Self {
            methods,
            cache_ttl_secs: 300,
            retry_attempts: 3,
            discovery_timeout: Duration::from_secs(5),
            env_overrides: HashMap::new(),
        }
    }
}

impl DiscoveryConfig {
    /// Create configuration from environment or defaults.
    #[must_use]
    pub fn from_env_or_default() -> Self {
        Self::from_explicit(
            env::var("SERVICE_REGISTRY_URL").ok().as_deref(),
            env::var("DISCOVERY_CACHE_TTL")
                .ok()
                .and_then(|s| s.parse().ok()),
        )
    }

    /// Create configuration from explicit values (pure, no env reads).
    #[must_use]
    pub fn from_explicit(registry_url: Option<&str>, cache_ttl: Option<u64>) -> Self {
        let mut config = Self::default();
        if let Some(url) = registry_url {
            config
                .methods
                .push(DiscoveryProtocol::ServiceRegistry(url.to_string()));
        }
        if let Some(ttl) = cache_ttl {
            config.cache_ttl_secs = ttl;
        }
        config
    }
}

/// Infant Discovery - starts with zero knowledge
pub struct InfantDiscovery {
    /// Our own capabilities (self-knowledge only)
    own_capabilities: Vec<LoamSpineCapability>,

    /// Discovered services (learned at runtime)
    pub(crate) cache: DiscoveryCache,

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
            cache: DiscoveryCache::new(),
            config,
        })
    }

    /// Get our own capabilities (self-knowledge)
    #[must_use]
    pub fn own_capabilities(&self) -> &[LoamSpineCapability] {
        &self.own_capabilities
    }

    /// Look up an environment variable, preferring `env_overrides` when present.
    fn env_lookup(&self, key: &str) -> Option<String> {
        self.config
            .env_overrides
            .get(key)
            .cloned()
            .or_else(|| env::var(key).ok())
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

        if let Some(fresh) = self.cache.get_fresh(capability).await {
            return Ok(fresh);
        }

        // Try each discovery method
        let mut all_services = Vec::new();

        for method in &self.config.methods {
            debug!("Trying discovery method: {:?}", method);

            let services = match method {
                DiscoveryProtocol::Environment => self.discover_via_environment(capability),
                DiscoveryProtocol::MDns => self.discover_via_mdns(capability).await,
                #[cfg(feature = "dns-srv")]
                DiscoveryProtocol::DnsSrv => self.discover_via_dns_srv(capability).await,
                #[cfg(not(feature = "dns-srv"))]
                DiscoveryProtocol::DnsSrv => {
                    debug!("DNS SRV discovery not available (feature not enabled)");
                    vec![]
                }
                DiscoveryProtocol::ServiceRegistry(url) => {
                    self.discover_via_registry(url, capability).await
                }
            };

            all_services.extend(services);

            // If we found services, we can stop (unless we want to aggregate)
            if !all_services.is_empty() {
                break;
            }
        }

        if all_services.is_empty() {
            warn!(
                "No services found for capability '{}', operating in degraded mode",
                capability
            );
        } else {
            self.cache
                .insert(capability.to_string(), all_services.clone())
                .await;
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

        if let Some(endpoint) = self.env_lookup(&capability_key) {
            debug!(
                "Found capability endpoint: {} = {}",
                capability_key, endpoint
            );

            services.push(DiscoveredService {
                id: format!("env-{capability}"),
                capability: capability.to_string(),
                endpoint,
                discovered_via: crate::constants::discovery_method::ENVIRONMENT.to_string(),
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

        if let Some(endpoint) = self.env_lookup(&service_key) {
            debug!("Found service URL: {service_key} = {endpoint}");

            services.push(DiscoveredService {
                id: format!("env-{capability}"),
                capability: capability.to_string(),
                endpoint,
                discovered_via: crate::constants::discovery_method::ENVIRONMENT.to_string(),
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
    /// Requires the `dns-srv` feature.
    #[cfg(feature = "dns-srv")]
    async fn discover_via_dns_srv(&self, capability: &str) -> Vec<DiscoveredService> {
        use hickory_resolver::proto::rr::rdata::SRV;

        debug!("Attempting DNS SRV discovery for '{}'", capability);

        let service_name = backends::capability_to_srv_name(capability);

        debug!("Querying DNS SRV for: {}", service_name);

        let resolver = match TokioResolver::builder_tokio()
            .and_then(hickory_resolver::ResolverBuilder::build)
        {
            Ok(r) => r,
            Err(e) => {
                debug!("Failed to create DNS resolver: {e}");
                return vec![];
            }
        };

        let lookup =
            match tokio::time::timeout(DNS_SRV_TIMEOUT, resolver.srv_lookup(service_name.as_str()))
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

        let mut records_data: Vec<(u16, u16, String, u16)> = Vec::new();
        for record in lookup.answers() {
            if let Some(srv_ref) = record.try_borrow::<SRV>() {
                let srv = srv_ref.data();
                let priority = srv.priority;
                let weight = srv.weight;
                let port = srv.port;
                let target = srv.target.to_utf8();
                records_data.push((priority, weight, target, port));
            }
        }

        // Sort by priority (lower is better), then by weight (higher is better)
        records_data.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));

        let mut services = Vec::new();
        for (priority, weight, target, port) in records_data.iter().take(5) {
            let endpoint = if *port == HTTPS_DEFAULT_PORT {
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
                discovered_via: crate::constants::discovery_method::DNS_SRV.to_string(),
                metadata: {
                    use crate::constants::srv_metadata;
                    let mut map = HashMap::new();
                    map.insert(srv_metadata::PRIORITY.to_string(), priority.to_string());
                    map.insert(srv_metadata::WEIGHT.to_string(), weight.to_string());
                    map.insert(srv_metadata::TARGET.to_string(), target.clone());
                    map.insert(srv_metadata::PORT.to_string(), port.to_string());
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
    async fn discover_via_mdns(&self, capability: &str) -> Vec<DiscoveredService> {
        debug!("Attempting mDNS discovery for '{}'", capability);

        #[cfg(feature = "mdns")]
        {
            let service_name = backends::capability_to_srv_name(capability);
            let capability = capability.to_string();
            let cache_ttl_secs = self.config.cache_ttl_secs;

            let services = tokio::task::spawn_blocking(move || {
                backends::mdns_discover_impl(&service_name, &capability, cache_ttl_secs)
            })
            .await;

            match services {
                Ok(svc) => svc,
                Err(e) => {
                    warn!("mDNS discovery task panicked: {e}");
                    vec![]
                }
            }
        }

        #[cfg(not(feature = "mdns"))]
        {
            let _ = capability;
            debug!("mDNS discovery not available (feature not enabled)");
            tokio::task::yield_now().await;
            vec![]
        }
    }

    /// Check if a service is still fresh (within TTL).
    #[cfg(test)]
    pub(crate) fn is_fresh(service: &DiscoveredService) -> bool {
        DiscoveryCache::is_fresh(service)
    }

    /// Clear cached discoveries (force rediscovery)
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
        info!("Discovery cache cleared");
    }

    /// Get all currently discovered services
    pub async fn all_discovered(&self) -> HashMap<String, Vec<DiscoveredService>> {
        self.cache.all().await
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "infant discovery integration tests use unwrap for conciseness"
)]
mod tests;

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "infant discovery integration tests use unwrap for conciseness"
)]
mod tests_coverage;
