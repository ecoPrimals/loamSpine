//! Service lifecycle management.
//!
//! This module handles LoamSpine service lifecycle operations:
//! - Startup (auto-advertisement to Songbird)
//! - Runtime (background heartbeat)
//! - Shutdown (deregistration from Songbird)

use crate::config::LoamSpineConfig;
use crate::error::LoamSpineResult;
use crate::service::LoamSpineService;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

/// Lifecycle manager for LoamSpine service.
///
/// Handles service startup, runtime tasks, and graceful shutdown following
/// the **infant discovery** philosophy: start with zero knowledge, discover everything.
pub struct LifecycleManager {
    /// Service instance.
    service: LoamSpineService,
    /// Configuration.
    config: LoamSpineConfig,
    /// Discovery service client (universal adapter).
    discovery_client: Option<crate::songbird::SongbirdClient>,
    /// Heartbeat task handle.
    heartbeat_task: Option<JoinHandle<()>>,
    /// Shutdown signal.
    shutdown: Arc<AtomicBool>,
}

impl LifecycleManager {
    /// Create a new lifecycle manager.
    #[must_use]
    pub fn new(service: LoamSpineService, config: LoamSpineConfig) -> Self {
        Self {
            service,
            config,
            discovery_client: None,
            heartbeat_task: None,
            shutdown: Arc::new(AtomicBool::new(false)),
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
    #[allow(clippy::option_if_let_else)]
    pub async fn start(&mut self) -> LoamSpineResult<()> {
        tracing::info!("🦴 Starting LoamSpine service lifecycle (infant discovery mode)...");

        // Check both new and deprecated fields for backward compatibility
        #[allow(deprecated)]
        let discovery_enabled = self.config.discovery.discovery_enabled 
            || self.config.discovery.songbird_enabled;
        
        #[allow(deprecated)]
        let discovery_endpoint = self.config.discovery.discovery_endpoint.clone()
            .or_else(|| self.config.discovery.songbird_endpoint.clone());

        // Connect to discovery service if enabled
        if discovery_enabled {
            // Try infant discovery if no endpoint configured
            let endpoint = if let Some(ep) = discovery_endpoint {
                ep
            } else {
                tracing::info!("📍 No discovery endpoint configured, attempting infant discovery...");
                
                // Use infant discovery to find the discovery service
                let infant = crate::service::InfantDiscovery::new(vec![
                    "persistent-ledger".to_string(),
                    "waypoint-anchoring".to_string(),
                    "certificate-manager".to_string(),
                ]);
                
                match infant.discover_discovery_service().await {
                    Ok(client) => {
                        tracing::info!("✅ Infant discovery successful!");
                        self.discovery_client = Some(client);
                        
                        // Advertise and start heartbeat
                        if self.config.discovery.auto_advertise {
                            if let Some(ref client) = self.discovery_client {
                                self.advertise_capabilities(client).await?;
                            }
                        }
                        
                        if self.config.discovery.heartbeat_interval_seconds > 0 {
                            if let Some(client) = self.discovery_client.clone() {
                                self.start_heartbeat_task(client);
                            }
                        }
                        
                        tracing::info!("✅ LoamSpine service lifecycle started");
                        return Ok(());
                    }
                    Err(e) => {
                        tracing::warn!(
                            "⚠️  Infant discovery failed: {e}. Continuing without discovery."
                        );
                        tracing::info!("✅ LoamSpine service lifecycle started (without discovery)");
                        return Ok(());
                    }
                }
            };
            
            // We have an endpoint, try to connect
            tracing::info!("📡 Connecting to discovery service at {endpoint}...");

            match crate::songbird::SongbirdClient::connect(&endpoint).await {
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

        tracing::info!("✅ LoamSpine service lifecycle started");
        Ok(())
    }

    /// Advertise capabilities to discovery service.
    async fn advertise_capabilities(
        &self,
        client: &crate::songbird::SongbirdClient,
    ) -> LoamSpineResult<()> {
        tracing::info!("📢 Advertising LoamSpine capabilities to discovery service...");

        // Get endpoints from config
        let tarpc_endpoint = &self.config.discovery.tarpc_endpoint;
        let jsonrpc_endpoint = &self.config.discovery.jsonrpc_endpoint;

        client
            .advertise_loamspine(tarpc_endpoint, jsonrpc_endpoint)
            .await?;

        tracing::info!("✅ Capabilities advertised to discovery service");
        Ok(())
    }

    /// Start background heartbeat task with retry logic.
    fn start_heartbeat_task(&mut self, client: crate::songbird::SongbirdClient) {
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
                            tracing::debug!("❤️  Heartbeat sent to Songbird");
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
                                "❌ Heartbeat failed {consecutive_failures} times. Giving up. Service may be deregistered by Songbird."
                            );
                            // Continue loop but stop trying to send heartbeats
                            // Service will be auto-deregistered by Songbird after timeout
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
    async fn send_heartbeat_with_retry(
        client: &crate::songbird::SongbirdClient,
        retry_config: &crate::config::HeartbeatRetryConfig,
        base_failures: u32,
    ) -> LoamSpineResult<()> {
        // Try immediate send first
        if matches!(client.heartbeat().await, Ok(())) {
            return Ok(());
        }

        // Retry with exponential backoff
        for (attempt, &backoff_secs) in retry_config.backoff_seconds.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let attempt_num = base_failures + (attempt as u32) + 1;

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
    /// 3. Deregister from Songbird (if connected)
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown operations fail.
    pub async fn stop(&mut self) -> LoamSpineResult<()> {
        tracing::info!("🛑 Stopping LoamSpine service lifecycle...");

        // Signal shutdown
        self.shutdown.store(true, Ordering::Relaxed);

        // Wait for heartbeat task to complete
        if let Some(task) = self.heartbeat_task.take() {
            tracing::debug!("Waiting for heartbeat task to complete...");
            let _ = task.await; // Ignore errors on shutdown
        }

        // Deregister from discovery service if connected
        if let Some(ref client) = self.discovery_client {
            tracing::info!("📢 Deregistering from discovery service...");
            match client.deregister().await {
                Ok(()) => tracing::info!("✅ Deregistered from discovery service"),
                Err(e) => tracing::warn!("⚠️  Deregister failed (non-fatal): {e}"),
            }
        }

        tracing::info!("✅ LoamSpine service lifecycle stopped");
        Ok(())
    }

    /// Get the service instance.
    #[must_use]
    pub const fn service(&self) -> &LoamSpineService {
        &self.service
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
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_manager_creation() {
        let service = LoamSpineService::new();
        let config = LoamSpineConfig::default();
        let manager = LifecycleManager::new(service, config);

        assert!(manager.heartbeat_task.is_none());
        assert!(!manager.shutdown.load(Ordering::Relaxed));
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn lifecycle_start_without_songbird() {
        let service = LoamSpineService::new();
        let mut config = LoamSpineConfig::default();
        config.discovery.songbird_enabled = false;

        let mut manager = LifecycleManager::new(service, config);
        let result = manager.start().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn lifecycle_stop() {
        let service = LoamSpineService::new();
        let mut config = LoamSpineConfig::default();
        config.discovery.songbird_enabled = false;

        let mut manager = LifecycleManager::new(service, config);
        manager.start().await.expect("Failed to start");

        let result = manager.stop().await;
        assert!(result.is_ok());
        assert!(manager.shutdown.load(Ordering::Relaxed));
    }

    #[test]
    fn lifecycle_manager_service_getter() {
        let service = LoamSpineService::new();
        let config = LoamSpineConfig::default();
        let manager = LifecycleManager::new(service, config);

        let _service_ref = manager.service();
        // Just verify we can get the service reference
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn lifecycle_start_with_songbird_unavailable() {
        let service = LoamSpineService::new();
        let mut config = LoamSpineConfig::default();
        config.discovery.songbird_enabled = true;
        config.discovery.songbird_endpoint = Some("http://localhost:9999".to_string());
        config.discovery.auto_advertise = true;

        let mut manager = LifecycleManager::new(service, config);
        let result = manager.start().await;

        // Should succeed even if discovery service is unavailable (graceful degradation)
        assert!(result.is_ok());
        assert!(manager.discovery_client.is_none());
    }

    #[tokio::test]
    async fn lifecycle_stop_without_start() {
        let service = LoamSpineService::new();
        let config = LoamSpineConfig::default();
        let mut manager = LifecycleManager::new(service, config);

        let result = manager.stop().await;
        assert!(result.is_ok());
    }

    #[test]
    fn lifecycle_manager_drop() {
        let service = LoamSpineService::new();
        let config = LoamSpineConfig::default();
        let manager = LifecycleManager::new(service, config);

        // Test that drop doesn't panic
        drop(manager);
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn lifecycle_multiple_stops() {
        let service = LoamSpineService::new();
        let mut config = LoamSpineConfig::default();
        config.discovery.songbird_enabled = false;

        let mut manager = LifecycleManager::new(service, config);
        manager.start().await.expect("Failed to start");

        let result1 = manager.stop().await;
        assert!(result1.is_ok());

        let result2 = manager.stop().await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn lifecycle_start_with_songbird_no_endpoint() {
        let service = LoamSpineService::new();
        let mut config = LoamSpineConfig::default();
        config.discovery.songbird_enabled = true;
        config.discovery.songbird_endpoint = None;

        let mut manager = LifecycleManager::new(service, config);
        let result = manager.start().await;

        assert!(result.is_ok());
        assert!(manager.discovery_client.is_none());
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn lifecycle_start_with_heartbeat_disabled() {
        let service = LoamSpineService::new();
        let mut config = LoamSpineConfig::default();
        config.discovery.songbird_enabled = false;
        config.discovery.heartbeat_interval_seconds = 0;

        let mut manager = LifecycleManager::new(service, config);
        let result = manager.start().await;

        assert!(result.is_ok());
        assert!(manager.heartbeat_task.is_none());
    }

    #[test]
    fn shutdown_signal_initial_state() {
        let service = LoamSpineService::new();
        let config = LoamSpineConfig::default();
        let manager = LifecycleManager::new(service, config);

        assert!(!manager.shutdown.load(Ordering::Relaxed));
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn shutdown_signal_after_stop() {
        let service = LoamSpineService::new();
        let mut config = LoamSpineConfig::default();
        config.discovery.songbird_enabled = false;

        let mut manager = LifecycleManager::new(service, config);
        manager.start().await.expect("Failed to start");
        manager.stop().await.expect("Failed to stop");

        assert!(manager.shutdown.load(Ordering::Relaxed));
    }

    #[test]
    fn config_endpoints_from_discovery_config() {
        let mut config = LoamSpineConfig::default();
        config.discovery.tarpc_endpoint = "http://localhost:9999".to_string();
        config.discovery.jsonrpc_endpoint = "http://localhost:7777".to_string();

        assert_eq!(config.discovery.tarpc_endpoint, "http://localhost:9999");
        assert_eq!(config.discovery.jsonrpc_endpoint, "http://localhost:7777");
    }
}
