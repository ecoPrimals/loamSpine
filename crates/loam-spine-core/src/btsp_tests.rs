// SPDX-License-Identifier: AGPL-3.0-or-later

//! Unit tests for BTSP types, config, frame I/O, and seed resolution.
//!
//! Integration tests with mock BTSP providers live in `btsp_tests_integration.rs`.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test assertions use expect/unwrap for failure clarity"
)]

use std::path::PathBuf;

use base64::Engine;
use tokio::io::AsyncWriteExt;

use super::config::provider_socket_name;
use super::frame::MAX_FRAME_SIZE;
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
// resolve_family_seed tests
// ---------------------------------------------------------------------------

#[test]
fn resolve_family_seed_from_primary_env() {
    temp_env::with_vars(
        [
            ("FAMILY_SEED", Some("abcdef1234567890")),
            ("BEARDOG_FAMILY_SEED", None::<&str>),
        ],
        || {
            let seed = super::handshake::resolve_family_seed().expect("should resolve");
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(&seed)
                .expect("valid base64");
            assert_eq!(decoded, b"abcdef1234567890");
        },
    );
}

#[test]
fn resolve_family_seed_falls_back_to_beardog() {
    temp_env::with_vars(
        [
            ("FAMILY_SEED", None::<&str>),
            ("BEARDOG_FAMILY_SEED", Some("fallback_seed")),
        ],
        || {
            let seed = super::handshake::resolve_family_seed().expect("should resolve");
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(&seed)
                .expect("valid base64");
            assert_eq!(decoded, b"fallback_seed");
        },
    );
}

#[test]
fn resolve_family_seed_primary_takes_precedence() {
    temp_env::with_vars(
        [
            ("FAMILY_SEED", Some("primary")),
            ("BEARDOG_FAMILY_SEED", Some("fallback")),
        ],
        || {
            let seed = super::handshake::resolve_family_seed().expect("should resolve");
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(&seed)
                .expect("valid base64");
            assert_eq!(decoded, b"primary");
        },
    );
}

#[test]
fn resolve_family_seed_missing_returns_error() {
    temp_env::with_vars(
        [
            ("FAMILY_SEED", None::<&str>),
            ("BEARDOG_FAMILY_SEED", None::<&str>),
        ],
        || {
            let result = super::handshake::resolve_family_seed();
            assert!(result.is_err());
            let err = result.unwrap_err().to_string();
            assert!(
                err.contains("FAMILY_SEED"),
                "error should mention env var: {err}"
            );
        },
    );
}

#[test]
fn resolve_family_seed_hex_roundtrip() {
    let hex_seed = "deadbeef01234567deadbeef01234567deadbeef01234567deadbeef01234567";
    temp_env::with_var("FAMILY_SEED", Some(hex_seed), || {
        let seed = super::handshake::resolve_family_seed().expect("should resolve");
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&seed)
            .expect("valid base64");
        assert_eq!(std::str::from_utf8(&decoded).expect("utf8"), hex_seed);
    });
}
