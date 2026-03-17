// SPDX-License-Identifier: AGPL-3.0-or-later

//! Service lifecycle management.
//!
//! This module handles LoamSpine service lifecycle operations:
//! - Startup (auto-advertisement to service registry)
//! - Runtime (background heartbeat)
//! - Shutdown (deregistration from service registry)

use crate::config::LoamSpineConfig;
use crate::error::LoamSpineResult;
use crate::service::LoamSpineService;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::{Duration, interval};

/// Service lifecycle state per `SERVICE_LIFECYCLE.md` specification.
///
/// Transitions: `Starting` → `Ready` → `Running` → `Stopping` → `Stopped`.
/// Error paths:  any state → `Degraded` (recoverable) or `Error` (needs restart).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ServiceState {
    /// Service is initializing (discovery, registration, storage warm-up).
    Starting,
    /// Initialization complete, ready to accept traffic.
    Ready,
    /// Actively serving requests.
    Running,
    /// Non-critical subsystem failure (e.g., heartbeat loss). Still serving.
    Degraded,
    /// Graceful shutdown in progress — draining in-flight requests.
    Stopping,
    /// Shutdown complete.
    Stopped,
    /// Unrecoverable failure — requires restart.
    Error,
}

impl std::fmt::Display for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Starting => "STARTING",
            Self::Ready => "READY",
            Self::Running => "RUNNING",
            Self::Degraded => "DEGRADED",
            Self::Stopping => "STOPPING",
            Self::Stopped => "STOPPED",
            Self::Error => "ERROR",
        })
    }
}

/// Lifecycle manager for LoamSpine service.
///
/// Handles service startup, runtime tasks, and graceful shutdown following
/// the **infant discovery** philosophy: start with zero knowledge, discover everything.
///
/// Exposes an observable [`ServiceState`] via a `watch` channel so that
/// health checks, readiness probes, and metrics can react to transitions.
pub struct LifecycleManager {
    /// Service instance.
    service: LoamSpineService,
    /// Configuration.
    config: LoamSpineConfig,
    /// Discovery service client (universal adapter).
    discovery_client: Option<crate::discovery_client::DiscoveryClient>,
    /// Heartbeat task handle.
    heartbeat_task: Option<JoinHandle<()>>,
    /// Shutdown signal.
    shutdown: Arc<AtomicBool>,
    /// Observable service state.
    state_tx: watch::Sender<ServiceState>,
    state_rx: watch::Receiver<ServiceState>,
}

impl LifecycleManager {
    /// Create a new lifecycle manager.
    #[must_use]
    pub fn new(service: LoamSpineService, config: LoamSpineConfig) -> Self {
        let (state_tx, state_rx) = watch::channel(ServiceState::Stopped);
        Self {
            service,
            config,
            discovery_client: None,
            heartbeat_task: None,
            shutdown: Arc::new(AtomicBool::new(false)),
            state_tx,
            state_rx,
        }
    }

    /// Current service state.
    #[must_use]
    pub fn state(&self) -> ServiceState {
        *self.state_rx.borrow()
    }

    /// Subscribe to state transitions (e.g., for health probes).
    #[must_use]
    pub fn subscribe_state(&self) -> watch::Receiver<ServiceState> {
        self.state_rx.clone()
    }

    /// Transition to a new state, logging the change.
    fn transition(&self, new: ServiceState) {
        let old = *self.state_rx.borrow();
        if old != new {
            tracing::info!("Service state: {old} → {new}");
            let _ = self.state_tx.send(new);
        }
    }

