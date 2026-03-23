// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use serial_test::serial;

const CLEAN: [(&str, Option<&str>); 4] = [
    ("LOAMSPINE_SOCKET", None),
    ("XDG_RUNTIME_DIR", None),
    ("BIOMEOS_NEURAL_API_SOCKET", None),
    ("BIOMEOS_FAMILY_ID", None),
];

#[test]
fn resolve_socket_path_returns_valid_path() {
    let path = resolve_socket_path();
    assert!(!path.as_os_str().is_empty());
    assert!(path.to_string_lossy().contains("loamspine"));
}

#[test]
fn capability_list_includes_all_expected() {
    let list = capability_list();
    assert!(list.get("primal").is_some());
    assert!(list.get("version").is_some());
    assert!(list.get("capabilities").is_some());
    assert!(list.get("methods").is_some());
    let caps = list["capabilities"].as_array().expect("capabilities array");
    assert!(caps.contains(&serde_json::json!("permanence")));
    assert!(caps.contains(&serde_json::json!("spine.create")));
    assert!(caps.contains(&serde_json::json!("capability.list")));
    assert_eq!(caps.len(), CAPABILITIES.len());
    let methods = list["methods"].as_array().expect("methods array");
    assert!(!methods.is_empty());
}

#[test]
fn primal_name_is_correct() {
    assert_eq!(PRIMAL_NAME, "loamspine");
    assert_eq!(PRIMAL_NAME, crate::primal_names::SELF_ID);
}

#[test]
fn capabilities_contains_expected_entries() {
    assert!(CAPABILITIES.contains(&"permanence"));
    assert!(CAPABILITIES.contains(&"spine.create"));
    assert!(CAPABILITIES.contains(&"spine.query"));
    assert!(CAPABILITIES.contains(&"certificate.issue"));
    assert!(CAPABILITIES.contains(&"capability.list"));
}

#[test]
#[serial]
fn socket_path_respects_env() {
    temp_env::with_vars(
        [("LOAMSPINE_SOCKET", Some("/custom/loamspine.sock"))],
        || {
            let path = resolve_socket_path();
            assert_eq!(path.to_string_lossy(), "/custom/loamspine.sock");
        },
    );
}

#[test]
fn capability_list_is_valid_json() {
    let list = capability_list();
    let s = serde_json::to_string(&list).expect("serialize");
    let _: serde_json::Value = serde_json::from_str(&s).expect("deserialize");
}

#[test]
#[serial]
fn resolve_socket_path_uses_xdg_runtime_dir_when_loamspine_socket_unset() {
    temp_env::with_vars(
        [
            ("LOAMSPINE_SOCKET", None),
            ("XDG_RUNTIME_DIR", Some("/run/user/1000")),
            ("BIOMEOS_FAMILY_ID", None),
        ],
        || {
            let path = resolve_socket_path();
            assert_eq!(
                path.to_string_lossy(),
                "/run/user/1000/biomeos/loamspine-default.sock"
            );
        },
    );
    temp_env::with_vars(
        [
            ("LOAMSPINE_SOCKET", None),
            ("XDG_RUNTIME_DIR", Some("/run/user/1000")),
            ("BIOMEOS_FAMILY_ID", Some("myfamily")),
        ],
        || {
            let path = resolve_socket_path();
            assert_eq!(
                path.to_string_lossy(),
                "/run/user/1000/biomeos/loamspine-myfamily.sock"
            );
        },
    );
}

#[test]
#[serial]
fn resolve_neural_api_socket_with_env() {
    temp_env::with_vars(
        [
            ("BIOMEOS_NEURAL_API_SOCKET", Some("/custom/neural.sock")),
            ("XDG_RUNTIME_DIR", None),
            ("BIOMEOS_FAMILY_ID", None),
        ],
        || {
            let path = super::resolve_neural_api_socket();
            assert!(path.is_some());
            assert_eq!(path.unwrap().to_string_lossy(), "/custom/neural.sock");
        },
    );
}

