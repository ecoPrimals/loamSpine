// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration tests for the BTSP handshake — framed and NDJSON paths.
//!
//! Uses mock BTSP provider UDS servers (aligned with `beardog_types::btsp::rpc`
//! response shapes) to test all handshake flows without actual crypto.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test assertions use expect/unwrap for failure clarity"
)]

use std::path::PathBuf;

use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};

use super::frame::{read_frame, write_frame};
use super::handshake::perform_server_handshake;
use super::wire::{
    ChallengeResponse, ClientHello, HandshakeComplete, HandshakeError, NdjsonClientHello,
    NdjsonServerHello,
};

// ---------------------------------------------------------------------------
// Mock BTSP provider server (beardog_types-compatible responses)
// ---------------------------------------------------------------------------

/// Spawn a mock BTSP provider that handles the three BTSP JSON-RPC methods.
///
/// Returns the socket path (in a temp dir) and a join handle.
async fn spawn_mock_provider(
    temp_dir: &std::path::Path,
    verify_ok: bool,
    cipher_allowed: bool,
) -> (PathBuf, tokio::task::JoinHandle<()>) {
    let socket_path = temp_dir.join("btsp-provider-test.sock");
    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path).expect("bind mock BTSP provider");

    let path = socket_path.clone();
    let handle = tokio::spawn(async move {
        for _ in 0..3 {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            let verify_ok = verify_ok;
            let cipher_allowed = cipher_allowed;
            tokio::spawn(async move {
                handle_mock_btsp_provider(stream, verify_ok, cipher_allowed).await;
            });
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    (path, handle)
}

async fn handle_mock_btsp_provider(stream: UnixStream, verify_ok: bool, cipher_allowed: bool) {
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
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "session_token": "tok_abcdef0123456789",
                "server_ephemeral_pub": "mock_server_pub_key",
                "challenge": "bW9ja19jaGFsbGVuZ2VfMzJfYnl0ZXM="
            }
        }),
        "btsp.session.verify" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "verified": verify_ok,
                "session_id": if verify_ok { Some("abcdef0123456789") } else { None::<&str> },
                "cipher": if verify_ok { Some("null") } else { None::<&str> }
            }
        }),
        "btsp.negotiate" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "cipher": "null",
                "accepted": cipher_allowed
            }
        }),
        _ => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": "method not found" }
        }),
    };

    let mut response_bytes = serde_json::to_vec(&response).expect("serialize mock response");
    response_bytes.push(b'\n');
    let _ = writer.write_all(&response_bytes).await;
    let _ = writer.flush().await;
}

// ---------------------------------------------------------------------------
// Full handshake integration tests (framed, with mock BTSP provider)
// ---------------------------------------------------------------------------

async fn mock_client_handshake(mut stream: UnixStream) -> bytes::Bytes {
    let client_hello = ClientHello {
        version: 1,
        client_ephemeral_pub: "mock_client_pub_key".to_string(),
    };
    let hello_bytes = serde_json::to_vec(&client_hello).expect("serialize hello");
    write_frame(&mut stream, &hello_bytes)
        .await
        .expect("write ClientHello");

    let _server_hello_bytes = read_frame(&mut stream).await.expect("read ServerHello");

    let cr = ChallengeResponse {
        response: "mock_hmac_response".to_string(),
        preferred_cipher: "null".to_string(),
    };
    let cr_bytes = serde_json::to_vec(&cr).expect("serialize cr");
    write_frame(&mut stream, &cr_bytes)
        .await
        .expect("write ChallengeResponse");

    read_frame(&mut stream).await.expect("read final frame")
}

const TEST_FAMILY_SEED: &str = "deadbeef01234567deadbeef01234567deadbeef01234567deadbeef01234567";

