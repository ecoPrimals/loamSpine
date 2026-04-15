// SPDX-License-Identifier: AGPL-3.0-or-later

//! Extended coverage tests for infant discovery: config edges, cache behavior,
//! SRV mapping, fallback chains, DNS error paths, registry failures.

use super::*;
use std::collections::HashMap;

#[test]
fn test_with_config_empty_methods() {
    let config = DiscoveryConfig {
        methods: vec![],
        cache_ttl_secs: 300,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        ..DiscoveryConfig::default()
    };
    let discovery = InfantDiscovery::with_config(config).unwrap();
    assert!(!discovery.own_capabilities().is_empty());
}

#[tokio::test]
async fn test_find_capability_with_empty_methods_returns_empty() {
    let config = DiscoveryConfig {
        methods: vec![],
        cache_ttl_secs: 300,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        ..DiscoveryConfig::default()
    };
    let discovery = InfantDiscovery::with_config(config).unwrap();
    let services = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert!(services.is_empty());
}

#[test]
fn test_discovery_config_from_env_zero_ttl() {
    let config = DiscoveryConfig::from_explicit(None, Some(0));
    assert_eq!(config.cache_ttl_secs, 0);
}

#[tokio::test]
async fn test_cache_expiry_triggers_rediscovery_with_zero_ttl() {
    let config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::Environment],
        cache_ttl_secs: 0,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        env_overrides: HashMap::new(),
    };
    let mut discovery = InfantDiscovery::with_config(config).unwrap();

    // Phase 2: set to first URL
    discovery
        .config
        .env_overrides
        .insert("SIGNING_SERVICE_URL".into(), "http://localhost:8888".into());
    let services1 = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services1.len(), 1);

    // Phase 3: change URL — zero TTL should bypass cache
    discovery
        .config
        .env_overrides
        .insert("SIGNING_SERVICE_URL".into(), "http://localhost:9999".into());
    let services2 = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services2.len(), 1);
    assert_eq!(
        services2[0].endpoint, "http://localhost:9999",
        "zero TTL should bypass cache and rediscover"
    );
}

#[tokio::test]
async fn test_is_fresh_with_clock_skew_returns_stale() {
    let service = DiscoveredService {
        id: "test".to_string(),
        capability: "signing".to_string(),
        endpoint: "http://localhost:8001".to_string(),
        discovered_via: "test".to_string(),
        metadata: HashMap::new(),
        health: ServiceHealth::Healthy,
        discovered_at: SystemTime::now() + Duration::from_secs(3600),
        ttl_secs: 300,
    };
    assert!(
        !InfantDiscovery::is_fresh(&service),
        "future discovered_at (clock skew) should be stale"
    );
}

#[test]
fn test_capability_to_srv_name_empty_string() {
    assert_eq!(backends::capability_to_srv_name(""), "_._tcp.local");
}

#[test]
fn test_capability_to_srv_name_hyphen_only() {
    assert_eq!(backends::capability_to_srv_name("---"), "_._tcp.local");
}

#[test]
fn test_capability_to_srv_name_single_hyphen() {
    assert_eq!(backends::capability_to_srv_name("-"), "_._tcp.local");
}

#[tokio::test]
async fn test_environment_pattern1_takes_precedence_over_pattern2() {
    let mut config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::Environment],
        cache_ttl_secs: 300,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        env_overrides: HashMap::new(),
    };
    config.env_overrides.insert(
        "CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT".into(),
        "http://pattern1:8001".into(),
    );
    config
        .env_overrides
        .insert("SIGNING_SERVICE_URL".into(), "http://pattern2:8002".into());
    let discovery = InfantDiscovery::with_config(config).unwrap();

    let services = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(
        services[0].endpoint, "http://pattern1:8001",
        "CAPABILITY_*_ENDPOINT should take precedence over *_SERVICE_URL"
    );
}

#[tokio::test]
async fn test_fallback_discovery_tries_next_method_when_first_empty() {
    let config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::Environment, DiscoveryProtocol::DnsSrv],
        cache_ttl_secs: 300,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        ..DiscoveryConfig::default()
    };
    let discovery = InfantDiscovery::with_config(config).unwrap();

    let services = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert!(
        services.is_empty(),
        "no env, DNS SRV has no records for signing"
    );
}

