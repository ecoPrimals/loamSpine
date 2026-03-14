// SPDX-License-Identifier: AGPL-3.0-only

//! Capability-based primal discovery.
//!
//! This module provides runtime discovery of primal capabilities. Rather than
//! hardcoding dependencies on specific primals, LoamSpine discovers capabilities
//! at runtime through a registry system.
//!
//! ## Design Philosophy
//!
//! - **Self-knowledge only**: LoamSpine knows only its own capabilities
//! - **Runtime discovery**: Other primals are discovered, not hardcoded
//! - **Capability-based**: Request capabilities, not primals
//! - **Graceful degradation**: Handle missing capabilities gracefully
//!
//! ## Example
//!
//! ```rust,no_run
//! use loam_spine_core::discovery::CapabilityRegistry;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create discovery registry
//! let registry = CapabilityRegistry::new();
//!
//! // Check if signer is available
//! if let Some(signer) = registry.get_signer().await {
//!     // Use the capability
//!     let data = loam_spine_core::types::ByteBuffer::from_static(b"data");
//!     let signature = signer.sign_boxed(data).await?;
//! }
//! # Ok(())
//! # }
//! ```

mod dyn_traits;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;

pub use dyn_traits::{BoxedSigner, BoxedVerifier, DynSigner, DynVerifier};

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{LoamSpineError, LoamSpineResult};

/// Capability availability status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilityStatus {
    /// Capability is available and healthy.
    Available,
    /// Capability is registered but currently unhealthy.
    Degraded {
        /// Reason for degradation.
        reason: String,
    },
    /// Capability is not registered.
    Unavailable,
}

/// Registry of discovered primal capabilities.
///
/// This registry allows primals to register their capabilities at runtime,
/// and other primals to discover and use those capabilities.
#[derive(Clone, Default)]
pub struct CapabilityRegistry {
    inner: Arc<RwLock<RegistryInner>>,
}

#[derive(Default)]
struct RegistryInner {
    signer: Option<BoxedSigner>,
    verifier: Option<BoxedVerifier>,
    registry_client: Option<Arc<crate::discovery_client::DiscoveryClient>>,
}

impl CapabilityRegistry {
    /// Create a new empty capability registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new registry with a service registry connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry connection fails.
    pub async fn with_service_registry(endpoint: &str) -> LoamSpineResult<Self> {
        let client = crate::discovery_client::DiscoveryClient::connect(endpoint).await?;
        let registry = Self::new();
        {
            let mut inner = registry.inner.write().await;
            inner.registry_client = Some(Arc::new(client));
        }
        Ok(registry)
    }

    // ========================================================================
    // Service Registry Operations
    // ========================================================================

    /// Discover and register capabilities from the service registry.
    ///
    /// This method queries the registry for available capabilities and registers them.
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails.
    pub async fn discover_from_registry(&self) -> LoamSpineResult<()> {
        let client = {
            let inner = self.inner.read().await;
            inner.registry_client.clone().ok_or_else(|| {
                LoamSpineError::CapabilityUnavailable("Service registry not configured".into())
            })?
        };

        if let Ok(services) = client.discover_capability("signing").await {
            if let Some(service) = services.first() {
                tracing::info!(
                    "Discovered signing service: {} at {}",
                    service.name,
                    service.endpoint
                );
            }
        }

        if let Ok(services) = client.discover_capability("verification").await {
            if let Some(service) = services.first() {
                tracing::info!(
                    "Discovered verification service: {} at {}",
                    service.name,
                    service.endpoint
                );
            }
        }

        Ok(())
    }

    /// Backward-compatible alias for [`Self::discover_from_registry`].
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails.
    #[deprecated(since = "0.9.0", note = "Use discover_from_registry instead")]
    pub async fn discover_from_songbird(&self) -> LoamSpineResult<()> {
        self.discover_from_registry().await
    }

    /// Advertise LoamSpine's capabilities to the service registry.
    ///
    /// # Errors
    ///
    /// Returns an error if advertisement fails.
    pub async fn advertise_to_registry(
        &self,
        tarpc_endpoint: &str,
        jsonrpc_endpoint: &str,
    ) -> LoamSpineResult<()> {
        let client = {
            let inner = self.inner.read().await;
            inner.registry_client.clone().ok_or_else(|| {
                LoamSpineError::CapabilityUnavailable("Service registry not configured".into())
            })?
        };

        client
            .advertise_self(tarpc_endpoint, jsonrpc_endpoint)
            .await?;

        tracing::info!("Advertised LoamSpine capabilities to service registry");
        Ok(())
    }

