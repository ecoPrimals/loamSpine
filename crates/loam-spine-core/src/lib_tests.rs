// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[tokio::test]
async fn lifecycle_start_stop() {
    let dir = tempfile::tempdir().unwrap_or_else(|_| unreachable!());
    let config = LoamSpineConfig::default().with_storage_path(dir.path());

    let mut spine = LoamSpine::new(config);
    assert_eq!(spine.state(), PrimalState::Created);

    spine.start().await.ok();
    assert_eq!(spine.state(), PrimalState::Running);
    assert!(spine.uptime_secs().is_some());

    spine.stop().await.ok();
    assert_eq!(spine.state(), PrimalState::Stopped);
}

#[tokio::test]
async fn health_check_when_running() {
    let dir = tempfile::tempdir().unwrap_or_else(|_| unreachable!());
    let config = LoamSpineConfig::default().with_storage_path(dir.path());

    let mut spine = LoamSpine::new(config);
    spine.start().await.ok();

    let report = spine.health_check().await;
    assert!(report.is_ok());

    let report = report.unwrap_or_else(|_| unreachable!());
    assert!(report.status.is_healthy());
    assert!(!report.components.is_empty());
}

#[test]
fn health_status_when_not_running() {
    let config = LoamSpineConfig::default();
    let spine = LoamSpine::new(config);

    assert!(spine.health_status().is_unhealthy());
}

#[test]
fn config_accessor() {
    let config = LoamSpineConfig::new("TestSpine");
    let spine = LoamSpine::new(config);

    assert_eq!(spine.config().name, "TestSpine");
}

#[tokio::test]
async fn capabilities_accessor() {
    let config = LoamSpineConfig::default();
    let spine = LoamSpine::new(config);

    assert_eq!(
        spine.capabilities().signer_status().await,
        CapabilityStatus::Unavailable
    );
}

#[tokio::test]
async fn with_capabilities_constructor() {
    let config = LoamSpineConfig::default();
    let caps = CapabilityRegistry::new();

    let signer = std::sync::Arc::new(MockSigner::new(Did::new("did:test")));
    caps.register_signer(signer).await;

    let spine = LoamSpine::with_capabilities(config, caps);
    assert_eq!(
        spine.capabilities().signer_status().await,
        CapabilityStatus::Available
    );
}

#[tokio::test]
async fn start_already_running() {
    let dir = tempfile::tempdir().unwrap_or_else(|_| unreachable!());
    let config = LoamSpineConfig::default().with_storage_path(dir.path());

    let mut spine = LoamSpine::new(config);
    spine.start().await.ok();
    assert_eq!(spine.state(), PrimalState::Running);

    let result = spine.start().await;
    assert!(result.is_ok());
    assert_eq!(spine.state(), PrimalState::Running);
}

#[tokio::test]
async fn stop_already_stopped() {
    let dir = tempfile::tempdir().unwrap_or_else(|_| unreachable!());
    let config = LoamSpineConfig::default().with_storage_path(dir.path());

    let mut spine = LoamSpine::new(config);
    spine.start().await.ok();
    spine.stop().await.ok();
    assert_eq!(spine.state(), PrimalState::Stopped);

    let result = spine.stop().await;
    assert!(result.is_ok());
    assert_eq!(spine.state(), PrimalState::Stopped);
}

#[tokio::test]
async fn uptime_when_not_started() {
    let config = LoamSpineConfig::default();
    let spine = LoamSpine::new(config);

    assert!(spine.uptime_secs().is_none());
}

#[tokio::test]
async fn health_check_storage_exists() {
    let dir = tempfile::tempdir().unwrap_or_else(|_| unreachable!());
    let config = LoamSpineConfig::default().with_storage_path(dir.path());

    let mut spine = LoamSpine::new(config);
    spine.start().await.ok();

    let report = spine
        .health_check()
        .await
        .unwrap_or_else(|_| unreachable!());

    let storage = report.components.iter().find(|c| c.name == "storage");
    assert!(storage.is_some());
}

#[tokio::test]
async fn health_check_with_capabilities() {
    let dir = tempfile::tempdir().unwrap_or_else(|_| unreachable!());
    let config = LoamSpineConfig::default().with_storage_path(dir.path());
    let caps = CapabilityRegistry::new();

    let signer = std::sync::Arc::new(MockSigner::new(Did::new("did:test")));
    caps.register_signer(signer).await;

    let mut spine = LoamSpine::with_capabilities(config, caps);
    spine.start().await.ok();

    let report = spine
        .health_check()
        .await
        .unwrap_or_else(|_| unreachable!());

    let signer_status = report.components.iter().find(|c| c.name == "Signer");
    assert!(signer_status.is_some());
}