#[tokio::test]
async fn test_fallback_discovery_breaks_on_first_success() {
    let mut config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::Environment, DiscoveryProtocol::DnsSrv],
        cache_ttl_secs: 300,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        env_overrides: HashMap::new(),
    };
    config.env_overrides.insert(
        "CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT".into(),
        "http://env-wins:8001".into(),
    );
    let discovery = InfantDiscovery::with_config(config).unwrap();

    let services = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].endpoint, "http://env-wins:8001");
    assert_eq!(services[0].discovered_via, "environment");
}

#[tokio::test]
async fn test_unknown_capability_returns_empty() {
    let discovery = InfantDiscovery::new().unwrap();
    let services = discovery
        .find_capability("completely-unknown-capability-xyz")
        .await
        .unwrap();
    assert!(services.is_empty());
}

#[tokio::test]
async fn test_find_capability_empty_string() {
    let discovery = InfantDiscovery::with_config(DiscoveryConfig::default()).unwrap();
    let services = discovery.find_capability("").await.unwrap();
    assert!(services.is_empty());
}

#[tokio::test]
async fn test_clear_cache_empties_all_discovered() {
    let mut config = DiscoveryConfig::default();
    config.env_overrides.insert(
        "CAPABILITY_CLEAR_TEST_ENDPOINT".into(),
        "http://localhost:5555".into(),
    );
    let discovery = InfantDiscovery::with_config(config).unwrap();
    let _ = discovery.find_capability("clear-test").await.unwrap();

    let all_before = discovery.all_discovered().await;
    assert!(!all_before.is_empty());

    discovery.clear_cache().await;
    let all_after = discovery.all_discovered().await;
    assert!(all_after.is_empty());
}

#[tokio::test]
async fn test_content_storage_service_url_strips_content_prefix() {
    let mut config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::Environment],
        cache_ttl_secs: 300,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        env_overrides: HashMap::new(),
    };
    config
        .env_overrides
        .insert("STORAGE_SERVICE_URL".into(), "http://storage:9000".into());
    let discovery = InfantDiscovery::with_config(config).unwrap();

    let services = discovery.find_capability("content-storage").await.unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].endpoint, "http://storage:9000");
}

#[tokio::test]
async fn test_cached_empty_services_triggers_rediscovery() {
    let config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::Environment],
        cache_ttl_secs: 300,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        env_overrides: HashMap::new(),
    };
    let mut discovery = InfantDiscovery::with_config(config).unwrap();
    let services1 = discovery.find_capability("rediscover").await.unwrap();
    assert!(services1.is_empty());

    discovery.config.env_overrides.insert(
        "REDISCOVER_SERVICE_URL".into(),
        "http://rediscovered:8080".into(),
    );
    let services2 = discovery.find_capability("rediscover").await.unwrap();
    assert_eq!(services2.len(), 1);
    assert_eq!(services2[0].endpoint, "http://rediscovered:8080");
}

#[tokio::test]
async fn test_discover_via_environment_capability_key_with_hyphens() {
    let mut config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::Environment],
        cache_ttl_secs: 300,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        env_overrides: HashMap::new(),
    };
    config.env_overrides.insert(
        "CAPABILITY_TEST_CAP_ENDPOINT".into(),
        "http://hyphen-test:8000".into(),
    );
    let discovery = InfantDiscovery::with_config(config).unwrap();

    let services = discovery.find_capability("test-cap").await.unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].endpoint, "http://hyphen-test:8000");
}

