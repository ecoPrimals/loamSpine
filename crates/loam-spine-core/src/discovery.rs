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

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{LoamSpineError, LoamSpineResult};
use crate::traits::{Signer, Verifier};

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

/// A boxed signer that can be stored and shared.
pub type BoxedSigner = Arc<dyn DynSigner>;

/// A boxed verifier that can be stored and shared.
pub type BoxedVerifier = Arc<dyn DynVerifier>;

/// Object-safe version of Signer for dynamic dispatch.
///
/// Uses `bytes::Bytes` for zero-copy data passing across async boundaries.
#[allow(async_fn_in_trait)]
pub trait DynSigner: Send + Sync {
    /// Sign data (takes `Bytes` for zero-copy object safety).
    fn sign_boxed(
        &self,
        data: crate::types::ByteBuffer,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = LoamSpineResult<crate::types::Signature>> + Send + '_>,
    >;

    /// Get the signer's DID.
    fn did(&self) -> &crate::types::Did;
}

/// Blanket implementation for any Signer.
impl<T: Signer> DynSigner for T {
    fn sign_boxed(
        &self,
        data: crate::types::ByteBuffer,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = LoamSpineResult<crate::types::Signature>> + Send + '_>,
    > {
        Box::pin(async move { self.sign(&data).await })
    }

    fn did(&self) -> &crate::types::Did {
        Signer::did(self)
    }
}

/// Object-safe version of Verifier for dynamic dispatch.
///
/// Uses `bytes::Bytes` for zero-copy data passing across async boundaries.
#[allow(async_fn_in_trait)]
pub trait DynVerifier: Send + Sync {
    /// Verify a signature (takes `Bytes` for zero-copy object safety).
    fn verify_boxed(
        &self,
        data: crate::types::ByteBuffer,
        signature: crate::types::Signature,
        signer: crate::types::Did,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = LoamSpineResult<crate::traits::SignatureVerification>>
                + Send
                + '_,
        >,
    >;

    /// Verify an entry signature (takes owned entry for object safety).
    fn verify_entry_boxed(
        &self,
        entry: crate::entry::Entry,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = LoamSpineResult<crate::traits::SignatureVerification>>
                + Send
                + '_,
        >,
    >;
}

/// Blanket implementation for any Verifier.
impl<T: Verifier> DynVerifier for T {
    fn verify_boxed(
        &self,
        data: crate::types::ByteBuffer,
        signature: crate::types::Signature,
        signer: crate::types::Did,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = LoamSpineResult<crate::traits::SignatureVerification>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move { self.verify(&data, &signature, &signer).await })
    }

    fn verify_entry_boxed(
        &self,
        entry: crate::entry::Entry,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = LoamSpineResult<crate::traits::SignatureVerification>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move { self.verify_entry(&entry).await })
    }
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
        // Currently, signing/verification are optional
        // Add required capabilities here as needed
        let _ = inner; // Silence unused warning
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_registry() {
        let registry = CapabilityRegistry::new();

        assert_eq!(
            registry.signer_status().await,
            CapabilityStatus::Unavailable
        );
        assert_eq!(
            registry.verifier_status().await,
            CapabilityStatus::Unavailable
        );
        assert!(registry.get_signer().await.is_none());
        assert!(registry.get_verifier().await.is_none());
    }

    #[tokio::test]
    async fn register_signer() {
        use crate::traits::signing::testing::MockSigner;
        use crate::types::Did;

        let registry = CapabilityRegistry::new();
        let signer = Arc::new(MockSigner::new(Did::new("did:key:test")));

        registry.register_signer(signer).await;

        assert_eq!(registry.signer_status().await, CapabilityStatus::Available);
        assert!(registry.get_signer().await.is_some());
    }

    #[tokio::test]
    async fn register_verifier() {
        use crate::traits::signing::testing::MockVerifier;

        let registry = CapabilityRegistry::new();
        let verifier = Arc::new(MockVerifier::permissive());

        registry.register_verifier(verifier).await;

        assert_eq!(
            registry.verifier_status().await,
            CapabilityStatus::Available
        );
        assert!(registry.get_verifier().await.is_some());
    }