    /// Start the service lifecycle.
    ///
    /// **Infant Discovery**: LoamSpine starts knowing only itself and discovers
    /// the discovery service (universal adapter) at runtime.
    ///
    /// This performs:
    /// 1. Discovery service connection (infant discovery if no endpoint configured)
    /// 2. Capability advertisement (if enabled)
    /// 3. Background heartbeat task (if enabled)
    ///
    /// # Errors
    ///
    /// Returns an error if startup operations fail.
    pub async fn start(&mut self) -> LoamSpineResult<()> {
        self.transition(ServiceState::Starting);
        tracing::info!("🦴 Starting LoamSpine service lifecycle (infant discovery mode)...");

        let discovery_enabled = self.config.discovery.discovery_enabled;
        let discovery_endpoint = self.config.discovery.discovery_endpoint.clone();

        // Connect to discovery service if enabled
        if discovery_enabled {
            // Try infant discovery if no endpoint configured
            let Some(endpoint) = discovery_endpoint else {
                tracing::info!(
                    "📍 No discovery endpoint configured, attempting infant discovery..."
                );

                // Use infant discovery to find the discovery service
                let infant = crate::service::InfantDiscovery::from_advertised();

                match infant.discover_discovery_service().await {
                    Ok(client) => {
                        tracing::info!("✅ Infant discovery successful!");
                        self.discovery_client = Some(client);

                        // Advertise and start heartbeat
                        if self.config.discovery.auto_advertise
                            && let Some(ref client) = self.discovery_client
                        {
                            self.advertise_capabilities(client).await?;
                        }

                        if self.config.discovery.heartbeat_interval_seconds > 0
                            && let Some(client) = self.discovery_client.clone()
                        {
                            self.start_heartbeat_task(client);
                        }

                        self.transition(ServiceState::Ready);
                        self.register_neural_api().await;
                        self.transition(ServiceState::Running);
                        return Ok(());
                    }
                    Err(e) => {
                        tracing::warn!(
                            "⚠️  Infant discovery failed: {e}. Continuing without discovery."
                        );
                        self.transition(ServiceState::Ready);
                        self.register_neural_api().await;
                        self.transition(ServiceState::Running);
                        return Ok(());
                    }
                }
            };

            // We have an endpoint, try to connect
            tracing::info!("📡 Connecting to discovery service at {endpoint}...");

            match crate::discovery_client::DiscoveryClient::connect(&endpoint).await {
                Ok(client) => {
                    tracing::info!("✅ Connected to discovery service");

                    // Advertise if enabled
                    if self.config.discovery.auto_advertise {
                        self.advertise_capabilities(&client).await?;
                    }

                    // Start background heartbeat
                    if self.config.discovery.heartbeat_interval_seconds > 0 {
                        self.start_heartbeat_task(client.clone());
                    }

                    // Store client for shutdown
                    self.discovery_client = Some(client);
                }
                Err(e) => {
                    tracing::warn!(
                        "⚠️  Discovery service unavailable at {endpoint}: {e}. Continuing without discovery."
                    );
                }
            }
        } else {
            tracing::debug!("Discovery service disabled");
        }

        self.transition(ServiceState::Ready);
        self.register_neural_api().await;
        self.transition(ServiceState::Running);

        Ok(())
    }

    /// Register with NeuralAPI (biomeOS orchestration) — non-fatal.
    async fn register_neural_api(&self) {
        match crate::neural_api::register_with_neural_api().await {
            Ok(true) => tracing::info!("✅ Registered with NeuralAPI"),
            Ok(false) => tracing::debug!("NeuralAPI not available, running standalone"),
            Err(err) => tracing::warn!("⚠️  NeuralAPI registration failed (non-fatal): {err}"),
        }
    }

    /// Advertise capabilities to discovery service.
    async fn advertise_capabilities(
        &self,
        client: &crate::discovery_client::DiscoveryClient,
    ) -> LoamSpineResult<()> {
        tracing::info!("📢 Advertising LoamSpine capabilities to discovery service...");

        // Get endpoints from config
        let tarpc_endpoint = &self.config.discovery.tarpc_endpoint;
        let jsonrpc_endpoint = &self.config.discovery.jsonrpc_endpoint;

        client
            .advertise_self(tarpc_endpoint, jsonrpc_endpoint)
            .await?;

        tracing::info!("✅ Capabilities advertised to discovery service");
        Ok(())
    }