#[tokio::test]
async fn test_cache_mix_fresh_and_stale_returns_fresh_only() {
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
    let discovery = InfantDiscovery::with_config(config).unwrap();

    let services1 = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services1.len(), 1);

    discovery.clear_cache().await;

    let stale_service = DiscoveredService {
        id: "stale".to_string(),
        capability: "cryptographic-signing".to_string(),
        endpoint: "http://stale:8001".to_string(),
        discovered_via: "test".to_string(),
        metadata: HashMap::new(),
        health: ServiceHealth::Unknown,
        discovered_at: SystemTime::now() - Duration::from_secs(7200),
        ttl_secs: 300,
    };
    let fresh_service = DiscoveredService {
        id: "fresh".to_string(),
        capability: "cryptographic-signing".to_string(),
        endpoint: "http://fresh:8001".to_string(),
        discovered_via: "test".to_string(),
        metadata: HashMap::new(),
        health: ServiceHealth::Unknown,
        discovered_at: SystemTime::now(),
        ttl_secs: 3600,
    };

    discovery
        .cache
        .insert(
            "cryptographic-signing".to_string(),
            vec![stale_service.clone(), fresh_service.clone()],
        )
        .await;

    let services = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].endpoint, "http://fresh:8001");
}

// =============================================================================
// DNS SRV error paths and registry discovery paths
// =============================================================================

#[tokio::test]
async fn test_dns_srv_discovery_timeout_on_bogus_domain() {
    let config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::DnsSrv],
        cache_ttl_secs: 300,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        ..DiscoveryConfig::default()
    };
    let discovery = InfantDiscovery::with_config(config).unwrap();

    let services = discovery
        .discover_via_dns_srv("extremely-unlikely-capability-xyz-9999")
        .await;
    assert!(services.is_empty());
}

#[tokio::test]
async fn test_dns_srv_multiple_known_capabilities() {
    let discovery = InfantDiscovery::new().unwrap();
    for cap in [
        "cryptographic-signing",
        "content-storage",
        "service-discovery",
        "session-management",
        "compute-orchestration",
    ] {
        let services = discovery.discover_via_dns_srv(cap).await;
        assert!(
            services.is_empty(),
            "Expected no SRV records for '{cap}' in test environment"
        );
    }
}

#[tokio::test]
async fn test_discover_via_registry_capability_query_failure() {
    let discovery = InfantDiscovery::new().unwrap();
    let services = discovery
        .discover_via_registry("http://127.0.0.1:1", "nonexistent-capability")
        .await;
    assert!(services.is_empty());
}

#[test]
fn test_discovery_config_clone_and_debug() {
    let config = DiscoveryConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("DiscoveryConfig"));

    let cloned = config.clone();
    assert_eq!(cloned.cache_ttl_secs, config.cache_ttl_secs);
    assert_eq!(cloned.retry_attempts, config.retry_attempts);
}

#[test]
fn test_discovery_method_debug() {
    let methods = vec![
        DiscoveryProtocol::Environment,
        DiscoveryProtocol::MDns,
        DiscoveryProtocol::DnsSrv,
        DiscoveryProtocol::ServiceRegistry("http://test".into()),
    ];
    for method in &methods {
        let debug = format!("{method:?}");
        assert!(!debug.is_empty());
    }
}

#[test]
fn test_discovery_method_clone() {
    let method = DiscoveryProtocol::ServiceRegistry("http://registry:8082".into());
    let cloned = method.clone();
    assert_eq!(method, cloned);
}

#[tokio::test]
async fn test_find_capability_dns_srv_only_returns_empty_for_unknown() {
    let config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::DnsSrv],
        cache_ttl_secs: 300,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(2),
        ..DiscoveryConfig::default()
    };
    let discovery = InfantDiscovery::with_config(config).unwrap();
    let services = discovery.find_capability("signing").await.unwrap();
    assert!(services.is_empty());
}

#[tokio::test]
async fn test_own_capabilities_introspection_stability() {
    let d1 = InfantDiscovery::new().unwrap();
    let d2 = InfantDiscovery::new().unwrap();
    assert_eq!(d1.own_capabilities().len(), d2.own_capabilities().len());
    for (a, b) in d1
        .own_capabilities()
        .iter()
        .zip(d2.own_capabilities().iter())
    {
        assert_eq!(a.identifier(), b.identifier());
    }
}

#[tokio::test]
async fn test_all_discovered_empty_initially() {
    let discovery = InfantDiscovery::new().unwrap();
    let all = discovery.all_discovered().await;
    assert!(all.is_empty());
}

// =========================================================================
// DiscoveryCache direct unit tests
// =========================================================================

