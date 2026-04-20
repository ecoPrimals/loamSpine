// SPDX-License-Identifier: AGPL-3.0-or-later

//! NeuralAPI registration for ecosystem orchestration.
//!
//! This module handles LoamSpine's registration with the ecosystem NeuralAPI,
//! advertising capabilities and socket endpoints so the orchestration layer
//! can route capability requests to LoamSpine.

use std::sync::OnceLock;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

mod mcp;
mod socket;

pub use mcp::{mcp_tool_to_rpc, mcp_tools_list};
pub use socket::{
    capability_domain_socket_name, domain_socket_name, legacy_socket_name,
    resolve_capability_symlink_path, resolve_legacy_symlink_path, resolve_neural_api_socket_with,
    resolve_socket_path, resolve_socket_path_with, validate_security_config,
    validate_security_config_from_env,
};

/// LoamSpine's primal identity for NeuralAPI registration.
///
/// Delegates to [`crate::primal_names::SELF_ID`] — single source of truth.
pub const PRIMAL_NAME: &str = crate::primal_names::SELF_ID;

/// Semantic capabilities LoamSpine provides to the ecosystem.
/// These map to the orchestrator's `capability_domains.rs` for NeuralAPI routing.
pub const CAPABILITIES: &[&str] = &[
    "permanence",
    "ledger",
    "session.commit",
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
    "anchor.publish",
    "anchor.verify",
    "bonding.ledger.store",
    "bonding.ledger.retrieve",
    "bonding.ledger.list",
    "health.check",
    "capability.list",
];

/// Register LoamSpine with the ecosystem NeuralAPI.
///
/// Sends a `lifecycle.register` JSON-RPC request to the NeuralAPI Unix socket.
/// If NeuralAPI is not available, logs a debug message and returns Ok (non-fatal).
///
/// # Errors
///
/// Returns an error only if registration was attempted but critically failed.
pub async fn register_with_neural_api() -> crate::error::LoamSpineResult<bool> {
    let Some(socket_path) = socket::resolve_neural_api_socket() else {
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
    register_at_socket(&socket_path, &our_socket).await
}

/// Inner registration logic (pure — no env reads, testable concurrently).
pub(crate) async fn register_at_socket(
    socket_path: &std::path::Path,
    our_socket: &std::path::Path,
) -> crate::error::LoamSpineResult<bool> {
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
    let Some(socket_path) = socket::resolve_neural_api_socket() else {
        tracing::debug!("NeuralAPI socket not resolvable, skipping deregister");
        return Ok(());
    };
    if !socket_path.exists() {
        tracing::debug!("NeuralAPI socket not found, skipping deregister");
        return Ok(());
    }
    deregister_at_socket(&socket_path).await
}

/// Inner deregistration logic (pure — no env reads, testable concurrently).
pub(crate) async fn deregister_at_socket(
    socket_path: &std::path::Path,
) -> crate::error::LoamSpineResult<()> {
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

    if let Some((_code, message)) = crate::error::extract_rpc_error(&response) {
        tracing::debug!("NeuralAPI deregister returned error: {message}");
    }

    Ok(())
}

/// Cached capability list — initialized once, reused for all subsequent calls.
static CAPABILITY_LIST_CACHE: OnceLock<serde_json::Value> = OnceLock::new();

/// Return the capability list as a JSON-RPC response payload.
/// Implements the `capability.list` semantic method.
///
/// Uses `OnceLock` to initialize the JSON value once and return a reference thereafter.
#[must_use]
pub fn capability_list() -> &'static serde_json::Value {
    CAPABILITY_LIST_CACHE.get_or_init(capability_list_inner)
}

/// Cached identity response — initialized once.
static IDENTITY_CACHE: OnceLock<serde_json::Value> = OnceLock::new();

/// Return the `identity.get` response payload per Capability Wire Standard v1.0.
#[must_use]
pub fn identity_response() -> &'static serde_json::Value {
    IDENTITY_CACHE.get_or_init(|| {
        serde_json::json!({
            "primal": PRIMAL_NAME,
            "version": env!("CARGO_PKG_VERSION"),
            "domain": "permanence",
            "license": env!("CARGO_PKG_LICENSE"),
        })
    })
}

