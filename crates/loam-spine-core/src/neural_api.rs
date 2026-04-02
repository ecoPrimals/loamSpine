// SPDX-License-Identifier: AGPL-3.0-or-later

//! NeuralAPI registration for biomeOS orchestration.
//!
//! This module handles LoamSpine's registration with biomeOS's NeuralAPI,
//! advertising capabilities and socket endpoints so the orchestration layer
//! can route capability requests to LoamSpine.

use std::path::PathBuf;
use std::sync::OnceLock;

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

/// Resolve the socket path from explicit config values (pure, no env reads).
///
/// Resolution order:
/// 1. `socket_override` (from `LOAMSPINE_SOCKET`)
/// 2. `runtime_dir/biomeos/loamspine-{family_id}.sock`
/// 3. `/run/user/{uid}/biomeos/...` (Linux)
/// 4. `temp_dir/biomeos/...`
#[must_use]
pub fn resolve_socket_path_with(
    socket_override: Option<&str>,
    family_id: Option<&str>,
    runtime_dir: Option<&str>,
) -> PathBuf {
    if let Some(s) = socket_override {
        return PathBuf::from(s);
    }
    let sock_name = match family_id {
        Some(fid) if !fid.is_empty() => {
            format!("{}-{fid}.sock", crate::primal_names::SELF_ID)
        }
        _ => format!("{}.sock", crate::primal_names::SELF_ID),
    };

    if let Some(rd) = runtime_dir {
        return PathBuf::from(rd)
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

/// Resolve the socket path for LoamSpine's IPC endpoint (reads env).
#[must_use]
pub fn resolve_socket_path() -> PathBuf {
    resolve_socket_path_with(
        std::env::var("LOAMSPINE_SOCKET").ok().as_deref(),
        std::env::var("BIOMEOS_FAMILY_ID").ok().as_deref(),
        std::env::var("XDG_RUNTIME_DIR").ok().as_deref(),
    )
}

/// Resolve the NeuralAPI socket from explicit config values (pure, no env reads).
#[must_use]
pub fn resolve_neural_api_socket_with(
    neural_socket: Option<&str>,
    runtime_dir: Option<&str>,
    family_id: Option<&str>,
) -> Option<PathBuf> {
    if let Some(s) = neural_socket {
        return Some(PathBuf::from(s));
    }
    let rd = runtime_dir?;
    let fid = family_id.unwrap_or("default");
    Some(PathBuf::from(format!("{rd}/biomeos/neural-api-{fid}.sock")))
}

/// Resolve the NeuralAPI socket path for connecting to biomeOS (reads env).
fn resolve_neural_api_socket() -> Option<PathBuf> {
    resolve_neural_api_socket_with(
        std::env::var("BIOMEOS_NEURAL_API_SOCKET").ok().as_deref(),
        std::env::var("XDG_RUNTIME_DIR").ok().as_deref(),
        std::env::var("BIOMEOS_FAMILY_ID").ok().as_deref(),
    )
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
    let Some(socket_path) = resolve_neural_api_socket() else {
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

/// Cached MCP tools list — initialized once, reused for all subsequent calls.
static MCP_TOOLS_CACHE: OnceLock<serde_json::Value> = OnceLock::new();

/// Return the capability list as a JSON-RPC response payload.
/// Implements the `capability.list` semantic method.
/// Aligns with ludoSpring's enhanced format: domain, method, dependencies, cost tier.
///
/// Uses `OnceLock` to initialize the JSON value once and return a reference thereafter,
/// avoiding re-building the `serde_json::Value` tree on every call.
#[must_use]
pub fn capability_list() -> &'static serde_json::Value {
    CAPABILITY_LIST_CACHE.get_or_init(capability_list_inner)
}

/// Build the capability list JSON (called once by `OnceLock`).
fn capability_list_inner() -> serde_json::Value {
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
    serde_json::to_string_pretty(capability_list()).unwrap_or_default()
}

// ============================================================================
// MCP (Model Context Protocol) tools — for AI agent visibility
// ============================================================================

/// Return MCP `tools/list` response payload.
///
/// Each tool maps to a JSON-RPC method with an `inputSchema` describing
/// the expected `params`. AI agents (e.g. Squirrel) call `tools/list` to
/// discover what operations are available, then invoke them via `tools/call`.
///
/// Absorbed from Squirrel/biomeOS MCP bridge pattern — primals advertise
/// tool schemas so agents can construct valid calls without hardcoded
/// knowledge of any specific primal.
///
/// Uses `OnceLock` to initialize the JSON value once and return a reference thereafter.
#[must_use]
pub fn mcp_tools_list() -> &'static serde_json::Value {
    MCP_TOOLS_CACHE.get_or_init(mcp_tools_list_inner)
}

/// Build the MCP tools JSON (called once by `OnceLock`).
#[expect(
    clippy::too_many_lines,
    reason = "declarative MCP tool schema definitions"
)]
fn mcp_tools_list_inner() -> serde_json::Value {
    serde_json::json!({
        "tools": [
            mcp_tool("spine_create", "Create a new sovereign spine (append-only ledger)", &serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Human-readable spine name" },
                    "owner": { "type": "string", "description": "DID of the spine owner" }
                },
                "required": ["name", "owner"]
            })),
            mcp_tool("spine_get", "Get a spine by ID", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Spine ID" }
                },
                "required": ["spine_id"]
            })),
            mcp_tool("spine_seal", "Seal a spine (make permanently read-only)", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Spine ID to seal" },
                    "sealer": { "type": "string", "description": "DID of the sealer" }
                },
                "required": ["spine_id", "sealer"]
            })),
            mcp_tool("entry_append", "Append an entry to a spine", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Target spine ID" },
                    "domain": { "type": "string", "description": "Entry domain (e.g. 'commit', 'certificate')" },
                    "payload": { "type": "string", "description": "Entry payload (base64 or JSON string)" }
                },
                "required": ["spine_id", "domain", "payload"]
            })),
            mcp_tool("entry_get", "Get an entry by spine ID and index", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Spine ID" },
                    "index": { "type": "integer", "description": "Entry index" }
                },
                "required": ["spine_id", "index"]
            })),
            mcp_tool("entry_get_tip", "Get the latest (tip) entry of a spine", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Spine ID" }
                },
                "required": ["spine_id"]
            })),
            mcp_tool("certificate_mint", "Mint a new certificate (memory-bound object)", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Spine to mint on" },
                    "owner": { "type": "string", "description": "Owner DID" },
                    "cert_type": { "type": "string", "description": "Certificate type" },
                    "name": { "type": "string", "description": "Certificate name" }
                },
                "required": ["spine_id", "owner", "cert_type"]
            })),
            mcp_tool("certificate_get", "Get certificate by ID", &serde_json::json!({
                "type": "object",
                "properties": {
                    "certificate_id": { "type": "string", "description": "Certificate ID" }
                },
                "required": ["certificate_id"]
            })),
            mcp_tool("certificate_transfer", "Transfer certificate ownership", &serde_json::json!({
                "type": "object",
                "properties": {
                    "certificate_id": { "type": "string", "description": "Certificate ID" },
                    "from": { "type": "string", "description": "Current owner DID" },
                    "to": { "type": "string", "description": "New owner DID" }
                },
                "required": ["certificate_id", "from", "to"]
            })),
            mcp_tool("certificate_loan", "Loan a certificate to another identity", &serde_json::json!({
                "type": "object",
                "properties": {
                    "certificate_id": { "type": "string", "description": "Certificate ID" },
                    "borrower": { "type": "string", "description": "Borrower DID" },
                    "duration_secs": { "type": "integer", "description": "Loan duration in seconds" }
                },
                "required": ["certificate_id", "borrower"]
            })),
            mcp_tool("certificate_return", "Return a loaned certificate", &serde_json::json!({
                "type": "object",
                "properties": {
                    "certificate_id": { "type": "string", "description": "Certificate ID" },
                    "returner": { "type": "string", "description": "Borrower DID returning the certificate" }
                },
                "required": ["certificate_id", "returner"]
            })),
            mcp_tool("certificate_verify", "Verify a certificate's chain of custody", &serde_json::json!({
                "type": "object",
                "properties": {
                    "certificate_id": { "type": "string", "description": "Certificate ID to verify" }
                },
                "required": ["certificate_id"]
            })),
            mcp_tool("certificate_lifecycle", "Get full lifecycle history of a certificate", &serde_json::json!({
                "type": "object",
                "properties": {
                    "certificate_id": { "type": "string", "description": "Certificate ID" }
                },
                "required": ["certificate_id"]
            })),
            mcp_tool("slice_anchor", "Anchor a slice on a waypoint spine", &serde_json::json!({
                "type": "object",
                "properties": {
                    "waypoint_spine_id": { "type": "integer", "description": "Waypoint spine ID" },
                    "slice_id": { "type": "string", "description": "Slice ID" },
                    "origin_spine_id": { "type": "integer", "description": "Origin spine ID" },
                    "committer": { "type": "string", "description": "Committer DID" }
                },
                "required": ["waypoint_spine_id", "slice_id", "origin_spine_id", "committer"]
            })),
            mcp_tool("slice_checkout", "Checkout a slice from a waypoint spine", &serde_json::json!({
                "type": "object",
                "properties": {
                    "waypoint_spine_id": { "type": "integer", "description": "Waypoint spine ID" },
                    "slice_id": { "type": "string", "description": "Slice ID" },
                    "requester": { "type": "string", "description": "Requester DID" }
                },
                "required": ["waypoint_spine_id", "slice_id", "requester"]
            })),
            mcp_tool("slice_record_operation", "Record an operation on a checked-out slice", &serde_json::json!({
                "type": "object",
                "properties": {
                    "waypoint_spine_id": { "type": "integer", "description": "Waypoint spine ID" },
                    "slice_id": { "type": "string", "description": "Slice ID" },
                    "operation": { "type": "string", "description": "Operation payload (JSON)" }
                },
                "required": ["waypoint_spine_id", "slice_id", "operation"]
            })),
            mcp_tool("slice_depart", "Depart (close) a slice and finalize waypoint entry", &serde_json::json!({
                "type": "object",
                "properties": {
                    "waypoint_spine_id": { "type": "integer", "description": "Waypoint spine ID" },
                    "slice_id": { "type": "string", "description": "Slice ID" }
                },
                "required": ["waypoint_spine_id", "slice_id"]
            })),
            mcp_tool("proof_generate_inclusion", "Generate an inclusion proof for an entry", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Spine ID" },
                    "index": { "type": "integer", "description": "Entry index to prove" }
                },
                "required": ["spine_id", "index"]
            })),
            mcp_tool("proof_verify_inclusion", "Verify an inclusion proof", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Spine ID" },
                    "entry_hash": { "type": "string", "description": "Entry hash (hex)" },
                    "proof": { "type": "object", "description": "Inclusion proof object" }
                },
                "required": ["spine_id", "entry_hash", "proof"]
            })),
            mcp_tool("session_commit", "Commit an ephemeral session to permanent storage", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Target spine ID" },
                    "session_id": { "type": "string", "description": "Session UUID" },
                    "session_hash": { "type": "string", "description": "Session DAG root hash (hex)" },
                    "vertex_count": { "type": "integer", "description": "Number of vertices" },
                    "committer": { "type": "string", "description": "Committer DID" }
                },
                "required": ["spine_id", "session_id", "session_hash", "committer"]
            })),
            mcp_tool("braid_commit", "Commit a semantic attribution braid", &serde_json::json!({
                "type": "object",
                "properties": {
                    "spine_id": { "type": "integer", "description": "Target spine ID" },
                    "braid_id": { "type": "string", "description": "Braid UUID" },
                    "braid_hash": { "type": "string", "description": "Braid hash (hex)" },
                    "subjects": { "type": "array", "items": { "type": "string" }, "description": "Subject DIDs" },
                    "committer": { "type": "string", "description": "Committer DID" }
                },
                "required": ["spine_id", "braid_id", "braid_hash", "committer"]
            })),
            mcp_tool("health_check", "Check LoamSpine health status", &serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            })),
            mcp_tool("capability_list", "List all LoamSpine capabilities and methods", &serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            })),
        ]
    })
}

