// SPDX-License-Identifier: AGPL-3.0-or-later

//! NeuralAPI registration for biomeOS orchestration.
//!
//! This module handles LoamSpine's registration with biomeOS's NeuralAPI,
//! advertising capabilities and socket endpoints so the orchestration layer
//! can route capability requests to LoamSpine.

use std::path::PathBuf;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// LoamSpine's primal identity for NeuralAPI registration.
///
/// Delegates to [`crate::primal_names::SELF_ID`] — single source of truth.
pub const PRIMAL_NAME: &str = crate::primal_names::SELF_ID;

/// Semantic capabilities LoamSpine provides to the ecosystem.
/// These map to biomeOS's `capability_domains.rs` for NeuralAPI routing.
pub const CAPABILITIES: &[&str] = &[
    "permanence",
    "commit.session",
    "spine.create",
    "spine.query",
    "certificate.issue",
    "certificate.transfer",
    "certificate.verify",
    "temporal_anchor",
    "selective-memory",
    "inclusion-proofs",
    "backup",
    "restore",
    "braid.commit",
    "slice.anchor",
    "slice.checkout",
    "proof.generate",
    "proof.verify",
    "health.check",
    "capability.list",
];

/// Resolve the socket path for LoamSpine's IPC endpoint.
///
/// 5-tier resolution order:
/// 1. `LOAMSPINE_SOCKET` environment variable (explicit override)
/// 2. `$XDG_RUNTIME_DIR/biomeos/loamspine-{family_id}.sock`
/// 3. `/run/user/{uid}/biomeos/loamspine-{family_id}.sock` (Linux)
/// 4. `{temp_dir}/biomeos/loamspine-{family_id}.sock`
#[must_use]
pub fn resolve_socket_path() -> PathBuf {
    if let Ok(s) = std::env::var("LOAMSPINE_SOCKET") {
        return PathBuf::from(s);
    }
    let family_id = std::env::var("BIOMEOS_FAMILY_ID").unwrap_or_else(|_| "default".to_string());
    let sock_name = format!("{}-{family_id}.sock", crate::primal_names::SELF_ID);

    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime_dir)
            .join(crate::primal_names::BIOMEOS_SOCKET_DIR)
            .join(&sock_name);
    }

    #[cfg(target_os = "linux")]
    if let Some(base) = crate::constants::network::linux_run_user_biomeos() {
        return base.join(&sock_name);
    }

    std::env::temp_dir()
        .join(crate::primal_names::BIOMEOS_SOCKET_DIR)
        .join(sock_name)
}

/// Resolve the NeuralAPI socket path for connecting to biomeOS.
fn resolve_neural_api_socket() -> Option<PathBuf> {
    if let Ok(s) = std::env::var("BIOMEOS_NEURAL_API_SOCKET") {
        return Some(PathBuf::from(s));
    }
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").ok()?;
    let family_id = std::env::var("BIOMEOS_FAMILY_ID").unwrap_or_else(|_| "default".to_string());
    Some(PathBuf::from(format!(
        "{runtime_dir}/biomeos/neural-api-{family_id}.sock"
    )))
}

