// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration tests for the BTSP client-side handshake.
//!
//! Spawns a mock bearDog server implementing the full server-side handshake
//! with HMAC verification, then runs `perform_client_handshake` against it.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "test assertions use unwrap/panic for failure clarity"
)]

use std::path::PathBuf;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

use crate::btsp_client::{BtspClientError, perform_client_handshake};
use crate::transport::stream::TransportStream;

type HmacSha256 = Hmac<Sha256>;

const MOCK_FAMILY_SEED: &str = "test_family_seed_for_btsp_client";

/// Spawn a mock BTSP server that implements the server side of the handshake.
fn spawn_mock_btsp_server(
    temp_dir: &std::path::Path,
    socket_name: &str,
    reject_challenge: bool,
) -> (PathBuf, tokio::task::JoinHandle<()>) {
    let socket_path = temp_dir.join(socket_name);
    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path).unwrap();
    let return_path = socket_path;

    let handle = tokio::spawn(async move {
        let Ok((stream, _)) = listener.accept().await else {
            return;
        };

        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);

        let mut line = String::new();
        buf_reader.read_line(&mut line).await.unwrap();
        let hello: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(hello["protocol"], "btsp");
        assert_eq!(hello["version"], 1);
        assert!(hello["client_ephemeral_pub"].is_string());

        let challenge_data = b"test_challenge_bytes_32_chars!!!";
        let server_hello = serde_json::json!({
            "version": 1,
            "server_ephemeral_pub": BASE64_STANDARD.encode(b"server_ephemeral_key_material_32"),
            "challenge": BASE64_STANDARD.encode(challenge_data),
            "session_id": "test-session-001",
        });
        let hello_line = serde_json::to_string(&server_hello).unwrap();
        writer
            .write_all(format!("{hello_line}\n").as_bytes())
            .await
            .unwrap();
        writer.flush().await.unwrap();

        let mut line = String::new();
        buf_reader.read_line(&mut line).await.unwrap();
        let response: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert!(response["response"].is_string());
        assert!(response["preferred_cipher"].is_string());

        if reject_challenge {
            let err = serde_json::json!({
                "error": "handshake_failed",
                "reason": "HMAC verification failed",
            });
            let err_line = serde_json::to_string(&err).unwrap();
            writer
                .write_all(format!("{err_line}\n").as_bytes())
                .await
                .unwrap();
            writer.flush().await.unwrap();
            return;
        }

        let response_b64 = response["response"].as_str().unwrap();
        let response_bytes = BASE64_STANDARD.decode(response_b64).unwrap();

        let mut mac = HmacSha256::new_from_slice(MOCK_FAMILY_SEED.as_bytes()).unwrap();
        mac.update(challenge_data);
        let expected = mac.finalize().into_bytes();
        assert_eq!(
            response_bytes.as_slice(),
            expected.as_slice(),
            "HMAC mismatch"
        );

        let complete = serde_json::json!({
            "cipher": "chacha20_poly1305",
            "session_id": "test-session-001",
        });
        let complete_line = serde_json::to_string(&complete).unwrap();
        writer
            .write_all(format!("{complete_line}\n").as_bytes())
            .await
            .unwrap();
        writer.flush().await.unwrap();
    });

    (return_path, handle)
}

async fn connect_to_mock(socket_path: &std::path::Path) -> TransportStream {
    let stream = tokio::net::UnixStream::connect(socket_path).await.unwrap();
    TransportStream::Uds(stream)
}

#[test]
fn client_handshake_success() {
    temp_env::with_vars(
        [
            ("FAMILY_SEED", Some(MOCK_FAMILY_SEED)),
            ("BTSP_FAMILY_SEED", None::<&str>),
            ("BEARDOG_FAMILY_SEED", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let temp = tempfile::tempdir().unwrap();
                let (socket_path, server_handle) =
                    spawn_mock_btsp_server(temp.path(), "success.sock", false);

                tokio::task::yield_now().await;

                let mut stream = connect_to_mock(&socket_path).await;
                let session = perform_client_handshake(&mut stream).await.unwrap();

                assert_eq!(session.session_id, "test-session-001");
                assert_eq!(session.cipher, "chacha20_poly1305");

                server_handle.await.unwrap();
            });
        },
    );
}

