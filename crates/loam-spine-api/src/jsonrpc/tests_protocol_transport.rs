// SPDX-License-Identifier: AGPL-3.0-or-later

//! Transport-level JSON-RPC tests: UDS server lifecycle, TCP server lifecycle,
//! HTTP/1.1 keep-alive, concurrent UDS load, `outcome_to_response` mapping,
//! and serialization helpers.

use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// =========================================================================
// UDS helper
// =========================================================================

/// Send a JSON-RPC request over a UDS stream and return the parsed response.
#[cfg(unix)]
async fn uds_rpc(stream: &mut tokio::net::UnixStream, request: &str) -> serde_json::Value {
    stream.write_all(request.as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = vec![0u8; 65536];
    let n = tokio::time::timeout(std::time::Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();

    serde_json::from_slice(&buf[..n]).unwrap()
}

// =========================================================================
// UDS server tests
// =========================================================================

#[cfg(unix)]
#[tokio::test]
async fn uds_server_starts_and_accepts_connections() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("test-jsonrpc.sock");
    let service = crate::service::LoamSpineRpcService::default_service();

    let handle = super::run_jsonrpc_uds_server(&sock_path, service, None)
        .await
        .unwrap();

    assert!(sock_path.exists());
    assert_eq!(handle.path(), sock_path);

    let mut stream = tokio::net::UnixStream::connect(&sock_path).await.unwrap();
    let response = uds_rpc(
        &mut stream,
        r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#,
    )
    .await;

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert_eq!(response["result"]["status"], "alive");

    handle.stop();
}

#[cfg(unix)]
#[tokio::test]
async fn uds_server_removes_stale_socket() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("stale.sock");

    std::fs::write(&sock_path, "stale socket data").unwrap();
    assert!(sock_path.exists());

    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_uds_server(&sock_path, service, None)
        .await
        .unwrap();

    assert!(sock_path.exists());
    handle.stop();
}

#[cfg(unix)]
#[tokio::test]
async fn uds_server_creates_parent_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("nested").join("dir").join("test.sock");

    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_uds_server(&sock_path, service, None)
        .await
        .unwrap();

    assert!(sock_path.exists());
    handle.stop();
}

#[cfg(unix)]
#[tokio::test]
async fn uds_server_drop_removes_socket() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("drop-test.sock");

    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_uds_server(&sock_path, service, None)
        .await
        .unwrap();

    assert!(sock_path.exists());
    drop(handle);
    assert!(!sock_path.exists());
}

#[cfg(unix)]
#[tokio::test]
async fn uds_server_shutdown_via_stop() {
    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("shutdown-test.sock");

    let service = crate::service::LoamSpineRpcService::default_service();
    let mut handle = super::run_jsonrpc_uds_server(&sock_path, service, None)
        .await
        .unwrap();

    handle.stop();
    let result = tokio::time::timeout(std::time::Duration::from_secs(2), handle.stopped()).await;
    assert!(result.is_ok(), "server should stop within timeout");
}

// =========================================================================
// TCP server handle
// =========================================================================