/// Register LoamSpine with biomeOS's NeuralAPI.
///
/// Sends a `lifecycle.register` JSON-RPC request to the NeuralAPI Unix socket.
/// If NeuralAPI is not available, logs a debug message and returns Ok (non-fatal).
///
/// # Errors
///
/// Returns an error only if registration was attempted but critically failed.
pub async fn register_with_neural_api() -> crate::error::LoamSpineResult<bool> {
    let Some(socket_path) = resolve_neural_api_socket() else {
        tracing::debug!("NeuralAPI socket not resolvable (XDG_RUNTIME_DIR unset)");
        return Ok(false);
    };
    if !socket_path.exists() {
        tracing::debug!(
            "NeuralAPI socket not found at {}, running standalone",
            socket_path.display()
        );
        return Ok(false);
    }

    let our_socket = resolve_socket_path();
    let pid = std::process::id();
    let capabilities: &[&str] = CAPABILITIES;

    let params = serde_json::json!({
        "name": PRIMAL_NAME,
        "socket_path": our_socket.to_string_lossy(),
        "pid": pid,
        "capabilities": capabilities,
    });

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "lifecycle.register",
        "params": params,
        "id": 1u64,
    });

    let request_bytes = serde_json::to_vec(&request).map_err(|e| {
        crate::error::LoamSpineError::Network(format!(
            "Failed to serialize NeuralAPI registration: {e}"
        ))
    })?;

    let mut stream = UnixStream::connect(&socket_path).await.map_err(|e| {
        crate::error::LoamSpineError::Network(format!(
            "NeuralAPI connection failed at {}: {e}",
            socket_path.display()
        ))
    })?;

    let len = u32::try_from(request_bytes.len()).map_err(|_| {
        crate::error::LoamSpineError::Network("Registration payload too large".into())
    })?;
    stream.write_all(&len.to_be_bytes()).await.map_err(|e| {
        crate::error::LoamSpineError::Network(format!("NeuralAPI write failed: {e}"))
    })?;
    stream.write_all(&request_bytes).await.map_err(|e| {
        crate::error::LoamSpineError::Network(format!("NeuralAPI write failed: {e}"))
    })?;
    stream.flush().await.map_err(|e| {
        crate::error::LoamSpineError::Network(format!("NeuralAPI flush failed: {e}"))
    })?;

    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await.map_err(|e| {
        crate::error::LoamSpineError::Network(format!("NeuralAPI response length read failed: {e}"))
    })?;
    let resp_len = usize::try_from(u32::from_be_bytes(len_buf)).map_err(|_| {
        crate::error::LoamSpineError::Network(
            "NeuralAPI response length exceeds platform capacity".into(),
        )
    })?;
    let mut resp_buf = vec![0u8; resp_len];
    stream.read_exact(&mut resp_buf).await.map_err(|e| {
        crate::error::LoamSpineError::Network(format!("NeuralAPI response read failed: {e}"))
    })?;

    let response: serde_json::Value = serde_json::from_slice(&resp_buf).map_err(|e| {
        crate::error::LoamSpineError::Network(format!("NeuralAPI response parse failed: {e}"))
    })?;

    if let Some(err) = response.get("error") {
        let msg = err
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        return Err(crate::error::LoamSpineError::Network(format!(
            "NeuralAPI registration error: {msg}"
        )));
    }

    Ok(true)
}

/// Deregister LoamSpine from NeuralAPI on shutdown.
///
/// Sends `lifecycle.deregister` if available. If NeuralAPI is not reachable
/// or the method does not exist, logs and returns Ok (non-fatal).
///
/// # Errors
///
/// Returns an error only if a critical failure occurred during shutdown.
pub async fn deregister_from_neural_api() -> crate::error::LoamSpineResult<()> {
    let Some(socket_path) = resolve_neural_api_socket() else {
        tracing::debug!("NeuralAPI socket not resolvable, skipping deregister");
        return Ok(());
    };
    if !socket_path.exists() {
        tracing::debug!("NeuralAPI socket not found, skipping deregister");
        return Ok(());
    }

    let params = serde_json::json!({ "name": crate::primal_names::SELF_ID });
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "lifecycle.deregister",
        "params": params,
        "id": 2u64,
    });

    let request_bytes = serde_json::to_vec(&request).map_err(|e| {
        crate::error::LoamSpineError::Network(format!(
            "Failed to serialize NeuralAPI deregister: {e}"
        ))
    })?;

    let mut stream = match UnixStream::connect(&socket_path).await {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!("NeuralAPI deregister connection failed: {e}");
            return Ok(());
        }
    };

    let len = u32::try_from(request_bytes.len()).map_err(|_| {
        crate::error::LoamSpineError::Network("Deregister payload too large".into())
    })?;
    if let Err(e) = stream.write_all(&len.to_be_bytes()).await {
        tracing::debug!("NeuralAPI deregister write failed: {e}");
        return Ok(());
    }
    if let Err(e) = stream.write_all(&request_bytes).await {
        tracing::debug!("NeuralAPI deregister write failed: {e}");
        return Ok(());
    }
    if let Err(e) = stream.flush().await {
        tracing::debug!("NeuralAPI deregister flush failed: {e}");
        return Ok(());
    }

    let mut len_buf = [0u8; 4];
    if let Err(e) = stream.read_exact(&mut len_buf).await {
        tracing::debug!("NeuralAPI deregister response read failed: {e}");
        return Ok(());
    }
    let Ok(resp_len) = usize::try_from(u32::from_be_bytes(len_buf)) else {
        tracing::debug!("NeuralAPI deregister response length overflow");
        return Ok(());
    };
    let mut resp_buf = vec![0u8; resp_len];
    if let Err(e) = stream.read_exact(&mut resp_buf).await {
        tracing::debug!("NeuralAPI deregister response read failed: {e}");
        return Ok(());
    }

    let response: serde_json::Value = match serde_json::from_slice(&resp_buf) {
        Ok(v) => v,
        Err(e) => {
            tracing::debug!("NeuralAPI deregister response parse failed: {e}");
            return Ok(());
        }
    };

    if response.get("error").is_some() {
        let msg = response
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("unknown");
        tracing::debug!("NeuralAPI deregister returned error: {msg}");
    }

    Ok(())
}