#[test]
fn handshake_success_full_sequence() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    temp_env::with_var("FAMILY_SEED", Some(TEST_FAMILY_SEED), || {
        rt.block_on(async {
            let tmp = tempfile::tempdir().expect("tempdir");
            let (provider_socket, _provider) = spawn_mock_provider(tmp.path(), true, true).await;

            let uds_path = tmp.path().join("loam-test.sock");
            let listener = UnixListener::bind(&uds_path).expect("bind loam test");

            let client_handle = tokio::spawn({
                let uds_path = uds_path.clone();
                async move {
                    let stream = UnixStream::connect(&uds_path).await.expect("connect");
                    mock_client_handshake(stream).await
                }
            });

            let (server_stream, _) = listener.accept().await.expect("accept");
            let (reader, mut writer) = server_stream.into_split();
            let mut buf_reader = tokio::io::BufReader::new(reader);
            let session = perform_server_handshake(&mut buf_reader, &mut writer, &provider_socket)
                .await
                .expect("handshake should succeed");

            assert_eq!(session.session_id, "abcdef0123456789");
            assert_eq!(session.cipher, "null");

            let final_bytes = client_handle.await.expect("client task");
            let complete: HandshakeComplete =
                serde_json::from_slice(&final_bytes).expect("parse HandshakeComplete");
            assert_eq!(complete.status, "ok");
            assert_eq!(complete.session_id, "abcdef0123456789");
            assert_eq!(complete.cipher, "null");
        });
    });
}

#[test]
fn handshake_failure_verify_rejected() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    temp_env::with_var("FAMILY_SEED", Some(TEST_FAMILY_SEED), || {
        rt.block_on(async {
            let tmp = tempfile::tempdir().expect("tempdir");
            let (provider_socket, _provider) = spawn_mock_provider(tmp.path(), false, true).await;

            let uds_path = tmp.path().join("loam-reject.sock");
            let listener = UnixListener::bind(&uds_path).expect("bind");

            let client_handle = tokio::spawn({
                let uds_path = uds_path.clone();
                async move {
                    let stream = UnixStream::connect(&uds_path).await.expect("connect");
                    mock_client_handshake(stream).await
                }
            });

            let (server_stream, _) = listener.accept().await.expect("accept");
            let (reader, mut writer) = server_stream.into_split();
            let mut buf_reader = tokio::io::BufReader::new(reader);
            let result =
                perform_server_handshake(&mut buf_reader, &mut writer, &provider_socket).await;

            assert!(result.is_err());
            let err_str = result.unwrap_err().to_string();
            assert!(
                err_str.contains("family verification") || err_str.contains("handshake failed"),
                "unexpected error: {err_str}"
            );

            let final_bytes = client_handle.await.expect("client task");
            let error: HandshakeError =
                serde_json::from_slice(&final_bytes).expect("parse HandshakeError");
            assert_eq!(error.error, "handshake_failed");
            assert_eq!(error.reason, "family_verification");
        });
    });
}

#[test]
fn handshake_failure_cipher_rejected() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    temp_env::with_var("FAMILY_SEED", Some(TEST_FAMILY_SEED), || {
        rt.block_on(async {
            let tmp = tempfile::tempdir().expect("tempdir");
            let (provider_socket, _provider) = spawn_mock_provider(tmp.path(), true, false).await;

            let uds_path = tmp.path().join("loam-cipher.sock");
            let listener = UnixListener::bind(&uds_path).expect("bind");

            let client_handle = tokio::spawn({
                let uds_path = uds_path.clone();
                async move {
                    let stream = UnixStream::connect(&uds_path).await.expect("connect");
                    mock_client_handshake(stream).await
                }
            });

            let (server_stream, _) = listener.accept().await.expect("accept");
            let (reader, mut writer) = server_stream.into_split();
            let mut buf_reader = tokio::io::BufReader::new(reader);
            let result =
                perform_server_handshake(&mut buf_reader, &mut writer, &provider_socket).await;

            assert!(result.is_err());
            let err_str = result.unwrap_err().to_string();
            assert!(err_str.contains("cipher"), "unexpected error: {err_str}");

            let final_bytes = client_handle.await.expect("client task");
            let error: HandshakeError =
                serde_json::from_slice(&final_bytes).expect("parse HandshakeError");
            assert_eq!(error.error, "cipher_rejected");
        });
    });
}

