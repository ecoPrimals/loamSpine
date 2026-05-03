// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration tests for the BTSP Phase 3 transport switch.
//!
//! Verifies that after `btsp.negotiate` returns `cipher: "chacha20-poly1305"`,
//! subsequent messages use encrypted framing (`read_encrypted_frame` /
//! `write_encrypted_frame`).

use crate::jsonrpc::LoamSpineJsonRpc;
use crate::service::LoamSpineRpcService;
use base64::Engine;
use loam_spine_core::btsp::{
    BtspSession, SessionKeys, read_encrypted_frame, write_encrypted_frame,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const TEST_SESSION_ID: &str = "test-session-001";

fn test_handshake_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    for (i, byte) in key.iter_mut().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "i < 32, fits in u8")]
        let idx = i as u8;
        *byte = idx.wrapping_mul(7).wrapping_add(42);
    }
    key
}

fn make_session_with_key() -> BtspSession {
    BtspSession {
        session_id: TEST_SESSION_ID.to_string(),
        cipher: "null".to_string(),
        handshake_key: Some(test_handshake_key()),
    }
}

fn make_session_without_key() -> BtspSession {
    BtspSession {
        session_id: TEST_SESSION_ID.to_string(),
        cipher: "null".to_string(),
        handshake_key: None,
    }
}

/// Verify that `handle_post_handshake` with a keyed session:
/// 1. Processes `btsp.negotiate` in plaintext → returns `chacha20-poly1305`
/// 2. Switches to encrypted framing for subsequent messages
/// 3. Successfully round-trips a `health.check` through encrypted frames
#[tokio::test]
async fn phase3_transport_switch_encrypted_roundtrip() {
    use tokio::net::UnixStream;

    let service = LoamSpineRpcService::default_service();
    let handler = LoamSpineJsonRpc::new(service);
    let session = make_session_with_key();
    let handshake_key = session.handshake_key.expect("has key");

    let (client_stream, server_stream) = UnixStream::pair().expect("unix socket pair");

    let server_task = tokio::spawn(async move {
        let (server_reader, mut server_writer) = server_stream.into_split();
        let mut server_buf_reader = BufReader::new(server_reader);
        super::uds::handle_post_handshake(
            &handler,
            &mut server_buf_reader,
            &mut server_writer,
            session,
        )
        .await
    });

    let (client_reader, mut client_writer) = client_stream.into_split();
    let mut client_buf_reader = BufReader::new(client_reader);

    // Step 1: Send btsp.negotiate as plaintext JSON-RPC
    let client_nonce = loam_spine_core::btsp::generate_nonce().expect("nonce");
    let client_nonce_b64 = base64::engine::general_purpose::STANDARD.encode(client_nonce);

    let negotiate_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "btsp.negotiate",
        "params": {
            "session_id": TEST_SESSION_ID,
            "ciphers": ["chacha20-poly1305"],
            "client_nonce": client_nonce_b64,
        },
        "id": 1
    });
    let mut line = serde_json::to_string(&negotiate_req).expect("serialize");
    line.push('\n');
    client_writer
        .write_all(line.as_bytes())
        .await
        .expect("write negotiate");

    // Step 2: Read the plaintext negotiate response
    let mut resp_line = String::new();
    client_buf_reader
        .read_line(&mut resp_line)
        .await
        .expect("read negotiate response");

    let resp: serde_json::Value =
        serde_json::from_str(resp_line.trim()).expect("parse negotiate response");

    let result = resp.get("result").expect("has result");
    let cipher = result
        .get("cipher")
        .expect("has cipher")
        .as_str()
        .expect("str");
    assert_eq!(
        cipher, "chacha20-poly1305",
        "negotiate must return chacha20-poly1305"
    );

    let server_nonce_b64 = result
        .get("server_nonce")
        .expect("has server_nonce")
        .as_str()
        .expect("str");
    let server_nonce = base64::engine::general_purpose::STANDARD
        .decode(server_nonce_b64)
        .expect("decode server nonce");

    // Step 3: Derive client-side session keys (is_server=false for client)
    let client_keys =
        SessionKeys::derive(&handshake_key, &client_nonce, &server_nonce, false).expect("derive");

    // Step 4: Send an encrypted frame containing a health.check request
    let health_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 2
    });
    let health_bytes = serde_json::to_vec(&health_req).expect("serialize health");
    write_encrypted_frame(&mut client_writer, &client_keys, &health_bytes)
        .await
        .expect("write encrypted frame");

    // Step 5: Read the encrypted response frame
    let resp_plaintext = read_encrypted_frame(&mut client_buf_reader, &client_keys)
        .await
        .expect("read encrypted response");

    let health_resp: serde_json::Value =
        serde_json::from_slice(&resp_plaintext).expect("parse health response");

    assert_eq!(
        health_resp.get("jsonrpc").and_then(|v| v.as_str()),
        Some("2.0")
    );
    assert!(
        health_resp.get("result").is_some(),
        "health.check must succeed"
    );
    assert_eq!(
        health_resp.get("id").and_then(serde_json::Value::as_u64),
        Some(2),
        "response id must match request"
    );

    // Step 6: Close the client side → server loop should exit cleanly
    drop(client_writer);
    let server_result = server_task.await.expect("server task join");
    assert!(
        server_result.is_ok(),
        "server should exit cleanly on client disconnect"
    );
}

