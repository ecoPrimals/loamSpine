// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn default_config() {
    let config = LoamSpineConfig::default();
    assert_eq!(config.name, "LoamSpine");
    assert!(!config.replication_enabled);
    assert_eq!(config.auto_rollup_threshold, Some(10_000));
}

#[test]
fn config_builder() {
    let config = LoamSpineConfig::new("TestSpine")
        .with_storage_path("/tmp/test")
        .with_auto_rollup(5000)
        .with_replication(true);

    assert_eq!(config.name, "TestSpine");
    assert_eq!(config.storage_path, PathBuf::from("/tmp/test"));
    assert_eq!(config.auto_rollup_threshold, Some(5000));
    assert!(config.replication_enabled);
}

#[test]
fn config_with_discovery() {
    let discovery = DiscoveryConfig {
        discovery_enabled: false,
        ..DiscoveryConfig::default()
    };
    let config = LoamSpineConfig::new("Test").with_discovery(discovery);
    assert!(!config.discovery.discovery_enabled);
}

#[test]
fn config_with_discovery_service() {
    let config = LoamSpineConfig::new("Test").with_discovery_service("http://registry.local:8082");
    assert!(config.discovery.discovery_enabled);
    assert_eq!(
        config.discovery.discovery_endpoint.as_deref(),
        Some("http://registry.local:8082")
    );
}

#[test]
fn heartbeat_retry_config_default() {
    let retry = HeartbeatRetryConfig::default();
    assert_eq!(retry.backoff_seconds, vec![10, 30, 60, 120]);
    assert_eq!(retry.max_failures_before_degraded, 3);
    assert_eq!(retry.max_failures_total, 10);
}

#[test]
fn discovery_config_default_methods() {
    let config = DiscoveryConfig::default();
    assert!(config.discovery_enabled);
    assert!(config.auto_advertise);
    assert_eq!(config.heartbeat_interval_seconds, 60);
    assert!(config.methods.contains(&DiscoveryMethod::Environment));
    assert!(config.methods.contains(&DiscoveryMethod::ServiceRegistry));
}

#[test]
fn discovery_method_serde_roundtrip() {
    let methods = vec![
        DiscoveryMethod::Environment,
        DiscoveryMethod::ServiceRegistry,
        DiscoveryMethod::Mdns,
        DiscoveryMethod::LocalBinaries,
        DiscoveryMethod::ConfigFile,
        DiscoveryMethod::Fallback,
    ];
    let json = serde_json::to_string(&methods).unwrap();
    let parsed: Vec<DiscoveryMethod> = serde_json::from_str(&json).unwrap();
    assert_eq!(methods, parsed);
}

#[test]
fn loamspine_config_serde_roundtrip() {
    let config = LoamSpineConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let parsed: LoamSpineConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, config.name);
    assert_eq!(parsed.storage_path, config.storage_path);
}
