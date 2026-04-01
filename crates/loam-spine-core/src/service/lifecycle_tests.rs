// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn lifecycle_manager_creation() {
    let service = LoamSpineService::new();
    let config = LoamSpineConfig::default();
    let manager = LifecycleManager::new(service, config);

    assert!(manager.heartbeat_task.is_none());
    assert!(!manager.shutdown.load(Ordering::Relaxed));
    assert_eq!(manager.state(), ServiceState::Stopped);
}

#[test]
fn service_state_display() {
    assert_eq!(ServiceState::Starting.to_string(), "STARTING");
    assert_eq!(ServiceState::Ready.to_string(), "READY");
    assert_eq!(ServiceState::Running.to_string(), "RUNNING");
    assert_eq!(ServiceState::Degraded.to_string(), "DEGRADED");
    assert_eq!(ServiceState::Stopping.to_string(), "STOPPING");
    assert_eq!(ServiceState::Stopped.to_string(), "STOPPED");
    assert_eq!(ServiceState::Error.to_string(), "ERROR");
}

#[test]
fn service_state_serialization() {
    let state = ServiceState::Running;
    let json = serde_json::to_string(&state).unwrap();
    let parsed: ServiceState = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, state);
}

#[test]
fn service_state_subscribe() {
    let service = LoamSpineService::new();
    let config = LoamSpineConfig::default();
    let manager = LifecycleManager::new(service, config);
    let rx = manager.subscribe_state();
    assert_eq!(*rx.borrow(), ServiceState::Stopped);
}

#[test]
fn lifecycle_transitions_through_states() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let service = LoamSpineService::new();
        let mut config = LoamSpineConfig::default();
        config.discovery.discovery_enabled = false;
        let mut manager = LifecycleManager::new(service, config);

        assert_eq!(manager.state(), ServiceState::Stopped);

        manager.start().await.unwrap();
        assert_eq!(manager.state(), ServiceState::Running);

        manager.stop().await.unwrap();
        assert_eq!(manager.state(), ServiceState::Stopped);
    });
}

#[tokio::test]
async fn lifecycle_start_without_discovery() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;

    let mut manager = LifecycleManager::new(service, config);
    let result = manager.start().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn lifecycle_stop() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;

    let mut manager = LifecycleManager::new(service, config);
    manager.start().await.expect("Failed to start");

    let result = manager.stop().await;
    assert!(result.is_ok());
    assert!(manager.shutdown.load(Ordering::Relaxed));
}

#[test]
fn lifecycle_manager_service_getter() {
    let service = LoamSpineService::new();
    let config = LoamSpineConfig::default();
    let manager = LifecycleManager::new(service, config);

    let _service_ref = manager.service();
}

#[tokio::test]
async fn lifecycle_start_with_registry_unavailable() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = true;
    config.discovery.discovery_endpoint = Some("http://localhost:9999".to_string());
    config.discovery.auto_advertise = true;

    let mut manager = LifecycleManager::new(service, config);
    let result = manager.start().await;

    assert!(result.is_ok());
    assert!(manager.discovery_client.is_none());
}

#[tokio::test]
async fn lifecycle_stop_without_start() {
    let service = LoamSpineService::new();
    let config = LoamSpineConfig::default();
    let mut manager = LifecycleManager::new(service, config);

    let result = manager.stop().await;
    assert!(result.is_ok());
}

#[test]
fn lifecycle_manager_drop() {
    let service = LoamSpineService::new();
    let config = LoamSpineConfig::default();
    let manager = LifecycleManager::new(service, config);

    drop(manager);
}

#[tokio::test]
async fn lifecycle_multiple_stops() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;

    let mut manager = LifecycleManager::new(service, config);
    manager.start().await.expect("Failed to start");

    let result1 = manager.stop().await;
    assert!(result1.is_ok());

    let result2 = manager.stop().await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn lifecycle_start_with_no_endpoint() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = true;
    config.discovery.discovery_endpoint = None;

    let mut manager = LifecycleManager::new(service, config);
    let result = manager.start().await;

    assert!(result.is_ok());
    assert!(manager.discovery_client.is_none());
}

#[tokio::test]
async fn lifecycle_start_with_heartbeat_disabled() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;
    config.discovery.heartbeat_interval_seconds = 0;

    let mut manager = LifecycleManager::new(service, config);
    let result = manager.start().await;

    assert!(result.is_ok());
    assert!(manager.heartbeat_task.is_none());
}

#[test]
fn shutdown_signal_initial_state() {
    let service = LoamSpineService::new();
    let config = LoamSpineConfig::default();
    let manager = LifecycleManager::new(service, config);

    assert!(!manager.shutdown.load(Ordering::Relaxed));
}

#[tokio::test]
async fn shutdown_signal_after_stop() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;

    let mut manager = LifecycleManager::new(service, config);
    manager.start().await.expect("Failed to start");
    manager.stop().await.expect("Failed to stop");

    assert!(manager.shutdown.load(Ordering::Relaxed));
}

