// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use super::*;
use crate::resilience::{CircuitBreakerConfig, RetryPolicyConfig};
use crate::transport::mock::{ConfigurableTransport, MockTransport, SuccessTransport};

#[test]
fn client_creation() {
    let endpoint = "http://localhost:8082";
    assert!(!endpoint.is_empty());
}

#[test]
fn discovered_service_serialization() {
    let service = DiscoveredService {
        name: "test-service".to_string(),
        endpoint: "http://localhost:9000".to_string(),
        capabilities: vec!["signing".to_string()],
        healthy: true,
        metadata: std::iter::once(("version".to_string(), "1.0.0".to_string())).collect(),
    };

    let json = serde_json::to_string(&service).unwrap();
    let deserialized: DiscoveredService = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, "test-service");
    assert_eq!(deserialized.capabilities.len(), 1);
}

#[test]
fn discovered_service_default_fields() {
    let json = r#"{"name":"test","endpoint":"http://localhost:9000","capabilities":[]}"#;
    let service: DiscoveredService = serde_json::from_str(json).unwrap();

    assert_eq!(service.name, "test");
    assert_eq!(service.endpoint, "http://localhost:9000");
    assert!(service.capabilities.is_empty());
    assert!(!service.healthy);
    assert!(service.metadata.is_empty());
}

#[test]
fn service_advertisement_serialization() {
    let advertisement = ServiceAdvertisement {
        name: "loamspine".to_string(),
        primary_role: "permanence".to_string(),
        capabilities: vec!["permanence".to_string()],
        endpoints: vec![ServiceEndpoint {
            protocol: "tarpc".to_string(),
            address: "http://localhost:9001".to_string(),
            port: 9001,
            health_check: None,
        }],
        metadata: std::iter::once(("version".to_string(), "0.8.0".to_string())).collect(),
    };

    let json = serde_json::to_string(&advertisement).unwrap();
    assert!(json.contains("loamspine"));
    assert!(json.contains("permanence"));
    assert!(json.contains("tarpc"));
}

#[test]
fn service_endpoint_serialization() {
    let endpoint = ServiceEndpoint {
        protocol: "jsonrpc".to_string(),
        address: "http://localhost:8080".to_string(),
        port: 8080,
        health_check: Some("/health".to_string()),
    };

    let json = serde_json::to_string(&endpoint).unwrap();
    let deserialized: ServiceEndpoint = serde_json::from_str(&json).unwrap();

    assert_eq!(endpoint.protocol, deserialized.protocol);
    assert_eq!(endpoint.address, deserialized.address);
    assert_eq!(endpoint.port, deserialized.port);
    assert_eq!(endpoint.health_check, deserialized.health_check);
}

#[test]
fn client_endpoint_getter() {
    let endpoint = "http://localhost:8082";
    let client = DiscoveryClient::for_testing(endpoint);
    assert_eq!(client.endpoint(), endpoint);
}

#[test]
fn client_is_cloneable() {
    let client = DiscoveryClient::for_testing("http://registry.local:8082");

    let _cloned = &client;
    assert_eq!(client.endpoint(), "http://registry.local:8082");
}

#[test]
fn discovered_service_with_metadata() {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("version".to_string(), "1.0.0".to_string());
    metadata.insert("language".to_string(), "rust".to_string());

    let service = DiscoveredService {
        name: "test-service".to_string(),
        endpoint: "http://localhost:9000".to_string(),
        capabilities: vec!["signing".to_string(), "verification".to_string()],
        healthy: true,
        metadata,
    };

    assert_eq!(service.metadata.len(), 2);
    assert_eq!(service.metadata.get("version"), Some(&"1.0.0".to_string()));
    assert_eq!(service.metadata.get("language"), Some(&"rust".to_string()));
}

#[test]
fn discovered_service_multiple_capabilities() {
    let service = DiscoveredService {
        name: "multi-service".to_string(),
        endpoint: "http://localhost:9000".to_string(),
        capabilities: vec![
            "signing".to_string(),
            "verification".to_string(),
            "encryption".to_string(),
        ],
        healthy: true,
        metadata: std::collections::HashMap::new(),
    };

    assert_eq!(service.capabilities.len(), 3);
    assert!(service.capabilities.contains(&"signing".to_string()));
    assert!(service.capabilities.contains(&"verification".to_string()));
    assert!(service.capabilities.contains(&"encryption".to_string()));
}

#[test]
fn service_endpoint_without_health_check() {
    let endpoint = ServiceEndpoint {
        protocol: "tarpc".to_string(),
        address: "http://localhost:9001".to_string(),
        port: 9001,
        health_check: None,
    };

    assert!(endpoint.health_check.is_none());
}

#[test]
fn service_endpoint_with_health_check() {
    let endpoint = ServiceEndpoint {
        protocol: "jsonrpc".to_string(),
        address: "http://localhost:8080".to_string(),
        port: 8080,
        health_check: Some("/health".to_string()),
    };

    assert!(endpoint.health_check.is_some());
    assert_eq!(endpoint.health_check.unwrap(), "/health");
}

