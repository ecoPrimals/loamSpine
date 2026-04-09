// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use std::collections::HashMap;

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
async fn test_discover_via_environment() {
    // Phase 1: clean overrides — ensure no signing endpoints exist
    let mut discovery = InfantDiscovery::with_config(DiscoveryConfig::default()).unwrap();
    let services = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert!(services.is_empty());

    // Phase 2: set override and rediscover
    discovery.clear_cache().await;
    discovery.config.env_overrides.insert(
        "CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT".into(),
        "http://localhost:8001".into(),
    );
    let services = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].endpoint, "http://localhost:8001");
    assert_eq!(services[0].discovered_via, "environment");
}

#[tokio::test]
async fn test_degraded_mode_when_no_services() {
    let discovery = InfantDiscovery::with_config(DiscoveryConfig::default()).unwrap();
    let services = discovery.find_capability("content-storage").await.unwrap();
    assert!(services.is_empty());
}

#[tokio::test]
async fn test_cache_functionality() {
    let mut config = DiscoveryConfig::default();
    config.env_overrides.insert(
        "CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT".into(),
        "http://localhost:8001".into(),
    );
    let discovery = InfantDiscovery::with_config(config).unwrap();

    let services1 = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services1.len(), 1);

    let services2 = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services2.len(), 1);

    discovery.clear_cache().await;

    let services3 = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services3.len(), 1);
}

#[tokio::test]
async fn test_discover_via_signing_service_url() {
    let mut discovery = InfantDiscovery::with_config(DiscoveryConfig::default()).unwrap();
    let services = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert!(services.is_empty());

    discovery.clear_cache().await;
    discovery
        .config
        .env_overrides
        .insert("SIGNING_SERVICE_URL".into(), "http://localhost:8002".into());
    let services = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].endpoint, "http://localhost:8002");
}

#[test]
fn test_discovery_config_default() {
    let config = DiscoveryConfig::default();
    assert!(!config.methods.is_empty());
    assert!(config.cache_ttl_secs > 0);
    assert!(config.retry_attempts > 0);
}

#[test]
fn test_discovery_config_from_env() {
    let config = DiscoveryConfig::from_explicit(Some("http://registry.example.com"), Some(600));
    assert!(config.methods.iter().any(|m| matches!(
        m,
        DiscoveryProtocol::ServiceRegistry(url) if url == "http://registry.example.com"
    )));
    assert_eq!(config.cache_ttl_secs, 600);
}

#[test]
fn test_capability_to_srv_name_indirect() {
    let discovery = InfantDiscovery::new().unwrap();
    let capabilities = discovery.own_capabilities();
    assert!(!capabilities.is_empty());
}

#[tokio::test]
async fn test_service_registry_discovery_returns_empty() {
    let config = DiscoveryConfig::from_explicit(Some("http://registry.test"), None);
    let discovery = InfantDiscovery::with_config(config).unwrap();
    discovery.clear_cache().await;

    let services = discovery.find_capability("signing").await.unwrap();
    assert!(services.is_empty());
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
        backends::capability_to_srv_name("cryptographic-signing"),
        "_signing._tcp.local"
    );
    assert_eq!(
        backends::capability_to_srv_name("content-storage"),
        "_storage._tcp.local"
    );
    assert_eq!(
        backends::capability_to_srv_name("simple"),
        "_simple._tcp.local"
    );
}

#[test]
fn test_own_capabilities_are_loamspine_specific() {
    let discovery = InfantDiscovery::new().unwrap();
    let caps = discovery.own_capabilities();
    let identifiers: Vec<&str> = caps.iter().map(LoamSpineCapability::identifier).collect();
    assert!(
        identifiers
            .iter()
            .any(|id| id.contains("ledger") || id.contains("permanence"))
    );
}

#[tokio::test]
async fn test_cache_hit_with_stale_services_triggers_rediscovery() {
    let mut config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::Environment],
        cache_ttl_secs: 0,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        env_overrides: HashMap::new(),
    };
    config
        .env_overrides
        .insert("SIGNING_SERVICE_URL".into(), "http://localhost:9999".into());
    let discovery = InfantDiscovery::with_config(config).unwrap();

    let services = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services.len(), 1);

    let services2 = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services2.len(), 1);
}

#[test]
fn test_discovery_config_from_env_invalid_ttl_uses_default() {
    let config = DiscoveryConfig::from_explicit(None, None);
    assert_eq!(
        config.cache_ttl_secs, 300,
        "invalid TTL should leave default"
    );
}

#[test]
fn test_capability_to_srv_name_all_known_capabilities() {
    assert_eq!(
        backends::capability_to_srv_name("session-management"),
        "_session._tcp.local"
    );
    assert_eq!(
        backends::capability_to_srv_name("compute-orchestration"),
        "_compute._tcp.local"
    );
    assert_eq!(
        backends::capability_to_srv_name("service-discovery"),
        "_discovery._tcp.local"
    );
}

