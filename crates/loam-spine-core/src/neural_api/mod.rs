// SPDX-License-Identifier: AGPL-3.0-or-later

//! NeuralAPI registration for ecosystem orchestration.
//!
//! This module handles LoamSpine's registration with the ecosystem NeuralAPI,
//! advertising capabilities and socket endpoints so the orchestration layer
//! can route capability requests to LoamSpine.

use std::sync::LazyLock;

#[cfg(unix)]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(unix)]
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

/// Signal tier for Neural API routing — determines scheduling priority.
/// `"nest"` = data-layer primal (storage/DAG/permanence).
pub const SIGNAL_TIERS: &[&str] = &["nest"];

/// Semantic capability domains for Neural API announce (Wave 43 schema).
/// These are the high-level routing labels biomeOS uses for `capability.call`.
pub const ANNOUNCE_CAPABILITIES: &[&str] = &["anchor", "ledger", "permanence"];

/// Per-domain cost hints for Neural API routing weights.
/// Higher values = more expensive operations.
pub const COST_HINTS: &[(&str, f64)] = &[("anchor", 20.0), ("ledger", 15.0), ("permanence", 30.0)];

/// Per-domain latency estimates (ms) for Neural API routing.
pub const LATENCY_ESTIMATES: &[(&str, u32)] =
    &[("anchor", 50), ("ledger", 20), ("permanence", 100)];

/// Semantic capabilities LoamSpine provides to the ecosystem.
/// These map to the orchestrator's `capability_domains.rs` for NeuralAPI routing.
pub const CAPABILITIES: &[&str] = &[
    crate::primal_names::LEGACY_DOMAIN,
    crate::primal_names::CAPABILITY_DOMAIN,
    "session.commit",
    "spine.create",
    "spine.get",
    "spine.list",
    "spine.seal",
    "entry.append",
    "entry.get",
    "entry.get_tip",
    "entry.list",
    "certificate.mint",
    "certificate.transfer",
    "inclusion-proofs",
    "braid.commit",
    "slice.anchor",
    "slice.checkout",
    "proof.generate_inclusion",
    "proof.verify_inclusion",
    "anchor.publish",
    "anchor.publish_batch",
    "anchor.verify",
    "bonding.ledger.store",
    "bonding.ledger.retrieve",
    "bonding.ledger.list",
    "trust.anchor",
    "trust.query",
    "trust.event_count",
    "btsp.negotiate",
    "btsp.capabilities",
    "primal.announce",
    "health.check",
    "lifecycle.status",
    "capabilities.list",
    "auth.check",
    "auth.mode",
    "auth.peer_info",
];

/// Register LoamSpine with the ecosystem NeuralAPI via `primal.announce`.
///
/// Sends a Wave 43 `primal.announce` JSON-RPC request to biomeOS's NeuralAPI
/// Unix socket. Includes semantic capabilities, signal tiers, cost hints,
/// latency estimates, and socket path for routing intelligence.
///
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

/// Build the `primal.announce` payload per Wave 43 Neural API schema.
#[must_use]
pub fn announce_payload(socket_path: &std::path::Path) -> serde_json::Value {
    let cost_hints: serde_json::Map<String, serde_json::Value> = COST_HINTS
        .iter()
        .map(|(k, v)| ((*k).to_owned(), serde_json::json!(v)))
        .collect();

    let latency_estimates: serde_json::Map<String, serde_json::Value> = LATENCY_ESTIMATES
        .iter()
        .map(|(k, v)| ((*k).to_owned(), serde_json::json!(v)))
        .collect();

    serde_json::json!({
        "primal": PRIMAL_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "socket": socket_path.to_string_lossy(),
        "capabilities": ANNOUNCE_CAPABILITIES,
        "methods": crate::niche::METHODS,
        "signal_tiers": SIGNAL_TIERS,
        "cost_hints": cost_hints,
        "latency_estimates": latency_estimates,
        "pid": std::process::id(),
        "domain": crate::primal_names::LEGACY_DOMAIN,
        "capability_domain": crate::primal_names::CAPABILITY_DOMAIN,
        "status": "running",
    })
}

/// Inner registration logic (pure — no env reads, testable concurrently).
#[cfg(unix)]
pub(crate) async fn register_at_socket(
    socket_path: &std::path::Path,
    our_socket: &std::path::Path,
) -> crate::error::LoamSpineResult<bool> {
    let params = announce_payload(our_socket);

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "primal.announce",
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
#[cfg(unix)]
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

#[cfg(not(unix))]
pub(crate) async fn register_at_socket(
    _socket_path: &std::path::Path,
    _our_socket: &std::path::Path,
) -> crate::error::LoamSpineResult<bool> {
    tracing::debug!("NeuralAPI registration not available on non-Unix platforms");
    Ok(false)
}

#[cfg(not(unix))]
pub(crate) async fn deregister_at_socket(
    _socket_path: &std::path::Path,
) -> crate::error::LoamSpineResult<()> {
    tracing::debug!("NeuralAPI deregistration not available on non-Unix platforms");
    Ok(())
}

/// Cached capability list — initialized once, reused for all subsequent calls.
static CAPABILITY_LIST_CACHE: LazyLock<serde_json::Value> = LazyLock::new(capability_list_inner);

/// Return the capability list as a JSON-RPC response payload.
/// Implements the `capability.list` semantic method.
#[must_use]
pub fn capability_list() -> &'static serde_json::Value {
    &CAPABILITY_LIST_CACHE
}