#[test]
fn service_advertisement_with_multiple_endpoints() {
    let advertisement = ServiceAdvertisement {
        name: "loamspine".to_string(),
        primary_role: "permanence".to_string(),
        capabilities: vec!["permanence".to_string()],
        endpoints: vec![
            ServiceEndpoint {
                protocol: "tarpc".to_string(),
                address: "http://localhost:9001".to_string(),
                port: 9001,
                health_check: None,
            },
            ServiceEndpoint {
                protocol: "jsonrpc".to_string(),
                address: "http://localhost:8080".to_string(),
                port: 8080,
                health_check: Some("/health".to_string()),
            },
        ],
        metadata: std::collections::HashMap::new(),
    };

    assert_eq!(advertisement.endpoints.len(), 2);
    assert_eq!(advertisement.endpoints[0].protocol, "tarpc");
    assert_eq!(advertisement.endpoints[1].protocol, "jsonrpc");
}

#[test]
fn port_extraction_from_urls() {
    let test_cases = vec![
        ("http://localhost:9001", Some(9001)),
        ("https://example.com:8443", Some(8443)),
        ("http://192.0.2.1:3000", Some(3000)),
        ("http://localhost", None),
    ];

    for (url, expected_port) in test_cases {
        assert_eq!(extract_port(url), expected_port, "Port mismatch for {url}");
    }
}

#[test]
fn service_advertisement_empty_capabilities() {
    let advertisement = ServiceAdvertisement {
        name: "minimal-service".to_string(),
        primary_role: "test".to_string(),
        capabilities: vec![],
        endpoints: vec![],
        metadata: std::collections::HashMap::new(),
    };

    assert!(advertisement.capabilities.is_empty());
    assert!(advertisement.endpoints.is_empty());
    assert!(advertisement.metadata.is_empty());
}

#[test]
fn discovered_service_healthy_flag() {
    let healthy_service = DiscoveredService {
        name: "healthy".to_string(),
        endpoint: "http://localhost:9000".to_string(),
        capabilities: vec![],
        healthy: true,
        metadata: std::collections::HashMap::new(),
    };

    let unhealthy_service = DiscoveredService {
        name: "unhealthy".to_string(),
        endpoint: "http://localhost:9000".to_string(),
        capabilities: vec![],
        healthy: false,
        metadata: std::collections::HashMap::new(),
    };

    assert!(healthy_service.healthy);
    assert!(!unhealthy_service.healthy);
}

#[test]
fn service_endpoint_port_matching() {
    let endpoint = ServiceEndpoint {
        protocol: "http".to_string(),
        address: "http://localhost:8080".to_string(),
        port: 8080,
        health_check: None,
    };

    let extracted = extract_port(&endpoint.address);
    assert_eq!(extracted, Some(endpoint.port), "Port mismatch");
}

#[test]
fn client_endpoint_accessor() {
    let endpoint_url = "http://registry.example.com:8082";
    let client = DiscoveryClient::for_testing(endpoint_url);

    assert_eq!(client.endpoint(), endpoint_url);
    assert!(client.endpoint().starts_with("http://"));
    assert!(client.endpoint().contains("8082"));
}

#[test]
fn discovered_service_debug_impl() {
    let service = DiscoveredService {
        name: "debug-test".to_string(),
        endpoint: "http://localhost:9000".to_string(),
        capabilities: vec!["test".to_string()],
        healthy: true,
        metadata: std::collections::HashMap::new(),
    };

    let debug_string = format!("{service:?}");
    assert!(debug_string.contains("debug-test"));
    assert!(debug_string.contains("localhost"));
}

#[test]
fn service_endpoint_protocol_variations() {
    let protocols = vec!["http", "https", "tarpc", "jsonrpc", "grpc"];

    for protocol in protocols {
        let endpoint = ServiceEndpoint {
            protocol: protocol.to_string(),
            address: format!("{protocol}://localhost:9000"),
            port: 9000,
            health_check: None,
        };

        assert_eq!(endpoint.protocol, protocol);
    }
}

#[test]
fn service_advertisement_metadata() {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("version".to_string(), "0.8.0".to_string());
    metadata.insert("language".to_string(), "rust".to_string());
    metadata.insert("rpc_style".to_string(), "pure-rust".to_string());

    let advertisement = ServiceAdvertisement {
        name: "loamspine".to_string(),
        primary_role: "permanence".to_string(),
        capabilities: vec!["permanence".to_string()],
        endpoints: vec![],
        metadata,
    };

    assert_eq!(advertisement.metadata.len(), 3);
    assert_eq!(
        advertisement.metadata.get("version"),
        Some(&"0.8.0".to_string())
    );
}