#[test]
fn handshake_failure_provider_unavailable() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    temp_env::with_var("FAMILY_SEED", Some(TEST_FAMILY_SEED), || {
        rt.block_on(async {
            let nonexistent = PathBuf::from("/tmp/btsp-no-such-socket-12345.sock");

            let (mut client, server) = tokio::io::duplex(8192);
            let (server_reader, mut server_writer) = tokio::io::split(server);
            let mut buf_reader = tokio::io::BufReader::new(server_reader);

            let client_handle = tokio::spawn(async move {
                let client_hello = ClientHello {
                    version: 1,
                    client_ephemeral_pub: "key".to_string(),
                };
                let bytes = serde_json::to_vec(&client_hello).expect("ser");
                write_frame(&mut client, &bytes).await.expect("write");
            });

            let result =
                perform_server_handshake(&mut buf_reader, &mut server_writer, &nonexistent).await;
            assert!(result.is_err());
            let err_str = result.unwrap_err().to_string();
            assert!(
                err_str.contains("unreachable") || err_str.contains("connect"),
                "unexpected error: {err_str}"
            );

            client_handle.await.expect("client task");
        });
    });
}

#[tokio::test]
async fn handshake_version_mismatch() {
    let (mut client, server) = tokio::io::duplex(8192);
    let (server_reader, mut server_writer) = tokio::io::split(server);
    let mut buf_reader = tokio::io::BufReader::new(server_reader);

    let client_handle = tokio::spawn(async move {
        let client_hello = ClientHello {
            version: 99,
            client_ephemeral_pub: "key".to_string(),
        };
        let bytes = serde_json::to_vec(&client_hello).expect("ser");
        write_frame(&mut client, &bytes).await.expect("write");

        let err_bytes = read_frame(&mut client).await.expect("read error frame");
        let err: HandshakeError = serde_json::from_slice(&err_bytes).expect("parse");
        assert_eq!(err.error, "unsupported_version");
    });

    let provider = PathBuf::from("/tmp/unused-btsp-provider.sock");
    let result = perform_server_handshake(&mut buf_reader, &mut server_writer, &provider).await;
    assert!(result.is_err());

    client_handle.await.expect("client task");
}

// ---------------------------------------------------------------------------
// NDJSON handshake integration tests (with mock BTSP provider)
// ---------------------------------------------------------------------------

async fn mock_ndjson_client_handshake(
    stream: UnixStream,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use tokio::io::AsyncBufReadExt;
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = tokio::io::BufReader::new(reader);

    let client_hello = NdjsonClientHello {
        protocol: "btsp".to_string(),
        version: 1,
        client_ephemeral_pub: "bW9ja19jbGllbnRfcHViX2tleQ==".to_string(),
    };
    let mut hello_line = serde_json::to_string(&client_hello)?;
    hello_line.push('\n');
    writer.write_all(hello_line.as_bytes()).await?;

    let mut server_hello_line = String::new();
    buf_reader.read_line(&mut server_hello_line).await?;
    let _server_hello: NdjsonServerHello = serde_json::from_str(server_hello_line.trim())?;

    let cr = ChallengeResponse {
        response: "mock_hmac_response".to_string(),
        preferred_cipher: "null".to_string(),
    };
    let mut cr_line = serde_json::to_string(&cr)?;
    cr_line.push('\n');
    writer.write_all(cr_line.as_bytes()).await?;

    let mut final_line = String::new();
    buf_reader.read_line(&mut final_line).await?;
    Ok(final_line)
}