#[test]
fn test_capability_to_srv_name_unknown_uses_last_segment() {
    assert_eq!(
        backends::capability_to_srv_name("custom-capability"),
        "_capability._tcp.local"
    );
    assert_eq!(
        backends::capability_to_srv_name("single"),
        "_single._tcp.local"
    );
}

#[tokio::test]
async fn test_is_fresh_with_zero_ttl() {
    let service = DiscoveredService {
        id: "test".to_string(),
        capability: "signing".to_string(),
        endpoint: "http://localhost:8001".to_string(),
        discovered_via: "test".to_string(),
        metadata: HashMap::new(),
        health: ServiceHealth::Healthy,
        discovered_at: SystemTime::now(),
        ttl_secs: 0,
    };
    assert!(
        !InfantDiscovery::is_fresh(&service),
        "zero TTL should be stale"
    );
}

#[tokio::test]
async fn test_all_discovered_returns_populated_cache() {
    let mut config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::Environment],
        cache_ttl_secs: 300,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        env_overrides: HashMap::new(),
    };
    config.env_overrides.insert(
        "CAPABILITY_TEST_ENDPOINT".into(),
        "http://localhost:1234".into(),
    );
    let discovery = InfantDiscovery::with_config(config).unwrap();

    let _ = discovery.find_capability("test").await.unwrap();

    let all = discovery.all_discovered().await;
    assert!(all.contains_key("test"));
    assert_eq!(all["test"].len(), 1);
}

#[test]
fn test_discovery_method_equality() {
    assert_eq!(
        DiscoveryProtocol::Environment,
        DiscoveryProtocol::Environment
    );
    assert_eq!(DiscoveryProtocol::MDns, DiscoveryProtocol::MDns);
    assert_eq!(DiscoveryProtocol::DnsSrv, DiscoveryProtocol::DnsSrv);
    assert_ne!(DiscoveryProtocol::Environment, DiscoveryProtocol::DnsSrv);
    assert_eq!(
        DiscoveryProtocol::ServiceRegistry("http://a".into()),
        DiscoveryProtocol::ServiceRegistry("http://a".into())
    );
    assert_ne!(
        DiscoveryProtocol::ServiceRegistry("http://a".into()),
        DiscoveryProtocol::ServiceRegistry("http://b".into())
    );
}

#[tokio::test]
async fn test_discover_via_environment_pattern2_service_url() {
    let mut config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::Environment],
        cache_ttl_secs: 300,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        env_overrides: HashMap::new(),
    };
    config
        .env_overrides
        .insert("STORAGE_SERVICE_URL".into(), "http://localhost:7777".into());
    let discovery = InfantDiscovery::with_config(config).unwrap();

    let services = discovery.find_capability("content-storage").await.unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].endpoint, "http://localhost:7777");
    assert_eq!(services[0].discovered_via, "environment");
}

#[tokio::test]
async fn test_cache_hit_with_fresh_services_skips_rediscovery() {
    let mut config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::Environment],
        cache_ttl_secs: 3600,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        env_overrides: HashMap::new(),
    };
    config
        .env_overrides
        .insert("SIGNING_SERVICE_URL".into(), "http://localhost:1111".into());
    let mut discovery = InfantDiscovery::with_config(config).unwrap();
    let services1 = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services1.len(), 1);
    assert_eq!(services1[0].endpoint, "http://localhost:1111");

    // Overrides now point at 2222, but cache should return the original 1111 without rediscovering.
    discovery
        .config
        .env_overrides
        .insert("SIGNING_SERVICE_URL".into(), "http://localhost:2222".into());
    let services2 = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services2.len(), 1);
    assert_eq!(
        services2[0].endpoint, "http://localhost:1111",
        "should use cached value, not re-read env"
    );
}

#[tokio::test]
async fn test_with_config_custom_timeout() {
    let config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::DnsSrv],
        cache_ttl_secs: 60,
        retry_attempts: 1,
        discovery_timeout: Duration::from_millis(100),
        ..DiscoveryConfig::default()
    };
    let discovery = InfantDiscovery::with_config(config).unwrap();
    assert_eq!(
        discovery.own_capabilities().len(),
        LoamSpineCapability::introspect().len()
    );
}

#[tokio::test]
async fn test_mdns_not_enabled_returns_empty() {
    let config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::MDns],
        cache_ttl_secs: 300,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        ..DiscoveryConfig::default()
    };
    let discovery = InfantDiscovery::with_config(config).unwrap();
    let services = discovery.find_capability("signing").await.unwrap();
    // Without mdns feature (or with it but no LAN services), should be empty
    assert!(services.is_empty());
}