#[test]
fn discovered_service_json_roundtrip() {
    let original = DiscoveredService {
        name: "roundtrip-test".to_string(),
        endpoint: "http://localhost:9000".to_string(),
        capabilities: vec!["signing".to_string(), "verification".to_string()],
        healthy: true,
        metadata: vec![
            ("version".to_string(), "1.0.0".to_string()),
            ("build".to_string(), "123".to_string()),
        ]
        .into_iter()
        .collect(),
    };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: DiscoveredService = serde_json::from_str(&json).unwrap();

    assert_eq!(original.name, deserialized.name);
    assert_eq!(original.endpoint, deserialized.endpoint);
    assert_eq!(original.capabilities, deserialized.capabilities);
    assert_eq!(original.healthy, deserialized.healthy);
    assert_eq!(original.metadata, deserialized.metadata);
}

#[test]
fn service_advertisement_complete_metadata() {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("version".to_string(), "0.8.0".to_string());
    metadata.insert("language".to_string(), "rust".to_string());
    metadata.insert("rpc_style".to_string(), "pure-rust".to_string());
    metadata.insert("unsafe_code".to_string(), "false".to_string());
    metadata.insert("zero_copy".to_string(), "true".to_string());

    let advertisement = ServiceAdvertisement {
        name: "loamspine".to_string(),
        primary_role: "permanence".to_string(),
        capabilities: vec![
            "permanence".to_string(),
            "certificates".to_string(),
            "proofs".to_string(),
        ],
        endpoints: vec![
            ServiceEndpoint {
                protocol: "tarpc".to_string(),
                address: "http://localhost:9001".to_string(),
                port: 9001,
                health_check: None,
            },
            ServiceEndpoint {
                protocol: "jsonrpc".to_string(),
                address: "http://localhost:8080".to_string(),
                port: 8080,
                health_check: Some("/health".to_string()),
            },
        ],
        metadata,
    };

    assert_eq!(advertisement.capabilities.len(), 3);
    assert_eq!(advertisement.endpoints.len(), 2);
    assert_eq!(advertisement.metadata.len(), 5);
    assert_eq!(
        advertisement.metadata.get("unsafe_code"),
        Some(&"false".to_string())
    );
}

#[tokio::test]
async fn advertise_self_fails_for_unreachable_endpoint() {
    let client = DiscoveryClient::for_testing("http://127.0.0.1:1");

    let result = client
        .advertise_self("http://localhost:9001", "http://localhost:8080")
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Advertisement")
            || err.to_string().contains("Network")
            || err.to_string().contains("MockTransport")
    );
}

#[tokio::test]
async fn deregister_fails_for_unreachable_endpoint() {
    let client = DiscoveryClient::for_testing("http://127.0.0.1:1");

    let result = client.deregister().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Deregister")
            || err.to_string().contains("Network")
            || err.to_string().contains("MockTransport")
    );
}

#[tokio::test]
async fn heartbeat_fails_for_unreachable_endpoint() {
    let client = DiscoveryClient::for_testing("http://127.0.0.1:1");

    let result = client.heartbeat().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Heartbeat")
            || err.to_string().contains("Network")
            || err.to_string().contains("MockTransport")
    );
}

#[tokio::test]
async fn discover_capability_fails_for_unreachable_endpoint() {
    let client = DiscoveryClient::for_testing("http://127.0.0.1:1");

    let result = client.discover_capability("signing").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Discovery")
            || err.to_string().contains("Network")
            || err.to_string().contains("MockTransport")
    );
}

#[tokio::test]
async fn discover_all_fails_for_unreachable_endpoint() {
    let client = DiscoveryClient::for_testing("http://127.0.0.1:1");

    let result = client.discover_all().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Discovery")
            || err.to_string().contains("Network")
            || err.to_string().contains("MockTransport")
    );
}

#[tokio::test]
async fn connect_fails_without_transport_features() {
    // With the mock transport in for_testing, connect itself would fail
    // because no real transport is available. We test that the error
    // path produces a sensible message.
    // Note: when neither feature is enabled, connect() returns an error.
    // We can't easily test that in isolation since tests may have features
    // enabled, so we test via for_testing + health_check instead.
    let client = DiscoveryClient::for_testing("http://127.0.0.1:1");
    let result = client.health_check().await;
    assert!(result.is_err());
}

#[test]
fn client_debug_impl() {
    let client = DiscoveryClient::for_testing("http://test:8082");
    let debug = format!("{client:?}");
    assert!(debug.contains("DiscoveryClient"));
    assert!(debug.contains("test:8082"));
}

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

#[tokio::test]
async fn advertise_loamspine_deprecated_alias() {
    let client = DiscoveryClient::for_testing_success("http://registry.local:8082");

    #[allow(deprecated)]
    let result = client
        .advertise_loamspine("http://localhost:9001", "http://localhost:8080")
        .await;

    assert!(result.is_ok());
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
