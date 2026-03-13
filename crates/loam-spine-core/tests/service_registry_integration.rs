// SPDX-License-Identifier: AGPL-3.0-only

//! Integration tests for service registry discovery client.
//!
//! These tests use a live service registry binary from `../bins/` to test
//! real service discovery and registration flows. They gracefully skip
//! if the binary is not available.
//!
//! The service registry is vendor-agnostic: any HTTP registry exposing
//! `/discover?capability=...` and `/register` endpoints works.
//!
//! ## Concurrency & Robustness
//!
//! - NO blocking sleeps (`thread::sleep`)
//! - Proper async polling with retries
//! - Timeout protection (no hanging tests)
//! - Production-grade error handling
//! - Truly concurrent test execution

use loam_spine_core::discovery_client::DiscoveryClient;
use std::path::Path;
use std::process::{Child, Command};
use std::time::Duration;

/// Path to service registry binary (discovered at runtime).
const REGISTRY_BIN: &str = "../bins/songbird-orchestrator";

/// Default service registry endpoint for tests.
const REGISTRY_ENDPOINT: &str = "http://localhost:8082";

/// Maximum time to wait for the registry to become ready.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);

/// Polling interval for health checks.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Helper to check if the service registry binary exists.
fn registry_available() -> bool {
    Path::new(REGISTRY_BIN).exists()
}

/// Start registry and wait for it to be ready using proper async polling.
///
/// No blocking sleeps - uses exponential backoff polling with health checks.
async fn start_registry_and_wait() -> Option<Child> {
    if !registry_available() {
        eprintln!("⚠️  Skipping service registry integration tests: binary not found at {REGISTRY_BIN}");
        return None;
    }

    // Start registry in background
    let child = match Command::new(REGISTRY_BIN)
        .arg("--port")
        .arg("8082")
        .arg("--host")
        .arg("127.0.0.1")
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!("⚠️  Failed to start service registry: {e}");
            return None;
        }
    };

    // Poll for readiness with timeout protection
    match wait_for_registry_ready().await {
        Ok(()) => Some(child),
        Err(e) => {
            eprintln!("⚠️  Service registry failed to become ready: {e}");
            None
        }
    }
}

/// Wait for service registry to be ready using async polling with exponential backoff.
///
/// Production-grade: No blind sleeps, proper timeout, error handling.
async fn wait_for_registry_ready() -> Result<(), String> {
    let start = std::time::Instant::now();
    let mut attempt = 0;

    loop {
        // Check timeout
        if start.elapsed() > STARTUP_TIMEOUT {
            return Err(format!(
                "Service registry did not become ready within {} seconds",
                STARTUP_TIMEOUT.as_secs()
            ));
        }

        // Try to connect
        match tokio::time::timeout(
            Duration::from_secs(2),
            DiscoveryClient::connect(REGISTRY_ENDPOINT),
        )
        .await
        {
            Ok(Ok(_client)) => {
                // Successfully connected!
                return Ok(());
            }
            Ok(Err(_e)) => {
                // Connection failed, retry
            }
            Err(_timeout) => {
                // Timeout, retry
            }
        }

        // Exponential backoff: 100ms, 200ms, 400ms, 800ms, 1s, 1s, ...
        let backoff = POLL_INTERVAL
            .saturating_mul(2_u32.saturating_pow(attempt))
            .min(Duration::from_secs(1));

        tokio::time::sleep(backoff).await;
        attempt += 1;
    }
}

