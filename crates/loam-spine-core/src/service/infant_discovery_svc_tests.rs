// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn infant_discovery_creation() {
    let infant = InfantDiscovery::new(vec!["test-capability".to_string()]);
    assert_eq!(infant.capabilities().len(), 1);
    assert_eq!(infant.capabilities()[0], "test-capability");
}

#[test]
fn infant_discovery_default() {
    let infant = InfantDiscovery::default();
    assert!(infant.capabilities().len() >= 3);
    assert!(
        infant.capabilities().contains(
            &crate::capabilities::identifiers::loamspine::PERMANENT_LEDGER.to_string()
        )
    );
}

#[test]
fn from_advertised_matches_canonical_set() {
    let infant = InfantDiscovery::from_advertised();
    let caps = infant.capabilities();
    for &expected in crate::capabilities::identifiers::loamspine::ADVERTISED {
        assert!(
            caps.contains(&expected.to_string()),
            "missing advertised capability: {expected}"
        );
    }
}

#[test]
fn environment_discovery_with_var() {
    let infant = InfantDiscovery::new(vec!["test".to_string()]);
    let result = infant.try_environment_discovery_with(Some("http://test.example.com:8082"));
    assert_eq!(result, Some("http://test.example.com:8082".to_string()));
}

#[test]
fn environment_discovery_without_var() {
    let infant = InfantDiscovery::new(vec!["test".to_string()]);
    let result = infant.try_environment_discovery_with(Some(""));
    assert!(result.is_none());
}

#[tokio::test]
async fn dns_srv_discovery_no_records() {
    let infant = InfantDiscovery::new(vec!["test".to_string()]);
    let result = infant.try_dns_srv_discovery().await;

    // Returns None in test environment (no DNS SRV records configured)
    // In production, would return endpoint if DNS is properly configured
    assert!(result.is_none());
}

#[tokio::test]
async fn mdns_discovery_not_configured() {
    let infant = InfantDiscovery::new(vec!["test".to_string()]);
    let result = infant.try_mdns_discovery().await;

    // Currently returns None (experimental/not fully implemented)
    assert!(result.is_none());
}

#[cfg(debug_assertions)]
#[tokio::test]
async fn development_fallback_in_debug() {
    let infant = InfantDiscovery::new(vec!["test".to_string()]);
    let result = infant.try_development_fallback();

    // Should return localhost in debug mode
    let expected_endpoint = format!(
        "http://{}:{}",
        crate::constants::LOCALHOST,
        crate::constants::DEFAULT_DISCOVERY_PORT
    );
    assert_eq!(result, Some(expected_endpoint));
}

#[tokio::test]
async fn discovery_service_full_chain() {
    let infant = InfantDiscovery::new(vec!["test".to_string()]);
    let result = infant
        .discover_discovery_service_with(Some("http://test.example.com:8082"))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn discover_discovery_service_unreachable_endpoint_returns_error() {
    let infant = InfantDiscovery::new(vec!["test".to_string()]);
    let result = infant
        .discover_discovery_service_with(Some("http://127.0.0.1:1"))
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("unavailable")
            || err_str.contains("registry")
            || err_str.contains("127"),
        "Expected connection error: {err_str}",
    );
}

#[tokio::test]
async fn discover_discovery_service_development_fallback_connection_fails() {
    let infant = InfantDiscovery::new(vec!["test".to_string()]);
    let result = infant.discover_discovery_service_with(Some("")).await;
    assert!(result.is_err());
}

#[test]
fn discover_discovery_service_empty_env_skipped() {
    let infant = InfantDiscovery::new(vec!["test".to_string()]);
    let result = infant.try_environment_discovery_with(Some(""));
    assert!(result.is_none());
}

#[tokio::test]
async fn development_fallback_returns_endpoint_in_test_mode() {
    let infant = InfantDiscovery::new(vec!["test".to_string()]);
    let result = infant.try_development_fallback();

    // In test mode, development fallback is enabled and returns localhost:8082
    let expected = format!(
        "http://{}:{}",
        crate::constants::LOCALHOST,
        crate::constants::DEFAULT_DISCOVERY_PORT
    );
    assert_eq!(result, Some(expected));
}

#[test]
fn default_includes_all_expected_capabilities() {
    use crate::capabilities::identifiers::loamspine;
    let infant = InfantDiscovery::default();
    let caps = infant.capabilities();

    assert!(caps.contains(&loamspine::PERMANENT_LEDGER.to_string()));
    assert!(caps.contains(&loamspine::WAYPOINT_ANCHORING.to_string()));
    assert!(caps.contains(&loamspine::CERTIFICATE_AUTHORITY.to_string()));
}

#[tokio::test]
async fn discover_discovery_service_env_takes_priority_over_fallback() {
    // When DISCOVERY_ENDPOINT is set, it should be used (even if unreachable)
    let infant = InfantDiscovery::new(vec!["test".to_string()]);
    let result = infant
        .discover_discovery_service_with(Some("http://env-priority-test.example:9999"))
        .await;
    // Should fail to connect (unreachable) but we used env, not fallback
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("env-priority-test")
            || err_str.contains("9999")
            || err_str.contains("unavailable"),
        "Expected env endpoint in error: {err_str}",
    );
}

#[tokio::test]
async fn discover_discovery_service_fallback_chain_when_env_unset() {
    // No env var -> DNS (None in test) -> mDNS (None) -> dev fallback -> connect fails
    let infant = InfantDiscovery::new(vec!["test".to_string()]);
    let result = infant.discover_discovery_service_with(Some("")).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string();
    // Connection to localhost:8082 should fail (no server)
    assert!(
        err_str.contains("unavailable")
            || err_str.contains("registry")
            || err_str.contains("localhost")
            || err_str.contains("8082")
            || err_str.contains("connection"),
        "Expected connection error from fallback: {err_str}",
    );
}

#[test]
fn capabilities_returns_owned_reference() {
    let infant = InfantDiscovery::new(vec!["cap-a".to_string(), "cap-b".to_string()]);
    let caps = infant.capabilities();

    assert_eq!(caps.len(), 2);
    assert_eq!(caps[0], "cap-a");
    assert_eq!(caps[1], "cap-b");
}

#[test]
fn infant_discovery_new_with_empty_capabilities() {
    let infant = InfantDiscovery::new(vec![]);
    assert!(infant.capabilities().is_empty());
}

#[test]
fn infant_discovery_new_with_many_capabilities() {
    let caps: Vec<String> = (0..10).map(|i| format!("cap-{i}")).collect();
    let infant = InfantDiscovery::new(caps);
    assert_eq!(infant.capabilities().len(), 10);
    for (i, c) in infant.capabilities().iter().enumerate() {
        assert_eq!(c, &format!("cap-{i}"));
    }
}

#[test]
fn environment_discovery_empty_string_skipped() {
    let infant = InfantDiscovery::new(vec!["test".to_string()]);
    let result = infant.try_environment_discovery_with(Some(""));
    assert!(result.is_none());
}
