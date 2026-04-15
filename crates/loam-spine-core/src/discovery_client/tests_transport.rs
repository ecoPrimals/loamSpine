// SPDX-License-Identifier: AGPL-3.0-or-later

//! Transport layer, status-code handling, JSON parse edge-cases, and
//! resilient discovery client tests.
//!
//! Extracted from `tests.rs` — these tests exercise the HTTP transport
//! abstraction layer (ConfigurableTransport, SuccessTransport), non-200
//! status handling, invalid JSON responses, and the circuit-breaker
//! resilient wrapper.

use std::sync::Arc;

use super::*;
use crate::resilience::{CircuitBreakerConfig, RetryPolicyConfig};
use crate::transport::mock::{ConfigurableTransport, MockTransport, SuccessTransport};

// ─────────────────────────────────────────────────────────────────────────────
// Constructor and transport tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn connect_with_transport_success() {
    let transport = Arc::new(SuccessTransport::new());
    let client =
        DiscoveryClient::connect_with_transport("http://registry.local:8082", transport).await;

    assert!(client.is_ok());
    let client = client.unwrap();
    assert_eq!(client.endpoint(), "http://registry.local:8082");
}

#[tokio::test]
async fn connect_with_transport_health_check_fails() {
    let transport = Arc::new(MockTransport::new("http://registry.local:8082"));
    let result =
        DiscoveryClient::connect_with_transport("http://registry.local:8082", transport).await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("unavailable") || err.contains("registry") || err.contains("MockTransport"),
        "Expected health check error: {err}",
    );
}

#[test]
fn for_testing_success_constructor() {
    let client = DiscoveryClient::for_testing_success("http://registry.local:8082");
    assert_eq!(client.endpoint(), "http://registry.local:8082");
}

// ─────────────────────────────────────────────────────────────────────────────
// Success paths with SuccessTransport
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn advertise_self_success() {
    let client = DiscoveryClient::for_testing_success("http://registry.local:8082");

    let result = client
        .advertise_self("http://localhost:9001", "http://localhost:8080")
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn advertise_self_with_urls_without_port_uses_defaults() {
    let client = DiscoveryClient::for_testing_success("http://registry.local:8082");

    let result = client
        .advertise_self("http://localhost", "http://localhost")
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn heartbeat_success() {
    let client = DiscoveryClient::for_testing_success("http://registry.local:8082");

    let result = client.heartbeat().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn deregister_success() {
    let client = DiscoveryClient::for_testing_success("http://registry.local:8082");

    let result = client.deregister().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn discover_capability_success() {
    let client = DiscoveryClient::for_testing_success("http://registry.local:8082");

    let result = client.discover_capability("signing").await;

    assert!(result.is_ok());
    let services = result.unwrap();
    assert!(services.is_empty());
}

#[tokio::test]
async fn discover_all_success() {
    let client = DiscoveryClient::for_testing_success("http://registry.local:8082");

    let result = client.discover_all().await;

    assert!(result.is_ok());
    let services = result.unwrap();
    assert!(services.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// Non-success status handling
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn discover_capability_returns_error_on_non_success_status() {
    let transport = Arc::new(ConfigurableTransport::status_code(404));
    let client =
        DiscoveryClient::for_testing_with_transport("http://registry.local:8082", transport);

    let result = client.discover_capability("signing").await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("404") || err.contains("Discovery"),
        "Expected status error: {err}"
    );
}

#[tokio::test]
async fn discover_all_returns_error_on_non_success_status() {
    let transport = Arc::new(ConfigurableTransport::status_code(500));
    let client =
        DiscoveryClient::for_testing_with_transport("http://registry.local:8082", transport);

    let result = client.discover_all().await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("500") || err.contains("Discovery"),
        "Expected status error: {err}"
    );
}

#[tokio::test]
async fn advertise_self_returns_error_on_non_success_status() {
    let transport = Arc::new(ConfigurableTransport::status_code(503));
    let client =
        DiscoveryClient::for_testing_with_transport("http://registry.local:8082", transport);

    let result = client
        .advertise_self("http://localhost:9001", "http://localhost:8080")
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("503") || err.contains("Advertisement"),
        "Expected status error: {err}",
    );
}

#[tokio::test]
async fn heartbeat_returns_error_on_non_success_status() {
    let transport = Arc::new(ConfigurableTransport::status_code(429));
    let client =
        DiscoveryClient::for_testing_with_transport("http://registry.local:8082", transport);

    let result = client.heartbeat().await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("429") || err.contains("Heartbeat"),
        "Expected status error: {err}"
    );
}

#[tokio::test]
async fn deregister_returns_error_on_non_success_status() {
    let transport = Arc::new(ConfigurableTransport::status_code(500));
    let client =
        DiscoveryClient::for_testing_with_transport("http://registry.local:8082", transport);

    let result = client.deregister().await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("500") || err.contains("Deregister"),
        "Expected status error: {err}",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// HTTP response parsing edge cases (invalid JSON)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn discover_capability_fails_on_invalid_json() {
    let transport = Arc::new(ConfigurableTransport::invalid_json());
    let client =
        DiscoveryClient::for_testing_with_transport("http://registry.local:8082", transport);

    let result = client.discover_capability("signing").await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("parse") || err.contains("JSON") || err.contains("Failed"),
        "Expected JSON parse error: {err}",
    );
}

#[tokio::test]
async fn discover_all_fails_on_invalid_json() {
    let transport = Arc::new(ConfigurableTransport::invalid_json());
    let client =
        DiscoveryClient::for_testing_with_transport("http://registry.local:8082", transport);

    let result = client.discover_all().await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("parse") || err.contains("JSON") || err.contains("Failed"),
        "Expected JSON parse error: {err}",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Resilient discovery client (circuit breaker + retry wrapper)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn resilient_discovery_client_success() {
    let client = DiscoveryClient::for_testing_success("http://registry.local:8082")
        .with_resilience(
            CircuitBreakerConfig::default(),
            RetryPolicyConfig::default(),
        );

    let result = client.discover_capability("signing").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());

    let result = client
        .advertise_self("http://localhost:9001", "http://localhost:8080")
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn resilient_discovery_client_circuit_opens_on_failures() {
    let client = DiscoveryClient::for_testing("http://127.0.0.1:1").with_resilience(
        CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout_secs: 3600,
            success_threshold: 2,
        },
        RetryPolicyConfig {
            max_retries: 1,
            base_delay_ms: 1,
            max_delay_ms: 10,
        },
    );

    let _ = client.discover_capability("signing").await;
    let _ = client.discover_capability("signing").await;
    let result = client.discover_capability("signing").await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("circuit breaker") || err_msg.contains("unavailable"),
        "expected circuit breaker or unavailable, got: {err_msg}"
    );
}

