// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for the BTSP handshake module.
//!
//! Uses mock BTSP provider UDS servers to test all handshake paths without
//! requiring actual cryptographic operations.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test assertions use expect/unwrap for failure clarity"
)]

use std::path::PathBuf;

use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};

use super::config::provider_socket_name;
use super::frame::MAX_FRAME_SIZE;
use super::handshake::generate_challenge;
use super::*;

// ---------------------------------------------------------------------------
// Pure function tests
// ---------------------------------------------------------------------------

#[test]
fn btsp_required_with_family_id() {
    assert!(is_btsp_required_with(Some("abc123")));
}

#[test]
fn btsp_not_required_without_family_id() {
    assert!(!is_btsp_required_with(None));
}

#[test]
fn btsp_not_required_with_default_family() {
    assert!(!is_btsp_required_with(Some("default")));
}

#[test]
fn btsp_not_required_with_empty_family() {
    assert!(!is_btsp_required_with(Some("")));
}

#[test]
fn config_from_values_none_when_no_family() {
    assert!(BtspHandshakeConfig::from_values(None, None, None).is_none());
}

#[test]
fn config_from_values_none_when_default_family() {
    assert!(BtspHandshakeConfig::from_values(Some("default"), None, None).is_none());
}

#[test]
fn config_from_values_some_with_real_family() {
    let cfg = BtspHandshakeConfig::from_values(Some("fam1"), None, Some("/tmp/bio"))
        .expect("should be Some");
    assert!(cfg.required);
    assert_eq!(cfg.family_id, "fam1");
    assert_eq!(
        cfg.provider_socket,
        PathBuf::from("/tmp/bio/btsp-provider-fam1.sock")
    );
}

#[test]
fn config_from_values_with_socket_override() {
    let cfg = BtspHandshakeConfig::from_values(Some("fam1"), Some("/custom/provider.sock"), None)
        .expect("should be Some");
    assert_eq!(cfg.provider_socket, PathBuf::from("/custom/provider.sock"));
}

#[test]
fn provider_socket_name_with_family() {
    assert_eq!(
        provider_socket_name(Some("abc"), None),
        "btsp-provider-abc.sock"
    );
}

#[test]
fn provider_socket_name_without_family() {
    assert_eq!(provider_socket_name(None, None), "btsp-provider.sock");
}

#[test]
fn provider_socket_name_default_family() {
    assert_eq!(
        provider_socket_name(Some("default"), None),
        "btsp-provider.sock"
    );
}

#[test]
fn provider_socket_name_custom_provider() {
    assert_eq!(
        provider_socket_name(Some("abc"), Some("custodian")),
        "custodian-abc.sock"
    );
}

#[test]
fn resolve_provider_socket_with_dir() {
    let path = resolve_provider_socket_with(Some("fam"), Some("/run/biomeos"), None);
    assert_eq!(path, PathBuf::from("/run/biomeos/btsp-provider-fam.sock"));
}

#[test]
fn resolve_provider_socket_no_family_with_dir() {
    let path = resolve_provider_socket_with(None, Some("/run/biomeos"), None);
    assert_eq!(path, PathBuf::from("/run/biomeos/btsp-provider.sock"));
}

#[test]
fn resolve_provider_socket_custom_provider() {
    let path = resolve_provider_socket_with(Some("fam"), Some("/run/biomeos"), Some("custodian"));
    assert_eq!(path, PathBuf::from("/run/biomeos/custodian-fam.sock"));
}

// ---------------------------------------------------------------------------
// Frame I/O tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn frame_roundtrip() {
    let (mut client, mut server) = tokio::io::duplex(4096);
    let payload = b"hello btsp";

    let write_task = tokio::spawn(async move {
        write_frame(&mut client, payload).await.expect("write ok");
    });

    let read_task = tokio::spawn(async move { read_frame(&mut server).await.expect("read ok") });

    write_task.await.expect("write task");
    let received = read_task.await.expect("read task");
    assert_eq!(received.as_ref(), payload.as_slice());
}

#[tokio::test]
async fn frame_empty_payload() {
    let (mut client, mut server) = tokio::io::duplex(4096);

    let write_task = tokio::spawn(async move {
        write_frame(&mut client, b"").await.expect("write ok");
    });

    let read_task = tokio::spawn(async move { read_frame(&mut server).await.expect("read ok") });

    write_task.await.expect("write task");
    let received = read_task.await.expect("read task");
    assert!(received.is_empty());
}

#[tokio::test]
async fn frame_too_large_on_read() {
    let (mut client, mut server) = tokio::io::duplex(4096);

    tokio::spawn(async move {
        let bad_len: u32 = MAX_FRAME_SIZE + 1;
        client.write_u32(bad_len).await.expect("write len");
    });

    let result = read_frame(&mut server).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("too large"), "expected 'too large' in: {err}");
}