#[tokio::test]
async fn tcp_server_starts_and_stops() {
    let addr = "127.0.0.1:0".parse().unwrap();
    let service = crate::service::LoamSpineRpcService::default_service();
    let mut handle = super::run_jsonrpc_server(addr, service).await.unwrap();

    let bound_addr = handle.local_addr();
    assert_ne!(bound_addr.port(), 0);

    handle.stop();
    let result = tokio::time::timeout(std::time::Duration::from_secs(2), handle.stopped()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn tcp_server_accepts_raw_jsonrpc() {
    let addr = "127.0.0.1:0".parse().unwrap();
    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_server(addr, service).await.unwrap();
    let bound_addr = handle.local_addr();

    let mut stream = tokio::net::TcpStream::connect(bound_addr).await.unwrap();
    let request = r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#;
    stream.write_all(request.as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = vec![0u8; 4096];
    let n = tokio::time::timeout(std::time::Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();
    assert!(n > 0);

    let response: serde_json::Value = serde_json::from_slice(&buf[..n]).unwrap();
    assert!(response.get("result").is_some());

    handle.stop();
}

#[tokio::test]
async fn tcp_server_accepts_http_post() {
    let addr = "127.0.0.1:0".parse().unwrap();
    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_server(addr, service).await.unwrap();
    let bound_addr = handle.local_addr();

    let body = r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#;
    let http_request = format!(
        "POST / HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );

    let mut stream = tokio::net::TcpStream::connect(bound_addr).await.unwrap();
    stream.write_all(http_request.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = vec![0u8; 8192];
    let n = tokio::time::timeout(std::time::Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();
    let response_str = String::from_utf8_lossy(&buf[..n]);
    assert!(response_str.contains("HTTP/1.1 200 OK"));
    assert!(response_str.contains("result"));

    handle.stop();
}

/// Regression test: HTTP/1.1 persistent connections (keep-alive).
///
/// Verifies that multiple HTTP POST requests can be sent on a single TCP
/// connection without the server closing after the first response.
/// See primalSpring audit: "loamSpine connection closes after first response".
#[tokio::test]
async fn tcp_http_persistent_connection_keepalive() {
    let addr = "127.0.0.1:0".parse().unwrap();
    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_server(addr, service).await.unwrap();
    let bound_addr = handle.local_addr();

    let mut stream = tokio::net::TcpStream::connect(bound_addr).await.unwrap();

    for req_id in 1..=3 {
        let body = format!(r#"{{"jsonrpc":"2.0","method":"health.liveness","id":{req_id}}}"#);
        let http_request = format!(
            "POST / HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );

        stream.write_all(http_request.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();

        let mut buf = vec![0u8; 8192];
        let n = tokio::time::timeout(std::time::Duration::from_secs(2), stream.read(&mut buf))
            .await
            .unwrap()
            .unwrap();

        let response_str = String::from_utf8_lossy(&buf[..n]);
        assert!(
            response_str.contains("HTTP/1.1 200 OK"),
            "request {req_id}: missing 200 OK in: {response_str}"
        );
        assert!(
            response_str.contains("keep-alive"),
            "request {req_id}: missing keep-alive header in: {response_str}"
        );
        assert!(
            response_str.contains("result"),
            "request {req_id}: missing result in: {response_str}"
        );
    }

    handle.stop();
}

/// Verify `Connection: close` in request headers terminates the connection
/// after that response (server respects client close intent).
#[tokio::test]
async fn tcp_http_connection_close_header_respected() {
    let addr = "127.0.0.1:0".parse().unwrap();
    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_server(addr, service).await.unwrap();
    let bound_addr = handle.local_addr();

    let body = r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#;
    let http_request = format!(
        "POST / HTTP/1.1\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );

    let mut stream = tokio::net::TcpStream::connect(bound_addr).await.unwrap();
    stream.write_all(http_request.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = vec![0u8; 8192];
    let n = tokio::time::timeout(std::time::Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();

    let response_str = String::from_utf8_lossy(&buf[..n]);
    assert!(response_str.contains("Connection: close"));
    assert!(response_str.contains("result"));

    // After `Connection: close`, server should have ended the handler.
    // A second read should return 0 (EOF) once the server task completes.
    let mut trailing = vec![0u8; 256];
    let eof = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        stream.read(&mut trailing),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(eof, 0, "expected EOF after Connection: close");

    handle.stop();
}

// =========================================================================
// outcome_to_response coverage
// =========================================================================

#[test]
fn outcome_to_response_ok() {
    let id = serde_json::Value::Number(1.into());
    let outcome = loam_spine_core::error::DispatchOutcome::Ok(serde_json::json!(true));
    let resp = outcome_to_response(id, outcome);
    assert!(resp.error.is_none());
    assert_eq!(resp.result.unwrap(), serde_json::json!(true));
}

#[test]
fn outcome_to_response_application_error() {
    let id = serde_json::Value::Number(2.into());
    let outcome = loam_spine_core::error::DispatchOutcome::ApplicationError {
        code: -32000,
        message: "app error".to_string(),
    };
    let resp = outcome_to_response(id, outcome);
    assert!(resp.result.is_none());
    let err = resp.error.unwrap();
    assert_eq!(err.code, -32000);
    assert_eq!(err.message, "app error");
}

#[test]
fn outcome_to_response_protocol_error_jsonrpc() {
    let id = serde_json::Value::Number(3.into());
    let outcome = loam_spine_core::error::DispatchOutcome::<serde_json::Value>::ProtocolError(
        loam_spine_core::error::LoamSpineError::ipc(
            loam_spine_core::error::IpcErrorPhase::JsonRpcError(-32601),
            "method not found",
        ),
    );
    let resp = outcome_to_response(id, outcome);
    let err = resp.error.unwrap();
    assert_eq!(err.code, -32601);
}

#[test]
fn outcome_to_response_protocol_error_other() {
    let id = serde_json::Value::Number(4.into());
    let outcome = loam_spine_core::error::DispatchOutcome::<serde_json::Value>::ProtocolError(
        loam_spine_core::error::LoamSpineError::Internal("some error".to_string()),
    );
    let resp = outcome_to_response(id, outcome);
    let err = resp.error.unwrap();
    assert_eq!(err.code, -32000);
}

// =========================================================================
// serialize_response / serialize_response_batch helpers
// =========================================================================

#[test]
fn serialize_response_produces_valid_json() {
    let resp = JsonRpcResponse::success(serde_json::Value::Number(1.into()), serde_json::json!(42));
    let bytes = serialize_response(&resp);
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["result"], 42);
}

#[test]
fn serialize_response_batch_produces_valid_json() {
    let responses = vec![
        JsonRpcResponse::success(serde_json::Value::Number(1.into()), serde_json::json!("a")),
        JsonRpcResponse::success(serde_json::Value::Number(2.into()), serde_json::json!("b")),
    ];
    let bytes = serialize_response_batch(&responses);
    let parsed: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed.len(), 2);
}

#[test]
fn is_notification_missing_id() {
    let val = serde_json::json!({"jsonrpc": "2.0", "method": "test"});
    assert!(is_notification(&val));
}

#[test]
fn is_notification_null_id() {
    let val = serde_json::json!({"jsonrpc": "2.0", "method": "test", "id": null});
    assert!(is_notification(&val));
}

#[test]
fn is_notification_with_id() {
    let val = serde_json::json!({"jsonrpc": "2.0", "method": "test", "id": 1});
    assert!(!is_notification(&val));
}

// =========================================================================
// Trio IPC stability: concurrent UDS load test (8 clients × 5 requests)
//
// Matches sweetGrass v0.7.27 pattern. Verifies that the UDS server handles
// concurrent persistent connections without dropped responses, buffering
// stalls, or semaphore starvation.
// =========================================================================

#[cfg(unix)]
#[tokio::test]
#[expect(
    clippy::panic,
    reason = "spawned tasks need contextual panic messages for debugging concurrent failures"
)]
async fn uds_concurrent_load_8x5() {
    const CLIENTS: usize = 8;
    const REQUESTS_PER_CLIENT: usize = 5;

    let tmp = tempfile::tempdir().unwrap();
    let sock_path = tmp.path().join("load-test.sock");
    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_uds_server(&sock_path, service, None)
        .await
        .unwrap();

    let mut handles = Vec::with_capacity(CLIENTS);
    for client_id in 0..CLIENTS {
        let path = sock_path.clone();
        handles.push(tokio::spawn(async move {
            let mut stream = tokio::net::UnixStream::connect(&path)
                .await
                .unwrap_or_else(|e| panic!("client {client_id} connect: {e}"));

            for req_id in 0..REQUESTS_PER_CLIENT {
                let id = client_id * REQUESTS_PER_CLIENT + req_id;
                let request =
                    format!(r#"{{"jsonrpc":"2.0","method":"health.liveness","id":{id}}}"#,);
                stream.write_all(request.as_bytes()).await.unwrap();
                stream.write_all(b"\n").await.unwrap();
                stream.flush().await.unwrap();

                let mut buf = vec![0u8; 4096];
                let n =
                    tokio::time::timeout(std::time::Duration::from_secs(5), stream.read(&mut buf))
                        .await
                        .unwrap_or_else(|_| panic!("client {client_id} req {req_id} timed out"))
                        .unwrap_or_else(|e| panic!("client {client_id} req {req_id} read: {e}"));

                assert!(n > 0, "client {client_id} req {req_id}: empty response");
                let resp: serde_json::Value =
                    serde_json::from_slice(&buf[..n]).unwrap_or_else(|e| {
                        panic!(
                            "client {client_id} req {req_id} parse: {e} — raw: {}",
                            String::from_utf8_lossy(&buf[..n])
                        )
                    });
                assert_eq!(resp["id"], id, "response id mismatch");
                assert_eq!(
                    resp["result"]["status"], "alive",
                    "client {client_id} req {req_id}: unexpected result"
                );
            }
        }));
    }

    let mut failures = Vec::new();
    for (i, h) in handles.into_iter().enumerate() {
        if let Err(e) = h.await {
            failures.push(format!("client {i}: {e}"));
        }
    }
    assert!(
        failures.is_empty(),
        "concurrent UDS load test failures: {failures:?}"
    );

    handle.stop();
}

// =========================================================================
// BTSP NDJSON with static btsp_config (regression: provider socket wiring)
// =========================================================================

/// Minimal mock BTSP provider that handles the three JSON-RPC methods
/// needed for a full NDJSON handshake.
#[cfg(unix)]
async fn handle_mock_btsp_provider_conn(stream: tokio::net::UnixStream) {
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = tokio::io::BufReader::new(reader);
    let mut line = String::new();
    let _ = tokio::io::AsyncBufReadExt::read_line(&mut buf_reader, &mut line).await;

    let request: serde_json::Value =
        serde_json::from_str(line.trim()).unwrap_or(serde_json::Value::Null);
    let method = request
        .get("method")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let id = request
        .get("id")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let response = match method {
        "btsp.session.create" => serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "session_token": "tok_test_001",
                "server_ephemeral_pub": "mock_server_pub",
                "challenge": "bW9ja19jaGFsbGVuZ2U="
            }
        }),
        "btsp.session.verify" => serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": { "verified": true, "session_id": "test_session_001", "cipher": "null" }
        }),
        "btsp.negotiate" => serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "result": { "cipher": "null", "accepted": true }
        }),
        _ => serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32601, "message": "method not found" }
        }),
    };

    let mut bytes = serde_json::to_vec(&response).unwrap();
    bytes.push(b'\n');
    let _ = writer.write_all(&bytes).await;
    let _ = writer.flush().await;
}

