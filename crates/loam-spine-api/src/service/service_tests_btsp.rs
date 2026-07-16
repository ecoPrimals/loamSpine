// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP Phase 3 negotiation tests.

use super::*;

#[tokio::test]
async fn test_btsp_negotiate_returns_null_when_no_session_key() {
    let service = LoamSpineRpcService::default_service();
    let resp = service
        .negotiate_btsp(BtspNegotiateRequest {
            session_id: "abc123".to_string(),
            preferred_cipher: "chacha20-poly1305".to_string(),
            ciphers: vec!["chacha20-poly1305".to_string()],
            client_nonce: Some("dGVzdA==".to_string()),
            bond_type: Some("Covalent".to_string()),
        })
        .await
        .expect("negotiate should succeed");
    assert_eq!(resp.cipher, "null");
    assert!(resp.server_nonce.is_none());
}

#[tokio::test]
async fn test_btsp_negotiate_with_minimal_params() {
    let service = LoamSpineRpcService::default_service();
    let resp = service
        .negotiate_btsp(BtspNegotiateRequest {
            session_id: "session-xyz".to_string(),
            preferred_cipher: "chacha20-poly1305".to_string(),
            ciphers: vec![],
            client_nonce: None,
            bond_type: None,
        })
        .await
        .expect("negotiate should succeed");
    assert_eq!(resp.cipher, "null");
}

#[tokio::test]
async fn test_btsp_negotiate_returns_chacha20_when_session_key_registered() {
    use base64::Engine;

    let service = LoamSpineRpcService::default_service();
    let handshake_key = [0x42u8; 32];
    service
        .register_btsp_session("sess-001".to_string(), handshake_key)
        .await;

    let resp = service
        .negotiate_btsp(BtspNegotiateRequest {
            session_id: "sess-001".to_string(),
            preferred_cipher: "chacha20-poly1305".to_string(),
            ciphers: vec!["chacha20-poly1305".to_string()],
            client_nonce: Some("dGVzdA==".to_string()),
            bond_type: Some("Ionic".to_string()),
        })
        .await
        .expect("negotiate should succeed");
    assert_eq!(resp.cipher, "chacha20-poly1305");
    assert!(resp.server_nonce.is_some());

    let nonce_bytes = base64::engine::general_purpose::STANDARD
        .decode(resp.server_nonce.expect("server_nonce present"))
        .expect("valid base64");
    assert_eq!(nonce_bytes.len(), 32, "server nonce should be 32 bytes");
}

#[tokio::test]
async fn test_btsp_negotiate_different_session_gets_null() {
    let service = LoamSpineRpcService::default_service();
    service
        .register_btsp_session("sess-known".to_string(), [0xAA; 32])
        .await;

    let resp = service
        .negotiate_btsp(BtspNegotiateRequest {
            session_id: "sess-unknown".to_string(),
            preferred_cipher: "chacha20-poly1305".to_string(),
            ciphers: vec![],
            client_nonce: None,
            bond_type: None,
        })
        .await
        .expect("negotiate should succeed");
    assert_eq!(resp.cipher, "null", "unknown session falls back to null");
}

#[tokio::test]
async fn test_btsp_session_key_derivation_interop() {
    use loam_spine_core::btsp::SessionKeys;

    let handshake_key = [0x42u8; 32];
    let client_nonce = [0x01u8; 32];
    let server_nonce = [0x02u8; 32];

    let server_keys = SessionKeys::derive(&handshake_key, &client_nonce, &server_nonce, true)
        .expect("server derive");
    let client_keys = SessionKeys::derive(&handshake_key, &client_nonce, &server_nonce, false)
        .expect("client derive");

    let plaintext = b"JSON-RPC over encrypted BTSP Phase 3";
    let encrypted = client_keys.encrypt(plaintext).expect("encrypt");
    let decrypted = server_keys.decrypt(&encrypted).expect("decrypt");
    assert_eq!(decrypted, plaintext);

    let response = b"response from loamSpine";
    let encrypted_resp = server_keys.encrypt(response).expect("encrypt response");
    let decrypted_resp = client_keys
        .decrypt(&encrypted_resp)
        .expect("decrypt response");
    assert_eq!(decrypted_resp, response);
}