/// Verify that multiple encrypted requests work in sequence.
#[tokio::test]
async fn phase3_multiple_encrypted_requests() {
    use tokio::net::UnixStream;

    let service = LoamSpineRpcService::default_service();
    let handler = LoamSpineJsonRpc::new(service);
    let session = make_session_with_key();
    let handshake_key = session.handshake_key.expect("has key");

    let (client_stream, server_stream) = UnixStream::pair().expect("unix socket pair");

    let server_task = tokio::spawn(async move {
        let (server_reader, mut server_writer) = server_stream.into_split();
        let mut server_buf_reader = BufReader::new(server_reader);
        super::uds::handle_post_handshake(
            &handler,
            &mut server_buf_reader,
            &mut server_writer,
            session,
        )
        .await
    });

    let (client_reader, mut client_writer) = client_stream.into_split();
    let mut client_buf_reader = BufReader::new(client_reader);

    // Negotiate
    let client_nonce = loam_spine_core::btsp::generate_nonce().expect("nonce");
    let client_nonce_b64 = base64::engine::general_purpose::STANDARD.encode(client_nonce);
    let negotiate_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "btsp.negotiate",
        "params": {
            "session_id": TEST_SESSION_ID,
            "ciphers": ["chacha20-poly1305"],
            "client_nonce": client_nonce_b64,
        },
        "id": 1
    });
    let mut line = serde_json::to_string(&negotiate_req).expect("serialize");
    line.push('\n');
    client_writer
        .write_all(line.as_bytes())
        .await
        .expect("write");

    let mut resp_line = String::new();
    client_buf_reader
        .read_line(&mut resp_line)
        .await
        .expect("read");
    let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).expect("parse");
    let result = resp.get("result").expect("result");
    let server_nonce_b64 = result
        .get("server_nonce")
        .expect("sn")
        .as_str()
        .expect("str");
    let server_nonce = base64::engine::general_purpose::STANDARD
        .decode(server_nonce_b64)
        .expect("decode");
    let client_keys =
        SessionKeys::derive(&handshake_key, &client_nonce, &server_nonce, false).expect("derive");

    // Send 3 encrypted requests and verify each response
    for i in 0u64..3 {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.check",
            "params": {},
            "id": i + 10
        });
        let req_bytes = serde_json::to_vec(&req).expect("serialize");
        write_encrypted_frame(&mut client_writer, &client_keys, &req_bytes)
            .await
            .expect("write");

        let resp_pt = read_encrypted_frame(&mut client_buf_reader, &client_keys)
            .await
            .expect("read");
        let resp: serde_json::Value = serde_json::from_slice(&resp_pt).expect("parse");
        assert_eq!(
            resp.get("id").and_then(serde_json::Value::as_u64),
            Some(i + 10)
        );
        assert!(resp.get("result").is_some());
    }

    drop(client_writer);
    server_task.await.expect("join").expect("server ok");
}

