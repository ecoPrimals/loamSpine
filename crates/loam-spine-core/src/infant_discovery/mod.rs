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
    TokioAsyncResolver,
    config::{ResolverConfig, ResolverOpts},
    lookup::SrvLookup,
};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::capabilities::identifiers::external;
use crate::constants::HTTPS_DEFAULT_PORT;

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
    #[must_use]
    pub fn from_env_or_default() -> Self {
        let mut config = Self::default();

        // Check for service registry URL
        if let Ok(registry_url) = env::var("SERVICE_REGISTRY_URL") {
            config
                .methods
                .push(DiscoveryMethod::ServiceRegistry(registry_url));
        }

        // Allow overriding cache TTL
        if let Ok(ttl_str) = env::var("DISCOVERY_CACHE_TTL")
            && let Ok(ttl) = ttl_str.parse::<u64>()
        {
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
    #[must_use]
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
            if let Some(services) = discovered.get(capability)
                && !services.is_empty()
            {
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
            {
                let mut discovered = self.discovered.write().await;
                discovered.insert(capability.to_string(), all_services.clone());
            }
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
    async fn discover_via_mdns(&self, capability: &str) -> Vec<DiscoveredService> {
        debug!("Attempting mDNS discovery for '{}'", capability);

        #[cfg(feature = "mdns")]
        {
            let service_name = capability_to_srv_name(capability);
            let capability = capability.to_string();
            let cache_ttl_secs = self.config.cache_ttl_secs;

            let services = tokio::task::spawn_blocking(move || {
                mdns_discover_impl(&service_name, &capability, cache_ttl_secs)
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

    /// Check if a service is still fresh (within TTL)
    fn is_fresh(service: &DiscoveredService) -> bool {
        let age = SystemTime::now()
            .duration_since(service.discovered_at)
            .unwrap_or(Duration::from_secs(u64::MAX));

        age.as_secs() < service.ttl_secs
    }

    /// Clear cached discoveries (force rediscovery)
    pub async fn clear_cache(&self) {
        self.discovered.write().await.clear();
        info!("Discovery cache cleared");
    }

    /// Get all currently discovered services
    pub async fn all_discovered(&self) -> HashMap<String, Vec<DiscoveredService>> {
        {
            let discovered = self.discovered.read().await;
            discovered.clone()
        }
    }
}

/// Real mDNS discovery implementation (runs in spawn_blocking).
///
/// Uses the mdns crate to query for DNS-SD services, parses SRV records,
/// and converts results to DiscoveredService. All errors are handled
/// gracefully (returns empty vec, logs warnings).
#[cfg(feature = "mdns")]
fn mdns_discover_impl(
    service_type: &str,
    capability: &str,
    cache_ttl_secs: u64,
) -> Vec<DiscoveredService> {
    use std::time::Instant;

    let discovery = match mdns::discover::all(service_type, Duration::from_secs(2)) {
        Ok(d) => d,
        Err(e) => {
            warn!("mDNS discovery failed for {service_type}: {e}");
            return vec![];
        }
    };

    let stream = discovery.listen();

    let services = async_std::task::block_on(async move {
        use futures_util::{pin_mut, stream::StreamExt};

        let mut services = Vec::new();
        pin_mut!(stream);

        let deadline = Instant::now() + Duration::from_secs(2);

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }

            let next = stream.next();
            match async_std::future::timeout(remaining, next).await {
                Ok(Some(Ok(response))) => {
                    if let Some(service) =
                        parse_mdns_response(&response, capability, cache_ttl_secs)
                    {
                        services.push(service);
                    }
                }
                Ok(Some(Err(e))) => {
                    warn!("mDNS response error: {e}");
                }
                Ok(None) | Err(_) => break,
            }
        }

        services
    });

    if services.is_empty() {
        debug!("No mDNS services found for {service_type}");
    } else {
        info!(
            "Found {} services via mDNS for '{capability}'",
            services.len()
        );
    }

    services
}

/// Parse a single mDNS response into a DiscoveredService.
///
/// Extracts SRV records for host/port and A/AAAA for address resolution.
/// Returns None if the response cannot be parsed into a valid service.
#[cfg(feature = "mdns")]
fn parse_mdns_response(
    response: &mdns::Response,
    capability: &str,
    cache_ttl_secs: u64,
) -> Option<DiscoveredService> {
    use mdns::RecordKind;

    let port = response.port()?;
    let endpoint = if let Some(addr) = response.ip_addr() {
        if port == HTTPS_DEFAULT_PORT {
            format!("https://{addr}")
        } else {
            format!("https://{addr}:{port}")
        }
    } else {
        let target = response.records().find_map(|r| match &r.kind {
            RecordKind::SRV { target, .. } => Some(target.clone()),
            _ => None,
        })?;
        if port == HTTPS_DEFAULT_PORT {
            format!("https://{target}")
        } else {
            format!("https://{target}:{port}")
        }
    };

    let id = response.ip_addr().map_or_else(
        || format!("mdns-{capability}-{port}"),
        |a| format!("mdns-{a}:{port}"),
    );

    Some(DiscoveredService {
        id,
        capability: capability.to_string(),
        endpoint,
        discovered_via: "mdns".to_string(),
        metadata: HashMap::new(),
        health: ServiceHealth::Unknown,
        discovered_at: SystemTime::now(),
        ttl_secs: cache_ttl_secs,
    })
}

/// Convert capability to DNS SRV service name (RFC 2782)
///
/// Maps capability identifiers from [`crate::capabilities::identifiers::external`]
/// to their corresponding SRV record names.
///
/// Examples:
/// - "cryptographic-signing" -> "_signing._tcp.local"
/// - "content-storage" -> "_storage._tcp.local"
/// - "service-discovery" -> "_discovery._tcp.local"
fn capability_to_srv_name(capability: &str) -> String {
    let service_part = match capability {
        external::SIGNING => "signing",
        external::STORAGE => "storage",
        external::DISCOVERY => "discovery",
        external::SESSION_MANAGEMENT => "session",
        external::COMPUTE => "compute",
        external::ATTESTATION => "attestation",
        other => other.split('-').next_back().unwrap_or("service"),
    };

    format!("_{service_part}._tcp.local")
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