#[test]
fn config_endpoints_from_discovery_config() {
    let mut config = LoamSpineConfig::default();
    config.discovery.tarpc_endpoint = "http://localhost:9999".to_string();
    config.discovery.jsonrpc_endpoint = "http://localhost:7777".to_string();

    assert_eq!(config.discovery.tarpc_endpoint, "http://localhost:9999");
    assert_eq!(config.discovery.jsonrpc_endpoint, "http://localhost:7777");
}

#[tokio::test]
async fn lifecycle_start_with_discovery_enabled_unreachable() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = true;
    config.discovery.discovery_endpoint = Some("http://127.0.0.1:1".to_string());
    config.discovery.auto_advertise = true;

    let mut manager = LifecycleManager::new(service, config);
    let result = manager.start().await;

    assert!(result.is_ok());
    assert!(manager.discovery_client.is_none());
}

#[tokio::test]
async fn lifecycle_stop_deregister_error_is_non_fatal() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;

    let mut manager = LifecycleManager::new(service, config);
    manager.start().await.expect("start failed");

    let result = manager.stop().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn lifecycle_stop_deregister_fails_but_stop_succeeds() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;

    let mut manager = LifecycleManager::new(service, config);
    manager.start().await.expect("start failed");

    let unreachable_client =
        crate::discovery_client::DiscoveryClient::for_testing("http://127.0.0.1:1");
    manager.inject_discovery_client_for_testing(unreachable_client);

    let result = manager.stop().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn send_heartbeat_with_retry_fails_for_unreachable() {
    let client = crate::discovery_client::DiscoveryClient::for_testing("http://127.0.0.1:1");
    let retry_config = crate::config::HeartbeatRetryConfig {
        backoff_seconds: vec![0],
        max_failures_before_degraded: 1,
        max_failures_total: 2,
    };

    let result = LifecycleManager::send_heartbeat_with_retry(&client, &retry_config, 0).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("retries"));
}

#[tokio::test]
async fn send_heartbeat_with_retry_exceeds_total_limit() {
    let client = crate::discovery_client::DiscoveryClient::for_testing("http://127.0.0.1:1");
    let retry_config = crate::config::HeartbeatRetryConfig {
        backoff_seconds: vec![0, 0, 0],
        max_failures_before_degraded: 1,
        max_failures_total: 1,
    };

    let result = LifecycleManager::send_heartbeat_with_retry(&client, &retry_config, 1).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn lifecycle_start_discovery_enabled_endpoint_provided_unreachable_no_advertise() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = true;
    config.discovery.discovery_endpoint = Some("http://127.0.0.1:1".to_string());
    config.discovery.auto_advertise = false;

    let mut manager = LifecycleManager::new(service, config);
    let result = manager.start().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn lifecycle_heartbeat_task_starts_with_connected_client() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = true;
    config.discovery.discovery_endpoint = Some("http://127.0.0.1:1".to_string());
    config.discovery.heartbeat_interval_seconds = 60;

    let mut manager = LifecycleManager::new(service, config);
    let result = manager.start().await;

    assert!(result.is_ok());
    assert!(manager.heartbeat_task.is_none());
}

#[tokio::test]
async fn lifecycle_stop_waits_for_heartbeat_task() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;
    config.discovery.heartbeat_retry.backoff_seconds = vec![0];
    config.discovery.heartbeat_retry.max_failures_total = 10;

    let mut manager = LifecycleManager::new(service, config);
    manager.start().await.expect("start failed");

    let client = crate::discovery_client::DiscoveryClient::for_testing("http://127.0.0.1:1");
    manager.start_heartbeat_for_testing(client, 1);

    let result = manager.stop().await;
    assert!(result.is_ok());
    assert!(manager.shutdown.load(Ordering::Relaxed));
}

#[tokio::test]
async fn lifecycle_drop_aborts_heartbeat_task() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;

    let mut manager = LifecycleManager::new(service, config);
    manager.start().await.expect("start failed");

    let client = crate::discovery_client::DiscoveryClient::for_testing("http://127.0.0.1:1");
    manager.start_heartbeat_for_testing(client, 60);

    drop(manager);
}

#[tokio::test]
async fn lifecycle_stop_deregister_success() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;

    let mut manager = LifecycleManager::new(service, config);
    manager.start().await.expect("start failed");

    let success_client =
        crate::discovery_client::DiscoveryClient::for_testing_success("http://test:8082");
    manager.inject_discovery_client_for_testing(success_client);

    let result = manager.stop().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn send_heartbeat_with_retry_success_immediate() {
    let client = crate::discovery_client::DiscoveryClient::for_testing_success("http://test:8082");
    let retry_config = crate::config::HeartbeatRetryConfig::default();

    let result = LifecycleManager::send_heartbeat_with_retry(&client, &retry_config, 0).await;
    assert!(result.is_ok());
}

#[test]
fn lifecycle_start_with_no_endpoint_clears_env() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let service = LoamSpineService::new();
        let mut config = LoamSpineConfig::default();
        config.discovery.discovery_enabled = true;
        config.discovery.discovery_endpoint = None;
        config.discovery.env_overrides.insert(
            "DISCOVERY_ENDPOINT".to_string(),
            String::new(),
        );

        let mut manager = LifecycleManager::new(service, config);
        let result = manager.start().await;

        assert!(result.is_ok());
        assert!(manager.discovery_client.is_none());
    });
}