    #[tokio::test]
    async fn register_and_unregister() {
        use crate::traits::signing::testing::MockSigner;
        use crate::types::Did;

        let registry = CapabilityRegistry::new();
        let signer = Arc::new(MockSigner::new(Did::new("did:key:test")));

        registry.register_signer(signer).await;
        assert_eq!(registry.signer_status().await, CapabilityStatus::Available);

        registry.unregister_signer().await;
        assert_eq!(
            registry.signer_status().await,
            CapabilityStatus::Unavailable
        );
    }

    #[tokio::test]
    async fn unregister_verifier() {
        use crate::traits::signing::testing::MockVerifier;

        let registry = CapabilityRegistry::new();
        let verifier = Arc::new(MockVerifier::permissive());

        registry.register_verifier(verifier).await;
        assert_eq!(
            registry.verifier_status().await,
            CapabilityStatus::Available
        );

        registry.unregister_verifier().await;
        assert_eq!(
            registry.verifier_status().await,
            CapabilityStatus::Unavailable
        );
    }

    #[tokio::test]
    async fn require_missing_capability() {
        let registry = CapabilityRegistry::new();

        let result = registry.require_signer().await;
        assert!(result.is_err());

        let result = registry.require_verifier().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn require_registered_capability() {
        use crate::traits::signing::testing::{MockSigner, MockVerifier};
        use crate::types::Did;

        let registry = CapabilityRegistry::new();
        let signer = Arc::new(MockSigner::new(Did::new("did:key:test")));
        let verifier = Arc::new(MockVerifier::permissive());

        registry.register_signer(signer).await;
        registry.register_verifier(verifier).await;

        assert!(registry.require_signer().await.is_ok());
        assert!(registry.require_verifier().await.is_ok());
    }

    #[tokio::test]
    async fn all_statuses() {
        use crate::traits::signing::testing::MockSigner;
        use crate::types::Did;

        let registry = CapabilityRegistry::new();
        let statuses = registry.all_statuses().await;
        assert_eq!(statuses.len(), 2);

        // Register a signer
        let signer = Arc::new(MockSigner::new(Did::new("did:key:test")));
        registry.register_signer(signer).await;

        let statuses = registry.all_statuses().await;
        let signer_status = statuses.iter().find(|(name, _)| *name == "Signer");
        assert_eq!(
            signer_status.map(|(_, s)| s),
            Some(&CapabilityStatus::Available)
        );
    }

    #[tokio::test]
    async fn all_required_available() {
        let registry = CapabilityRegistry::new();

        // Currently all required are optional, so should always be true
        assert!(registry.all_required_available().await);
    }

    #[test]
    fn registry_debug() {
        let registry = CapabilityRegistry::new();
        let debug = format!("{registry:?}");
        assert!(debug.contains("CapabilityRegistry"));
    }

    #[test]
    fn capability_status_equality() {
        assert_eq!(CapabilityStatus::Available, CapabilityStatus::Available);
        assert_eq!(CapabilityStatus::Unavailable, CapabilityStatus::Unavailable);
        assert_eq!(
            CapabilityStatus::Degraded {
                reason: "test".into()
            },
            CapabilityStatus::Degraded {
                reason: "test".into()
            }
        );
        assert_ne!(CapabilityStatus::Available, CapabilityStatus::Unavailable);
    }

    #[tokio::test]
    async fn dyn_signer_sign_boxed() {
        use crate::traits::signing::testing::MockSigner;
        use crate::types::Did;

        let did = Did::new("did:key:test");
        let signer = MockSigner::new(did.clone());

        // Test DynSigner trait through Arc
        let boxed: BoxedSigner = Arc::new(signer);

        let data = crate::types::ByteBuffer::from_static(b"test data");
        let sig = boxed.sign_boxed(data).await;
        assert!(sig.is_ok());

        // Check did() method
        assert_eq!(boxed.did(), &did);
    }

    #[tokio::test]
    async fn dyn_verifier_verify_boxed() {
        use crate::traits::signing::testing::MockVerifier;
        use crate::types::{Did, Signature};

        let verifier = MockVerifier::permissive();
        let boxed: BoxedVerifier = Arc::new(verifier);

        let data = crate::types::ByteBuffer::from_static(b"test data");
        let sig = Signature::from_vec(vec![1, 2, 3]);
        let did = Did::new("did:key:test");

        let result = boxed.verify_boxed(data, sig, did).await;
        assert!(result.is_ok());
        assert!(result.unwrap_or_else(|_| unreachable!()).valid);
    }

    #[tokio::test]
    async fn dyn_verifier_verify_entry_boxed() {
        use crate::entry::{Entry, EntryType};
        use crate::traits::signing::testing::MockVerifier;
        use crate::types::Did;

        let verifier = MockVerifier::permissive();
        let boxed: BoxedVerifier = Arc::new(verifier);

        let entry = Entry::new(
            0,
            None,
            Did::new("did:test"),
            EntryType::SpineSealed { reason: None },
        );

        let result = boxed.verify_entry_boxed(entry).await;
        assert!(result.is_ok());
        assert!(result.unwrap_or_else(|_| unreachable!()).valid);
    }

    #[tokio::test]
    async fn dyn_verifier_strict_fails() {
        use crate::traits::signing::testing::MockVerifier;
        use crate::types::{Did, Signature};

        let verifier = MockVerifier::strict();
        let boxed: BoxedVerifier = Arc::new(verifier);

        let data = crate::types::ByteBuffer::from_static(b"test data");
        let sig = Signature::from_vec(vec![1, 2, 3]);
        let did = Did::new("did:key:test");

        let result = boxed.verify_boxed(data, sig, did).await;
        assert!(result.is_ok());
        assert!(!result.unwrap_or_else(|_| unreachable!()).valid);
    }

    #[test]
    fn capability_status_debug_clone() {
        let status = CapabilityStatus::Degraded {
            reason: "test".into(),
        };
        let debug_str = format!("{status:?}");
        assert!(debug_str.contains("Degraded"));

        #[allow(clippy::redundant_clone)]
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[tokio::test]
    async fn registry_clone() {
        use crate::traits::signing::testing::MockSigner;
        use crate::types::Did;

        let registry = CapabilityRegistry::new();
        let signer = Arc::new(MockSigner::new(Did::new("did:key:test")));
        registry.register_signer(signer).await;

        // Clone the registry
        #[allow(clippy::redundant_clone)]
        let cloned = registry.clone();

        // Both should have the signer
        assert!(registry.get_signer().await.is_some());
        assert!(cloned.get_signer().await.is_some());
    }

    #[tokio::test]
    async fn discover_from_registry_fails_when_not_configured() {
        let registry = CapabilityRegistry::new();

        let result = registry.discover_from_registry().await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("registry") || err.to_string().contains("unavailable"));
    }

    #[tokio::test]
    async fn advertise_to_registry_fails_when_not_configured() {
        let registry = CapabilityRegistry::new();

        let result = registry
            .advertise_to_registry("http://localhost:9001", "http://localhost:8080")
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("registry") || err.to_string().contains("unavailable"));
    }