/// Return the capability list as a JSON-RPC response payload.
/// Implements the `capability.list` semantic method.
/// Aligns with ludoSpring's enhanced format: domain, method, dependencies, cost tier.
#[must_use]
pub fn capability_list() -> serde_json::Value {
    serde_json::json!({
        "primal": PRIMAL_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "capabilities": CAPABILITIES,
        "methods": [
            { "method": "spine.create", "domain": "spine", "cost": "low", "deps": [] },
            { "method": "spine.get", "domain": "spine", "cost": "low", "deps": [] },
            { "method": "spine.seal", "domain": "spine", "cost": "low", "deps": [] },
            { "method": "entry.append", "domain": "entry", "cost": "low", "deps": ["spine.create"] },
            { "method": "entry.get", "domain": "entry", "cost": "low", "deps": [] },
            { "method": "entry.get_tip", "domain": "entry", "cost": "low", "deps": [] },
            { "method": "certificate.mint", "domain": "certificate", "cost": "low", "deps": ["spine.create"] },
            { "method": "certificate.transfer", "domain": "certificate", "cost": "low", "deps": ["certificate.mint"] },
            { "method": "certificate.loan", "domain": "certificate", "cost": "low", "deps": ["certificate.mint"] },
            { "method": "certificate.return", "domain": "certificate", "cost": "low", "deps": ["certificate.loan"] },
            { "method": "certificate.get", "domain": "certificate", "cost": "low", "deps": [] },
            { "method": "certificate.verify", "domain": "certificate", "cost": "medium", "deps": [] },
            { "method": "certificate.lifecycle", "domain": "certificate", "cost": "medium", "deps": [] },
            { "method": "slice.anchor", "domain": "waypoint", "cost": "low", "deps": ["spine.create"] },
            { "method": "slice.checkout", "domain": "waypoint", "cost": "low", "deps": [] },
            { "method": "slice.record_operation", "domain": "waypoint", "cost": "low", "deps": ["slice.anchor"] },
            { "method": "slice.depart", "domain": "waypoint", "cost": "low", "deps": ["slice.anchor"] },
            { "method": "proof.generate_inclusion", "domain": "proof", "cost": "medium", "deps": ["entry.append"] },
            { "method": "proof.verify_inclusion", "domain": "proof", "cost": "medium", "deps": [] },
            { "method": "session.commit", "domain": "integration", "cost": "medium", "deps": ["spine.create"] },
            { "method": "braid.commit", "domain": "integration", "cost": "medium", "deps": ["spine.create"] },
            { "method": "health.check", "domain": "health", "cost": "low", "deps": [] },
            { "method": "capability.list", "domain": "meta", "cost": "low", "deps": [] },
        ],
    })
}

/// Return the capability list as a pretty-printed JSON string.
/// Used by the `loamspine capabilities` CLI subcommand.
#[must_use]
pub fn capability_list_pretty() -> String {
    serde_json::to_string_pretty(&capability_list()).unwrap_or_default()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
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
}