#[tokio::test]
async fn discover_capability_returns_services_when_valid_json() {
    let transport = Arc::new(ConfigurableTransport::new(
        200,
        r#"[{"name":"signing-svc","endpoint":"http://localhost:9000","capabilities":["signing"],"healthy":true}]"#,
    ));
    let client =
        DiscoveryClient::for_testing_with_transport("http://registry.local:8082", transport);

    let result = client.discover_capability("signing").await;

    assert!(result.is_ok());
    let services = result.unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].name, "signing-svc");
    assert_eq!(services[0].endpoint, "http://localhost:9000");
    assert!(services[0].capabilities.contains(&"signing".to_string()));
    assert!(services[0].healthy);
}

#[tokio::test]
async fn discover_all_returns_services_when_valid_json() {
    let transport = Arc::new(ConfigurableTransport::new(
        200,
        r#"[{"name":"svc-a","endpoint":"http://localhost:9000","capabilities":["a"],"healthy":true},{"name":"svc-b","endpoint":"http://localhost:9001","capabilities":["b"],"healthy":false}]"#,
    ));
    let client =
        DiscoveryClient::for_testing_with_transport("http://registry.local:8082", transport);

    let result = client.discover_all().await;

    assert!(result.is_ok());
    let services = result.unwrap();
    assert_eq!(services.len(), 2);
    assert_eq!(services[0].name, "svc-a");
    assert_eq!(services[0].endpoint, "http://localhost:9000");
    assert!(services[0].healthy);
    assert_eq!(services[1].name, "svc-b");
    assert_eq!(services[1].endpoint, "http://localhost:9001");
    assert!(!services[1].healthy);
}

#[tokio::test]
async fn resilient_discovery_client_discover_all() {
    let client = DiscoveryClient::for_testing_success("http://registry.local:8082")
        .with_resilience(
            CircuitBreakerConfig::default(),
            RetryPolicyConfig::default(),
        );

    let result = client.discover_all().await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn resilient_discovery_client_heartbeat_and_deregister() {
    let client = DiscoveryClient::for_testing_success("http://registry.local:8082")
        .with_resilience(
            CircuitBreakerConfig::default(),
            RetryPolicyConfig::default(),
        );

    let result = client.heartbeat().await;
    assert!(result.is_ok());

    let result = client.deregister().await;
    assert!(result.is_ok());
}