fn capability_list_inner() -> serde_json::Value {
    serde_json::json!({
        "primal": PRIMAL_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        // Wire Standard L2: flat string array of all callable methods (primary ecosystem routing signal)
        "methods": crate::niche::METHODS,
        // Wire Standard L3: capability groupings for structured routing
        "provided_capabilities": [
            { "type": "spine", "methods": ["create", "get", "seal"], "version": env!("CARGO_PKG_VERSION"), "description": "Append-only spine lifecycle" },
            { "type": "entry", "methods": ["append", "get", "get_tip"], "version": env!("CARGO_PKG_VERSION"), "description": "Content-addressed entry management" },
            { "type": "certificate", "methods": ["mint", "transfer", "loan", "return", "get", "verify", "lifecycle"], "version": env!("CARGO_PKG_VERSION"), "description": "Certificate lifecycle and provenance" },
            { "type": "proof", "methods": ["generate_inclusion", "verify_inclusion"], "version": env!("CARGO_PKG_VERSION"), "description": "Merkle inclusion proofs" },
            { "type": "anchor", "methods": ["publish", "verify"], "version": env!("CARGO_PKG_VERSION"), "description": "Public chain anchoring" },
            { "type": "session", "methods": ["commit"], "version": env!("CARGO_PKG_VERSION"), "description": "Provenance trio session commit" },
            { "type": "braid", "methods": ["commit"], "version": env!("CARGO_PKG_VERSION"), "description": "Provenance trio braid commit" },
            { "type": "slice", "methods": ["anchor", "checkout", "record_operation", "depart"], "version": env!("CARGO_PKG_VERSION"), "description": "Waypoint slice operations" },
            { "type": "bonding", "methods": ["ledger.store", "ledger.retrieve", "ledger.list"], "version": env!("CARGO_PKG_VERSION"), "description": "Ionic bond contract ledger persistence" },
            { "type": "health", "methods": ["check", "liveness", "readiness"], "version": env!("CARGO_PKG_VERSION"), "description": "Health probes" },
        ],
        // Wire Standard L3: consumed capabilities for composition completeness validation
        "consumed_capabilities": crate::niche::CONSUMED_CAPABILITIES,
        // Backward-compatible: semantic capability labels for ecosystem domain registration
        "capabilities": CAPABILITIES,
        // Wire Standard L3: per-method cost hints for AI advisors and scheduler
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
            "anchor.publish":            { "latency_ms": 2, "cpu": "low", "memory_bytes": 8192, "gpu_eligible": false },
            "anchor.verify":             { "latency_ms": 2, "cpu": "low", "memory_bytes": 8192, "gpu_eligible": false },
            "bonding.ledger.store":      { "latency_ms": 2, "cpu": "low", "memory_bytes": 8192, "gpu_eligible": false },
            "bonding.ledger.retrieve":   { "latency_ms": 1, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "bonding.ledger.list":       { "latency_ms": 1, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "capability.list":           { "latency_ms": 1, "cpu": "low", "memory_bytes": 1024, "gpu_eligible": false },
            "identity.get":              { "latency_ms": 1, "cpu": "low", "memory_bytes": 1024, "gpu_eligible": false },
        },
        // Wire Standard L3: method dependency DAG for execution planners
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
            "anchor.publish": ["spine.create"],
            "anchor.verify": ["anchor.publish"],
            "bonding.ledger.retrieve": ["bonding.ledger.store"],
            "bonding.ledger.list": ["bonding.ledger.store"],
        },
    })
}

/// Return the capability list as a pretty-printed JSON string.
/// Used by the `loamspine capabilities` CLI subcommand.
#[must_use]
pub fn capability_list_pretty() -> String {
    serde_json::to_string_pretty(capability_list()).unwrap_or_default()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests use unwrap for conciseness"
)]
#[path = "tests.rs"]
mod tests;