#[tokio::test]
async fn lifecycle_subscribe_receives_state_transitions() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;

    let mut manager = LifecycleManager::new(service, config);
    let rx = manager.subscribe_state();

    assert_eq!(*rx.borrow(), ServiceState::Stopped);

    manager.start().await.expect("start failed");
    assert_eq!(*rx.borrow(), ServiceState::Running);

    manager.stop().await.expect("stop failed");
    assert_eq!(*rx.borrow(), ServiceState::Stopped);
}

#[tokio::test]
async fn lifecycle_start_stop_start_cycle() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;

    let mut manager = LifecycleManager::new(service, config);

    manager.start().await.expect("first start");
    assert_eq!(manager.state(), ServiceState::Running);

    manager.stop().await.expect("first stop");
    assert_eq!(manager.state(), ServiceState::Stopped);

    manager.start().await.expect("second start");
    assert_eq!(manager.state(), ServiceState::Running);

    manager.stop().await.expect("second stop");
    assert_eq!(manager.state(), ServiceState::Stopped);
}

#[tokio::test]
async fn lifecycle_heartbeat_recovery_resets_failures() {
    let client = crate::discovery_client::DiscoveryClient::for_testing_success("http://test:8082");
    let retry_config = crate::config::HeartbeatRetryConfig {
        backoff_seconds: vec![0],
        max_failures_before_degraded: 5,
        max_failures_total: 10,
    };

    let result = LifecycleManager::send_heartbeat_with_retry(&client, &retry_config, 3).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn transition_idempotent_same_state() {
    let service = LoamSpineService::new();
    let config = LoamSpineConfig::default();
    let manager = LifecycleManager::new(service, config);

    assert_eq!(manager.state(), ServiceState::Stopped);
    manager.transition(ServiceState::Stopped);
    assert_eq!(manager.state(), ServiceState::Stopped);
}

#[tokio::test]
async fn send_heartbeat_with_retry_fails_at_total_limit_boundary() {
    let client = crate::discovery_client::DiscoveryClient::for_testing("http://127.0.0.1:1");
    let retry_config = crate::config::HeartbeatRetryConfig {
        backoff_seconds: vec![0, 0, 0, 0, 0],
        max_failures_before_degraded: 2,
        max_failures_total: 3,
    };

    let result = LifecycleManager::send_heartbeat_with_retry(&client, &retry_config, 2).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn send_heartbeat_with_retry_empty_backoff_still_fails() {
    let client = crate::discovery_client::DiscoveryClient::for_testing("http://127.0.0.1:1");
    let retry_config = crate::config::HeartbeatRetryConfig {
        backoff_seconds: vec![],
        max_failures_before_degraded: 1,
        max_failures_total: 5,
    };

    let result = LifecycleManager::send_heartbeat_with_retry(&client, &retry_config, 0).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn lifecycle_state_serialization_roundtrip() {
    let states = [
        ServiceState::Starting,
        ServiceState::Ready,
        ServiceState::Running,
        ServiceState::Degraded,
        ServiceState::Stopping,
        ServiceState::Stopped,
        ServiceState::Error,
    ];
    for state in &states {
        let json = serde_json::to_string(state).unwrap();
        let parsed: ServiceState = serde_json::from_str(&json).unwrap();
        assert_eq!(*state, parsed);
    }
}

#[tokio::test]
async fn lifecycle_subscribe_tracks_multiple_transitions() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;

    let mut manager = LifecycleManager::new(service, config);
    let rx = manager.subscribe_state();

    assert_eq!(*rx.borrow(), ServiceState::Stopped);

    manager.start().await.expect("start");
    assert_eq!(*rx.borrow(), ServiceState::Running);

    manager.stop().await.expect("stop");
    assert_eq!(*rx.borrow(), ServiceState::Stopped);

    manager.start().await.expect("start again");
    assert_eq!(*rx.borrow(), ServiceState::Running);
}

#[tokio::test]
async fn advertise_capabilities_calls_client() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.discovery_enabled = false;
    config.discovery.tarpc_endpoint = "http://localhost:9001".to_string();
    config.discovery.jsonrpc_endpoint = "http://localhost:8080".to_string();

    let mut manager = LifecycleManager::new(service, config);
    manager.start().await.expect("start");

    let success_client =
        crate::discovery_client::DiscoveryClient::for_testing_success("http://test:8082");
    let result = manager.advertise_capabilities(&success_client).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn advertise_capabilities_propagates_error() {
    let service = LoamSpineService::new();
    let mut config = LoamSpineConfig::default();
    config.discovery.tarpc_endpoint = "http://localhost:9001".to_string();
    config.discovery.jsonrpc_endpoint = "http://localhost:8080".to_string();

    let manager = LifecycleManager::new(service, config);

    let fail_client = crate::discovery_client::DiscoveryClient::for_testing("http://127.0.0.1:1");
    let result = manager.advertise_capabilities(&fail_client).await;
    assert!(result.is_err());
}

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
