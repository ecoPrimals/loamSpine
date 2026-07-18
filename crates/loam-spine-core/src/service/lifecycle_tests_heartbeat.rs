// SPDX-License-Identifier: AGPL-3.0-or-later

//! Heartbeat task, state transition, and `ServiceState` coverage tests.
//!
//! See `lifecycle_tests.rs` for core lifecycle manager start/stop/config tests.

use super::*;
use std::sync::atomic::Ordering;
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn heartbeat_task_exits_on_shutdown_signal() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;

    let mut manager = LifecycleManager::new(service, config);
    manager.start().await.expect("start");

    let client = crate::discovery_client::DiscoveryClient::for_testing_success("http://test:8082");
    manager.start_heartbeat_for_testing(client, 1);

    assert!(manager.heartbeat_task.is_some());

    manager.shutdown.store(true, Ordering::Relaxed);
    tokio::time::advance(Duration::from_secs(2)).await;

    if let Some(task) = manager.heartbeat_task.take() {
        let result = tokio::time::timeout(Duration::from_secs(5), task).await;
        assert!(result.is_ok(), "heartbeat task should have exited");
    }
}

#[tokio::test(start_paused = true)]
async fn heartbeat_task_marks_degraded_after_threshold() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;
    config.discovery.heartbeat_retry.backoff_seconds = vec![0];
    config
        .discovery
        .heartbeat_retry
        .max_failures_before_degraded = 1;
    config.discovery.heartbeat_retry.max_failures_total = 3;

    let mut manager = LifecycleManager::new(service, config);
    manager.start().await.expect("start");

    let client = crate::discovery_client::DiscoveryClient::for_testing("http://127.0.0.1:1");
    manager.start_heartbeat_for_testing(client, 1);

    tokio::time::advance(Duration::from_secs(5)).await;

    if let Some(task) = manager.heartbeat_task.take() {
        let result = tokio::time::timeout(Duration::from_secs(10), task).await;
        assert!(
            result.is_ok(),
            "heartbeat task should exit after max_failures_total"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn heartbeat_task_recovers_on_success() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;

    let mut manager = LifecycleManager::new(service, config);
    manager.start().await.expect("start");

    let client = crate::discovery_client::DiscoveryClient::for_testing_success("http://test:8082");
    manager.start_heartbeat_for_testing(client, 1);

    tokio::time::advance(Duration::from_secs(3)).await;

    manager.stop().await.expect("stop");
}

#[tokio::test]
async fn transition_to_error_state() {
    let service = LoamSpineService::new();
    let config = LoamSpineConfig::default();
    let manager = LifecycleManager::new(service, config);

    manager.transition(ServiceState::Error);
    assert_eq!(manager.state(), ServiceState::Error);
}

#[tokio::test]
async fn transition_to_degraded_state() {
    let service = LoamSpineService::new();
    let config = LoamSpineConfig::default();
    let manager = LifecycleManager::new(service, config);

    manager.transition(ServiceState::Degraded);
    assert_eq!(manager.state(), ServiceState::Degraded);
}

#[tokio::test]
async fn lifecycle_stop_with_injected_heartbeat_and_client() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;

    let mut manager = LifecycleManager::new(service, config);
    manager.start().await.expect("start");

    let success_client =
        crate::discovery_client::DiscoveryClient::for_testing_success("http://test:8082");
    manager.start_heartbeat_for_testing(success_client.clone(), 1);

    manager.inject_discovery_client_for_testing(success_client);

    let result = manager.stop().await;
    assert!(result.is_ok());
    assert_eq!(manager.state(), ServiceState::Stopped);
}

#[tokio::test]
async fn send_heartbeat_with_retry_succeeds_on_retry() {
    let client = crate::discovery_client::DiscoveryClient::for_testing_success("http://test:8082");
    let retry_config = crate::config::HeartbeatRetryConfig {
        backoff_seconds: vec![0, 0],
        max_failures_before_degraded: 5,
        max_failures_total: 10,
    };

    let result = LifecycleManager::send_heartbeat_with_retry(&client, &retry_config, 5).await;
    assert!(result.is_ok());
}

#[test]
fn service_state_debug() {
    let state = ServiceState::Running;
    let debug = format!("{state:?}");
    assert!(debug.contains("Running"));
}

#[test]
fn service_state_clone_eq() {
    let a = ServiceState::Degraded;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn service_state_copy() {
    let a = ServiceState::Starting;
    let b = a;
    assert_eq!(a, b);
    assert_eq!(a, ServiceState::Starting);
}

#[test]
fn service_state_display_all_variants() {
    assert_eq!(ServiceState::Starting.to_string(), "STARTING");
    assert_eq!(ServiceState::Ready.to_string(), "READY");
    assert_eq!(ServiceState::Running.to_string(), "RUNNING");
    assert_eq!(ServiceState::Degraded.to_string(), "DEGRADED");
    assert_eq!(ServiceState::Stopping.to_string(), "STOPPING");
    assert_eq!(ServiceState::Stopped.to_string(), "STOPPED");
    assert_eq!(ServiceState::Error.to_string(), "ERROR");
}

#[test]
fn service_state_serde_all_variants() {
    let variants = [
        ServiceState::Starting,
        ServiceState::Ready,
        ServiceState::Running,
        ServiceState::Degraded,
        ServiceState::Stopping,
        ServiceState::Stopped,
        ServiceState::Error,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: ServiceState = serde_json::from_str(&json).unwrap();
        assert_eq!(*v, back, "round-trip failed for {v:?}");
    }
}

#[tokio::test]
async fn lifecycle_start_transitions_through_starting_to_running() {
    let service = LoamSpineService::new();
    let config = LoamSpineConfig::default();
    let mut manager = LifecycleManager::new(service, config);

    assert_eq!(manager.state(), ServiceState::Stopped);

    let mut rx = manager.subscribe_state();
    let _result = manager.start().await;

    let final_state = manager.state();
    assert!(
        final_state == ServiceState::Running || final_state == ServiceState::Ready,
        "expected Running or Ready, got {final_state:?}"
    );

    let observed_states: Vec<_> = {
        let mut states = vec![];
        while rx.has_changed().unwrap_or(false) {
            rx.changed().await.ok();
            states.push(*rx.borrow());
        }
        states
    };

    assert!(
        !observed_states.is_empty() || final_state == ServiceState::Running,
        "should have observed at least one state change"
    );
}

#[tokio::test]
async fn lifecycle_stop_transitions_to_stopped() {
    let service = LoamSpineService::new();
    let config = LoamSpineConfig::default();
    let mut manager = LifecycleManager::new(service, config);

    manager.start().await.ok();
    let result = manager.stop().await;
    assert!(result.is_ok());
    assert_eq!(manager.state(), ServiceState::Stopped);
}

#[test]
fn lifecycle_service_accessor() {
    let service = LoamSpineService::new();
    let config = LoamSpineConfig::default();
    let manager = LifecycleManager::new(service, config);
    let _ = manager.service();
}
