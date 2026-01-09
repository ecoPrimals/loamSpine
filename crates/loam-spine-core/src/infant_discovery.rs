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
//! 4. **Service Registry** - Universal adapter (Songbird)
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

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::capabilities::{DiscoveredService, LoamSpineCapability, ServiceHealth};
use crate::error::LoamSpineResult;

/// Discovery method types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// Environment variables (highest priority)
    Environment,
    /// mDNS/Bonjour for local network
    #[allow(dead_code)]
    MDns,
    /// DNS SRV records for production
    #[allow(dead_code)]
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
        Self {
            methods: vec![
                DiscoveryMethod::Environment,
                // mDNS and DNS-SRV would be added when implemented
            ],
            cache_ttl_secs: 300, // 5 minutes
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
                DiscoveryMethod::MDns => {
                    // TODO: Implement mDNS discovery
                    warn!("mDNS discovery not yet implemented");
                    vec![]
                }
                DiscoveryMethod::DnsSrv => {
                    // TODO: Implement DNS-SRV discovery
                    warn!("DNS-SRV discovery not yet implemented");
                    vec![]
                }
                DiscoveryMethod::ServiceRegistry(url) => {
                    // TODO: Implement registry query
                    warn!("Service registry discovery not yet implemented for {}", url);
                    vec![]
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
}