#[test]
#[serial]
fn resolve_neural_api_socket_with_xdg_runtime_dir() {
    temp_env::with_vars(
        [
            ("BIOMEOS_NEURAL_API_SOCKET", None),
            ("XDG_RUNTIME_DIR", Some("/run/user/1000")),
            ("BIOMEOS_FAMILY_ID", None),
        ],
        || {
            let path = super::resolve_neural_api_socket();
            assert!(path.is_some());
            assert_eq!(
                path.unwrap().to_string_lossy(),
                "/run/user/1000/biomeos/neural-api-default.sock"
            );
        },
    );
}

#[test]
#[serial]
fn resolve_neural_api_socket_without_env_returns_none() {
    temp_env::with_vars(CLEAN, || {
        let path = super::resolve_neural_api_socket();
        assert!(path.is_none());
    });
}

#[test]
fn capability_list_pretty_output_validation() {
    let pretty = capability_list_pretty();
    assert!(!pretty.is_empty());
    assert!(pretty.contains('\n'));
    assert!(pretty.contains("  "));
    let parsed: serde_json::Value =
        serde_json::from_str(&pretty).expect("pretty output must be valid JSON");
    assert!(parsed.get("capabilities").is_some());
    assert!(parsed.get("primal").is_some());
    assert!(parsed.get("version").is_some());
    assert!(parsed.get("methods").is_some());
    assert_eq!(parsed["primal"], PRIMAL_NAME);
    let caps = parsed["capabilities"]
        .as_array()
        .expect("capabilities array");
    assert_eq!(caps.len(), CAPABILITIES.len());
}

#[test]
#[serial]
fn register_with_neural_api_returns_ok_false_when_xdg_runtime_dir_unset() {
    temp_env::with_vars(CLEAN, || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(register_with_neural_api());
        assert!(result.is_ok());
        assert!(!result.unwrap());
    });
}

#[test]
#[serial]
fn deregister_from_neural_api_returns_ok_when_xdg_runtime_dir_unset() {
    temp_env::with_vars(CLEAN, || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(deregister_from_neural_api());
        assert!(result.is_ok());
    });
}

#[test]
#[serial]
fn resolve_socket_path_fallback_when_xdg_unset() {
    temp_env::with_vars(CLEAN, || {
        let path = resolve_socket_path();
        assert!(
            path.to_string_lossy()
                .ends_with("biomeos/loamspine-default.sock"),
            "got: {}",
            path.display()
        );
    });
}

#[test]
#[serial]
fn resolve_socket_path_with_custom_family_id() {
    temp_env::with_vars(
        [
            ("LOAMSPINE_SOCKET", None),
            ("XDG_RUNTIME_DIR", None),
            ("BIOMEOS_NEURAL_API_SOCKET", None),
            ("BIOMEOS_FAMILY_ID", Some("custom-family")),
        ],
        || {
            let path = resolve_socket_path();
            assert!(
                path.to_string_lossy()
                    .ends_with("biomeos/loamspine-custom-family.sock"),
                "got: {}",
                path.display()
            );
        },
    );
}

#[test]
#[serial]
fn resolve_socket_path_loamspine_socket_overrides_xdg_and_family() {
    temp_env::with_vars(
        [
            ("LOAMSPINE_SOCKET", Some("/override/path.sock")),
            ("XDG_RUNTIME_DIR", Some("/run/user/1000")),
            ("BIOMEOS_FAMILY_ID", Some("ignored")),
        ],
        || {
            let path = resolve_socket_path();
            assert_eq!(path.to_string_lossy(), "/override/path.sock");
        },
    );
}

#[test]
#[serial]
fn resolve_neural_api_socket_with_family_id() {
    temp_env::with_vars(
        [
            ("BIOMEOS_NEURAL_API_SOCKET", None),
            ("XDG_RUNTIME_DIR", Some("/run/user/42")),
            ("BIOMEOS_FAMILY_ID", Some("my-family")),
        ],
        || {
            let path = super::resolve_neural_api_socket();
            assert!(path.is_some());
            assert_eq!(
                path.unwrap().to_string_lossy(),
                "/run/user/42/biomeos/neural-api-my-family.sock"
            );
        },
    );
}

#[test]
fn capability_list_pretty_contains_primal_and_capabilities() {
    let pretty = capability_list_pretty();
    assert!(pretty.contains(PRIMAL_NAME));
    assert!(pretty.contains("permanence"));
    assert!(pretty.contains("capability.list"));
}