#[test]
fn client_handshake_rejected_by_server() {
    temp_env::with_vars(
        [
            ("FAMILY_SEED", Some(MOCK_FAMILY_SEED)),
            ("BTSP_FAMILY_SEED", None::<&str>),
            ("BEARDOG_FAMILY_SEED", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let temp = tempfile::tempdir().unwrap();
                let (socket_path, server_handle) =
                    spawn_mock_btsp_server(temp.path(), "reject.sock", true);

                tokio::task::yield_now().await;

                let mut stream = connect_to_mock(&socket_path).await;
                let result = perform_client_handshake(&mut stream).await;

                assert!(result.is_err());
                match result.unwrap_err() {
                    BtspClientError::Rejected(reason) => {
                        assert!(reason.contains("HMAC"), "reason: {reason}");
                    }
                    other => panic!("expected Rejected, got: {other}"),
                }

                server_handle.await.unwrap();
            });
        },
    );
}

#[test]
fn client_handshake_no_family_seed() {
    temp_env::with_vars(
        [
            ("FAMILY_SEED", None::<&str>),
            ("BTSP_FAMILY_SEED", None::<&str>),
            ("BEARDOG_FAMILY_SEED", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let temp = tempfile::tempdir().unwrap();
                let (socket_path, _server_handle) =
                    spawn_mock_btsp_server(temp.path(), "noseed.sock", false);

                tokio::task::yield_now().await;

                let mut stream = connect_to_mock(&socket_path).await;
                let result = perform_client_handshake(&mut stream).await;

                assert!(result.is_err());
                match result.unwrap_err() {
                    BtspClientError::NoFamilySeed => {}
                    other => panic!("expected NoFamilySeed, got: {other}"),
                }
            });
        },
    );
}

#[test]
fn client_handshake_server_sends_error_on_hello() {
    temp_env::with_vars(
        [
            ("FAMILY_SEED", Some(MOCK_FAMILY_SEED)),
            ("BTSP_FAMILY_SEED", None::<&str>),
            ("BEARDOG_FAMILY_SEED", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let temp = tempfile::tempdir().unwrap();
                let socket_path = temp.path().join("error-hello.sock");
                let _ = std::fs::remove_file(&socket_path);
                let listener = UnixListener::bind(&socket_path).unwrap();

                let sp = socket_path.clone();
                let server_handle = tokio::spawn(async move {
                    let Ok((stream, _)) = listener.accept().await else {
                        return;
                    };
                    let (reader, mut writer) = stream.into_split();
                    let mut buf_reader = BufReader::new(reader);

                    let mut line = String::new();
                    buf_reader.read_line(&mut line).await.unwrap();

                    let err = serde_json::json!({
                        "error": "version_mismatch",
                        "reason": "unsupported protocol version",
                    });
                    let err_line = serde_json::to_string(&err).unwrap();
                    writer
                        .write_all(format!("{err_line}\n").as_bytes())
                        .await
                        .unwrap();
                    writer.flush().await.unwrap();
                });

                tokio::task::yield_now().await;

                let mut stream = connect_to_mock(&sp).await;
                let result = perform_client_handshake(&mut stream).await;

                assert!(result.is_err());
                match result.unwrap_err() {
                    BtspClientError::Rejected(reason) => {
                        assert!(
                            reason.contains("unsupported protocol version"),
                            "reason: {reason}"
                        );
                    }
                    other => panic!("expected Rejected, got: {other}"),
                }

                server_handle.await.unwrap();
            });
        },
    );
}

#[test]
fn client_handshake_server_disconnects() {
    temp_env::with_vars(
        [
            ("FAMILY_SEED", Some(MOCK_FAMILY_SEED)),
            ("BTSP_FAMILY_SEED", None::<&str>),
            ("BEARDOG_FAMILY_SEED", None::<&str>),
        ],
        || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let temp = tempfile::tempdir().unwrap();
                let socket_path = temp.path().join("disconnect.sock");
                let _ = std::fs::remove_file(&socket_path);
                let listener = UnixListener::bind(&socket_path).unwrap();

                let sp = socket_path.clone();
                let server_handle = tokio::spawn(async move {
                    let Ok((stream, _)) = listener.accept().await else {
                        return;
                    };
                    let (reader, _writer) = stream.into_split();
                    let mut buf_reader = BufReader::new(reader);

                    let mut line = String::new();
                    let _ = buf_reader.read_line(&mut line).await;
                });

                tokio::task::yield_now().await;

                let mut stream = connect_to_mock(&sp).await;
                let result = perform_client_handshake(&mut stream).await;

                assert!(result.is_err());
                let err_str = result.unwrap_err().to_string();
                assert!(
                    err_str.contains("empty") || err_str.contains("I/O"),
                    "unexpected error: {err_str}"
                );

                server_handle.await.unwrap();
            });
        },
    );
}
