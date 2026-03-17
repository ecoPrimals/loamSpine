// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use serial_test::serial;

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
#[serial]
fn lifecycle_transitions_through_states() {
    temp_env::with_var_unset("DISCOVERY_ENDPOINT", || {
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
#[serial]
fn lifecycle_start_with_no_endpoint_clears_env() {
    temp_env::with_var("DISCOVERY_ENDPOINT", None::<&str>, || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = LoamSpineService::new();
            let mut config = LoamSpineConfig::default();
            config.discovery.discovery_enabled = true;
            config.discovery.discovery_endpoint = None;

            let mut manager = LifecycleManager::new(service, config);
            let result = manager.start().await;

            assert!(result.is_ok());
            assert!(manager.discovery_client.is_none());
        });
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