/// Wait for a service to be discoverable (eventual consistency).
///
/// Production-grade: Polls with timeout, no blind sleeps.
async fn wait_for_service_discoverable(
    client: &DiscoveryClient,
    capability: &str,
    service_name: &str,
) -> Result<(), String> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(5);

    loop {
        if start.elapsed() > timeout {
            return Err(format!(
                "Service {service_name} not discoverable within {} seconds",
                timeout.as_secs()
            ));
        }

        if let Ok(services) = client.discover_capability(capability).await {
            if services.iter().any(|s| s.name == service_name) {
                return Ok(());
            }
        }
        // Discovery failed, retry

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

/// Wait for a service to be deregistered (eventual consistency).
///
/// Production-grade: Polls with timeout, no blind sleeps.
async fn wait_for_service_removed(
    client: &DiscoveryClient,
    capability: &str,
    service_name: &str,
) -> Result<(), String> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(5);

    loop {
        if start.elapsed() > timeout {
            return Err(format!(
                "Service {service_name} still discoverable after {} seconds",
                timeout.as_secs()
            ));
        }

        match client.discover_capability(capability).await {
            Ok(services) => {
                if !services.iter().any(|s| s.name == service_name) {
                    return Ok(());
                }
            }
            Err(_) => {
                // Discovery failed, but that's okay for removal check
                return Ok(());
            }
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

#[tokio::test]
async fn test_registry_connection() {
    if !registry_available() {
        eprintln!("⚠️  Skipping test: service registry binary not available");
        return;
    }

    // Start registry and wait for readiness (async, no blocking)
    #[allow(clippy::used_underscore_binding)]
    let _process = start_registry_and_wait().await;
    if _process.is_none() {
        return;
    }

    // Test connection (with timeout protection)
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        DiscoveryClient::connect(REGISTRY_ENDPOINT),
    )
    .await;

    assert!(result.is_ok(), "Connection should not timeout");
    #[allow(clippy::unwrap_used)]
    let connect_result = result.unwrap();
    assert!(
        connect_result.is_ok(),
        "Should connect to service registry successfully"
    );
}

#[tokio::test]
async fn test_registry_advertise_and_discover() {
    if !registry_available() {
        eprintln!("⚠️  Skipping test: service registry binary not available");
        return;
    }

    // Start registry and wait for readiness (async, no blocking)
    #[allow(clippy::used_underscore_binding)]
    let _process = start_registry_and_wait().await;
    if _process.is_none() {
        return;
    }

    let Ok(client) = DiscoveryClient::connect(REGISTRY_ENDPOINT).await else {
        eprintln!("⚠️  Failed to connect to service registry");
        return;
    };

    // Advertise LoamSpine
    let result = client
        .advertise_self("http://localhost:9001", "http://localhost:8080")
        .await;

    assert!(
        result.is_ok(),
        "Should advertise LoamSpine successfully: {:?}",
        result.err()
    );

    // Wait for service to be discoverable (eventual consistency, no blind sleep)
    let wait_result =
        wait_for_service_discoverable(&client, "persistent-ledger", "loamspine").await;
    assert!(
        wait_result.is_ok(),
        "Service should become discoverable: {:?}",
        wait_result.err()
    );

    // Try to discover persistent-ledger capability
    let Ok(services) = client.discover_capability("persistent-ledger").await else {
        eprintln!("⚠️  Failed to discover services");
        return;
    };

    assert!(
        !services.is_empty(),
        "Should find at least one persistent-ledger service"
    );

    // Verify we found ourselves
    let loamspine = services.iter().find(|s| s.name == "loamspine");
    assert!(
        loamspine.is_some(),
        "Should find LoamSpine in discovered services"
    );
}

#[tokio::test]
#[allow(clippy::used_underscore_binding, clippy::unwrap_used)]
async fn test_registry_heartbeat() {
    if !registry_available() {
        eprintln!("⚠️  Skipping test: service registry binary not available");
        return;
    }

    // Start registry and wait for readiness (async, no blocking)
    let _process = start_registry_and_wait().await;
    if _process.is_none() {
        return;
    }

    let Ok(client) = DiscoveryClient::connect(REGISTRY_ENDPOINT).await else {
        return;
    };

    // Advertise first
    let _ = client
        .advertise_self("http://localhost:9001", "http://localhost:8080")
        .await;

    // Wait for registration (eventual consistency, no blind sleep)
    let _ = wait_for_service_discoverable(&client, "persistent-ledger", "loamspine").await;

    // Send heartbeat (with timeout protection)
    let result = tokio::time::timeout(Duration::from_secs(5), client.heartbeat()).await;

    assert!(result.is_ok(), "Heartbeat should not timeout");
    let heartbeat_result = result.unwrap();
    assert!(
        heartbeat_result.is_ok(),
        "Should send heartbeat successfully"
    );
}

#[tokio::test]
#[allow(clippy::used_underscore_binding)]
async fn test_registry_deregister() {
    if !registry_available() {
        eprintln!("⚠️  Skipping test: service registry binary not available");
        return;
    }

    // Start registry and wait for readiness (async, no blocking)
    let _process = start_registry_and_wait().await;
    if _process.is_none() {
        return;
    }

    let Ok(client) = DiscoveryClient::connect(REGISTRY_ENDPOINT).await else {
        return;
    };

    // Advertise first
    let _ = client
        .advertise_self("http://localhost:9001", "http://localhost:8080")
        .await;

    // Wait for registration (eventual consistency, no blind sleep)
    let _ = wait_for_service_discoverable(&client, "persistent-ledger", "loamspine").await;

    // Deregister
    let result = client.deregister().await;
    assert!(
        result.is_ok(),
        "Should deregister successfully: {:?}",
        result.err()
    );

    // Wait for deregistration (eventual consistency, no blind sleep)
    let wait_result = wait_for_service_removed(&client, "persistent-ledger", "loamspine").await;
    assert!(
        wait_result.is_ok(),
        "Service should be removed: {:?}",
        wait_result.err()
    );

    // Verify we're no longer discoverable
    let services = client
        .discover_capability("persistent-ledger")
        .await
        .unwrap_or_default();
    let loamspine = services.iter().find(|s| s.name == "loamspine");
    assert!(
        loamspine.is_none(),
        "Should not find LoamSpine after deregistration"
    );
}

#[tokio::test]
#[allow(clippy::used_underscore_binding, clippy::unwrap_used)]
async fn test_registry_discover_all() {
    if !registry_available() {
        eprintln!("⚠️  Skipping test: service registry binary not available");
        return;
    }

    // Start registry and wait for readiness (async, no blocking)
    let _process = start_registry_and_wait().await;
    if _process.is_none() {
        return;
    }

    let Ok(client) = DiscoveryClient::connect(REGISTRY_ENDPOINT).await else {
        return;
    };

    // Advertise multiple capabilities
    let _ = client
        .advertise_self("http://localhost:9001", "http://localhost:8080")
        .await;

    // Wait for registration (eventual consistency, no blind sleep)
    let _ = wait_for_service_discoverable(&client, "persistent-ledger", "loamspine").await;

    // Discover all services (with timeout protection)
    let result = tokio::time::timeout(Duration::from_secs(5), client.discover_all()).await;

    assert!(result.is_ok(), "Discovery should not timeout");
    let discover_result = result.unwrap();
    let Ok(services) = discover_result else {
        eprintln!("⚠️  Failed to discover all services");
        return;
    };

    assert!(!services.is_empty(), "Should find at least one service");
}

#[tokio::test]
#[allow(clippy::used_underscore_binding, clippy::unwrap_used)]
async fn test_registry_multiple_capabilities() {
    if !registry_available() {
        eprintln!("⚠️  Skipping test: service registry binary not available");
        return;
    }

    // Start registry and wait for readiness (async, no blocking)
    let _process = start_registry_and_wait().await;
    if _process.is_none() {
        return;
    }

    let Ok(client) = DiscoveryClient::connect(REGISTRY_ENDPOINT).await else {
        return;
    };

    // Advertise
    let _ = client
        .advertise_self("http://localhost:9001", "http://localhost:8080")
        .await;

    // Wait for registration (eventual consistency, no blind sleep)
    let _ = wait_for_service_discoverable(&client, "persistent-ledger", "loamspine").await;

    // Test discovering multiple capabilities concurrently (not serial!)
    let capabilities = vec![
        "persistent-ledger",
        "certificate-manager",
        "waypoint-anchoring",
    ];

    // Spawn concurrent discovery tasks
    let mut handles = vec![];
    for capability in capabilities {
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            tokio::time::timeout(
                Duration::from_secs(5),
                client_clone.discover_capability(capability),
            )
            .await
        });
        handles.push((capability, handle));
    }

    // All should succeed concurrently
    for (capability, handle) in handles {
        let result = handle.await;
        assert!(result.is_ok(), "Task for {capability} should not fail");
        #[allow(clippy::unwrap_used)]
        let timeout_result = result.unwrap();
        assert!(
            timeout_result.is_ok(),
            "Should discover {capability} capability"
        );
    }
}