/// Verify that when no handshake key is present, the transport stays plaintext.
#[tokio::test]
async fn phase3_no_key_stays_plaintext() {
    use tokio::net::UnixStream;

    let service = LoamSpineRpcService::default_service();
    let handler = LoamSpineJsonRpc::new(service);
    let session = make_session_without_key();

    let (client_stream, server_stream) = UnixStream::pair().expect("unix socket pair");

    let server_task = tokio::spawn(async move {
        let (server_reader, mut server_writer) = server_stream.into_split();
        let mut server_buf_reader = BufReader::new(server_reader);
        super::uds::handle_post_handshake(
            &handler,
            &mut server_buf_reader,
            &mut server_writer,
            session,
        )
        .await
    });

    let (client_reader, mut client_writer) = client_stream.into_split();
    let mut client_buf_reader = BufReader::new(client_reader);

    // Send a plaintext JSON-RPC request
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 1
    });
    let mut line = serde_json::to_string(&req).expect("serialize");
    line.push('\n');
    client_writer
        .write_all(line.as_bytes())
        .await
        .expect("write");

    // Should get a plaintext response
    let mut resp_line = String::new();
    client_buf_reader
        .read_line(&mut resp_line)
        .await
        .expect("read");
    let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).expect("parse");
    assert!(
        resp.get("result").is_some(),
        "plaintext health.check must work"
    );

    drop(client_writer);
    server_task.await.expect("join").expect("server ok");
}

/// Verify that `btsp.negotiate` returning `null` (no key registered for session)
/// keeps the transport in plaintext for subsequent messages.
#[tokio::test]
async fn phase3_negotiate_null_stays_plaintext() {
    use tokio::net::UnixStream;

    let service = LoamSpineRpcService::default_service();
    let handler = LoamSpineJsonRpc::new(service);

    let session = BtspSession {
        session_id: TEST_SESSION_ID.to_string(),
        cipher: "null".to_string(),
        handshake_key: Some(test_handshake_key()),
    };

    let (client_stream, server_stream) = UnixStream::pair().expect("unix socket pair");

    let server_task = tokio::spawn(async move {
        let (server_reader, mut server_writer) = server_stream.into_split();
        let mut server_buf_reader = BufReader::new(server_reader);
        super::uds::handle_post_handshake(
            &handler,
            &mut server_buf_reader,
            &mut server_writer,
            session,
        )
        .await
    });

    let (client_reader, mut client_writer) = client_stream.into_split();
    let mut client_buf_reader = BufReader::new(client_reader);

    // Send btsp.negotiate with a DIFFERENT session_id so key lookup fails → null cipher
    let client_nonce = loam_spine_core::btsp::generate_nonce().expect("nonce");
    let client_nonce_b64 = base64::engine::general_purpose::STANDARD.encode(client_nonce);
    let negotiate_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "btsp.negotiate",
        "params": {
            "session_id": "different-session-999",
            "ciphers": ["chacha20-poly1305"],
            "client_nonce": client_nonce_b64,
        },
        "id": 1
    });
    let mut line = serde_json::to_string(&negotiate_req).expect("serialize");
    line.push('\n');
    client_writer
        .write_all(line.as_bytes())
        .await
        .expect("write");

    let mut resp_line = String::new();
    client_buf_reader
        .read_line(&mut resp_line)
        .await
        .expect("read");
    let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).expect("parse");
    let cipher = resp["result"]["cipher"].as_str().expect("cipher");
    assert_eq!(cipher, "null", "mismatched session should get null cipher");

    // Subsequent plaintext request should work
    let health_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 2
    });
    let mut line2 = serde_json::to_string(&health_req).expect("serialize");
    line2.push('\n');
    client_writer
        .write_all(line2.as_bytes())
        .await
        .expect("write");

    let mut resp_line2 = String::new();
    client_buf_reader
        .read_line(&mut resp_line2)
        .await
        .expect("read");
    let resp2: serde_json::Value = serde_json::from_str(resp_line2.trim()).expect("parse");
    assert!(
        resp2.get("result").is_some(),
        "plaintext follow-up must work"
    );

    drop(client_writer);
    server_task.await.expect("join").expect("server ok");
}