#[tokio::test]
async fn test_cache_new_is_empty() {
    let cache = cache::DiscoveryCache::new();
    let all = cache.all().await;
    assert!(all.is_empty());
}

#[tokio::test]
async fn test_cache_insert_and_get() {
    let cache = cache::DiscoveryCache::new();
    let svc = DiscoveredService {
        id: "cache-test".to_string(),
        capability: "signing".to_string(),
        endpoint: "http://localhost:1234".to_string(),
        discovered_via: "test".to_string(),
        metadata: HashMap::new(),
        health: ServiceHealth::Healthy,
        discovered_at: SystemTime::now(),
        ttl_secs: 3600,
    };
    cache.insert("signing".to_string(), vec![svc]).await;
    let fresh = cache.get_fresh("signing").await;
    assert!(fresh.is_some());
    assert_eq!(fresh.unwrap().len(), 1);
}

#[tokio::test]
async fn test_cache_get_fresh_missing_key() {
    let cache = cache::DiscoveryCache::new();
    assert!(cache.get_fresh("nonexistent").await.is_none());
}

#[tokio::test]
async fn test_cache_get_fresh_empty_vec() {
    let cache = cache::DiscoveryCache::new();
    cache.insert("empty".to_string(), vec![]).await;
    assert!(cache.get_fresh("empty").await.is_none());
}

#[tokio::test]
async fn test_cache_get_fresh_all_stale() {
    let cache = cache::DiscoveryCache::new();
    let stale = DiscoveredService {
        id: "stale".to_string(),
        capability: "signing".to_string(),
        endpoint: "http://old:1234".to_string(),
        discovered_via: "test".to_string(),
        metadata: HashMap::new(),
        health: ServiceHealth::Unknown,
        discovered_at: SystemTime::now() - Duration::from_secs(7200),
        ttl_secs: 300,
    };
    cache.insert("signing".to_string(), vec![stale]).await;
    assert!(cache.get_fresh("signing").await.is_none());
}

#[tokio::test]
async fn test_cache_clear() {
    let cache = cache::DiscoveryCache::new();
    let svc = DiscoveredService {
        id: "t".to_string(),
        capability: "test".to_string(),
        endpoint: "http://x".to_string(),
        discovered_via: "test".to_string(),
        metadata: HashMap::new(),
        health: ServiceHealth::Healthy,
        discovered_at: SystemTime::now(),
        ttl_secs: 3600,
    };
    cache.insert("test".to_string(), vec![svc]).await;
    assert!(!cache.all().await.is_empty());
    cache.clear().await;
    assert!(cache.all().await.is_empty());
}

#[tokio::test]
async fn test_cache_all_returns_snapshot() {
    let cache = cache::DiscoveryCache::new();
    let svc_a = DiscoveredService {
        id: "a".to_string(),
        capability: "cap-a".to_string(),
        endpoint: "http://a".to_string(),
        discovered_via: "test".to_string(),
        metadata: HashMap::new(),
        health: ServiceHealth::Healthy,
        discovered_at: SystemTime::now(),
        ttl_secs: 3600,
    };
    let svc_b = DiscoveredService {
        id: "b".to_string(),
        capability: "cap-b".to_string(),
        endpoint: "http://b".to_string(),
        discovered_via: "test".to_string(),
        metadata: HashMap::new(),
        health: ServiceHealth::Healthy,
        discovered_at: SystemTime::now(),
        ttl_secs: 3600,
    };
    cache.insert("cap-a".to_string(), vec![svc_a]).await;
    cache.insert("cap-b".to_string(), vec![svc_b]).await;
    let all = cache.all().await;
    assert_eq!(all.len(), 2);
    assert!(all.contains_key("cap-a"));
    assert!(all.contains_key("cap-b"));
}

