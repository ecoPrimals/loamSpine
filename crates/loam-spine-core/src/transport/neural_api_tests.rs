// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn neural_api_transport_requires_socket() {
    // Without any env vars pointing to a socket, creation should still succeed
    // if we provide an explicit path
    let transport = NeuralApiTransport::new(Some(PathBuf::from("/tmp/test.sock")));
    assert!(transport.is_ok());
}

#[test]
fn neural_api_transport_debug() {
    let transport = NeuralApiTransport::new(Some(PathBuf::from("/tmp/test.sock"))).unwrap();
    let debug = format!("{transport:?}");
    assert!(debug.contains("NeuralApiTransport"));
    assert!(debug.contains("test.sock"));
}

#[test]
fn base64_decode_roundtrip() {
    let encoded = "SGVsbG8gV29ybGQ=";
    let decoded = base64_decode(encoded).unwrap();
    assert_eq!(decoded, b"Hello World");
}

#[test]
fn base64_decode_empty() {
    let decoded = base64_decode("").unwrap();
    assert!(decoded.is_empty());
}

#[test]
fn base64_decode_no_padding() {
    let decoded = base64_decode("YQ").unwrap();
    assert_eq!(decoded, b"a");
}

#[test]
fn urlencoding_basic() {
    assert_eq!(urlencoding_encode("hello"), "hello");
    assert_eq!(urlencoding_encode("hello world"), "hello%20world");
    assert_eq!(urlencoding_encode("a=b&c"), "a%3Db%26c");
}

#[test]
fn urlencoding_safe_chars() {
    assert_eq!(urlencoding_encode("a-b_c.d~e"), "a-b_c.d~e");
}

#[test]
fn parse_http_result_success() {
    let result = serde_json::json!({
        "status": 200,
        "body": "eyJrZXkiOiJ2YWx1ZSJ9", // {"key":"value"} in base64
    });
    let resp = NeuralApiTransport::parse_http_result(&result).unwrap();
    assert_eq!(resp.status, 200);
    assert!(resp.is_success());
    let parsed: serde_json::Value = resp.json().unwrap();
    assert_eq!(parsed["key"], "value");
}

#[test]
fn parse_http_result_no_body() {
    let result = serde_json::json!({ "status": 204 });
    let resp = NeuralApiTransport::parse_http_result(&result).unwrap();
    assert_eq!(resp.status, 204);
    assert!(resp.body.is_empty());
}

#[test]
fn parse_http_result_missing_status() {
    let result = serde_json::json!({ "body": "dGVzdA==" });
    let resp = NeuralApiTransport::parse_http_result(&result);
    assert!(resp.is_err());
}

#[tokio::test]
async fn neural_api_get_fails_no_socket() {
    let transport =
        NeuralApiTransport::new(Some(PathBuf::from("/tmp/nonexistent-neural-api-test.sock")))
            .unwrap();
    let result = transport.get("http://registry:8082/health").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("NeuralAPI") || err.contains("socket"),
        "error should mention NeuralAPI: {err}"
    );
}

#[tokio::test]
async fn neural_api_post_fails_no_socket() {
    let transport =
        NeuralApiTransport::new(Some(PathBuf::from("/tmp/nonexistent-neural-api-test.sock")))
            .unwrap();
    let body = serde_json::json!({"name": "test"});
    let result = transport
        .post_json("http://registry:8082/register", &body)
        .await;
    assert!(result.is_err());
}

#[test]
fn request_id_increments() {
    let transport = NeuralApiTransport::new(Some(PathBuf::from("/tmp/test.sock"))).unwrap();
    let id1 = transport.next_id();
    let id2 = transport.next_id();
    assert_eq!(id2, id1 + 1);
}

#[test]
fn base64_decode_invalid_character() {
    let result = base64_decode("SGVsbG8!!!!");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("base64") || err.contains("Invalid"));
}

#[test]
fn base64_decode_with_newlines() {
    let decoded = base64_decode("SGVs\nbG8=\r\n").unwrap();
    assert_eq!(decoded, b"Hello");
}

#[test]
fn base64_decode_standard_vectors() {
    assert_eq!(base64_decode("").unwrap(), b"");
    assert_eq!(base64_decode("Zg==").unwrap(), b"f");
    assert_eq!(base64_decode("Zm8=").unwrap(), b"fo");
    assert_eq!(base64_decode("Zm9v").unwrap(), b"foo");
    assert_eq!(base64_decode("Zm9vYg==").unwrap(), b"foob");
    assert_eq!(base64_decode("Zm9vYmE=").unwrap(), b"fooba");
    assert_eq!(base64_decode("Zm9vYmFy").unwrap(), b"foobar");
}

#[test]
fn parse_http_result_status_overflow() {
    let result = serde_json::json!({
        "status": 99999,
        "body": "dGVzdA==",
    });
    let resp = NeuralApiTransport::parse_http_result(&result).unwrap();
    assert_eq!(
        resp.status, 500,
        "overflowing status should fallback to 500"
    );
}

#[test]
fn parse_http_result_invalid_base64_body() {
    let result = serde_json::json!({
        "status": 200,
        "body": "not!!valid!!base64",
    });
    let resp = NeuralApiTransport::parse_http_result(&result);
    assert!(resp.is_err(), "invalid base64 should produce an error");
}

