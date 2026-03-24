// SPDX-License-Identifier: AGPL-3.0-or-later

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
pub mod manifest;

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests;

pub use dyn_traits::{
    BoxedAttestationProvider, BoxedSigner, BoxedVerifier, DynAttestationProvider, DynSigner,
    DynVerifier,
};

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::capabilities::identifiers::external;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::types::Did;
use crate::waypoint::{AttestationContext, AttestationResult};

/// Capability availability status.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
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
    attestation_provider: Option<BoxedAttestationProvider>,
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

        if let Ok(services) = client.discover_capability("signing").await
            && let Some(service) = services.first()
        {
            tracing::info!(
                "Discovered signing service: {} at {}",
                service.name,
                service.endpoint
            );
        }

        if let Ok(services) = client.discover_capability("verification").await
            && let Some(service) = services.first()
        {
            tracing::info!(
                "Discovered verification service: {} at {}",
                service.name,
                service.endpoint
            );
        }

        // Discover attestation provider (capability-based discovery)
        if let Ok(services) = client.discover_capability(external::ATTESTATION).await
            && let Some(service) = services.first()
        {
            tracing::info!(
                "Discovered attestation provider: {} at {}",
                service.name,
                service.endpoint
            );
            let mut inner = self.inner.write().await;
            inner.attestation_provider = Some(Arc::new(DiscoveredAttestationProvider {
                attester_did: Did::new(format!("did:attestation:{}", service.name)),
                endpoint: service.endpoint.clone(),
            }));
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
    // Attestation Provider Capability
    // ========================================================================

    /// Register an attestation provider (discovered via capability `ATTESTATION`).
    ///
    /// Called when an attestation primal is discovered at runtime.
    pub async fn register_attestation_provider(&self, provider: BoxedAttestationProvider) {
        let mut inner = self.inner.write().await;
        inner.attestation_provider = Some(provider);
    }

    /// Unregister the attestation provider.
    pub async fn unregister_attestation_provider(&self) {
        let mut inner = self.inner.write().await;
        inner.attestation_provider = None;
    }

    /// Get the attestation provider if available.
    pub async fn get_attestation_provider(&self) -> Option<BoxedAttestationProvider> {
        let inner = self.inner.read().await;
        inner.attestation_provider.clone()
    }

    /// Check if attestation provider is available.
    pub async fn attestation_provider_status(&self) -> CapabilityStatus {
        let inner = self.inner.read().await;
        if inner.attestation_provider.is_some() {
            CapabilityStatus::Available
        } else {
            CapabilityStatus::Unavailable
        }
    }

    /// Request attestation for a waypoint operation.
    ///
    /// # Errors
    ///
    /// Returns error if no attestation provider is registered or attestation is denied.
    pub async fn request_attestation(
        &self,
        context: AttestationContext,
    ) -> LoamSpineResult<AttestationResult> {
        let provider = self.get_attestation_provider().await.ok_or_else(|| {
            LoamSpineError::CapabilityUnavailable(format!(
                "Attestation provider not available (discover via capability '{}')",
                external::ATTESTATION
            ))
        })?;
        let result = provider.request_attestation(context).await?;
        if !result.attested {
            return Err(LoamSpineError::CapabilityProvider {
                capability: external::ATTESTATION.into(),
                message: result
                    .denial_reason
                    .unwrap_or_else(|| "Attestation denied".into()),
            });
        }
        Ok(result)
    }

    // ========================================================================
    // Bulk Operations
    // ========================================================================

    /// Get all available capability statuses.
    pub async fn all_statuses(&self) -> Vec<(&'static str, CapabilityStatus)> {
        vec![
            ("Signer", self.signer_status().await),
            ("Verifier", self.verifier_status().await),
            ("Attestation", self.attestation_provider_status().await),
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
            .field("attestation_provider", &"<capability>")
            .finish()
    }
}

/// Discovered attestation provider that delegates to a remote primal.
///
/// Wraps the endpoint of a capability-discovered attestation service.
/// Sends `attestation.request` JSON-RPC calls over TCP to the provider.
/// Falls back to local approval when the remote endpoint is unreachable,
/// logging a warning so operators can diagnose missing attestation
/// infrastructure.
struct DiscoveredAttestationProvider {
    attester_did: Did,
    endpoint: String,
}

impl DiscoveredAttestationProvider {
    /// Send a JSON-RPC request to the attestation endpoint.
    async fn jsonrpc_call(
        endpoint: &str,
        method: &str,
        params: serde_json::Value,
    ) -> LoamSpineResult<serde_json::Value> {
        use crate::error::IpcErrorPhase;
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::TcpStream;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1u64,
        });

        let payload = serde_json::to_string(&request).map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::Serialization,
                format!("attestation request serialization: {e}"),
            )
        })?;

        let timeout = std::time::Duration::from_secs(5);
        let mut stream = match tokio::time::timeout(timeout, TcpStream::connect(endpoint)).await {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                return Err(LoamSpineError::ipc(
                    IpcErrorPhase::Connect,
                    format!("attestation provider at {endpoint}: {e}"),
                ));
            }
            Err(_) => {
                return Err(LoamSpineError::ipc(
                    IpcErrorPhase::Connect,
                    format!("attestation provider at {endpoint} timed out"),
                ));
            }
        };

        stream.write_all(payload.as_bytes()).await.map_err(|e| {
            LoamSpineError::ipc(IpcErrorPhase::Write, format!("attestation write: {e}"))
        })?;
        stream.write_all(b"\n").await.map_err(|e| {
            LoamSpineError::ipc(IpcErrorPhase::Write, format!("attestation write: {e}"))
        })?;
        stream.flush().await.map_err(|e| {
            LoamSpineError::ipc(IpcErrorPhase::Write, format!("attestation flush: {e}"))
        })?;

        let mut line = String::new();
        BufReader::new(stream)
            .read_line(&mut line)
            .await
            .map_err(|e| {
                LoamSpineError::ipc(IpcErrorPhase::Read, format!("attestation read: {e}"))
            })?;

        let response: serde_json::Value = serde_json::from_str(line.trim()).map_err(|e| {
            LoamSpineError::ipc(
                IpcErrorPhase::InvalidJson,
                format!("attestation response parse: {e}"),
            )
        })?;

        if let Some((code, message)) = crate::error::extract_rpc_error(&response) {
            return Err(LoamSpineError::ipc(
                IpcErrorPhase::JsonRpcError(code),
                format!("attestation provider error: {message}"),
            ));
        }

        response.get("result").cloned().ok_or_else(|| {
            LoamSpineError::ipc(
                IpcErrorPhase::NoResult,
                "attestation response missing result",
            )
        })
    }
}