#[tokio::test]
async fn frame_truncated_body() {
    let (mut client, mut server) = tokio::io::duplex(4096);

    tokio::spawn(async move {
        client.write_u32(100).await.expect("write len");
        client.write_all(b"short").await.expect("write partial");
        drop(client);
    });

    let result = read_frame(&mut server).await;
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Wire type serde tests
// ---------------------------------------------------------------------------

#[test]
fn client_hello_roundtrip() {
    let msg = ClientHello {
        version: 1,
        client_ephemeral_pub: "AAAA".to_string(),
    };
    let bytes = serde_json::to_vec(&msg).expect("serialize");
    let decoded: ClientHello = serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(decoded.version, 1);
    assert_eq!(decoded.client_ephemeral_pub, "AAAA");
}

#[test]
fn server_hello_roundtrip() {
    let msg = ServerHello {
        version: 1,
        server_ephemeral_pub: "BBBB".to_string(),
        challenge: "CCCC".to_string(),
    };
    let bytes = serde_json::to_vec(&msg).expect("serialize");
    let decoded: ServerHello = serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(decoded.version, 1);
    assert_eq!(decoded.server_ephemeral_pub, "BBBB");
    assert_eq!(decoded.challenge, "CCCC");
}

#[test]
fn challenge_response_roundtrip() {
    let msg = ChallengeResponse {
        response: "hmac".to_string(),
        preferred_cipher: "null".to_string(),
    };
    let bytes = serde_json::to_vec(&msg).expect("serialize");
    let decoded: ChallengeResponse = serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(decoded.response, "hmac");
    assert_eq!(decoded.preferred_cipher, "null");
}

#[test]
fn handshake_complete_roundtrip() {
    let msg = HandshakeComplete {
        cipher: "null".to_string(),
        session_id: "deadbeef".to_string(),
    };
    let bytes = serde_json::to_vec(&msg).expect("serialize");
    let decoded: HandshakeComplete = serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(decoded.cipher, "null");
    assert_eq!(decoded.session_id, "deadbeef");
}

#[test]
fn handshake_error_roundtrip() {
    let msg = HandshakeError {
        error: "handshake_failed".to_string(),
        reason: "family_verification".to_string(),
    };
    let bytes = serde_json::to_vec(&msg).expect("serialize");
    let decoded: HandshakeError = serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(decoded.error, "handshake_failed");
    assert_eq!(decoded.reason, "family_verification");
}

// ---------------------------------------------------------------------------
// Mock BTSP provider server
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
        // Handle 3 sequential connections (one per provider_call).
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
                "session_id": "abcdef0123456789",
                "server_ephemeral_pub": "mock_server_pub_key",
                "handshake_key": "mock_handshake_key_base64"
            }
        }),
        "btsp.session.verify" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "verified": verify_ok,
                "session_key": if verify_ok { Some("mock_session_key") } else { None::<&str> }
            }
        }),
        "btsp.negotiate" => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "cipher": "null",
                "allowed": cipher_allowed
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
// Full handshake integration tests (with mock BTSP provider)
// ---------------------------------------------------------------------------

/// Simulates a BTSP client sending ClientHello, reading ServerHello,
/// sending ChallengeResponse, and reading the final frame.
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

#[tokio::test]
async fn handshake_success_full_sequence() {
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

    let (mut server_stream, _) = listener.accept().await.expect("accept");
    let session = perform_server_handshake(&mut server_stream, &provider_socket)
        .await
        .expect("handshake should succeed");

    assert_eq!(session.session_id, "abcdef0123456789");
    assert_eq!(session.cipher, "null");

    let final_bytes = client_handle.await.expect("client task");
    let complete: HandshakeComplete =
        serde_json::from_slice(&final_bytes).expect("parse HandshakeComplete");
    assert_eq!(complete.session_id, "abcdef0123456789");
    assert_eq!(complete.cipher, "null");
}

#[tokio::test]
async fn handshake_failure_verify_rejected() {
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

    let (mut server_stream, _) = listener.accept().await.expect("accept");
    let result = perform_server_handshake(&mut server_stream, &provider_socket).await;

    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("family verification") || err_str.contains("handshake failed"),
        "unexpected error: {err_str}"
    );

    let final_bytes = client_handle.await.expect("client task");
    let error: HandshakeError = serde_json::from_slice(&final_bytes).expect("parse HandshakeError");
    assert_eq!(error.error, "handshake_failed");
    assert_eq!(error.reason, "family_verification");
}