#[test]
fn neural_api_transport_from_env_fallback() {
    let transport = NeuralApiTransport::new(None);
    // Without any env vars this may or may not succeed depending on XDG_RUNTIME_DIR
    // The important thing is it doesn't panic
    let _ = transport;
}

/// `NeuralApiTransport::new(None)` errors only when neither explicit socket nor env-based path exists.
#[test]
fn neural_api_transport_new_none_errors_without_runtime_or_explicit_socket() {
    let has_socket = std::env::var("BIOMEOS_NEURAL_API_SOCKET")
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let has_runtime = std::env::var("XDG_RUNTIME_DIR").is_ok();
    if has_socket || has_runtime {
        return;
    }
    let err =
        NeuralApiTransport::new(None).expect_err("expected connect error from missing socket path");
    let msg = err.to_string();
    assert!(
        msg.contains("NeuralAPI") && msg.contains("socket"),
        "unexpected error: {msg}"
    );
}

#[test]
fn urlencoding_encode_non_ascii_and_reserved() {
    assert_eq!(urlencoding_encode("a/b"), "a%2Fb");
    assert_eq!(urlencoding_encode("café"), "caf%C3%A9");
    assert_eq!(urlencoding_encode("100%"), "100%25");
}

#[test]
fn parse_http_result_status_must_be_json_number() {
    let result = serde_json::json!({
        "status": "200",
        "body": "dGVzdA==",
    });
    let err = NeuralApiTransport::parse_http_result(&result).expect_err("string status is invalid");
    assert!(err.to_string().contains("status") || err.to_string().contains("Missing"));
}

#[test]
fn parse_http_result_empty_object_body() {
    let result = serde_json::json!({ "status": 200 });
    let resp = NeuralApiTransport::parse_http_result(&result).expect("parse");
    assert_eq!(resp.status, 200);
    assert!(resp.body.is_empty());
}

#[test]
fn parse_http_result_status_exceeds_u16_max_maps_to_500() {
    let result = serde_json::json!({
        "status": u64::from(u16::MAX) + 1,
        "body": null,
    });
    let resp = NeuralApiTransport::parse_http_result(&result).expect("parse");
    assert_eq!(resp.status, 500);
}

/// Helper: spawn a mock NeuralAPI socket that responds to `http.request`
fn spawn_mock_transport_server(
    socket_path: &std::path::Path,
    response: &serde_json::Value,
) -> tokio::task::JoinHandle<()> {
    let listener = tokio::net::UnixListener::bind(socket_path).unwrap();
    let resp_bytes = serde_json::to_vec(&response).unwrap();

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
async fn jsonrpc_call_success_via_mock_socket() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-transport-test.sock");

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": {
            "status": 200,
            "body": "eyJvayI6dHJ1ZX0="  // {"ok":true}
        },
        "id": 1
    });
    let handle = spawn_mock_transport_server(&sock, &response);

    let transport = NeuralApiTransport::new(Some(sock)).unwrap();
    let result = transport.get("http://registry:8082/health").await;

    assert!(result.is_ok(), "GET should succeed: {:?}", result.err());
    let resp = result.unwrap();
    assert_eq!(resp.status, 200);
    assert!(!resp.body.is_empty());

    handle.abort();
}

#[tokio::test]
async fn jsonrpc_call_returns_error_on_rpc_error() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-transport-err.sock");

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": -32601, "message": "method not found" },
        "id": 1
    });
    let handle = spawn_mock_transport_server(&sock, &response);

    let transport = NeuralApiTransport::new(Some(sock)).unwrap();
    let result = transport.get("http://registry:8082/health").await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("method not found"), "error: {err}");

    handle.abort();
}

#[tokio::test]
async fn get_with_query_builds_url_and_calls() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-query-test.sock");

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": {
            "status": 200,
            "body": "e30="  // {}
        },
        "id": 1
    });
    let handle = spawn_mock_transport_server(&sock, &response);

    let transport = NeuralApiTransport::new(Some(sock)).unwrap();
    let result = transport
        .get_with_query(
            "http://registry:8082/discover",
            &[("capability", "signing"), ("format", "json")],
        )
        .await;

    assert!(
        result.is_ok(),
        "get_with_query should succeed: {:?}",
        result.err()
    );
    let resp = result.unwrap();
    assert_eq!(resp.status, 200);

    handle.abort();
}

#[tokio::test]
async fn post_json_via_mock_socket() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-post-test.sock");

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": {
            "status": 201,
            "body": "eyJjcmVhdGVkIjp0cnVlfQ=="  // {"created":true}
        },
        "id": 1
    });
    let handle = spawn_mock_transport_server(&sock, &response);

    let transport = NeuralApiTransport::new(Some(sock)).unwrap();
    let body = serde_json::json!({"name": "test-service"});
    let result = transport
        .post_json("http://registry:8082/register", &body)
        .await;

    assert!(result.is_ok(), "POST should succeed: {:?}", result.err());
    let resp = result.unwrap();
    assert_eq!(resp.status, 201);

    handle.abort();
}

#[tokio::test]
async fn jsonrpc_call_missing_result_field() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-noresult.sock");

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1
    });
    let handle = spawn_mock_transport_server(&sock, &response);

    let transport = NeuralApiTransport::new(Some(sock)).unwrap();
    let result = transport.get("http://registry:8082/health").await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("missing") || err.contains("result"),
        "error: {err}"
    );

    handle.abort();
}
