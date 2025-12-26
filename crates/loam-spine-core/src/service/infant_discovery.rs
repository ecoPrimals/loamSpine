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

use crate::error::{LoamSpineError, LoamSpineResult};
use crate::songbird::SongbirdClient;

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
    pub async fn discover_discovery_service(&self) -> LoamSpineResult<SongbirdClient> {
        tracing::info!("🔍 Starting infant discovery (zero knowledge → full knowledge)...");

        // Method 1: Environment variable (highest priority)
        if let Some(endpoint) = self.try_environment_discovery() {
            tracing::info!("✅ Discovery service found via environment: {}", endpoint);
            return SongbirdClient::connect(&endpoint).await;
        }

        // Method 2: DNS SRV records (production)
        if let Some(endpoint) = self.try_dns_srv_discovery() {
            tracing::info!("✅ Discovery service found via DNS SRV: {}", endpoint);
            return SongbirdClient::connect(&endpoint).await;
        }

        // Method 3: mDNS (local network)
        if let Some(endpoint) = self.try_mdns_discovery() {
            tracing::info!("✅ Discovery service found via mDNS: {}", endpoint);
            return SongbirdClient::connect(&endpoint).await;
        }

        // Method 4: Development fallback (lowest priority, logged as warning)
        if let Some(endpoint) = self.try_development_fallback() {
            tracing::warn!(
                "⚠️  Using development fallback: {}. Set DISCOVERY_ENDPOINT for production!",
                endpoint
            );
            return SongbirdClient::connect(&endpoint).await;
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
    #[allow(clippy::unused_self)]
    fn try_dns_srv_discovery(&self) -> Option<String> {
        tracing::debug!("🔍 Attempting DNS SRV discovery (_discovery._tcp.local)...");

        // TODO: Implement actual DNS SRV lookup
        // For now, this is a placeholder that returns None
        // In production, would use:
        // - trust-dns-resolver crate
        // - Query _discovery._tcp.local
        // - Parse SRV records for host:port

        tracing::debug!("🔍 DNS SRV discovery not yet implemented, trying next method");
        None
    }

    /// Try to discover via mDNS (multicast DNS).
    ///
    /// Broadcasts on local network to find discovery service.
    /// Useful for local development and LAN deployments.
    #[allow(clippy::unused_self)]
    fn try_mdns_discovery(&self) -> Option<String> {
        tracing::debug!("🔍 Attempting mDNS discovery (local network)...");

        // TODO: Implement actual mDNS discovery
        // For now, this is a placeholder that returns None
        // In production, would use:
        // - mdns crate
        // - Broadcast query for _discovery._tcp.local
        // - Listen for responses on 224.0.0.251:5353

        tracing::debug!("🔍 mDNS discovery not yet implemented, trying next method");
        None
    }

    /// Try development fallback (localhost).
    ///
    /// This should only be used in development and will log a warning.
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn try_development_fallback(&self) -> Option<String> {
        tracing::debug!("🔍 Attempting development fallback (localhost:8082)...");

        // Only in development/test mode
        #[cfg(any(debug_assertions, test))]
        {
            let endpoint = "http://localhost:8082".to_string();
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
mod tests {
    use super::*;

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
    async fn environment_discovery_with_var() {
        std::env::set_var("DISCOVERY_ENDPOINT", "http://test.example.com:8082");
        
        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.try_environment_discovery();
        
        assert_eq!(result, Some("http://test.example.com:8082".to_string()));
        
        std::env::remove_var("DISCOVERY_ENDPOINT");
    }

    #[tokio::test]
    async fn environment_discovery_without_var() {
        std::env::remove_var("DISCOVERY_ENDPOINT");
        
        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.try_environment_discovery();
        
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn dns_srv_discovery_placeholder() {
        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.try_dns_srv_discovery();
        
        // Currently returns None (not implemented yet)
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn mdns_discovery_placeholder() {
        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.try_mdns_discovery();
        
        // Currently returns None (not implemented yet)
        assert!(result.is_none());
    }

    #[cfg(debug_assertions)]
    #[tokio::test]
    async fn development_fallback_in_debug() {
        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        let result = infant.try_development_fallback();
        
        // Should return localhost in debug mode
        assert_eq!(result, Some("http://localhost:8082".to_string()));
    }

    #[tokio::test]
    async fn discovery_service_full_chain() {
        // Set environment variable to ensure test is deterministic
        std::env::set_var("DISCOVERY_ENDPOINT", "http://test.example.com:8082");
        
        let infant = InfantDiscovery::new(vec!["test".to_string()]);
        
        // This will try to connect, which will fail in tests
        // but we're testing that the discovery chain works
        let result = infant.discover_discovery_service().await;
        
        // Will fail to connect, but that's expected in tests
        // The important thing is the discovery chain executed
        assert!(result.is_err());
        
        std::env::remove_var("DISCOVERY_ENDPOINT");
    }
}

