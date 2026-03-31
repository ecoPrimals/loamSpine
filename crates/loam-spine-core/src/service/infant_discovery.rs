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
        tracing::info!("🔍 Starting infant discovery (zero knowledge → full knowledge)...");

        // Method 1: Environment variable (highest priority)
        if let Some(endpoint) = self.try_environment_discovery() {
            tracing::info!("✅ Discovery service found via environment: {}", endpoint);
            return DiscoveryClient::connect(&endpoint).await;
        }

        // Method 2: DNS SRV records (production)
        if let Some(endpoint) = self.try_dns_srv_discovery().await {
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
    #[expect(
        clippy::unused_self,
        reason = "method on self for consistent discovery chain API"
    )]
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
    #[allow(
        clippy::unused_async,
        reason = "async required for dns-srv feature builds; lint fires only in no-feature builds"
    )]
    async fn try_dns_srv_discovery(&self) -> Option<String> {
        tracing::debug!("🔍 Attempting DNS SRV discovery (_discovery._tcp.local)...");

        #[cfg(test)]
        {
            tracing::debug!("🔍 DNS SRV discovery disabled in test mode, trying next method");
            None
        }

        #[cfg(all(not(test), feature = "dns-srv"))]
        {
            use hickory_resolver::TokioAsyncResolver;
            use hickory_resolver::config::{ResolverConfig, ResolverOpts};

            let resolver =
                TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

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
                        let endpoint = format!("http://{}:{}", target.trim_end_matches('.'), port);
                        tracing::info!("✅ DNS SRV discovery successful: {}", endpoint);
                        Some(endpoint)
                    },
                ),
                Err(e) => {
                    tracing::debug!("🔍 DNS SRV lookup failed: {}, trying next method", e);
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

    /// Try to discover via mDNS (multicast DNS).
    ///
    /// Broadcasts on local network to find discovery service.
    /// Useful for local development and LAN deployments.
    ///
    /// Note: This is currently experimental. The mDNS crate's API has some
    /// complexity with async streams, so we use a simple thread-based polling
    /// approach for now. Future versions may improve this implementation.
    /// For production use, prefer DNS SRV discovery or environment variables.
    #[expect(
        clippy::unused_self,
        reason = "method on self for consistent discovery chain API"
    )]
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

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
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
        assert!(
            infant.capabilities().contains(
                &crate::capabilities::identifiers::loamspine::PERMANENT_LEDGER.to_string()
            )
        );
    }

    #[test]
    fn from_advertised_matches_canonical_set() {
        let infant = InfantDiscovery::from_advertised();
        let caps = infant.capabilities();
        for &expected in crate::capabilities::identifiers::loamspine::ADVERTISED {
            assert!(
                caps.contains(&expected.to_string()),
                "missing advertised capability: {expected}"
            );
        }
    }

    #[test]
    #[serial]
    fn environment_discovery_with_var() {
        temp_env::with_var(
            "DISCOVERY_ENDPOINT",
            Some("http://test.example.com:8082"),
            || {
                let infant = InfantDiscovery::new(vec!["test".to_string()]);
                let result = infant.try_environment_discovery();
                assert_eq!(result, Some("http://test.example.com:8082".to_string()));
            },
        );
    }

    #[test]
    #[serial]
    fn environment_discovery_without_var() {
        temp_env::with_var("DISCOVERY_ENDPOINT", None::<&str>, || {
            let infant = InfantDiscovery::new(vec!["test".to_string()]);
            let result = infant.try_environment_discovery();
            assert!(result.is_none());
        });
    }

    #[tokio::test]
    async fn dns_srv_discovery_no_records() {
        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.try_dns_srv_discovery().await;

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

    #[test]
    #[serial]
    fn discovery_service_full_chain() {
        temp_env::with_var(
            "DISCOVERY_ENDPOINT",
            Some("http://test.example.com:8082"),
            || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let infant = InfantDiscovery::new(vec!["test".to_string()]);
                    let result = infant.discover_discovery_service().await;
                    assert!(result.is_err());
                });
            },
        );
    }

    #[test]
    #[serial]
    fn discover_discovery_service_unreachable_endpoint_returns_error() {
        temp_env::with_var("DISCOVERY_ENDPOINT", Some("http://127.0.0.1:1"), || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
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
            });
        });
    }

    #[test]
    #[serial]
    fn discover_discovery_service_development_fallback_connection_fails() {
        temp_env::with_var("DISCOVERY_ENDPOINT", None::<&str>, || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let infant = InfantDiscovery::new(vec!["test".to_string()]);
                let result = infant.discover_discovery_service().await;
                assert!(result.is_err());
            });
        });
    }

    #[test]
    #[serial]
    fn discover_discovery_service_empty_env_skipped() {
        temp_env::with_var("DISCOVERY_ENDPOINT", Some(""), || {
            let infant = InfantDiscovery::new(vec!["test".to_string()]);
            let result = infant.try_environment_discovery();
            assert!(result.is_none());
        });
    }

    #[tokio::test]
    async fn development_fallback_returns_endpoint_in_test_mode() {
        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.try_development_fallback();

        // In test mode, development fallback is enabled and returns localhost:8082
        let expected = format!(
            "http://{}:{}",
            crate::constants::LOCALHOST,
            crate::constants::DEFAULT_DISCOVERY_PORT
        );
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn default_includes_all_expected_capabilities() {
        use crate::capabilities::identifiers::loamspine;
        let infant = InfantDiscovery::default();
        let caps = infant.capabilities();

        assert!(caps.contains(&loamspine::PERMANENT_LEDGER.to_string()));
        assert!(caps.contains(&loamspine::WAYPOINT_ANCHORING.to_string()));
        assert!(caps.contains(&loamspine::CERTIFICATE_AUTHORITY.to_string()));
    }

    #[test]
    #[serial]
    fn discover_discovery_service_env_takes_priority_over_fallback() {
        // When DISCOVERY_ENDPOINT is set, it should be used (even if unreachable)
        temp_env::with_var(
            "DISCOVERY_ENDPOINT",
            Some("http://env-priority-test.example:9999"),
            || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let infant = InfantDiscovery::new(vec!["test".to_string()]);
                    let result = infant.discover_discovery_service().await;
                    // Should fail to connect (unreachable) but we used env, not fallback
                    assert!(result.is_err());
                    let err_str = result.unwrap_err().to_string();
                    assert!(
                        err_str.contains("env-priority-test")
                            || err_str.contains("9999")
                            || err_str.contains("unavailable"),
                        "Expected env endpoint in error: {err_str}",
                    );
                });
            },
        );
    }

    #[test]
    #[serial]
    fn discover_discovery_service_fallback_chain_when_env_unset() {
        // No env var -> DNS (None in test) -> mDNS (None) -> dev fallback -> connect fails
        temp_env::with_var("DISCOVERY_ENDPOINT", None::<&str>, || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let infant = InfantDiscovery::new(vec!["test".to_string()]);
                let result = infant.discover_discovery_service().await;
                assert!(result.is_err());
                let err = result.unwrap_err();
                let err_str = err.to_string();
                // Connection to localhost:8082 should fail (no server)
                assert!(
                    err_str.contains("unavailable")
                        || err_str.contains("registry")
                        || err_str.contains("localhost")
                        || err_str.contains("8082")
                        || err_str.contains("connection"),
                    "Expected connection error from fallback: {err_str}",
                );
            });
        });
    }

    #[test]
    fn capabilities_returns_owned_reference() {
        let infant = InfantDiscovery::new(vec!["cap-a".to_string(), "cap-b".to_string()]);
        let caps = infant.capabilities();

        assert_eq!(caps.len(), 2);
        assert_eq!(caps[0], "cap-a");
        assert_eq!(caps[1], "cap-b");
    }

    #[test]
    fn infant_discovery_new_with_empty_capabilities() {
        let infant = InfantDiscovery::new(vec![]);
        assert!(infant.capabilities().is_empty());
    }

    #[test]
    fn infant_discovery_new_with_many_capabilities() {
        let caps: Vec<String> = (0..10).map(|i| format!("cap-{i}")).collect();
        let infant = InfantDiscovery::new(caps);
        assert_eq!(infant.capabilities().len(), 10);
        for (i, c) in infant.capabilities().iter().enumerate() {
            assert_eq!(c, &format!("cap-{i}"));
        }
    }

    #[test]
    #[serial]
    fn environment_discovery_empty_string_skipped() {
        temp_env::with_var("DISCOVERY_ENDPOINT", Some(""), || {
            let infant = InfantDiscovery::new(vec!["test".to_string()]);
            let result = infant.try_environment_discovery();
            assert!(result.is_none());
        });
    }
}
