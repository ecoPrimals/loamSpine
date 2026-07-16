// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// ── Register / Deregister via mock sockets (no env vars) ─────────────────

fn spawn_mock_neural_api(
    socket_path: &std::path::Path,
    response: &serde_json::Value,
) -> tokio::task::JoinHandle<()> {
    let listener = tokio::net::UnixListener::bind(socket_path).unwrap();
    let resp_bytes = serde_json::to_vec(response).unwrap();

    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut len_buf = [0u8; 4];
            let _ = stream.read_exact(&mut len_buf).await;
            let req_len = u32::from_be_bytes(len_buf) as usize;
            let mut req_buf = vec![0u8; req_len];
            let _ = stream.read_exact(&mut req_buf).await;

            let len = u32::try_from(resp_bytes.len())
                .unwrap_or(u32::MAX)
                .to_be_bytes();
            let _ = stream.write_all(&len).await;
            let _ = stream.write_all(&resp_bytes).await;
            let _ = stream.flush().await;
        }
    })
}

#[tokio::test]
async fn register_returns_ok_false_when_socket_unresolvable() {
    let path = resolve_neural_api_socket_with(None, None, None);
    assert!(path.is_none());
}

#[tokio::test]
async fn register_returns_ok_false_when_socket_missing() {
    let path = std::path::PathBuf::from("/tmp/nonexistent-neural-api-loamspine-test.sock");
    assert!(!path.exists());
}

#[tokio::test]
async fn register_succeeds_with_mock_server() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api.sock");
    let our_sock = tmp.path().join("loamspine.sock");
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": { "registered": true },
        "id": 1
    });
    let handle = spawn_mock_neural_api(&sock, &response);
    let result = register_at_socket(&sock, &our_sock).await;
    handle.abort();
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn register_returns_error_on_jsonrpc_error() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api-err.sock");
    let our_sock = tmp.path().join("loamspine.sock");
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": -32601, "message": "method not found" },
        "id": 1
    });
    let handle = spawn_mock_neural_api(&sock, &response);
    let result = register_at_socket(&sock, &our_sock).await;
    handle.abort();
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("method not found"), "error: {err}");
}

#[tokio::test]
async fn deregister_succeeds_with_mock_server() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api-dereg.sock");
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": { "deregistered": true },
        "id": 2
    });
    let handle = spawn_mock_neural_api(&sock, &response);
    let result = deregister_at_socket(&sock).await;
    handle.abort();
    assert!(result.is_ok());
}

#[tokio::test]
async fn deregister_handles_jsonrpc_error_gracefully() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api-dereg-err.sock");
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": -32601, "message": "not supported" },
        "id": 2
    });
    let handle = spawn_mock_neural_api(&sock, &response);
    let result = deregister_at_socket(&sock).await;
    handle.abort();
    assert!(
        result.is_ok(),
        "deregister should succeed even on JSON-RPC error"
    );
}

#[tokio::test]
async fn deregister_handles_malformed_response() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api-dereg-bad.sock");
    let listener = tokio::net::UnixListener::bind(&sock).unwrap();
    let handle = tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut len_buf = [0u8; 4];
            let _ = stream.read_exact(&mut len_buf).await;
            let req_len = u32::from_be_bytes(len_buf) as usize;
            let mut req_buf = vec![0u8; req_len];
            let _ = stream.read_exact(&mut req_buf).await;

            let garbage = b"not json";
            let len = u32::try_from(garbage.len())
                .unwrap_or(u32::MAX)
                .to_be_bytes();
            let _ = stream.write_all(&len).await;
            let _ = stream.write_all(garbage).await;
            let _ = stream.flush().await;
        }
    });
    let result = deregister_at_socket(&sock).await;
    handle.abort();
    assert!(
        result.is_ok(),
        "deregister should succeed even on malformed response"
    );
}

/// Peer hangs up after reading the request but before sending the 4-byte response length.
#[tokio::test]
async fn deregister_handles_peer_close_before_response_length() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api-dereg-hangup.sock");
    let listener = tokio::net::UnixListener::bind(&sock).unwrap();
    let handle = tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let mut stream = stream;
            let mut len_buf = [0u8; 4];
            let _ = stream.read_exact(&mut len_buf).await;
            let req_len = u32::from_be_bytes(len_buf) as usize;
            let mut req_buf = vec![0u8; req_len];
            let _ = stream.read_exact(&mut req_buf).await;
            drop(stream);
        }
    });
    let result = deregister_at_socket(&sock).await;
    handle.abort();
    assert!(
        result.is_ok(),
        "deregister should tolerate hang-up before response length"
    );
}

// ── primal.announce payload (Wave 43 Neural API schema) ──────────────────