/// Spawn a mock BTSP provider and return its socket path.
#[cfg(unix)]
async fn spawn_test_btsp_provider(
    dir: &std::path::Path,
) -> (std::path::PathBuf, tokio::task::JoinHandle<()>) {
    let socket = dir.join("btsp-mock-provider.sock");
    let _ = std::fs::remove_file(&socket);
    let listener = tokio::net::UnixListener::bind(&socket).unwrap();

    let handle = tokio::spawn(async move {
        for _ in 0..3 {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(handle_mock_btsp_provider_conn(stream));
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    (socket, handle)
}

/// NDJSON BTSP handshake through the UDS server when `btsp_config` is `Some`.
///
/// Before the fix, this path was unreachable — static BTSP mode sent NDJSON
/// data into the binary length-prefixed handshake, which failed.
#[cfg(unix)]
#[test]
fn uds_ndjson_btsp_through_static_config() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    temp_env::with_var(
        "FAMILY_SEED",
        Some("test_seed_for_btsp_integration"),
        || {
            rt.block_on(async {
                let tmp = tempfile::tempdir().unwrap();
                let (provider_socket, _provider_handle) =
                    spawn_test_btsp_provider(tmp.path()).await;

                let btsp_config = loam_spine_core::btsp::BtspHandshakeConfig {
                    required: true,
                    provider_socket: provider_socket.clone(),
                    family_id: "test-fam".into(),
                };

                let sock_path = tmp.path().join("ndjson-btsp-static.sock");
                let service = crate::service::LoamSpineRpcService::default_service();
                let handle = super::run_jsonrpc_uds_server(&sock_path, service, Some(btsp_config))
                    .await
                    .unwrap();

                let stream = tokio::net::UnixStream::connect(&sock_path).await.unwrap();
                let (reader, mut writer) = stream.into_split();
                let mut buf_reader = tokio::io::BufReader::new(reader);

                let client_hello = serde_json::json!({
                    "protocol": "btsp",
                    "version": 1,
                    "client_ephemeral_pub": "dGVzdC1rZXk="
                });
                let mut line = serde_json::to_string(&client_hello).unwrap();
                line.push('\n');
                writer.write_all(line.as_bytes()).await.unwrap();

                let mut server_hello_line = String::new();
                tokio::io::AsyncBufReadExt::read_line(&mut buf_reader, &mut server_hello_line)
                    .await
                    .unwrap();
                let server_hello: serde_json::Value =
                    serde_json::from_str(server_hello_line.trim()).unwrap();
                assert_eq!(server_hello["version"], 1);
                assert!(
                    server_hello.get("session_id").is_some(),
                    "ServerHello should contain session_id"
                );

                let cr = serde_json::json!({
                    "response": "mock_hmac",
                    "preferred_cipher": "null"
                });
                let mut cr_line = serde_json::to_string(&cr).unwrap();
                cr_line.push('\n');
                writer.write_all(cr_line.as_bytes()).await.unwrap();

                let mut complete_line = String::new();
                tokio::io::AsyncBufReadExt::read_line(&mut buf_reader, &mut complete_line)
                    .await
                    .unwrap();
                let complete: serde_json::Value =
                    serde_json::from_str(complete_line.trim()).unwrap();
                assert_eq!(complete["cipher"], "null");
                assert_eq!(complete["session_id"], "test_session_001");

                handle.stop();
            });
        },
    );
}

/// Plain JSON-RPC still works when `btsp_config` is `Some` but the client
/// sends JSON-RPC (not NDJSON BTSP). The auto-detect routes to JSON-RPC dispatch.
#[cfg(unix)]
#[tokio::test]
async fn uds_jsonrpc_with_static_btsp_config() {
    let tmp = tempfile::tempdir().unwrap();

    let btsp_config = loam_spine_core::btsp::BtspHandshakeConfig {
        required: true,
        provider_socket: tmp.path().join("no-such-provider.sock"),
        family_id: "test-fam".into(),
    };

    let sock_path = tmp.path().join("jsonrpc-with-btsp.sock");
    let service = crate::service::LoamSpineRpcService::default_service();
    let handle = super::run_jsonrpc_uds_server(&sock_path, service, Some(btsp_config))
        .await
        .unwrap();

    let mut stream = tokio::net::UnixStream::connect(&sock_path).await.unwrap();
    let response = uds_rpc(
        &mut stream,
        r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#,
    )
    .await;

    assert_eq!(response["result"]["status"], "alive");

    handle.stop();
}
