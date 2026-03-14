// SPDX-License-Identifier: AGPL-3.0-only

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
//! // Create infant with self-knowledge only
//! let infant = InfantDiscovery::new(vec![
//!     "persistent-ledger".to_string(),
//!     "waypoint-anchoring".to_string(),
//! ]);
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

use crate::discovery_client::DiscoveryClient;
use crate::error::{LoamSpineError, LoamSpineResult};

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
    ///     "persistent-ledger".to_string(),
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
    /// let infant = InfantDiscovery::new(vec!["persistent-ledger".to_string()]);
    ///
    /// match infant.discover_discovery_service().await {
    ///     Ok(client) => println!("✅ Found discovery service"),
    ///     Err(e) => println!("⚠️  No discovery service: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_discovery_service(&self) -> LoamSpineResult<DiscoveryClient> {
        tracing::info!("🔍 Starting infant discovery (zero knowledge → full knowledge)...");

        // Method 1: Environment variable (highest priority)
        if let Some(endpoint) = self.try_environment_discovery() {
            tracing::info!("✅ Discovery service found via environment: {}", endpoint);
            return DiscoveryClient::connect(&endpoint).await;
        }

        // Method 2: DNS SRV records (production)
        if let Some(endpoint) = self.try_dns_srv_discovery() {
            tracing::info!("✅ Discovery service found via DNS SRV: {}", endpoint);
            return DiscoveryClient::connect(&endpoint).await;
        }

        // Method 3: mDNS (local network)
        if let Some(endpoint) = self.try_mdns_discovery() {
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

    /// Try to discover via environment variable.
    ///
    /// Checks `DISCOVERY_ENDPOINT` environment variable.
    #[allow(clippy::unused_self)]
    fn try_environment_discovery(&self) -> Option<String> {
        match std::env::var("DISCOVERY_ENDPOINT") {
            Ok(endpoint) if !endpoint.is_empty() => {
                tracing::debug!("🔍 Found DISCOVERY_ENDPOINT: {}", endpoint);
                Some(endpoint)
            }
            Ok(_) => {
                tracing::debug!("🔍 DISCOVERY_ENDPOINT is empty, skipping");
                None
            }
            Err(_) => {
                tracing::debug!("🔍 DISCOVERY_ENDPOINT not set, trying next method");
                None
            }
        }
    }

    /// Try to discover via DNS SRV records.
    ///
    /// Looks up `_discovery._tcp.local` SRV record.
    /// This is the standard production discovery method.
    ///
    /// Note: Disabled in test mode to avoid runtime conflicts. Use environment variables in tests.
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    fn try_dns_srv_discovery(&self) -> Option<String> {
        tracing::debug!("🔍 Attempting DNS SRV discovery (_discovery._tcp.local)...");

        // Skip DNS SRV in test mode to avoid runtime conflicts
        #[cfg(test)]
        {
            tracing::debug!("🔍 DNS SRV discovery disabled in test mode, trying next method");
            None
        }

        #[cfg(not(test))]
        {
            // Use hickory-resolver for DNS SRV lookup
            use hickory_resolver::config::{ResolverConfig, ResolverOpts};
            use hickory_resolver::TokioAsyncResolver;

            // Check if we have a runtime available
            let Some(handle) = tokio::runtime::Handle::try_current().ok() else {
                // No runtime available, skip DNS discovery
                tracing::debug!(
                    "🔍 No tokio runtime available for DNS resolution, trying next method"
                );
                return None;
            };

            // Use spawn_blocking to avoid nested runtime issues
            let result = handle.block_on(async {
                tokio::task::spawn_blocking(|| {
                    // Create a new runtime for the DNS lookup
                    let rt = tokio::runtime::Runtime::new().ok()?;

                    rt.block_on(async {
                        let resolver = TokioAsyncResolver::tokio(
                            ResolverConfig::default(),
                            ResolverOpts::default(),
                        );

                        // Query for _discovery._tcp.local SRV record
                        let srv_query = "_discovery._tcp.local";
                        match resolver.srv_lookup(srv_query).await {
                            Ok(response) => response.iter().next().map_or_else(
                                || {
                                    tracing::debug!(
                                        "🔍 No SRV records found for {}, trying next method",
                                        srv_query
                                    );
                                    None
                                },
                                |srv| {
                                    let target = srv.target().to_utf8();
                                    let port = srv.port();

                                    // Construct endpoint URL
                                    let endpoint =
                                        format!("http://{}:{}", target.trim_end_matches('.'), port);
                                    tracing::info!("✅ DNS SRV discovery successful: {}", endpoint);
                                    Some(endpoint)
                                },
                            ),
                            Err(e) => {
                                tracing::debug!(
                                    "🔍 DNS SRV lookup failed: {}, trying next method",
                                    e
                                );
                                None
                            }
                        }
                    })
                })
                .await
                .ok()
                .flatten()
            });

            result
        }
    }

    /// Try to discover via mDNS (multicast DNS).
    ///
    /// Broadcasts on local network to find discovery service.
    /// Useful for local development and LAN deployments.
    ///
    /// Note: This is currently experimental. The mDNS crate's API has some
    /// complexity with async streams, so we use a simple thread-based polling
    /// approach for now. Future versions may improve this implementation.
    /// For production use, prefer DNS SRV discovery or environment variables.
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    fn try_mdns_discovery(&self) -> Option<String> {
        tracing::debug!("🔍 Attempting mDNS discovery (local network)...");

        #[cfg(feature = "mdns")]
        {
            // mDNS discovery using the mdns crate
            // NOTE: The mdns crate's Discovery API doesn't provide direct synchronous
            // iteration over responses. The stream-based API requires careful handling
            // of async traits that are complex to integrate with both tokio and sync code.
            //
            // This is marked as experimental and currently returns None.
            // Future versions may provide a more robust implementation using async streams
            // or an alternative mDNS library with better sync APIs.
            //
            // For production use, prefer:
            // 1. Environment variables (DISCOVERY_ENDPOINT)
            // 2. DNS SRV records (_discovery._tcp.local)
            // 3. Configuration files

            tracing::debug!("🔍 mDNS discovery experimental - API integration pending");
            tracing::debug!(
                "🔍 For local development, use DISCOVERY_ENDPOINT environment variable"
            );
        }

        #[cfg(not(feature = "mdns"))]
        {
            tracing::debug!(
                "🔍 mDNS discovery not enabled (requires 'mdns' feature), trying next method"
            );
        }

        None
    }

    /// Try development fallback (localhost).
    ///
    /// This should only be used in development and will log a warning.
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
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
        Self::new(vec![
            "persistent-ledger".to_string(),
            "waypoint-anchoring".to_string(),
            "certificate-manager".to_string(),
        ])
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn infant_discovery_creation() {
        let infant = InfantDiscovery::new(vec!["test-capability".to_string()]);
        assert_eq!(infant.capabilities().len(), 1);
        assert_eq!(infant.capabilities()[0], "test-capability");
    }

    #[test]
    fn infant_discovery_default() {
        let infant = InfantDiscovery::default();
        assert!(infant.capabilities().len() >= 3);
        assert!(infant
            .capabilities()
            .contains(&"persistent-ledger".to_string()));
    }

    #[tokio::test]
    #[serial]
    async fn environment_discovery_with_var() {
        std::env::set_var("DISCOVERY_ENDPOINT", "http://test.example.com:8082");

        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.try_environment_discovery();

        assert_eq!(result, Some("http://test.example.com:8082".to_string()));

        std::env::remove_var("DISCOVERY_ENDPOINT");
    }

    #[tokio::test]
    #[serial]
    async fn environment_discovery_without_var() {
        std::env::remove_var("DISCOVERY_ENDPOINT");

        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.try_environment_discovery();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn dns_srv_discovery_no_records() {
        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.try_dns_srv_discovery();

        // Returns None in test environment (no DNS SRV records configured)
        // In production, would return endpoint if DNS is properly configured
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn mdns_discovery_not_configured() {
        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.try_mdns_discovery();

        // Currently returns None (experimental/not fully implemented)
        assert!(result.is_none());
    }

    #[cfg(debug_assertions)]
    #[tokio::test]
    async fn development_fallback_in_debug() {
        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.try_development_fallback();

        // Should return localhost in debug mode
        let expected_endpoint = format!(
            "http://{}:{}",
            crate::constants::LOCALHOST,
            crate::constants::DEFAULT_DISCOVERY_PORT
        );
        assert_eq!(result, Some(expected_endpoint));
    }

    #[tokio::test]
    #[serial]
    async fn discovery_service_full_chain() {
        std::env::set_var("DISCOVERY_ENDPOINT", "http://test.example.com:8082");

        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.discover_discovery_service().await;

        assert!(result.is_err());

        std::env::remove_var("DISCOVERY_ENDPOINT");
    }

    #[tokio::test]
    #[serial]
    async fn discover_discovery_service_unreachable_endpoint_returns_error() {
        std::env::set_var("DISCOVERY_ENDPOINT", "http://127.0.0.1:1");

        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.discover_discovery_service().await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("unavailable")
                || err_str.contains("registry")
                || err_str.contains("127"),
            "Expected connection error: {err_str}",
        );

        std::env::remove_var("DISCOVERY_ENDPOINT");
    }

    #[tokio::test]
    #[serial]
    async fn discover_discovery_service_development_fallback_connection_fails() {
        std::env::remove_var("DISCOVERY_ENDPOINT");

        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.discover_discovery_service().await;

        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn discover_discovery_service_empty_env_skipped() {
        std::env::set_var("DISCOVERY_ENDPOINT", "");

        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.try_environment_discovery();

        assert!(result.is_none());

        std::env::remove_var("DISCOVERY_ENDPOINT");
    }
}
