// SPDX-License-Identifier: AGPL-3.0-or-later

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

#[test]
#[serial]
fn test_discover_via_environment() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Phase 1: clean env — ensure no signing endpoints exist
    let discovery = temp_env::with_vars(
        [
            ("CAPABILITY_SIGNING_ENDPOINT", None::<&str>),
            ("CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT", None::<&str>),
            ("SIGNING_SERVICE_URL", None::<&str>),
        ],
        || {
            rt.block_on(async {
                let discovery = InfantDiscovery::new().unwrap();
                let services = discovery
                    .find_capability("cryptographic-signing")
                    .await
                    .unwrap();
                assert!(services.is_empty());
                discovery
            })
        },
    );

    // Phase 2: set env var and rediscover
    temp_env::with_vars(
        [
            ("CAPABILITY_SIGNING_ENDPOINT", None::<&str>),
            (
                "CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT",
                Some("http://localhost:8001"),
            ),
            ("SIGNING_SERVICE_URL", None::<&str>),
        ],
        || {
            rt.block_on(async {
                discovery.clear_cache().await;
                let services = discovery
                    .find_capability("cryptographic-signing")
                    .await
                    .unwrap();
                assert_eq!(services.len(), 1);
                assert_eq!(services[0].endpoint, "http://localhost:8001");
                assert_eq!(services[0].discovered_via, "environment");
            });
        },
    );
}

#[test]
#[serial]
fn test_degraded_mode_when_no_services() {
    temp_env::with_var("CAPABILITY_STORAGE_ENDPOINT", None::<&str>, || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let discovery = InfantDiscovery::new().unwrap();
            let services = discovery.find_capability("content-storage").await.unwrap();
            assert!(services.is_empty());
        });
    });
}

#[test]
#[serial]
fn test_cache_functionality() {
    temp_env::with_vars(
        [
            (
                "CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT",
                Some("http://localhost:8001"),
            ),
            ("CAPABILITY_SIGNING_ENDPOINT", None::<&str>),
            ("SIGNING_SERVICE_URL", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let discovery = InfantDiscovery::new().unwrap();

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
            });
        },
    );
}

#[test]
#[serial]
fn test_discover_via_signing_service_url() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let discovery = temp_env::with_vars(
        [
            ("CAPABILITY_SIGNING_ENDPOINT", None::<&str>),
            ("CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT", None::<&str>),
            ("SIGNING_SERVICE_URL", None::<&str>),
        ],
        || {
            rt.block_on(async {
                let discovery = InfantDiscovery::new().unwrap();
                let services = discovery
                    .find_capability("cryptographic-signing")
                    .await
                    .unwrap();
                assert!(services.is_empty());
                discovery
            })
        },
    );

    temp_env::with_vars(
        [
            ("CAPABILITY_SIGNING_ENDPOINT", None::<&str>),
            ("CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT", None::<&str>),
            ("SIGNING_SERVICE_URL", Some("http://localhost:8002")),
        ],
        || {
            rt.block_on(async {
                discovery.clear_cache().await;
                let services = discovery
                    .find_capability("cryptographic-signing")
                    .await
                    .unwrap();
                assert_eq!(services.len(), 1);
                assert_eq!(services[0].endpoint, "http://localhost:8002");
            });
        },
    );
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
    temp_env::with_vars(
        [
            ("SERVICE_REGISTRY_URL", Some("http://registry.example.com")),
            ("DISCOVERY_CACHE_TTL", Some("600")),
        ],
        || {
            let config = DiscoveryConfig::from_env_or_default();
            assert!(config.methods.iter().any(|m| matches!(
                m,
                DiscoveryMethod::ServiceRegistry(url) if url == "http://registry.example.com"
            )));
            assert_eq!(config.cache_ttl_secs, 600);
        },
    );
}

#[test]
fn test_capability_to_srv_name_indirect() {
    let discovery = InfantDiscovery::new().unwrap();
    let capabilities = discovery.own_capabilities();
    assert!(!capabilities.is_empty());
}

#[test]
#[serial]
fn test_service_registry_discovery_returns_empty() {
    temp_env::with_vars(
        [
            ("CAPABILITY_SIGNING_ENDPOINT", None::<&str>),
            ("SIGNING_SERVICE_URL", None::<&str>),
            ("SERVICE_REGISTRY_URL", Some("http://registry.test")),
        ],
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let config = DiscoveryConfig::from_env_or_default();
                let discovery = InfantDiscovery::with_config(config).unwrap();
                discovery.clear_cache().await;

                let services = discovery.find_capability("signing").await.unwrap();
                assert!(services.is_empty());
            });
        },
    );
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
    assert!(
        identifiers
            .iter()
            .any(|id| id.contains("ledger") || id.contains("permanence"))
    );
}

#[test]
#[serial]
fn test_cache_hit_with_stale_services_triggers_rediscovery() {
    temp_env::with_vars(
        [
            ("CAPABILITY_CRYPTOGRAPHIC_SIGNING_ENDPOINT", None::<&str>),
            ("SIGNING_SERVICE_URL", Some("http://localhost:9999")),
            ("SERVICE_REGISTRY_URL", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let config = DiscoveryConfig {
                    methods: vec![DiscoveryMethod::Environment],
                    cache_ttl_secs: 0,
                    retry_attempts: 1,
                    discovery_timeout: Duration::from_secs(1),
                };
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
            });
        },
    );
}

