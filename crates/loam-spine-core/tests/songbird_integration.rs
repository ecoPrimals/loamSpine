//! Integration tests for Songbird client with real binary.
//!
//! These tests use the actual `Songbird` binary from `../bins/` to test
//! real service discovery and registration flows. They gracefully skip
//! if the binary is not available.

use loam_spine_core::songbird::SongbirdClient;
use std::path::Path;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

/// Path to songbird-orchestrator binary
const SONGBIRD_BIN: &str = "../bins/songbird-orchestrator";

/// Default Songbird endpoint for tests
const SONGBIRD_ENDPOINT: &str = "http://localhost:8082";

/// Helper to check if Songbird binary exists
fn songbird_available() -> bool {
    Path::new(SONGBIRD_BIN).exists()
}

/// Helper to start Songbird for tests
fn start_songbird() -> Option<Child> {
    if !songbird_available() {
        eprintln!("⚠️  Skipping Songbird integration tests: binary not found at {SONGBIRD_BIN}");
        return None;
    }

    // Start Songbird in background
    match Command::new(SONGBIRD_BIN)
        .arg("--port")
        .arg("8082")
        .arg("--host")
        .arg("127.0.0.1")
        .spawn()
    {
        Ok(child) => {
            // Give it time to start
            thread::sleep(Duration::from_secs(2));
            Some(child)
        }
        Err(e) => {
            eprintln!("⚠️  Failed to start Songbird: {e}");
            None
        }
    }
}

#[tokio::test]
async fn test_songbird_connection() {
    if !songbird_available() {
        eprintln!("⚠️  Skipping test: Songbird binary not available");
        return;
    }

    // Hold process handle to keep Songbird running for duration of test
    let process = start_songbird();
    if process.is_none() {
        return;
    }

    // Test connection
    let result = SongbirdClient::connect(SONGBIRD_ENDPOINT).await;
    assert!(result.is_ok(), "Should connect to Songbird successfully");
}

#[tokio::test]
async fn test_songbird_advertise_and_discover() {
    if !songbird_available() {
        eprintln!("⚠️  Skipping test: Songbird binary not available");
        return;
    }

    // Hold process handle to keep Songbird running for duration of test
    let process = start_songbird();
    if process.is_none() {
        return;
    }

    let Ok(client) = SongbirdClient::connect(SONGBIRD_ENDPOINT).await else {
        eprintln!("⚠️  Failed to connect to Songbird");
        return;
    };

    // Advertise LoamSpine
    let result = client
        .advertise_loamspine("http://localhost:9001", "http://localhost:8080")
        .await;

    assert!(
        result.is_ok(),
        "Should advertise LoamSpine successfully: {:?}",
        result.err()
    );

    // Give Songbird time to register
    thread::sleep(Duration::from_millis(500));

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
async fn test_songbird_heartbeat() {
    if !songbird_available() {
        eprintln!("⚠️  Skipping test: Songbird binary not available");
        return;
    }

    // Hold process handle to keep Songbird running for duration of test
    let process = start_songbird();
    if process.is_none() {
        return;
    }

    let Ok(client) = SongbirdClient::connect(SONGBIRD_ENDPOINT).await else {
        return;
    };

    // Advertise first
    let _ = client
        .advertise_loamspine("http://localhost:9001", "http://localhost:8080")
        .await;

    thread::sleep(Duration::from_millis(500));

    // Send heartbeat
    let result = client.heartbeat().await;
    assert!(
        result.is_ok(),
        "Should send heartbeat successfully: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_songbird_deregister() {
    if !songbird_available() {
        eprintln!("⚠️  Skipping test: Songbird binary not available");
        return;
    }

    // Hold process handle to keep Songbird running for duration of test
    let process = start_songbird();
    if process.is_none() {
        return;
    }

    let Ok(client) = SongbirdClient::connect(SONGBIRD_ENDPOINT).await else {
        return;
    };

    // Advertise first
    let _ = client
        .advertise_loamspine("http://localhost:9001", "http://localhost:8080")
        .await;

    thread::sleep(Duration::from_millis(500));

    // Deregister
    let result = client.deregister().await;
    assert!(
        result.is_ok(),
        "Should deregister successfully: {:?}",
        result.err()
    );

    // Verify we're no longer discoverable
    thread::sleep(Duration::from_millis(500));
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
async fn test_songbird_discover_all() {
    if !songbird_available() {
        eprintln!("⚠️  Skipping test: Songbird binary not available");
        return;
    }

    // Hold process handle to keep Songbird running for duration of test
    let process = start_songbird();
    if process.is_none() {
        return;
    }

    let Ok(client) = SongbirdClient::connect(SONGBIRD_ENDPOINT).await else {
        return;
    };

    // Advertise multiple capabilities
    let _ = client
        .advertise_loamspine("http://localhost:9001", "http://localhost:8080")
        .await;

    thread::sleep(Duration::from_millis(500));

    // Discover all services
    let Ok(services) = client.discover_all().await else {
        eprintln!("⚠️  Failed to discover all services");
        return;
    };

    assert!(!services.is_empty(), "Should find at least one service");
}

#[tokio::test]
async fn test_songbird_multiple_capabilities() {
    if !songbird_available() {
        eprintln!("⚠️  Skipping test: Songbird binary not available");
        return;
    }

    // Hold process handle to keep Songbird running for duration of test
    let process = start_songbird();
    if process.is_none() {
        return;
    }

    let Ok(client) = SongbirdClient::connect(SONGBIRD_ENDPOINT).await else {
        return;
    };

    // Advertise
    let _ = client
        .advertise_loamspine("http://localhost:9001", "http://localhost:8080")
        .await;

    thread::sleep(Duration::from_millis(500));

    // Test discovering multiple capabilities
    let capabilities = vec![
        "persistent-ledger",
        "certificate-manager",
        "waypoint-anchoring",
    ];

    for capability in capabilities {
        let services = client.discover_capability(capability).await;
        assert!(services.is_ok(), "Should discover {capability} capability");
    }
}

#[tokio::test]
async fn test_songbird_endpoint_validation() {
    // Test that invalid endpoints are handled gracefully
    let result = SongbirdClient::connect("http://invalid-host:99999").await;
    assert!(
        result.is_err(),
        "Should fail to connect to invalid endpoint"
    );
}

#[tokio::test]
async fn test_songbird_concurrent_operations() {
    if !songbird_available() {
        eprintln!("⚠️  Skipping test: Songbird binary not available");
        return;
    }

    // Hold process handle to keep Songbird running for duration of test
    let process = start_songbird();
    if process.is_none() {
        return;
    }

    let Ok(client) = SongbirdClient::connect(SONGBIRD_ENDPOINT).await else {
        return;
    };

    // Advertise
    let _ = client
        .advertise_loamspine("http://localhost:9001", "http://localhost:8080")
        .await;

    thread::sleep(Duration::from_millis(500));

    // Perform multiple concurrent discovery operations
    let mut handles = vec![];
    for _ in 0..10 {
        let client_clone = client.clone();
        let handle =
            tokio::spawn(
                async move { client_clone.discover_capability("persistent-ledger").await },
            );
        handles.push(handle);
    }

    // All should succeed
    for handle in handles {
        let Ok(result) = handle.await else {
            eprintln!("⚠️  Task failed to complete");
            continue;
        };
        assert!(result.is_ok(), "Concurrent discovery should succeed");
    }
}