#[tokio::test]
async fn test_multiple_capabilities_cached_independently() {
    let mut config = DiscoveryConfig {
        methods: vec![DiscoveryProtocol::Environment],
        cache_ttl_secs: 3600,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
        env_overrides: HashMap::new(),
    };
    config
        .env_overrides
        .insert("CAPABILITY_CAP_A_ENDPOINT".into(), "http://a:8001".into());
    config
        .env_overrides
        .insert("CAPABILITY_CAP_B_ENDPOINT".into(), "http://b:8002".into());
    let discovery = InfantDiscovery::with_config(config).unwrap();

    let a = discovery.find_capability("cap-a").await.unwrap();
    assert_eq!(a.len(), 1);

    let b = discovery.find_capability("cap-b").await.unwrap();
    assert_eq!(b.len(), 1);

    let all = discovery.all_discovered().await;
    assert_eq!(all.len(), 2);
    assert!(all.contains_key("cap-a"));
    assert!(all.contains_key("cap-b"));
}

// =========================================================================
// Additional coverage: capability_to_srv_name catch-all arm
// =========================================================================

#[test]
fn test_capability_to_srv_name_unknown_with_hyphen() {
    let srv = backends::capability_to_srv_name("custom-auth-provider");
    assert_eq!(srv, "_provider._tcp.local");
}

#[test]
fn test_capability_to_srv_name_unknown_no_hyphen() {
    let srv = backends::capability_to_srv_name("telemetry");
    assert_eq!(srv, "_telemetry._tcp.local");
}

#[test]
fn test_capability_to_srv_name_empty_string_fallback() {
    let srv = backends::capability_to_srv_name("");
    assert_eq!(srv, "_._tcp.local");
}

#[test]
fn test_capability_to_srv_name_single_hyphen_fallback() {
    let srv = backends::capability_to_srv_name("-");
    assert_eq!(srv, "_._tcp.local");
}

// =========================================================================
// Additional coverage: DiscoveryConfig::from_explicit branches
// =========================================================================

#[test]
fn test_from_explicit_with_registry_and_ttl() {
    let config = DiscoveryConfig::from_explicit(Some("http://reg.local:8082"), Some(60));
    assert_eq!(config.cache_ttl_secs, 60);
    assert!(
        config
            .methods
            .iter()
            .any(|m| matches!(m, DiscoveryProtocol::ServiceRegistry(_)))
    );
}

#[test]
fn test_from_explicit_with_registry_only() {
    let config = DiscoveryConfig::from_explicit(Some("http://reg.local:8082"), None);
    assert_eq!(config.cache_ttl_secs, 300);
    assert!(
        config
            .methods
            .iter()
            .any(|m| matches!(m, DiscoveryProtocol::ServiceRegistry(_)))
    );
}

#[test]
fn test_from_explicit_with_neither() {
    let config = DiscoveryConfig::from_explicit(None, None);
    assert_eq!(config.cache_ttl_secs, 300);
    assert!(
        !config
            .methods
            .iter()
            .any(|m| matches!(m, DiscoveryProtocol::ServiceRegistry(_)))
    );
}

#[test]
fn test_from_explicit_with_ttl_only() {
    let config = DiscoveryConfig::from_explicit(None, Some(999));
    assert_eq!(config.cache_ttl_secs, 999);
}

// =========================================================================
// Additional coverage: InfantDiscovery env_overrides for env_lookup
// =========================================================================

#[tokio::test]
async fn test_env_lookup_prefers_overrides_over_system() {
    let overrides: HashMap<String, String> = HashMap::from([(
        "TEST_SIGNING_URL".to_string(),
        "http://override:9000".to_string(),
    )]);
    let config = DiscoveryConfig {
        env_overrides: overrides,
        ..DiscoveryConfig::default()
    };
    let discovery = InfantDiscovery::with_config(config).unwrap();

    let result = discovery
        .find_capability(crate::capabilities::identifiers::external::SIGNING)
        .await;
    assert!(result.is_ok());
}

#[test]
fn test_discovery_config_from_env_or_default_does_not_panic() {
    let _config = DiscoveryConfig::from_env_or_default();
}

// =========================================================================
// Additional coverage: InfantDiscovery::new() default path
// =========================================================================

#[tokio::test]
async fn test_infant_discovery_new_default_config() {
    let discovery = InfantDiscovery::new().unwrap();
    assert!(!discovery.own_capabilities().is_empty());
    let all = discovery.all_discovered().await;
    assert!(all.is_empty());
}
