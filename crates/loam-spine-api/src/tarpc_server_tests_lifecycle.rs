// SPDX-License-Identifier: AGPL-3.0-or-later

//! Server lifecycle, configuration, and TCP integration tests for the tarpc server.
//!
//! Extracted from `tarpc_server_tests.rs` to keep each file under 1000 lines.
//! Tests here cover: server bind, config, TCP client round-trip, commit
//! delegation via RPC trait, and bind-collision error handling.

use super::*;

use crate::rpc::LoamSpineRpcClient;
use tarpc::client;
use tarpc::serde_transport::tcp;
use tarpc::tokio_serde::formats::Json;

#[tokio::test]
async fn test_run_tarpc_server_binds() {
    use std::net::SocketAddr;

    let addr: SocketAddr = "127.0.0.1:0".parse().expect("parse");
    let service = LoamSpineRpcService::default_service();

    let result = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        run_tarpc_server(addr, service),
    )
    .await;

    assert!(result.is_err(), "server runs until timeout");
}

#[test]
fn test_tarpc_server_config_default() {
    let config = TarpcServerConfig::default();
    assert_eq!(
        config.max_concurrent_requests,
        DEFAULT_MAX_CONCURRENT_REQUESTS
    );
    assert_eq!(config.max_channels_per_ip, DEFAULT_MAX_CHANNELS_PER_IP);
}

#[test]
fn test_tarpc_server_config_clone_and_debug() {
    let config = TarpcServerConfig {
        max_concurrent_requests: 50,
        max_channels_per_ip: 5,
    };
    let cloned = config.clone();
    assert_eq!(cloned.max_concurrent_requests, 50);
    assert_eq!(cloned.max_channels_per_ip, 5);
    let debug = format!("{config:?}");
    assert!(debug.contains("TarpcServerConfig"));
}

#[tokio::test]
async fn test_run_tarpc_server_with_custom_config() {
    use std::net::SocketAddr;

    let addr: SocketAddr = "127.0.0.1:0".parse().expect("parse");
    let service = LoamSpineRpcService::default_service();
    let config = TarpcServerConfig {
        max_concurrent_requests: 10,
        max_channels_per_ip: 2,
    };

    let result = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        run_tarpc_server_with_config(addr, service, config),
    )
    .await;

    assert!(result.is_err(), "server runs until timeout");
}

#[tokio::test]
async fn test_tarpc_commit_session_via_rpc() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();

    let create_request = CreateSpineRequest {
        owner: Did::new("did:key:z6MkCommitTest"),
        name: "Commit Spine".to_string(),
        config: None,
    };
    let create_resp = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .expect("create spine");

    let commit_request = CommitSessionRequest {
        spine_id: create_resp.spine_id,
        session_id: uuid::Uuid::now_v7(),
        session_hash: [0u8; 32],
        vertex_count: 42,
        committer: Did::new("did:key:z6MkCommitTest"),
    };
    let result = LoamSpineRpc::commit_session(server, ctx, commit_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tarpc_commit_braid_via_rpc() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();

    let create_request = CreateSpineRequest {
        owner: Did::new("did:key:z6MkBraidTest"),
        name: "Braid Spine".to_string(),
        config: None,
    };
    let create_resp = LoamSpineRpc::create_spine(server.clone(), ctx, create_request)
        .await
        .expect("create spine");

    let braid_request = CommitBraidRequest {
        spine_id: create_resp.spine_id,
        braid_id: uuid::Uuid::now_v7(),
        braid_hash: [0u8; 32],
        subjects: vec![Did::new("did:key:z6MkSubject1")],
        committer: Did::new("did:key:z6MkBraidTest"),
    };
    let result = LoamSpineRpc::commit_braid(server, ctx, braid_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tarpc_get_certificate_not_found() {
    let server = LoamSpineTarpcServer::default_server();
    let ctx = tarpc::context::current();

    let request = GetCertificateRequest {
        certificate_id: uuid::Uuid::now_v7(),
    };
    let result = LoamSpineRpc::get_certificate(server, ctx, request).await;
    assert!(result.is_ok());
    assert!(!result.expect("ok").found);
}

#[tokio::test]
async fn run_tarpc_server_processes_tcp_client_health_check() {
    use std::net::SocketAddr;
    use std::time::Duration;

    let port = portpicker::pick_unused_port().expect("unused port");
    let addr: SocketAddr = format!("127.0.0.1:{port}")
        .parse()
        .expect("parse socket addr");

    let server = tokio::spawn(run_tarpc_server(
        addr,
        LoamSpineRpcService::default_service(),
    ));

    let mut transport = None;
    for _ in 0..60 {
        match tcp::connect(addr, Json::default).await {
            Ok(t) => {
                transport = Some(t);
                break;
            }
            Err(_) => tokio::time::sleep(Duration::from_millis(25)).await,
        }
    }
    let transport = transport.expect("connect to tarpc listener");
    let client = LoamSpineRpcClient::new(client::Config::default(), transport).spawn();

    let ctx = tarpc::context::current();
    let response = client
        .health_check(
            ctx,
            HealthCheckRequest {
                include_details: false,
            },
        )
        .await
        .expect("tarpc transport")
        .expect("health_check api");
    assert!(response.status.is_healthy());

    server.abort();
}

#[tokio::test]
async fn run_tarpc_server_bind_fails_when_address_in_use() {
    use std::net::SocketAddr;
    use std::time::Duration;

    let port = portpicker::pick_unused_port().expect("unused port");
    let addr: SocketAddr = format!("127.0.0.1:{port}")
        .parse()
        .expect("parse socket addr");

    let first = tokio::spawn(run_tarpc_server(
        addr,
        LoamSpineRpcService::default_service(),
    ));
    tokio::time::sleep(Duration::from_millis(100)).await;

    let err = run_tarpc_server(addr, LoamSpineRpcService::default_service())
        .await
        .expect_err("second bind should fail");
    let msg = err.to_string();
    assert!(msg.contains("bind failed"), "unexpected bind error: {msg}");
    let source = std::error::Error::source(&err);
    assert!(
        source.is_some(),
        "Bind error should preserve io::Error source"
    );

    first.abort();
}