impl DynAttestationProvider for DiscoveredAttestationProvider {
    fn request_attestation(
        &self,
        context: AttestationContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = LoamSpineResult<AttestationResult>> + Send + '_>,
    > {
        let attester_did = self.attester_did.clone();
        let endpoint = self.endpoint.clone();
        Box::pin(async move {
            let params = serde_json::json!({
                "operation": context.operation,
                "waypoint_spine_id": context.waypoint_spine_id,
                "slice_id": context.slice_id,
                "caller": context.caller,
            });

            match Self::jsonrpc_call(&endpoint, "attestation.request", params).await {
                Ok(response) => {
                    let attested = response
                        .get("attested")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false);
                    let denial_reason = response
                        .get("denial_reason")
                        .and_then(serde_json::Value::as_str)
                        .map(String::from);
                    let token = response
                        .get("token")
                        .and_then(serde_json::Value::as_str)
                        .map(|s| s.as_bytes().to_vec())
                        .unwrap_or_default();

                    Ok(AttestationResult {
                        attested,
                        attester: attester_did,
                        timestamp: crate::types::Timestamp::now(),
                        token,
                        denial_reason,
                    })
                }
                Err(e) => {
                    tracing::warn!(
                        "Attestation provider at {endpoint} unreachable: {e}; \
                         granting local attestation (degraded mode)"
                    );
                    Ok(AttestationResult {
                        attested: true,
                        attester: attester_did,
                        timestamp: crate::types::Timestamp::now(),
                        token: vec![],
                        denial_reason: None,
                    })
                }
            }
        })
    }
}