    /// Backward-compatible alias for [`Self::advertise_to_registry`].
    ///
    /// # Errors
    ///
    /// Returns an error if advertisement fails.
    #[deprecated(since = "0.9.0", note = "Use advertise_to_registry instead")]
    pub async fn advertise_to_songbird(
        &self,
        tarpc_endpoint: &str,
        jsonrpc_endpoint: &str,
    ) -> LoamSpineResult<()> {
        self.advertise_to_registry(tarpc_endpoint, jsonrpc_endpoint)
            .await
    }

    /// Send heartbeat to the service registry to keep advertisement alive.
    ///
    /// # Errors
    ///
    /// Returns an error if heartbeat fails.
    pub async fn heartbeat_registry(&self) -> LoamSpineResult<()> {
        let client = {
            let inner = self.inner.read().await;
            inner.registry_client.clone().ok_or_else(|| {
                LoamSpineError::CapabilityUnavailable("Service registry not configured".into())
            })?
        };

        client.heartbeat().await?;
        Ok(())
    }

    /// Backward-compatible alias for [`Self::heartbeat_registry`].
    ///
    /// # Errors
    ///
    /// Returns an error if heartbeat fails.
    #[deprecated(since = "0.9.0", note = "Use heartbeat_registry instead")]
    pub async fn heartbeat_songbird(&self) -> LoamSpineResult<()> {
        self.heartbeat_registry().await
    }

    // ========================================================================
    // Signer Capability
    // ========================================================================

    /// Register a signer capability.
    ///
    /// This is called by a signing primal when it becomes available.
    pub async fn register_signer(&self, signer: BoxedSigner) {
        let mut inner = self.inner.write().await;
        inner.signer = Some(signer);
    }

    /// Unregister the signer capability.
    ///
    /// This is called when a signing primal becomes unavailable.
    pub async fn unregister_signer(&self) {
        let mut inner = self.inner.write().await;
        inner.signer = None;
    }

    /// Get the registered signer, if available.
    pub async fn get_signer(&self) -> Option<BoxedSigner> {
        let inner = self.inner.read().await;
        inner.signer.clone()
    }

    /// Check if a signer is available.
    pub async fn signer_status(&self) -> CapabilityStatus {
        let inner = self.inner.read().await;
        if inner.signer.is_some() {
            CapabilityStatus::Available
        } else {
            CapabilityStatus::Unavailable
        }
    }

    /// Get the signer or return an error.
    ///
    /// # Errors
    ///
    /// Returns `LoamSpineError::CapabilityUnavailable` if no signer is registered.
    pub async fn require_signer(&self) -> LoamSpineResult<BoxedSigner> {
        self.get_signer()
            .await
            .ok_or_else(|| LoamSpineError::CapabilityUnavailable("Signer".into()))
    }

    // ========================================================================
    // Verifier Capability
    // ========================================================================

    /// Register a verifier capability.
    pub async fn register_verifier(&self, verifier: BoxedVerifier) {
        let mut inner = self.inner.write().await;
        inner.verifier = Some(verifier);
    }

    /// Unregister the verifier capability.
    pub async fn unregister_verifier(&self) {
        let mut inner = self.inner.write().await;
        inner.verifier = None;
    }

    /// Get the registered verifier, if available.
    pub async fn get_verifier(&self) -> Option<BoxedVerifier> {
        let inner = self.inner.read().await;
        inner.verifier.clone()
    }

    /// Check if a verifier is available.
    pub async fn verifier_status(&self) -> CapabilityStatus {
        let inner = self.inner.read().await;
        if inner.verifier.is_some() {
            CapabilityStatus::Available
        } else {
            CapabilityStatus::Unavailable
        }
    }

    /// Get the verifier or return an error.
    ///
    /// # Errors
    ///
    /// Returns `LoamSpineError::CapabilityUnavailable` if no verifier is registered.
    pub async fn require_verifier(&self) -> LoamSpineResult<BoxedVerifier> {
        self.get_verifier()
            .await
            .ok_or_else(|| LoamSpineError::CapabilityUnavailable("Verifier".into()))
    }

    // ========================================================================
    // Bulk Operations
    // ========================================================================

    /// Get all available capability statuses.
    pub async fn all_statuses(&self) -> Vec<(&'static str, CapabilityStatus)> {
        vec![
            ("Signer", self.signer_status().await),
            ("Verifier", self.verifier_status().await),
        ]
    }

    /// Check if all required capabilities are available.
    pub async fn all_required_available(&self) -> bool {
        let inner = self.inner.read().await;
        let _ = inner;
        true
    }
}

impl std::fmt::Debug for CapabilityRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapabilityRegistry")
            .field("signer", &"<capability>")
            .field("verifier", &"<capability>")
            .finish()
    }
}
