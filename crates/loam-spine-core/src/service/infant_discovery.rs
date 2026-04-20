// SPDX-License-Identifier: AGPL-3.0-or-later

//! Infant discovery - start with zero knowledge, discover everything.
//!
//! This module implements the "infant discovery" philosophy: LoamSpine starts
//! knowing only itself and discovers all external services at runtime.
//!
//! ## Philosophy
//!
//! Like an infant learning about the world, LoamSpine:
//! 1. **Knows only itself** - Its own capabilities and identity
//! 2. **Discovers the discovery service** - The universal adapter for finding others
//! 3. **Discovers capabilities** - What services are available
//! 4. **Adapts gracefully** - Works with reduced capabilities if services unavailable
//!
//! ## Discovery Methods
//!
//! The discovery service itself is found through a priority chain:
//! 1. **Environment variables** - `DISCOVERY_ENDPOINT` (highest priority)
//! 2. **DNS SRV records** - `_discovery._tcp.local` (production)
//! 3. **mDNS** - Multicast DNS on local network (local development)
//! 4. **Development fallback** - `localhost:8082` (logged as warning)
//!
//! ## Example
//!
//! ```rust,no_run
//! use loam_spine_core::service::infant_discovery::InfantDiscovery;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create infant from canonical advertised capabilities
//! let infant = InfantDiscovery::from_advertised();
//!
//! // Discover the discovery service (universal adapter)
//! match infant.discover_discovery_service().await {
//!     Ok(discovery) => {
//!         // Register ourselves and discover capabilities
//!         println!("✅ Discovery service found!");
//!     }
//!     Err(e) => {
//!         // Continue with reduced capabilities
//!         println!("⚠️  No discovery service: {}", e);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;

use crate::discovery_client::DiscoveryClient;
use crate::error::{LoamSpineError, LoamSpineResult};

/// DNS SRV query for the discovery service (RFC 2782).
const DISCOVERY_SRV_QUERY: &str = "_discovery._tcp.local";

/// Infant discovery - discovers the discovery service at runtime.
///
/// This struct embodies the "infant discovery" philosophy: start with
/// self-knowledge only and discover everything else dynamically.
pub struct InfantDiscovery {
    /// Self-knowledge: our own capabilities (the only thing we know at start).
    self_capabilities: Vec<String>,
}

impl InfantDiscovery {
    /// Create a new infant discovery instance.
    ///
    /// The infant starts knowing only its own capabilities.
    ///
    /// # Arguments
    ///
    /// * `self_capabilities` - List of capabilities this service provides
    ///
    /// # Example
    ///
    /// ```
    /// use loam_spine_core::service::infant_discovery::InfantDiscovery;
    ///
    /// let infant = InfantDiscovery::new(vec![
    ///     "permanent-ledger".to_string(),
    ///     "waypoint-anchoring".to_string(),
    /// ]);
    /// ```
    #[must_use]
    pub fn new(self_capabilities: Vec<String>) -> Self {
        tracing::debug!(
            "🧒 Infant discovery initialized with {} self-capabilities",
            self_capabilities.len()
        );
        Self { self_capabilities }
    }

    /// Create from the canonical advertised capability set.
    ///
    /// Uses [`crate::capabilities::identifiers::loamspine::ADVERTISED`] as
    /// the single source of truth for what capabilities this primal provides.
    #[must_use]
    pub fn from_advertised() -> Self {
        Self::new(
            crate::capabilities::identifiers::loamspine::ADVERTISED
                .iter()
                .map(|&s| s.to_string())
                .collect(),
        )
    }