/// Cached identity response — initialized once.
static IDENTITY_CACHE: LazyLock<serde_json::Value> = LazyLock::new(|| {
    serde_json::json!({
        "primal": PRIMAL_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "domain": crate::primal_names::LEGACY_DOMAIN,
        "capability_domain": crate::primal_names::CAPABILITY_DOMAIN,
        "license": env!("CARGO_PKG_LICENSE"),
        "ecobin_grade": "A+",
        "edition": "2024",
    })
});

/// Return the `identity.get` response payload per Capability Wire Standard v1.0.
#[must_use]
pub fn identity_response() -> &'static serde_json::Value {
    &IDENTITY_CACHE
}

fn capability_list_inner() -> serde_json::Value {
    serde_json::json!({
        "primal": PRIMAL_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "count": crate::niche::METHODS.len(),
        // Wire Standard L2: flat string array of all callable methods (primary ecosystem routing signal)
        "methods": crate::niche::METHODS,
        // Wire Standard L3: capability groupings for structured routing
        "provided_capabilities": [
            { "type": "spine", "methods": ["create", "get", "list", "seal"], "version": env!("CARGO_PKG_VERSION"), "stability": "stable", "description": "Append-only spine lifecycle" },
            { "type": "entry", "methods": ["append", "get", "get_tip", "list"], "version": env!("CARGO_PKG_VERSION"), "stability": "stable", "description": "Content-addressed entry management" },
            { "type": "certificate", "methods": ["mint", "transfer", "loan", "return", "get"], "version": env!("CARGO_PKG_VERSION"), "stability": "stable", "description": "Certificate lifecycle and provenance" },
            { "type": "proof", "methods": ["generate_inclusion", "verify_inclusion"], "version": env!("CARGO_PKG_VERSION"), "stability": "stable", "description": "Merkle inclusion proofs" },
            { "type": "anchor", "methods": ["publish", "publish_batch", "verify"], "version": env!("CARGO_PKG_VERSION"), "stability": "stable", "description": "Public chain anchoring (single + aggregate batch)" },
            { "type": "session", "methods": ["dehydrate", "commit"], "version": env!("CARGO_PKG_VERSION"), "stability": "stable", "description": "Session dehydration and commit" },
            { "type": "braid", "methods": ["commit"], "version": env!("CARGO_PKG_VERSION"), "stability": "stable", "description": "Provenance trio braid commit" },
            { "type": "slice", "methods": ["anchor", "checkout"], "version": env!("CARGO_PKG_VERSION"), "stability": "evolving", "description": "Waypoint slice operations" },
            { "type": "btsp", "methods": ["negotiate", "capabilities"], "version": env!("CARGO_PKG_VERSION"), "stability": "stable", "description": "BTSP Phase 3 cipher negotiation" },
            { "type": "lifecycle", "methods": ["status"], "version": env!("CARGO_PKG_VERSION"), "stability": "stable", "description": "Service lifecycle status" },
            { "type": "bonding", "methods": ["ledger.store", "ledger.retrieve", "ledger.list"], "version": env!("CARGO_PKG_VERSION"), "stability": "stable", "description": "Ionic bond contract ledger persistence" },
            { "type": "health", "methods": ["check", "liveness", "readiness"], "version": env!("CARGO_PKG_VERSION"), "stability": "stable", "description": "Health probes" },
            { "type": "auth", "methods": ["check", "mode", "peer_info"], "version": env!("CARGO_PKG_VERSION"), "stability": "stable", "description": "JH-0 method gate introspection" },
            { "type": "primal", "methods": ["announce"], "version": env!("CARGO_PKG_VERSION"), "stability": "stable", "description": "Self-registration / announce" },
        ],
        // Wire Standard L3: consumed capabilities for composition completeness validation
        "consumed_capabilities": crate::niche::CONSUMED_CAPABILITIES,
        // Backward-compatible: semantic capability labels for ecosystem domain registration
        "capabilities": CAPABILITIES,
        // Wire Standard L3: per-method cost hints for AI advisors and scheduler
        "cost_estimates": {
            "spine.create":              { "latency_ms": 1, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "spine.get":                 { "latency_ms": 1, "cpu": "low", "memory_bytes": 2048, "gpu_eligible": false },
            "spine.list":                { "latency_ms": 2, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "spine.seal":                { "latency_ms": 1, "cpu": "low", "memory_bytes": 2048, "gpu_eligible": false },
            "entry.append":              { "latency_ms": 2, "cpu": "low", "memory_bytes": 8192, "gpu_eligible": false },
            "entry.get":                 { "latency_ms": 1, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "entry.get_tip":             { "latency_ms": 1, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "entry.list":                { "latency_ms": 5, "cpu": "low", "memory_bytes": 32768, "gpu_eligible": false },
            "certificate.mint":          { "latency_ms": 2, "cpu": "low", "memory_bytes": 8192, "gpu_eligible": false },
            "certificate.transfer":      { "latency_ms": 2, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "certificate.loan":          { "latency_ms": 2, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "certificate.return":        { "latency_ms": 2, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "certificate.get":           { "latency_ms": 1, "cpu": "low", "memory_bytes": 2048, "gpu_eligible": false },
            "proof.generate_inclusion":  { "latency_ms": 10, "cpu": "medium", "memory_bytes": 32768, "gpu_eligible": false },
            "proof.verify_inclusion":    { "latency_ms": 5, "cpu": "medium", "memory_bytes": 16384, "gpu_eligible": false },
            "session.dehydrate":         { "latency_ms": 5, "cpu": "medium", "memory_bytes": 32768, "gpu_eligible": false },
            "session.commit":            { "latency_ms": 5, "cpu": "medium", "memory_bytes": 16384, "gpu_eligible": false },
            "braid.commit":              { "latency_ms": 5, "cpu": "medium", "memory_bytes": 16384, "gpu_eligible": false },
            "health.check":              { "latency_ms": 1, "cpu": "low", "memory_bytes": 1024, "gpu_eligible": false },
            "anchor.publish":            { "latency_ms": 2, "cpu": "low", "memory_bytes": 8192, "gpu_eligible": false },
            "anchor.publish_batch":      { "latency_ms": 10, "cpu": "medium", "memory_bytes": 65536, "gpu_eligible": false },
            "anchor.verify":             { "latency_ms": 2, "cpu": "low", "memory_bytes": 8192, "gpu_eligible": false },
            "bonding.ledger.store":      { "latency_ms": 2, "cpu": "low", "memory_bytes": 8192, "gpu_eligible": false },
            "bonding.ledger.retrieve":   { "latency_ms": 1, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "bonding.ledger.list":       { "latency_ms": 1, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "trust.anchor":              { "latency_ms": 2, "cpu": "low", "memory_bytes": 8192, "gpu_eligible": false },
            "trust.query":               { "latency_ms": 5, "cpu": "low", "memory_bytes": 16384, "gpu_eligible": false },
            "trust.event_count":         { "latency_ms": 1, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "btsp.negotiate":            { "latency_ms": 2, "cpu": "low", "memory_bytes": 4096, "gpu_eligible": false },
            "btsp.capabilities":         { "latency_ms": 1, "cpu": "low", "memory_bytes": 512, "gpu_eligible": false },
            "primal.announce":           { "latency_ms": 1, "cpu": "low", "memory_bytes": 1024, "gpu_eligible": false },
            "capabilities.list":         { "latency_ms": 1, "cpu": "low", "memory_bytes": 1024, "gpu_eligible": false },
            "identity.get":              { "latency_ms": 1, "cpu": "low", "memory_bytes": 1024, "gpu_eligible": false },
            "lifecycle.status":          { "latency_ms": 1, "cpu": "low", "memory_bytes": 512, "gpu_eligible": false },
            "auth.check":                { "latency_ms": 1, "cpu": "low", "memory_bytes": 512, "gpu_eligible": false },
            "auth.mode":                 { "latency_ms": 1, "cpu": "low", "memory_bytes": 512, "gpu_eligible": false },
            "auth.peer_info":            { "latency_ms": 1, "cpu": "low", "memory_bytes": 512, "gpu_eligible": false },
        },
        // Wire Standard L3: method dependency DAG for execution planners
        "operation_dependencies": {
            "entry.append": ["spine.create"],
            "certificate.mint": ["spine.create"],
            "certificate.transfer": ["certificate.mint"],
            "certificate.loan": ["certificate.mint"],
            "certificate.return": ["certificate.loan"],
            "slice.anchor": ["spine.create"],
            "proof.generate_inclusion": ["entry.append"],
            "session.dehydrate": ["spine.create"],
            "session.commit": ["spine.create"],
            "braid.commit": ["spine.create"],
            "anchor.publish": ["spine.create"],
            "anchor.publish_batch": ["spine.create"],
            "anchor.verify": ["anchor.publish"],
            "bonding.ledger.retrieve": ["bonding.ledger.store"],
            "bonding.ledger.list": ["bonding.ledger.store"],
            "trust.query": ["trust.anchor"],
            "trust.event_count": ["trust.anchor"],
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

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
#[path = "tests_socket.rs"]
mod tests_socket;

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
#[path = "tests_registration.rs"]
mod tests_registration;

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests use unwrap for conciseness"
)]
#[path = "tests_mcp.rs"]
mod tests_mcp;
