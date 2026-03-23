// SPDX-License-Identifier: AGPL-3.0-or-later

//! Signal handling utilities for graceful shutdown.
//!
//! Provides helpers for handling SIGTERM, SIGINT, and other shutdown signals.

use crate::error::LoamSpineResult;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Signal handler for graceful shutdown.
///
/// Listens for SIGTERM and SIGINT signals and triggers shutdown.
pub struct SignalHandler {
    /// Shutdown flag.
    shutdown: Arc<AtomicBool>,
}

impl SignalHandler {
    /// Create a new signal handler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Wait for shutdown signal (SIGTERM or SIGINT).
    ///
    /// This method blocks until either:
    /// - SIGTERM is received (kill command)
    /// - SIGINT is received (Ctrl+C)
    ///
    /// # Errors
    ///
    /// Returns error if signal setup fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use loam_spine_core::service::signals::SignalHandler;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let handler = SignalHandler::new();
    ///
    /// // Run service...
    ///
    /// // Wait for shutdown signal
    /// handler.wait_for_shutdown().await?;
    ///
    /// // Perform graceful shutdown...
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_shutdown(&self) -> LoamSpineResult<()> {
        #[cfg(unix)]
        {
            self.wait_for_shutdown_unix().await
        }

        #[cfg(not(unix))]
        {
            self.wait_for_shutdown_windows().await
        }
    }

    /// Unix-specific signal handling (SIGTERM + SIGINT).
    #[cfg(unix)]
    async fn wait_for_shutdown_unix(&self) -> LoamSpineResult<()> {
        use tokio::signal::unix::{SignalKind, signal};

        let mut sigterm = signal(SignalKind::terminate()).map_err(|e| {
            crate::error::LoamSpineError::Internal(format!("Failed to setup SIGTERM handler: {e}"))
        })?;

        let mut sigint = signal(SignalKind::interrupt()).map_err(|e| {
            crate::error::LoamSpineError::Internal(format!("Failed to setup SIGINT handler: {e}"))
        })?;

        tokio::select! {
            _ = sigterm.recv() => {
                tracing::info!("Received SIGTERM, initiating graceful shutdown...");
                self.shutdown.store(true, Ordering::Relaxed);
            }
            _ = sigint.recv() => {
                tracing::info!("Received SIGINT (Ctrl+C), initiating graceful shutdown...");
                self.shutdown.store(true, Ordering::Relaxed);
            }
        }

        Ok(())
    }

    /// Windows-specific signal handling (Ctrl+C only).
    #[cfg(not(unix))]
    async fn wait_for_shutdown_windows(&self) -> LoamSpineResult<()> {
        signal::ctrl_c().await.map_err(|e| {
            crate::error::LoamSpineError::Internal(format!("Failed to setup Ctrl+C handler: {e}"))
        })?;

        tracing::info!("Received Ctrl+C, initiating graceful shutdown...");
        self.shutdown.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Check if shutdown has been signaled.
    #[must_use]
    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }

    /// Get a clone of the shutdown flag for sharing.
    #[must_use]
    pub fn shutdown_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.shutdown)
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to run a service with automatic signal handling.
///
/// This function:
/// 1. Starts the lifecycle manager
/// 2. Waits for shutdown signal (SIGTERM/SIGINT)
/// 3. Stops the lifecycle manager gracefully
///
/// # Errors
///
/// Returns error if startup, shutdown, or signal handling fails.
///
/// # Example
///
/// ```no_run
/// use loam_spine_core::service::{LoamSpineService, signals};
/// use loam_spine_core::config::LoamSpineConfig;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let service = LoamSpineService::new();
/// let config = LoamSpineConfig::default();
///
/// // Run with automatic signal handling
/// signals::run_with_signals(service, config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn run_with_signals(
    service: crate::service::LoamSpineService,
    config: crate::config::LoamSpineConfig,
) -> LoamSpineResult<()> {
    use super::lifecycle::LifecycleManager;

    let mut lifecycle = LifecycleManager::new(service, config);

    // Start lifecycle
    lifecycle.start().await?;
    tracing::info!("✅ Service started, waiting for shutdown signal...");

    // Wait for shutdown signal
    let handler = SignalHandler::new();
    handler.wait_for_shutdown().await?;

    // Stop lifecycle gracefully
    lifecycle.stop().await?;
    tracing::info!("✅ Service stopped gracefully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_handler_creation() {
        let handler = SignalHandler::new();
        assert!(!handler.is_shutdown());
    }

    #[test]
    fn signal_handler_default() {
        let handler = SignalHandler::default();
        assert!(!handler.is_shutdown());
    }

    #[test]
    fn signal_handler_shutdown_flag() {
        let handler = SignalHandler::new();
        let flag = handler.shutdown_flag();

        // Initially false
        assert!(!flag.load(Ordering::Relaxed));

        // Can be set
        flag.store(true, Ordering::Relaxed);
        assert!(handler.is_shutdown());
    }

    #[test]
    fn signal_handler_default_equals_new() {
        let from_new = SignalHandler::new();
        let from_default = SignalHandler::default();
        assert_eq!(from_new.is_shutdown(), from_default.is_shutdown());
    }

    #[test]
    fn signal_handler_state_transition_via_flag() {
        let handler = SignalHandler::new();
        let flag = handler.shutdown_flag();

        assert!(!handler.is_shutdown());
        flag.store(true, Ordering::Relaxed);
        assert!(handler.is_shutdown());
        flag.store(false, Ordering::Relaxed);
        assert!(!handler.is_shutdown());
    }

    #[test]
    fn signal_handler_multiple_flag_clones_share_state() {
        let handler = SignalHandler::new();
        let flag1 = handler.shutdown_flag();
        let flag2 = handler.shutdown_flag();

        flag1.store(true, Ordering::Relaxed);
        assert!(flag2.load(Ordering::Relaxed));
        assert!(handler.is_shutdown());
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn run_with_signals_requires_real_signal() {
        // This test verifies the function compiles and can be invoked.
        // Actual signal testing requires integration tests with real OS signals.
        // We use a timeout to avoid blocking forever - the function will block
        // on wait_for_shutdown until SIGTERM/SIGINT is received.
        let service = crate::service::LoamSpineService::new();
        let mut config = crate::config::LoamSpineConfig::default();
        config.discovery.discovery_enabled = false;

        let result = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            run_with_signals(service, config),
        )
        .await;

        // Timeout is expected - we didn't send a signal
        assert!(result.is_err());
    }
}
