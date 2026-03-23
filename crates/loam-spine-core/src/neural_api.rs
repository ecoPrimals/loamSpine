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
        crate::error::LoamSpineError::ipc(
            crate::error::IpcErrorPhase::Serialization,
            format!("Failed to serialize NeuralAPI registration: {e}"),
        )
    })?;

    let mut stream = UnixStream::connect(&socket_path).await.map_err(|e| {
        crate::error::LoamSpineError::ipc(
            crate::error::IpcErrorPhase::Connect,
            format!(
                "NeuralAPI connection failed at {}: {e}",
                socket_path.display()
            ),
        )
    })?;

    let len = u32::try_from(request_bytes.len()).map_err(|_| {
        crate::error::LoamSpineError::ipc(
            crate::error::IpcErrorPhase::Serialization,
            "Registration payload too large",
        )
    })?;
    stream.write_all(&len.to_be_bytes()).await.map_err(|e| {
        crate::error::LoamSpineError::ipc(
            crate::error::IpcErrorPhase::Write,
            format!("NeuralAPI write failed: {e}"),
        )
    })?;
    stream.write_all(&request_bytes).await.map_err(|e| {
        crate::error::LoamSpineError::ipc(
            crate::error::IpcErrorPhase::Write,
            format!("NeuralAPI write failed: {e}"),
        )
    })?;
    stream.flush().await.map_err(|e| {
        crate::error::LoamSpineError::ipc(
            crate::error::IpcErrorPhase::Write,
            format!("NeuralAPI flush failed: {e}"),
        )
    })?;

    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await.map_err(|e| {
        crate::error::LoamSpineError::ipc(
            crate::error::IpcErrorPhase::Read,
            format!("NeuralAPI response length read failed: {e}"),
        )
    })?;
    let resp_len = usize::try_from(u32::from_be_bytes(len_buf)).map_err(|_| {
        crate::error::LoamSpineError::ipc(
            crate::error::IpcErrorPhase::Read,
            "NeuralAPI response length exceeds platform capacity",
        )
    })?;
    let mut resp_buf = vec![0u8; resp_len];
    stream.read_exact(&mut resp_buf).await.map_err(|e| {
        crate::error::LoamSpineError::ipc(
            crate::error::IpcErrorPhase::Read,
            format!("NeuralAPI response read failed: {e}"),
        )
    })?;

    let response: serde_json::Value = serde_json::from_slice(&resp_buf).map_err(|e| {
        crate::error::LoamSpineError::ipc(
            crate::error::IpcErrorPhase::InvalidJson,
            format!("NeuralAPI response parse failed: {e}"),
        )
    })?;

    if let Some((code, message)) = crate::error::extract_rpc_error(&response) {
        return Err(crate::error::LoamSpineError::ipc(
            crate::error::IpcErrorPhase::JsonRpcError(code),
            format!("NeuralAPI registration error: {message}"),
        ));
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
        crate::error::LoamSpineError::ipc(
            crate::error::IpcErrorPhase::Serialization,
            format!("Failed to serialize NeuralAPI deregister: {e}"),
        )
    })?;

    let mut stream = match UnixStream::connect(&socket_path).await {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!("NeuralAPI deregister connection failed: {e}");
            return Ok(());
        }
    };

    let len = u32::try_from(request_bytes.len()).map_err(|_| {
        crate::error::LoamSpineError::ipc(
            crate::error::IpcErrorPhase::Serialization,
            "Deregister payload too large",
        )
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
        "operation_dependencies": {
            "entry.append": ["spine.create"],
            "certificate.mint": ["spine.create"],
            "certificate.transfer": ["certificate.mint"],
            "certificate.loan": ["certificate.mint"],
            "certificate.return": ["certificate.loan"],
            "slice.anchor": ["spine.create"],
            "slice.record_operation": ["slice.anchor"],
            "slice.depart": ["slice.anchor"],
            "proof.generate_inclusion": ["entry.append"],
            "session.commit": ["spine.create"],
            "braid.commit": ["spine.create"],
        },
        "cost_estimates": {
            "spine.create":              { "latency_ms": 1, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "spine.get":                 { "latency_ms": 1, "cpu": "low", "memory_bytes": 2048, "gpu_eligible": false },
            "spine.seal":                { "latency_ms": 1, "cpu": "low", "memory_bytes": 2048, "gpu_eligible": false },
            "entry.append":              { "latency_ms": 2, "cpu": "low", "memory_bytes": 8192, "gpu_eligible": false },
            "entry.get":                 { "latency_ms": 1, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "entry.get_tip":             { "latency_ms": 1, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "certificate.mint":          { "latency_ms": 2, "cpu": "low", "memory_bytes": 8192, "gpu_eligible": false },
            "certificate.transfer":      { "latency_ms": 2, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "certificate.loan":          { "latency_ms": 2, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "certificate.return":        { "latency_ms": 2, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "certificate.get":           { "latency_ms": 1, "cpu": "low", "memory_bytes": 2048, "gpu_eligible": false },
            "certificate.verify":        { "latency_ms": 5, "cpu": "medium", "memory_bytes": 16384, "gpu_eligible": false },
            "proof.generate_inclusion":  { "latency_ms": 10, "cpu": "medium", "memory_bytes": 32768, "gpu_eligible": false },
            "proof.verify_inclusion":    { "latency_ms": 5, "cpu": "medium", "memory_bytes": 16384, "gpu_eligible": false },
            "session.commit":            { "latency_ms": 5, "cpu": "medium", "memory_bytes": 16384, "gpu_eligible": false },
            "braid.commit":              { "latency_ms": 5, "cpu": "medium", "memory_bytes": 16384, "gpu_eligible": false },
            "health.check":              { "latency_ms": 1, "cpu": "low", "memory_bytes": 1024, "gpu_eligible": false },
            "capability.list":           { "latency_ms": 1, "cpu": "low", "memory_bytes": 1024, "gpu_eligible": false },
        },
    })
}

/// Return the capability list as a pretty-printed JSON string.
/// Used by the `loamspine capabilities` CLI subcommand.
#[must_use]
pub fn capability_list_pretty() -> String {
    serde_json::to_string_pretty(&capability_list()).unwrap_or_default()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests use unwrap for conciseness"
)]
#[path = "neural_api_tests.rs"]
mod tests;