#[test]
#[serial]
fn test_discovery_config_from_env_invalid_ttl_uses_default() {
    temp_env::with_vars(
        [
            ("SERVICE_REGISTRY_URL", None::<&str>),
            ("DISCOVERY_CACHE_TTL", Some("not-a-number")),
        ],
        || {
            let config = DiscoveryConfig::from_env_or_default();
            assert_eq!(
                config.cache_ttl_secs, 300,
                "invalid TTL should leave default"
            );
        },
    );
}

#[test]
fn test_capability_to_srv_name_all_known_capabilities() {
    assert_eq!(
        capability_to_srv_name("session-management"),
        "_session._tcp.local"
    );
    assert_eq!(
        capability_to_srv_name("compute-orchestration"),
        "_compute._tcp.local"
    );
    assert_eq!(
        capability_to_srv_name("service-discovery"),
        "_discovery._tcp.local"
    );
}

#[test]
fn test_capability_to_srv_name_unknown_uses_last_segment() {
    assert_eq!(
        capability_to_srv_name("custom-capability"),
        "_capability._tcp.local"
    );
    assert_eq!(capability_to_srv_name("single"), "_single._tcp.local");
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

#[test]
fn test_all_discovered_returns_populated_cache() {
    temp_env::with_var(
        "CAPABILITY_TEST_ENDPOINT",
        Some("http://localhost:1234"),
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let config = DiscoveryConfig {
                    methods: vec![DiscoveryMethod::Environment],
                    cache_ttl_secs: 300,
                    retry_attempts: 1,
                    discovery_timeout: Duration::from_secs(1),
                };
                let discovery = InfantDiscovery::with_config(config).unwrap();

                let _ = discovery.find_capability("test").await.unwrap();

                let all = discovery.all_discovered().await;
                assert!(all.contains_key("test"));
                assert_eq!(all["test"].len(), 1);
            });
        },
    );
}

#[test]
fn test_discovery_method_equality() {
    assert_eq!(DiscoveryMethod::Environment, DiscoveryMethod::Environment);
    assert_eq!(DiscoveryMethod::MDns, DiscoveryMethod::MDns);
    assert_eq!(DiscoveryMethod::DnsSrv, DiscoveryMethod::DnsSrv);
    assert_ne!(DiscoveryMethod::Environment, DiscoveryMethod::DnsSrv);
    assert_eq!(
        DiscoveryMethod::ServiceRegistry("http://a".into()),
        DiscoveryMethod::ServiceRegistry("http://a".into())
    );
    assert_ne!(
        DiscoveryMethod::ServiceRegistry("http://a".into()),
        DiscoveryMethod::ServiceRegistry("http://b".into())
    );
}

#[test]
#[serial]
fn test_discover_via_environment_pattern2_service_url() {
    temp_env::with_vars(
        [
            ("CAPABILITY_STORAGE_ENDPOINT", None::<&str>),
            ("CAPABILITY_CONTENT_STORAGE_ENDPOINT", None::<&str>),
            ("STORAGE_SERVICE_URL", Some("http://localhost:7777")),
        ],
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let config = DiscoveryConfig {
                    methods: vec![DiscoveryMethod::Environment],
                    cache_ttl_secs: 300,
                    retry_attempts: 1,
                    discovery_timeout: Duration::from_secs(1),
                };
                let discovery = InfantDiscovery::with_config(config).unwrap();

                let services = discovery.find_capability("content-storage").await.unwrap();
                assert_eq!(services.len(), 1);
                assert_eq!(services[0].endpoint, "http://localhost:7777");
                assert_eq!(services[0].discovered_via, "environment");
            });
        },
    );
}

#[tokio::test]
#[serial]
#[expect(
    unsafe_code,
    reason = "async test requires multiple sequential env changes (set A, await, set B, await); temp-env cannot wrap per-await mutation"
)]
async fn test_cache_hit_with_fresh_services_skips_rediscovery() {
    unsafe {
        env::remove_var("CAPABILITY_SIGNING_ENDPOINT");
        env::remove_var("SIGNING_SERVICE_URL");
    }

    let config = DiscoveryConfig {
        methods: vec![DiscoveryMethod::Environment],
        cache_ttl_secs: 3600,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
    };
    let discovery = InfantDiscovery::with_config(config).unwrap();

    unsafe {
        env::set_var("SIGNING_SERVICE_URL", "http://localhost:1111");
    }
    let services1 = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services1.len(), 1);

    unsafe {
        env::set_var("SIGNING_SERVICE_URL", "http://localhost:2222");
    }
    let services2 = discovery
        .find_capability("cryptographic-signing")
        .await
        .unwrap();
    assert_eq!(services2.len(), 1);
    assert_eq!(
        services2[0].endpoint, "http://localhost:1111",
        "should use cached value"
    );

    unsafe {
        env::remove_var("SIGNING_SERVICE_URL");
    }
}

#[tokio::test]
async fn test_with_config_custom_timeout() {
    let config = DiscoveryConfig {
        methods: vec![DiscoveryMethod::DnsSrv],
        cache_ttl_secs: 60,
        retry_attempts: 1,
        discovery_timeout: Duration::from_millis(100),
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
        methods: vec![DiscoveryMethod::MDns],
        cache_ttl_secs: 300,
        retry_attempts: 1,
        discovery_timeout: Duration::from_secs(1),
    };
    let discovery = InfantDiscovery::with_config(config).unwrap();
    let services = discovery.find_capability("signing").await.unwrap();
    // Without mdns feature (or with it but no LAN services), should be empty
    assert!(services.is_empty());
}