#[test]
fn ndjson_handshake_success_full_sequence() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    temp_env::with_var("FAMILY_SEED", Some(TEST_FAMILY_SEED), || {
        rt.block_on(async {
            let tmp = tempfile::tempdir().expect("tempdir");
            let (provider_socket, _provider) = spawn_mock_provider(tmp.path(), true, true).await;

            let uds_path = tmp.path().join("loam-ndjson-test.sock");
            let listener = UnixListener::bind(&uds_path).expect("bind");

            let client_handle = tokio::spawn({
                let uds_path = uds_path.clone();
                async move {
                    let stream = UnixStream::connect(&uds_path).await.expect("connect");
                    mock_ndjson_client_handshake(stream).await
                }
            });

            let (server_stream, _) = listener.accept().await.expect("accept");
            let (reader, mut writer) = server_stream.into_split();
            let mut buf_reader = tokio::io::BufReader::new(reader);

            let mut first_line = String::new();
            tokio::io::AsyncBufReadExt::read_line(&mut buf_reader, &mut first_line)
                .await
                .expect("read first line");
            assert!(first_line.contains("\"protocol\""));
            assert!(first_line.contains("\"btsp\""));

            let session = super::handshake::perform_ndjson_server_handshake(
                &mut buf_reader,
                &mut writer,
                &provider_socket,
                &first_line,
            )
            .await
            .expect("ndjson handshake should succeed");

            assert_eq!(session.session_id, "abcdef0123456789");
            assert_eq!(session.cipher, "null");

            let final_line = client_handle
                .await
                .expect("client task")
                .expect("client ok");
            let complete: HandshakeComplete =
                serde_json::from_str(final_line.trim()).expect("parse HandshakeComplete");
            assert_eq!(complete.status, "ok");
            assert_eq!(complete.session_id, "abcdef0123456789");
            assert_eq!(complete.cipher, "null");
        });
    });
}

#[test]
fn ndjson_handshake_failure_verify_rejected() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    temp_env::with_var("FAMILY_SEED", Some(TEST_FAMILY_SEED), || {
        rt.block_on(async {
            let tmp = tempfile::tempdir().expect("tempdir");
            let (provider_socket, _provider) = spawn_mock_provider(tmp.path(), false, true).await;

            let uds_path = tmp.path().join("loam-ndjson-reject.sock");
            let listener = UnixListener::bind(&uds_path).expect("bind");

            let client_handle = tokio::spawn({
                let uds_path = uds_path.clone();
                async move {
                    let stream = UnixStream::connect(&uds_path).await.expect("connect");
                    mock_ndjson_client_handshake(stream).await
                }
            });

            let (server_stream, _) = listener.accept().await.expect("accept");
            let (reader, mut writer) = server_stream.into_split();
            let mut buf_reader = tokio::io::BufReader::new(reader);

            let mut first_line = String::new();
            tokio::io::AsyncBufReadExt::read_line(&mut buf_reader, &mut first_line)
                .await
                .expect("read first line");

            let result = super::handshake::perform_ndjson_server_handshake(
                &mut buf_reader,
                &mut writer,
                &provider_socket,
                &first_line,
            )
            .await;

            assert!(result.is_err());
            let err_str = result.unwrap_err().to_string();
            assert!(
                err_str.contains("family verification") || err_str.contains("handshake failed"),
                "unexpected error: {err_str}"
            );

            let final_line = client_handle
                .await
                .expect("client task")
                .expect("client ok");
            let error: HandshakeError =
                serde_json::from_str(final_line.trim()).expect("parse HandshakeError");
            assert_eq!(error.error, "handshake_failed");
        });
    });
}

#[tokio::test]
async fn ndjson_handshake_version_mismatch() {
    let (mut client, server) = tokio::io::duplex(8192);
    let (reader, mut writer) = tokio::io::split(server);
    let mut buf_reader = tokio::io::BufReader::new(reader);

    let client_handle = tokio::spawn(async move {
        use tokio::io::AsyncBufReadExt;
        let hello = r#"{"protocol":"btsp","version":99,"client_ephemeral_pub":"key"}"#;
        client
            .write_all(format!("{hello}\n").as_bytes())
            .await
            .expect("write");
        let mut reader = tokio::io::BufReader::new(client);
        let mut err_line = String::new();
        reader
            .read_line(&mut err_line)
            .await
            .expect("read error line");
        let err: HandshakeError =
            serde_json::from_str(err_line.trim()).expect("parse HandshakeError");
        assert_eq!(err.error, "unsupported_version");
    });

    let first_line = r#"{"protocol":"btsp","version":99,"client_ephemeral_pub":"key"}"#;
    let provider = PathBuf::from("/tmp/unused-btsp-provider.sock");
    let result = super::handshake::perform_ndjson_server_handshake(
        &mut buf_reader,
        &mut writer,
        &provider,
        first_line,
    )
    .await;
    assert!(result.is_err());

    client_handle.await.expect("client task");
}