    #[tokio::test]
    async fn heartbeat_registry_fails_when_not_configured() {
        let registry = CapabilityRegistry::new();

        let result = registry.heartbeat_registry().await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("registry") || err.to_string().contains("unavailable"));
    }

    #[tokio::test]
    async fn with_service_registry_fails_for_unreachable_endpoint() {
        let result = CapabilityRegistry::with_service_registry("http://127.0.0.1:1").await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("unavailable")
                || err.to_string().contains("registry")
                || err.to_string().contains("transport"),
            "Expected connection/transport error: {err}",
        );
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn deprecated_songbird_aliases_work() {
        let registry = CapabilityRegistry::new();
        assert!(registry.discover_from_songbird().await.is_err());
        assert!(registry.heartbeat_songbird().await.is_err());
        assert!(registry
            .advertise_to_songbird("http://localhost:9001", "http://localhost:8080")
            .await
            .is_err());
        assert!(
            CapabilityRegistry::with_service_registry("http://127.0.0.1:1")
                .await
                .is_err()
        );
    }

    #[test]
    fn capability_status_degraded_variant() {
        let degraded = CapabilityStatus::Degraded {
            reason: "heartbeat failed".to_string(),
        };
        assert!(matches!(degraded, CapabilityStatus::Degraded { .. }));
        assert_eq!(
            degraded,
            CapabilityStatus::Degraded {
                reason: "heartbeat failed".to_string(),
            }
        );
    }
}