    /// Start background heartbeat task with retry logic.
    fn start_heartbeat_task(&mut self, client: crate::discovery_client::DiscoveryClient) {
        let interval_secs = self.config.discovery.heartbeat_interval_seconds;
        let shutdown = Arc::clone(&self.shutdown);
        let retry_config = self.config.discovery.heartbeat_retry.clone();

        tracing::info!("❤️  Starting heartbeat task (interval: {interval_secs}s)");

        let task = tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));
            let mut consecutive_failures = 0u32;
            let mut is_degraded = false;

            loop {
                ticker.tick().await;

                // Check for shutdown signal
                if shutdown.load(Ordering::Relaxed) {
                    tracing::info!("Heartbeat task shutting down");
                    break;
                }

                // Attempt heartbeat with retry logic
                match Self::send_heartbeat_with_retry(&client, &retry_config, consecutive_failures)
                    .await
                {
                    Ok(()) => {
                        // Success - reset failure counter
                        if consecutive_failures > 0 {
                            tracing::info!(
                                "✅ Heartbeat recovered after {consecutive_failures} failures"
                            );
                            consecutive_failures = 0;
                            is_degraded = false;
                        } else {
                            tracing::debug!("❤️  Heartbeat sent to service registry");
                        }
                    }
                    Err(e) => {
                        consecutive_failures += 1;
                        tracing::warn!(
                            "⚠️  Heartbeat failed (attempt {consecutive_failures}): {e}"
                        );

                        // Mark as degraded after threshold
                        if !is_degraded
                            && consecutive_failures >= retry_config.max_failures_before_degraded
                        {
                            tracing::warn!(
                                "⚠️  Service marked as DEGRADED after {consecutive_failures} consecutive heartbeat failures"
                            );
                            is_degraded = true;
                        }

                        // Check if we've exceeded total failure limit
                        if consecutive_failures >= retry_config.max_failures_total {
                            tracing::error!(
                                "❌ Heartbeat failed {consecutive_failures} times. Giving up. Service may be deregistered by registry."
                            );
                            // Continue loop but stop trying to send heartbeats
                            // Service will be auto-deregistered by registry after timeout
                            break;
                        }
                    }
                }
            }

            tracing::info!("Heartbeat task ended");
        });

        self.heartbeat_task = Some(task);
    }

    /// Send heartbeat with exponential backoff retry logic.
    #[cfg_attr(test, allow(dead_code))]
    pub(crate) async fn send_heartbeat_with_retry(
        client: &crate::discovery_client::DiscoveryClient,
        retry_config: &crate::config::HeartbeatRetryConfig,
        base_failures: u32,
    ) -> LoamSpineResult<()> {
        // Try immediate send first
        if matches!(client.heartbeat().await, Ok(())) {
            return Ok(());
        }

        // Retry with exponential backoff
        for (attempt, &backoff_secs) in retry_config.backoff_seconds.iter().enumerate() {
            let attempt_num = base_failures + attempt.try_into().unwrap_or(u32::MAX) + 1;

            // Check if we've exceeded total failure limit
            if attempt_num >= retry_config.max_failures_total {
                break;
            }

            tracing::debug!("Retrying heartbeat in {backoff_secs}s (attempt {attempt_num})...");
            tokio::time::sleep(Duration::from_secs(backoff_secs)).await;

            if matches!(client.heartbeat().await, Ok(())) {
                tracing::info!("✅ Heartbeat succeeded after retry (attempt {attempt_num})");
                return Ok(());
            }
            tracing::debug!("Retry {attempt_num} failed");
        }

        // All retries exhausted
        Err(crate::error::LoamSpineError::Network(
            "Heartbeat failed after all retries".to_string(),
        ))
    }

    /// Stop the service lifecycle.
    ///
    /// This performs:
    /// 1. Signal shutdown to background tasks
    /// 2. Wait for heartbeat task to complete
    /// 3. Deregister from service registry (if connected)
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown operations fail.
    pub async fn stop(&mut self) -> LoamSpineResult<()> {
        self.transition(ServiceState::Stopping);

        // Signal shutdown
        self.shutdown.store(true, Ordering::Relaxed);

        // Wait for heartbeat task to complete
        if let Some(task) = self.heartbeat_task.take() {
            tracing::debug!("Waiting for heartbeat task to complete...");
            let _ = task.await; // Ignore errors on shutdown
        }

        // Deregister from NeuralAPI
        if let Err(e) = crate::neural_api::deregister_from_neural_api().await {
            tracing::warn!("⚠️  NeuralAPI deregistration failed (non-fatal): {e}");
        }

        // Deregister from discovery service if connected
        if let Some(ref client) = self.discovery_client {
            tracing::info!("📢 Deregistering from discovery service...");
            match client.deregister().await {
                Ok(()) => tracing::info!("✅ Deregistered from discovery service"),
                Err(e) => tracing::warn!("⚠️  Deregister failed (non-fatal): {e}"),
            }
        }

        self.transition(ServiceState::Stopped);
        Ok(())
    }

    /// Get the service instance.
    #[must_use]
    pub const fn service(&self) -> &LoamSpineService {
        &self.service
    }

    /// Inject a discovery client for testing (e.g. to exercise deregister error path).
    #[cfg(test)]
    pub fn inject_discovery_client_for_testing(
        &mut self,
        client: crate::discovery_client::DiscoveryClient,
    ) {
        self.discovery_client = Some(client);
    }

    /// Start heartbeat task for testing (exercises stop-waits-for-task and Drop paths).
    #[cfg(test)]
    pub fn start_heartbeat_for_testing(
        &mut self,
        client: crate::discovery_client::DiscoveryClient,
        interval_secs: u64,
    ) {
        self.config.discovery.heartbeat_interval_seconds = interval_secs;
        self.start_heartbeat_task(client);
    }
}

impl Drop for LifecycleManager {
    fn drop(&mut self) {
        // Signal shutdown if not already done
        self.shutdown.store(true, Ordering::Relaxed);

        // Note: We can't await the task here in Drop, so we just abort it
        if let Some(task) = self.heartbeat_task.take() {
            task.abort();
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "tests use unwrap/expect for conciseness"
)]
#[path = "lifecycle_tests.rs"]
mod tests;
