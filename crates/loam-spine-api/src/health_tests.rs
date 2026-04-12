// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[tokio::test]
async fn health_checker_creation() {
    let checker = HealthChecker::new();
    let health = checker.check_health().await;
    assert!(health.is_ok());
}

#[tokio::test]
async fn liveness_probe_always_alive() {
    let checker = HealthChecker::new();
    let liveness = checker.check_liveness();
    assert_eq!(liveness.status, "alive");
}

#[tokio::test]
async fn readiness_probe_ready_when_storage_healthy() {
    let checker = HealthChecker::new();
    let readiness = checker.check_readiness().await;
    assert!(readiness.is_ok());
    assert!(readiness.unwrap().ready);
}

#[tokio::test]
async fn health_status_includes_version() {
    let checker = HealthChecker::new();
    let health = checker.check_health().await.unwrap();
    assert!(!health.version.is_empty());
}

#[tokio::test]
async fn health_status_includes_capabilities() {
    let checker = HealthChecker::new();
    let health = checker.check_health().await.unwrap();
    assert!(!health.capabilities.is_empty());
    assert!(health.capabilities.contains(
        &loam_spine_core::capabilities::identifiers::loamspine::PERMANENT_LEDGER.to_string()
    ));
}

#[tokio::test(start_paused = true)]
async fn health_status_tracks_uptime() {
    let checker = HealthChecker::new();

    let health1 = checker.check_health().await.unwrap();
    assert!(
        health1.uptime_seconds < 60,
        "Initial uptime should be less than a minute"
    );

    tokio::time::advance(Duration::from_millis(100)).await;

    let health2 = checker.check_health().await.unwrap();
    assert!(
        health2.uptime_seconds >= health1.uptime_seconds,
        "Uptime should increase or stay the same"
    );
    assert!(
        health2.uptime_seconds < 60,
        "Uptime should still be less than a minute"
    );
}

#[test]
fn service_status_serialization() {
    let status = ServiceStatus::Healthy;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"healthy\"");

    let status = ServiceStatus::Degraded;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"degraded\"");

    let status = ServiceStatus::Error;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"error\"");
}

#[tokio::test]
async fn health_checker_with_storage_check() {
    let storage = Arc::new(|| true);
    let checker = HealthChecker::with_storage_check(storage);
    let health = checker.check_health().await.unwrap();
    assert_eq!(health.status, ServiceStatus::Healthy);
    assert!(health.dependencies.storage);
}

#[tokio::test]
async fn health_checker_with_checks_all_healthy() {
    let storage = Arc::new(|| true);
    let discovery = Arc::new(|| Some(true));
    let checker = HealthChecker::with_checks(storage, discovery);
    let health = checker.check_health().await.unwrap();
    assert_eq!(health.status, ServiceStatus::Healthy);
    assert!(health.dependencies.storage);
    assert_eq!(health.dependencies.discovery, Some(true));
}

#[tokio::test]
async fn health_checker_degraded_when_discovery_unavailable() {
    let storage = Arc::new(|| true);
    let discovery = Arc::new(|| Some(false));
    let checker = HealthChecker::with_checks(storage, discovery);
    let health = checker.check_health().await.unwrap();
    assert_eq!(health.status, ServiceStatus::Degraded);
    assert_eq!(health.dependencies.discovery, Some(false));
}

#[tokio::test]
async fn health_checker_error_when_storage_unavailable() {
    let storage = Arc::new(|| false);
    let checker = HealthChecker::with_storage_check(storage);
    let health = checker.check_health().await.unwrap();
    assert_eq!(health.status, ServiceStatus::Error);
    assert!(!health.dependencies.storage);
}

#[tokio::test]
async fn readiness_not_ready_when_storage_down() {
    let storage = Arc::new(|| false);
    let checker = HealthChecker::with_storage_check(storage);
    let readiness = checker.check_readiness().await.unwrap();
    assert!(!readiness.ready);
    assert!(readiness.reason.is_some());
}

#[test]
fn health_checker_default_same_as_new() {
    let checker = HealthChecker::default();
    let liveness = checker.check_liveness();
    assert_eq!(liveness.status, "alive");
}