#[tokio::test]
async fn handshake_failure_cipher_rejected() {
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

    let (mut server_stream, _) = listener.accept().await.expect("accept");
    let result = perform_server_handshake(&mut server_stream, &provider_socket).await;

    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(err_str.contains("cipher"), "unexpected error: {err_str}");

    let final_bytes = client_handle.await.expect("client task");
    let error: HandshakeError = serde_json::from_slice(&final_bytes).expect("parse HandshakeError");
    assert_eq!(error.error, "cipher_rejected");
}

#[tokio::test]
async fn handshake_failure_provider_unavailable() {
    let nonexistent = PathBuf::from("/tmp/btsp-no-such-socket-12345.sock");

    let (mut client, mut server) = tokio::io::duplex(8192);

    let client_handle = tokio::spawn(async move {
        let client_hello = ClientHello {
            version: 1,
            client_ephemeral_pub: "key".to_string(),
        };
        let bytes = serde_json::to_vec(&client_hello).expect("ser");
        write_frame(&mut client, &bytes).await.expect("write");
    });

    let result = perform_server_handshake(&mut server, &nonexistent).await;
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("unreachable") || err_str.contains("connect"),
        "unexpected error: {err_str}"
    );

    client_handle.await.expect("client task");
}

#[tokio::test]
async fn handshake_version_mismatch() {
    let (mut client, mut server) = tokio::io::duplex(8192);

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
    let result = perform_server_handshake(&mut server, &provider).await;
    assert!(result.is_err());

    client_handle.await.expect("client task");
}

// ---------------------------------------------------------------------------
// NDJSON wire type serde tests
// ---------------------------------------------------------------------------

#[test]
fn ndjson_client_hello_roundtrip() {
    let msg = NdjsonClientHello {
        protocol: "btsp".to_string(),
        version: 1,
        client_ephemeral_pub: "AAAA".to_string(),
    };
    let json = serde_json::to_string(&msg).expect("serialize");
    assert!(json.contains("\"protocol\":\"btsp\""));
    let decoded: NdjsonClientHello = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded.protocol, "btsp");
    assert_eq!(decoded.version, 1);
    assert_eq!(decoded.client_ephemeral_pub, "AAAA");
}

#[test]
fn ndjson_server_hello_roundtrip() {
    let msg = NdjsonServerHello {
        version: 1,
        server_ephemeral_pub: "BBBB".to_string(),
        challenge: "CCCC".to_string(),
        session_id: "sess123".to_string(),
    };
    let json = serde_json::to_string(&msg).expect("serialize");
    assert!(json.contains("\"session_id\""));
    let decoded: NdjsonServerHello = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded.version, 1);
    assert_eq!(decoded.server_ephemeral_pub, "BBBB");
    assert_eq!(decoded.challenge, "CCCC");
    assert_eq!(decoded.session_id, "sess123");
}

#[test]
fn ndjson_client_hello_deserializes_primalspring_format() {
    let primalspring_line =
        r#"{"protocol":"btsp","version":1,"client_ephemeral_pub":"dGVzdC1rZXk="}"#;
    let hello: NdjsonClientHello =
        serde_json::from_str(primalspring_line).expect("parse primalSpring format");
    assert_eq!(hello.protocol, "btsp");
    assert_eq!(hello.version, 1);
    assert_eq!(hello.client_ephemeral_pub, "dGVzdC1rZXk=");
}

#[test]
fn ndjson_client_hello_version_u8_u32_compat() {
    let v_u8 = r#"{"protocol":"btsp","version":1,"client_ephemeral_pub":"key"}"#;
    let hello: NdjsonClientHello = serde_json::from_str(v_u8).expect("parse u8 version");
    assert_eq!(hello.version, 1);
}

// ---------------------------------------------------------------------------
// NDJSON handshake integration tests (with mock BTSP provider)
// ---------------------------------------------------------------------------

/// Simulates a primalSpring-style NDJSON BTSP client.
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

#[tokio::test]
async fn ndjson_handshake_success_full_sequence() {
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
    assert_eq!(complete.session_id, "abcdef0123456789");
    assert_eq!(complete.cipher, "null");
}

#[tokio::test]
async fn ndjson_handshake_failure_verify_rejected() {
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

#[test]
fn generate_challenge_is_not_empty() {
    let challenge = generate_challenge();
    assert!(!challenge.is_empty());
    assert_eq!(challenge.len(), 64, "blake3 hash hex = 64 chars");
}

#[test]
fn generate_challenge_is_unique() {
    let a = generate_challenge();
    let b = generate_challenge();
    assert_ne!(a, b, "two challenges must differ (OS entropy)");
}