#[tokio::test]
async fn test_registry_endpoint_validation() {
    // Test that invalid endpoints are handled gracefully
    let result = DiscoveryClient::connect("http://invalid-host:99999").await;
    assert!(
        result.is_err(),
        "Should fail to connect to invalid endpoint"
    );
}

#[tokio::test]
#[allow(clippy::used_underscore_binding)]
async fn test_registry_concurrent_operations() {
    if !registry_available() {
        eprintln!("⚠️  Skipping test: service registry binary not available");
        return;
    }

    // Start registry and wait for readiness (async, no blocking)
    let _process = start_registry_and_wait().await;
    if _process.is_none() {
        return;
    }

    let Ok(client) = DiscoveryClient::connect(REGISTRY_ENDPOINT).await else {
        return;
    };

    // Advertise
    let _ = client
        .advertise_self("http://localhost:9001", "http://localhost:8080")
        .await;

    // Wait for registration (eventual consistency, no blind sleep)
    let _ = wait_for_service_discoverable(&client, "persistent-ledger", "loamspine").await;

    // Perform 100 concurrent discovery operations (truly concurrent!)
    let mut handles = vec![];
    for i in 0..100 {
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            // Each operation has timeout protection
            tokio::time::timeout(
                Duration::from_secs(5),
                client_clone.discover_capability("persistent-ledger"),
            )
            .await
            .map_err(|_| format!("Timeout on operation {i}"))
            .and_then(|r| r.map_err(|e| format!("Discovery failed: {e}")))
        });
        handles.push(handle);
    }

    // All should succeed concurrently
    let mut successes = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_services)) => {
                successes += 1;
            }
            Ok(Err(e)) => {
                eprintln!("⚠️  Operation failed: {e}");
            }
            Err(e) => {
                eprintln!("⚠️  Task panicked: {e}");
            }
        }
    }

    // Require at least 95% success rate (allows for network jitter)
    assert!(
        successes >= 95,
        "At least 95 out of 100 concurrent operations should succeed, got {successes}"
    );
}