    /// Discover the discovery service (universal adapter).
    ///
    /// Tries multiple methods in priority order:
    /// 1. Environment variable (`DISCOVERY_ENDPOINT`)
    /// 2. DNS SRV records (`_discovery._tcp.local`)
    /// 3. mDNS (multicast DNS on local network)
    /// 4. Development fallback (`localhost:8082`, logged as warning)
    ///
    /// # Errors
    ///
    /// Returns an error if all discovery methods fail.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use loam_spine_core::service::infant_discovery::InfantDiscovery;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let infant = InfantDiscovery::from_advertised();
    ///
    /// match infant.discover_discovery_service().await {
    ///     Ok(client) => println!("✅ Found discovery service"),
    ///     Err(e) => println!("⚠️  No discovery service: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_discovery_service(&self) -> LoamSpineResult<DiscoveryClient> {
        self.discover_discovery_service_with(None).await
    }

    /// Discover the discovery service with an optional pre-resolved endpoint.
    ///
    /// When `endpoint_override` is `Some`, it is used as the first-priority
    /// endpoint (equivalent to a non-empty `DISCOVERY_ENDPOINT`). When `None`,
    /// `DISCOVERY_ENDPOINT` is read from the environment. An override of `Some("")`
    /// skips the environment step (same as an empty variable).
    ///
    /// # Errors
    ///
    /// Returns [`LoamSpineError`] if no discovery method succeeds.
    pub async fn discover_discovery_service_with(
        &self,
        endpoint_override: Option<&str>,
    ) -> LoamSpineResult<DiscoveryClient> {
        self.discover_discovery_service_with_env(endpoint_override, &HashMap::new())
            .await
    }

    /// Like [`Self::discover_discovery_service_with`], but resolves `DISCOVERY_ENDPOINT` from
    /// `env_overrides` before reading the process environment (for config injection / tests).
    ///
    /// # Errors
    ///
    /// Returns [`LoamSpineError`] if no discovery method succeeds.
    pub async fn discover_discovery_service_with_env(
        &self,
        endpoint_override: Option<&str>,
        env_overrides: &HashMap<String, String>,
    ) -> LoamSpineResult<DiscoveryClient> {
        tracing::info!("🔍 Starting infant discovery (zero knowledge → full knowledge)...");

        // Method 1: Environment variable (highest priority)
        if let Some(endpoint) = match endpoint_override {
            None => Self::try_environment_discovery_from(None, Some(env_overrides)),
            Some(s) => Self::try_environment_discovery_from(Some(s), Some(env_overrides)),
        } {
            tracing::info!("✅ Discovery service found via environment: {}", endpoint);
            return DiscoveryClient::connect(&endpoint).await;
        }

        // Method 2: DNS SRV records (production)
        if let Some(endpoint) = self.try_dns_srv_discovery().await {
            tracing::info!("✅ Discovery service found via DNS SRV: {}", endpoint);
            return DiscoveryClient::connect(&endpoint).await;
        }

        // Method 3: mDNS (local network)
        if let Some(endpoint) = self.try_mdns_discovery().await {
            tracing::info!("✅ Discovery service found via mDNS: {}", endpoint);
            return DiscoveryClient::connect(&endpoint).await;
        }

        // Method 4: Development fallback (lowest priority, logged as warning)
        if let Some(endpoint) = self.try_development_fallback() {
            tracing::warn!(
                "⚠️  Using development fallback: {}. Set DISCOVERY_ENDPOINT for production!",
                endpoint
            );
            return DiscoveryClient::connect(&endpoint).await;
        }

        // All methods failed
        Err(LoamSpineError::Internal(
            "No discovery service found. Set DISCOVERY_ENDPOINT environment variable or ensure \
             DNS SRV records are configured."
                .to_string(),
        ))
    }

    /// Get self-capabilities (what this service provides).
    #[must_use]
    pub fn capabilities(&self) -> &[String] {
        &self.self_capabilities
    }

    /// Try environment discovery with an explicit endpoint value (pure, no env reads when set).
    ///
    /// When `endpoint` is `None`, falls back to `DISCOVERY_ENDPOINT`. When `Some("")`,
    /// the environment step is skipped (same as an unset or empty variable).
    #[must_use]
    pub fn try_environment_discovery_with(&self, endpoint: Option<&str>) -> Option<String> {
        Self::try_environment_discovery_from(endpoint, None)
    }

    fn try_environment_discovery_from(
        endpoint: Option<&str>,
        env_overrides: Option<&HashMap<String, String>>,
    ) -> Option<String> {
        let value = endpoint.map(String::from).or_else(|| {
            env_overrides
                .and_then(|m| m.get("DISCOVERY_ENDPOINT").cloned())
                .or_else(|| std::env::var("DISCOVERY_ENDPOINT").ok())
        });
        match value {
            Some(v) if !v.is_empty() => {
                tracing::info!("🔍 Discovery endpoint from environment: {v}");
                Some(v)
            }
            _ => None,
        }
    }

    /// Try to discover via DNS SRV records.
    ///
    /// Looks up `_discovery._tcp.local` SRV record.
    /// This is the standard production discovery method.
    ///
    /// Note: Disabled in test mode to avoid runtime conflicts. Use environment variables in tests.
    #[allow(
        clippy::unused_async,
        reason = "async required for dns-srv feature builds; lint fires only in no-feature builds"
    )]
    async fn try_dns_srv_discovery(&self) -> Option<String> {
        tracing::debug!("🔍 Attempting DNS SRV discovery ({DISCOVERY_SRV_QUERY})...");

        #[cfg(test)]
        {
            tracing::debug!("🔍 DNS SRV discovery disabled in test mode, trying next method");
            None
        }

        #[cfg(all(not(test), feature = "dns-srv"))]
        {
            use hickory_resolver::TokioResolver;

            let resolver = match TokioResolver::builder_tokio()
                .and_then(hickory_resolver::ResolverBuilder::build)
            {
                Ok(r) => r,
                Err(e) => {
                    tracing::debug!("🔍 Failed to create DNS resolver: {e}, trying next method");
                    return None;
                }
            };

            match resolver.srv_lookup(DISCOVERY_SRV_QUERY).await {
                Ok(response) => {
                    use hickory_resolver::proto::rr::rdata::SRV;

                    response
                        .answers()
                        .iter()
                        .find_map(hickory_resolver::proto::rr::Record::try_borrow::<SRV>)
                        .map_or_else(
                            || {
                                tracing::debug!(
                                    "🔍 No SRV records found for {DISCOVERY_SRV_QUERY}, trying next method",
                                );
                                None
                            },
                            |srv_ref| {
                                let srv = srv_ref.data();
                                let target = srv.target.to_utf8();
                                let port = srv.port;
                                let endpoint =
                                    format!("http://{}:{}", target.trim_end_matches('.'), port);
                                tracing::info!("✅ DNS SRV discovery successful: {endpoint}");
                                Some(endpoint)
                            },
                        )
                }
                Err(e) => {
                    tracing::debug!("🔍 DNS SRV lookup failed: {e}, trying next method");
                    None
                }
            }
        }

        #[cfg(all(not(test), not(feature = "dns-srv")))]
        {
            tracing::debug!("DNS SRV discovery not available (dns-srv feature not enabled)");
            None
        }
    }

    /// Try to discover via mDNS-SD (multicast DNS service discovery).
    ///
    /// Broadcasts `_discovery._tcp.local.` on the local network to find the
    /// discovery service. Uses `mdns-sd` which manages its own daemon thread
    /// and delivers results via async-compatible flume channels.
    ///
    /// For production, prefer DNS SRV or environment variables.
    #[allow(
        clippy::unused_async,
        reason = "async required for mdns feature builds; lint fires only in no-feature builds"
    )]
    async fn try_mdns_discovery(&self) -> Option<String> {
        tracing::debug!("🔍 Attempting mDNS-SD discovery (local network)...");

        #[cfg(feature = "mdns")]
        {
            match mdns_discover_service_impl(DISCOVERY_SRV_QUERY).await {
                Some(endpoint) => return Some(endpoint),
                None => {
                    tracing::debug!("🔍 mDNS-SD: no discovery service found on LAN");
                }
            }
        }

        #[cfg(not(feature = "mdns"))]
        tracing::debug!(
            "🔍 mDNS discovery not enabled (requires 'mdns' feature), trying next method"
        );

        None
    }

    /// Try development fallback (localhost).
    ///
    /// This should only be used in development and will log a warning.
    #[expect(
        clippy::unused_self,
        clippy::unnecessary_wraps,
        reason = "consistent discovery chain API; Option for cfg-conditional return"
    )]
    fn try_development_fallback(&self) -> Option<String> {
        tracing::debug!(
            "🔍 Attempting development fallback ({}:{})...",
            crate::constants::LOCALHOST,
            crate::constants::DEFAULT_DISCOVERY_PORT
        );

        // Only in development/test mode
        #[cfg(any(debug_assertions, test))]
        {
            let endpoint = format!(
                "http://{}:{}",
                crate::constants::LOCALHOST,
                crate::constants::DEFAULT_DISCOVERY_PORT
            );
            tracing::debug!("🔍 Development fallback available: {}", endpoint);
            Some(endpoint)
        }

        #[cfg(not(any(debug_assertions, test)))]
        {
            tracing::debug!("🔍 Development fallback disabled in release mode");
            None
        }
    }
}