/// Helper: spawn a mock NeuralAPI Unix socket server.
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

#[test]
#[serial]
fn register_with_neural_api_succeeds_with_mock_server() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api.sock");
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": { "registered": true },
        "id": 1
    });
    let sock_path = sock.to_str().unwrap().to_string();
    temp_env::with_vars(
        [("BIOMEOS_NEURAL_API_SOCKET", Some(sock_path.as_str()))],
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let handle = spawn_mock_neural_api(&sock, &response);
                let result = register_with_neural_api().await;
                handle.abort();
                assert!(result.is_ok());
                assert!(result.unwrap());
            });
        },
    );
}

#[test]
#[serial]
fn register_with_neural_api_returns_error_on_jsonrpc_error() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api-err.sock");
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": -32601, "message": "method not found" },
        "id": 1
    });
    let sock_path = sock.to_str().unwrap().to_string();
    temp_env::with_vars(
        [("BIOMEOS_NEURAL_API_SOCKET", Some(sock_path.as_str()))],
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let handle = spawn_mock_neural_api(&sock, &response);
                let result = register_with_neural_api().await;
                handle.abort();
                assert!(result.is_err());
                let err = result.unwrap_err().to_string();
                assert!(err.contains("method not found"), "error: {err}");
            });
        },
    );
}

#[test]
#[serial]
fn deregister_from_neural_api_succeeds_with_mock_server() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api-dereg.sock");
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "result": { "deregistered": true },
        "id": 2
    });
    let sock_path = sock.to_str().unwrap().to_string();
    temp_env::with_vars(
        [("BIOMEOS_NEURAL_API_SOCKET", Some(sock_path.as_str()))],
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let handle = spawn_mock_neural_api(&sock, &response);
                let result = deregister_from_neural_api().await;
                handle.abort();
                assert!(result.is_ok());
            });
        },
    );
}

#[test]
#[serial]
fn deregister_from_neural_api_handles_jsonrpc_error_gracefully() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api-dereg-err.sock");
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": -32601, "message": "not supported" },
        "id": 2
    });
    let sock_path = sock.to_str().unwrap().to_string();
    temp_env::with_vars(
        [("BIOMEOS_NEURAL_API_SOCKET", Some(sock_path.as_str()))],
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let handle = spawn_mock_neural_api(&sock, &response);
                let result = deregister_from_neural_api().await;
                handle.abort();
                assert!(
                    result.is_ok(),
                    "deregister should succeed even on JSON-RPC error"
                );
            });
        },
    );
}

#[test]
#[serial]
fn deregister_from_neural_api_handles_malformed_response() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("neural-api-dereg-bad.sock");
    let sock_path = sock.to_str().unwrap().to_string();
    temp_env::with_vars(
        [("BIOMEOS_NEURAL_API_SOCKET", Some(sock_path.as_str()))],
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
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
                let result = deregister_from_neural_api().await;
                handle.abort();
                assert!(
                    result.is_ok(),
                    "deregister should succeed even on malformed response"
                );
            });
        },
    );
}

#[test]
fn registration_gracefully_handles_missing_socket() {
    temp_env::with_vars(
        [(
            "BIOMEOS_NEURAL_API_SOCKET",
            Some("/tmp/nonexistent-neural-api-loamspine-test.sock"),
        )],
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(register_with_neural_api());
            assert!(result.is_ok());
            assert!(!result.unwrap());
        },
    );
}

#[test]
fn deregistration_gracefully_handles_missing_socket() {
    temp_env::with_vars(
        [(
            "BIOMEOS_NEURAL_API_SOCKET",
            Some("/tmp/nonexistent-neural-api-loamspine-test.sock"),
        )],
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(deregister_from_neural_api());
            assert!(result.is_ok());
        },
    );
}

#[test]
fn capabilities_has_no_duplicates() {
    let mut seen = std::collections::HashSet::new();
    for cap in CAPABILITIES {
        assert!(seen.insert(cap), "duplicate capability: {cap}");
    }
}

#[test]
fn capabilities_follow_semantic_naming() {
    for cap in CAPABILITIES {
        assert!(
            !cap.contains("loamspine"),
            "capability '{cap}' should not reference primal name"
        );
        assert!(!cap.is_empty());
    }
}