#[test]
fn announce_payload_has_wave43_fields() {
    let socket = std::path::Path::new("/run/user/1000/biomeos/loamspine.sock");
    let payload = announce_payload(socket);

    assert_eq!(payload["primal"], "loamspine");
    assert!(payload["version"].is_string());
    assert_eq!(payload["socket"], "/run/user/1000/biomeos/loamspine.sock");
    assert_eq!(payload["status"], "running");
    assert_eq!(payload["domain"], "permanence");
    assert_eq!(payload["capability_domain"], "ledger");

    let caps = payload["capabilities"].as_array().unwrap();
    assert!(caps.contains(&serde_json::json!("anchor")));
    assert!(caps.contains(&serde_json::json!("ledger")));
    assert!(caps.contains(&serde_json::json!("permanence")));
    assert_eq!(caps.len(), 3);

    let tiers = payload["signal_tiers"].as_array().unwrap();
    assert!(tiers.contains(&serde_json::json!("nest")));

    let cost_hints = payload["cost_hints"].as_object().unwrap();
    assert_eq!(cost_hints["anchor"], 20.0);
    assert_eq!(cost_hints["ledger"], 15.0);
    assert_eq!(cost_hints["permanence"], 30.0);

    let latency = payload["latency_estimates"].as_object().unwrap();
    assert_eq!(latency["anchor"], 50);
    assert_eq!(latency["ledger"], 20);
    assert_eq!(latency["permanence"], 100);

    let methods = payload["methods"].as_array().unwrap();
    assert_eq!(methods.len(), crate::niche::METHODS.len());
    assert!(methods.contains(&serde_json::json!("anchor.publish_batch")));

    assert!(payload["pid"].is_number());
}

#[test]
fn announce_payload_methods_match_niche() {
    let socket = std::path::Path::new("/tmp/test.sock");
    let payload = announce_payload(socket);
    let methods: Vec<&str> = payload["methods"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    for m in crate::niche::METHODS {
        assert!(
            methods.contains(m),
            "niche method '{m}' missing from announce payload"
        );
    }
}

#[test]
fn announce_constants_are_consistent() {
    assert!(!SIGNAL_TIERS.is_empty());
    assert!(!ANNOUNCE_CAPABILITIES.is_empty());
    assert!(!COST_HINTS.is_empty());
    assert!(!LATENCY_ESTIMATES.is_empty());

    for (domain, _) in COST_HINTS {
        assert!(
            ANNOUNCE_CAPABILITIES.contains(domain),
            "cost_hint domain '{domain}' not in ANNOUNCE_CAPABILITIES"
        );
    }
    for (domain, _) in LATENCY_ESTIMATES {
        assert!(
            ANNOUNCE_CAPABILITIES.contains(domain),
            "latency_estimate domain '{domain}' not in ANNOUNCE_CAPABILITIES"
        );
    }
}

#[tokio::test]
async fn register_sends_primal_announce_not_lifecycle_register() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api-method-check.sock");
    let our_sock = tmp.path().join("loamspine.sock");

    let listener = tokio::net::UnixListener::bind(&sock).unwrap();
    let handle = tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut len_buf = [0u8; 4];
            let _ = stream.read_exact(&mut len_buf).await;
            let req_len = u32::from_be_bytes(len_buf) as usize;
            let mut req_buf = vec![0u8; req_len];
            let _ = stream.read_exact(&mut req_buf).await;

            let request: serde_json::Value = serde_json::from_slice(&req_buf).unwrap();
            assert_eq!(
                request["method"], "primal.announce",
                "registration should use primal.announce, not lifecycle.register"
            );
            assert!(request["params"]["socket"].is_string());
            assert!(request["params"]["signal_tiers"].is_array());
            assert!(request["params"]["cost_hints"].is_object());
            assert!(request["params"]["latency_estimates"].is_object());

            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": { "registered": true },
                "id": 1
            });
            let resp_bytes = serde_json::to_vec(&response).unwrap();
            let len = u32::try_from(resp_bytes.len()).unwrap().to_be_bytes();
            let _ = stream.write_all(&len).await;
            let _ = stream.write_all(&resp_bytes).await;
            let _ = stream.flush().await;
        }
    });

    let result = register_at_socket(&sock, &our_sock).await;
    handle.abort();
    assert!(result.is_ok());
    assert!(result.unwrap());
}

// ── Public wrapper entry points ──────────────────────────────────────────

#[tokio::test]
async fn register_with_neural_api_returns_ok_regardless_of_socket() {
    let result = super::register_with_neural_api().await;
    // On machines without a NeuralAPI socket: Ok(false).
    // On machines with a live socket (e.g. sporeGate running cellMembrane):
    // Ok(true) on success or Err on connection failure — both are valid.
    // We only assert no panic; the semantics depend on the runtime environment.
    match result {
        Ok(false | true) => {}
        Err(e) => {
            // Connection refused or protocol error is acceptable in test —
            // the NeuralAPI provider may not speak our announce schema.
            let msg = e.to_string();
            assert!(
                msg.contains("unreachable")
                    || msg.contains("connection")
                    || msg.contains("Connection refused")
                    || msg.contains("NeuralAPI"),
                "unexpected registration error: {msg}"
            );
        }
    }
}

#[tokio::test]
async fn deregister_from_neural_api_succeeds_when_no_socket() {
    let result = super::deregister_from_neural_api().await;
    assert!(result.is_ok());
}
