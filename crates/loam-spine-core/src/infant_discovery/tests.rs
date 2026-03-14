// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
use serial_test::serial;

#[tokio::test]
async fn test_infant_starts_with_zero_knowledge() {
    let discovery = InfantDiscovery::new().unwrap();

    // Should know only its own capabilities
    assert!(!discovery.own_capabilities().is_empty());

    // Should have no discovered services initially
    let all = discovery.all_discovered().await;
    assert!(all.is_empty());
}

#[tokio::test]
#[serial]
async fn test_discover_via_environment() {
    // Clean environment first
    env::remove_var("CAPABILITY_SIGNING_ENDPOINT");
    env::remove_var("CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT");
    env::remove_var("SIGNING_SERVICE_URL");

    let discovery = InfantDiscovery::new().unwrap();

    // Should NOT find anything initially
    let services = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert!(services.is_empty());

    // Now set the environment variable
    env::set_var(
        "CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT",
        "http://localhost:8001",
    );

    // Clear cache to force rediscovery
    discovery.clear_cache().await;

    let services = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();

    assert_eq!(services.len(), 1);
    assert_eq!(services[0].endpoint, "http://localhost:8001");
    assert_eq!(services[0].discovered_via, "environment");

    // Cleanup
    env::remove_var("CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT");
}

#[tokio::test]
#[serial]
async fn test_degraded_mode_when_no_services() {
    // Don't set any environment variables
    env::remove_var("CAPABILITY_STORAGE_ENDPOINT");

    let discovery = InfantDiscovery::new().unwrap();
    let services = discovery.find_capability("content-storage").await.unwrap();

    // Should return empty, not error (graceful degradation)
    assert!(services.is_empty());
}

#[tokio::test]
#[serial]
async fn test_cache_functionality() {
    // Clean up any existing env vars first
    env::remove_var("CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT");
    env::remove_var("CAPABILITY_SIGNING_ENDPOINT");
    env::remove_var("SIGNING_SERVICE_URL");

    env::set_var(
        "CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT",
        "http://localhost:8001",
    );

    let discovery = InfantDiscovery::new().unwrap();

    // First discovery
    let services1 = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services1.len(), 1);

    // Second discovery (should hit cache)
    let services2 = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services2.len(), 1);

    // Clear cache
    discovery.clear_cache().await;

    // Third discovery (cache cleared, should rediscover)
    let services3 = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services3.len(), 1);

    // Clean up
    env::remove_var("CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT");
}

#[tokio::test]
#[serial]
async fn test_discover_via_signing_service_url() {
    env::remove_var("CAPABILITY_SIGNING_ENDPOINT");
    env::remove_var("CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT");
    env::remove_var("SIGNING_SERVICE_URL");

    let discovery = InfantDiscovery::new().unwrap();
    let services = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert!(services.is_empty());

    env::set_var("SIGNING_SERVICE_URL", "http://localhost:8002");
    discovery.clear_cache().await;

    let services = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].endpoint, "http://localhost:8002");

    env::remove_var("SIGNING_SERVICE_URL");
}

#[test]
fn test_discovery_config_default() {
    let config = DiscoveryConfig::default();
    assert!(!config.methods.is_empty());
    assert!(config.cache_ttl_secs > 0);
    assert!(config.retry_attempts > 0);
}

#[test]
#[serial]
fn test_discovery_config_from_env() {
    env::set_var("SERVICE_REGISTRY_URL", "http://registry.example.com");
    env::set_var("DISCOVERY_CACHE_TTL", "600");

    let config = DiscoveryConfig::from_env_or_default();
    assert!(config.methods.iter().any(|m| matches!(
        m,
        DiscoveryMethod::ServiceRegistry(url) if url == "http://registry.example.com"
    )));
    assert_eq!(config.cache_ttl_secs, 600);

    env::remove_var("SERVICE_REGISTRY_URL");
    env::remove_var("DISCOVERY_CACHE_TTL");
}

#[test]
fn test_capability_to_srv_name_indirect() {
    let discovery = InfantDiscovery::new().unwrap();
    let capabilities = discovery.own_capabilities();
    assert!(!capabilities.is_empty());
}

#[tokio::test]
#[serial]
async fn test_service_registry_discovery_returns_empty() {
    env::remove_var("CAPABILITY_SIGNING_ENDPOINT");
    env::remove_var("SIGNING_SERVICE_URL");
    env::set_var("SERVICE_REGISTRY_URL", "http://registry.test");

    let config = DiscoveryConfig::from_env_or_default();
    let discovery = InfantDiscovery::with_config(config).unwrap();
    discovery.clear_cache().await;

    let services = discovery.find_capability("signing").await.unwrap();
    assert!(services.is_empty());

    env::remove_var("SERVICE_REGISTRY_URL");
}

#[tokio::test]
async fn test_dns_srv_discovery_no_records() {
    let discovery = InfantDiscovery::new().unwrap();
    let services = discovery
        .discover_via_dns_srv("nonexistent-capability")
        .await;
    assert!(services.is_empty());
}

#[tokio::test]
async fn test_registry_discovery_unreachable() {
    let discovery = InfantDiscovery::new().unwrap();
    let services = discovery
        .discover_via_registry("http://127.0.0.1:1", "signing")
        .await;
    assert!(services.is_empty());
}

#[tokio::test]
async fn test_is_fresh_with_recent_service() {
    let service = DiscoveredService {
        id: "test".to_string(),
        capability: "signing".to_string(),
        endpoint: "http://localhost:8001".to_string(),
        discovered_via: "test".to_string(),
        metadata: HashMap::new(),
        health: ServiceHealth::Healthy,
        discovered_at: SystemTime::now(),
        ttl_secs: 300,
    };
    assert!(InfantDiscovery::is_fresh(&service));
}

#[tokio::test]
async fn test_is_fresh_with_expired_service() {
    let service = DiscoveredService {
        id: "test".to_string(),
        capability: "signing".to_string(),
        endpoint: "http://localhost:8001".to_string(),
        discovered_via: "test".to_string(),
        metadata: HashMap::new(),
        health: ServiceHealth::Healthy,
        discovered_at: SystemTime::now() - std::time::Duration::from_secs(600),
        ttl_secs: 300,
    };
    assert!(!InfantDiscovery::is_fresh(&service));
}

#[test]
fn test_capability_to_srv_name() {
    assert_eq!(
        capability_to_srv_name("cryptographic-signing"),
        "_signing._tcp.local"
    );
    assert_eq!(
        capability_to_srv_name("content-storage"),
        "_storage._tcp.local"
    );
    assert_eq!(capability_to_srv_name("simple"), "_simple._tcp.local");
}

#[test]
fn test_own_capabilities_are_loamspine_specific() {
    let discovery = InfantDiscovery::new().unwrap();
    let caps = discovery.own_capabilities();
    let identifiers: Vec<&str> = caps.iter().map(LoamSpineCapability::identifier).collect();
    assert!(identifiers
        .iter()
        .any(|id| id.contains("ledger") || id.contains("permanence")));
}