fn mcp_tool(name: &str, description: &str, input_schema: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema,
    })
}

/// Handle an MCP `tools/call` by mapping the tool name to a JSON-RPC method.
///
/// Returns `(method, params)` suitable for dispatching through the JSON-RPC
/// handler. Returns `None` if the tool name is unrecognized.
#[must_use]
pub fn mcp_tool_to_rpc(
    tool_name: &str,
    arguments: serde_json::Value,
) -> Option<(&'static str, serde_json::Value)> {
    let method = match tool_name {
        "spine_create" => "spine.create",
        "spine_get" => "spine.get",
        "spine_seal" => "spine.seal",
        "entry_append" => "entry.append",
        "entry_get" => "entry.get",
        "entry_get_tip" => "entry.get_tip",
        "certificate_mint" => "certificate.mint",
        "certificate_get" => "certificate.get",
        "certificate_transfer" => "certificate.transfer",
        "certificate_loan" => "certificate.loan",
        "certificate_return" => "certificate.return",
        "certificate_verify" => "certificate.verify",
        "certificate_lifecycle" => "certificate.lifecycle",
        "slice_anchor" => "slice.anchor",
        "slice_checkout" => "slice.checkout",
        "slice_record_operation" => "slice.record_operation",
        "slice_depart" => "slice.depart",
        "proof_generate_inclusion" => "proof.generate_inclusion",
        "proof_verify_inclusion" => "proof.verify_inclusion",
        "session_commit" => "session.commit",
        "braid_commit" => "braid.commit",
        "health_check" => "health.check",
        "capability_list" => "capability.list",
        _ => return None,
    };
    Some((method, arguments))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests use unwrap for conciseness"
)]
#[path = "neural_api_tests.rs"]
mod tests;