impl Default for InfantDiscovery {
    fn default() -> Self {
        Self::from_advertised()
    }
}

/// mDNS-SD discovery for finding the discovery service on the local network.
///
/// Queries for `service_query` (e.g. `_discovery._tcp.local.`) via mDNS-SD.
/// Returns the first resolved endpoint, or `None` after a 2-second timeout.
/// Uses `mdns-sd` which manages its own daemon thread — fully tokio-compatible.
#[cfg(feature = "mdns")]
async fn mdns_discover_service_impl(service_query: &str) -> Option<String> {
    use std::time::Duration;

    let daemon = match mdns_sd::ServiceDaemon::new() {
        Ok(d) => d,
        Err(e) => {
            tracing::debug!("mDNS-SD daemon creation failed: {e}");
            return None;
        }
    };

    let receiver = match daemon.browse(service_query) {
        Ok(r) => r,
        Err(e) => {
            tracing::debug!("mDNS-SD browse failed for {service_query}: {e}");
            let _ = daemon.shutdown();
            return None;
        }
    };

    let result = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match receiver.recv_async().await {
                Ok(mdns_sd::ServiceEvent::ServiceResolved(info)) => {
                    if let Some(addr) = info.addresses.iter().next() {
                        let port = info.port;
                        return Some(format!("http://{addr}:{port}"));
                    }
                }
                Ok(mdns_sd::ServiceEvent::SearchStopped(_)) | Err(_) => return None,
                Ok(_) => {}
            }
        }
    })
    .await;

    let _ = daemon.shutdown();
    result.unwrap_or(None)
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
#[path = "infant_discovery_svc_tests.rs"]
mod tests;
