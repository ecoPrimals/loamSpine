// SPDX-License-Identifier: AGPL-3.0-or-later

//! Network configuration helpers with environment-first discovery.
//!
//! Each public function has a pure inner variant (prefixed `resolve_`) that
//! accepts pre-fetched values instead of reading `std::env::var` directly.
//! The outer wrapper reads env once and delegates.  Tests exercise the inner
//! functions for concurrency safety.

use std::borrow::Cow;
use std::env;
use tracing::{debug, warn};

use crate::constants::{DEFAULT_JSONRPC_PORT, DEFAULT_TARPC_PORT, OS_ASSIGNED_PORT};

// ──────────────────────────────────────────────────────────────────────────────
// Inner pure functions (no env reads)
// ──────────────────────────────────────────────────────────────────────────────

/// Resolve JSON-RPC port from optional environment values.
///
/// Priority: `loamspine_port` > `generic_port` > [`DEFAULT_JSONRPC_PORT`].
/// Invalid values fall through to the next tier.
#[must_use]
pub fn resolve_jsonrpc_port(loamspine_port: Option<&str>, generic_port: Option<&str>) -> u16 {
    if let Some(port_str) = loamspine_port {
        if let Ok(port) = port_str.parse::<u16>() {
            debug!("Using JSON-RPC port from LOAMSPINE_JSONRPC_PORT: {port}");
            return port;
        }
        warn!("Invalid LOAMSPINE_JSONRPC_PORT value: {port_str}, using default");
    }
    if let Some(port_str) = generic_port {
        if let Ok(port) = port_str.parse::<u16>() {
            debug!("Using JSON-RPC port from JSONRPC_PORT: {port}");
            return port;
        }
        warn!("Invalid JSONRPC_PORT value: {port_str}, using default");
    }
    debug!("Using default JSON-RPC port: {DEFAULT_JSONRPC_PORT}");
    DEFAULT_JSONRPC_PORT
}

/// Resolve tarpc port from optional environment values.
///
/// Priority: `loamspine_port` > `generic_port` > [`DEFAULT_TARPC_PORT`].
#[must_use]
pub fn resolve_tarpc_port(loamspine_port: Option<&str>, generic_port: Option<&str>) -> u16 {
    if let Some(port_str) = loamspine_port {
        if let Ok(port) = port_str.parse::<u16>() {
            debug!("Using tarpc port from LOAMSPINE_TARPC_PORT: {port}");
            return port;
        }
        warn!("Invalid LOAMSPINE_TARPC_PORT value: {port_str}, using default");
    }
    if let Some(port_str) = generic_port {
        if let Ok(port) = port_str.parse::<u16>() {
            debug!("Using tarpc port from TARPC_PORT: {port}");
            return port;
        }
        warn!("Invalid TARPC_PORT value: {port_str}, using default");
    }
    debug!("Using default tarpc port: {DEFAULT_TARPC_PORT}");
    DEFAULT_TARPC_PORT
}

/// Resolve bind address from optional environment values.
///
/// Priority: `loamspine_addr` > `generic_addr` > `"0.0.0.0"`.
#[must_use]
pub fn resolve_bind_address(
    loamspine_addr: Option<&str>,
    generic_addr: Option<&str>,
) -> Cow<'static, str> {
    if let Some(addr) = loamspine_addr {
        debug!("Using bind address from LOAMSPINE_BIND_ADDRESS: {addr}");
        return Cow::Owned(addr.to_owned());
    }
    if let Some(addr) = generic_addr {
        debug!("Using bind address from BIND_ADDRESS: {addr}");
        return Cow::Owned(addr.to_owned());
    }
    debug!(
        "Using default bind address: {}",
        crate::constants::BIND_ALL_IPV4
    );
    Cow::Borrowed(crate::constants::BIND_ALL_IPV4)
}

/// Resolve whether OS-assigned ports should be used.
///
/// Returns `true` when either input is a truthy string (`"true"`, `"1"`, `"yes"`).
#[must_use]
pub fn resolve_use_os_assigned_ports(use_os: Option<&str>, loamspine_os: Option<&str>) -> bool {
    fn is_truthy(v: &str) -> bool {
        let low = v.to_lowercase();
        low == "true" || v == "1" || low == "yes"
    }
    use_os.or(loamspine_os).is_some_and(is_truthy)
}

/// Resolve actual JSON-RPC port considering OS assignment preference.
#[must_use]
pub fn resolve_actual_jsonrpc_port(os_assigned: bool, port: u16) -> u16 {
    if os_assigned {
        debug!("Using OS-assigned port for JSON-RPC");
        OS_ASSIGNED_PORT
    } else {
        port
    }
}

/// Resolve actual tarpc port considering OS assignment preference.
#[must_use]
pub fn resolve_actual_tarpc_port(os_assigned: bool, port: u16) -> u16 {
    if os_assigned {
        debug!("Using OS-assigned port for tarpc");
        OS_ASSIGNED_PORT
    } else {
        port
    }
}

/// Resolve the biomeos socket base directory from an optional runtime dir.
///
/// 3-tier resolution:
/// 1. `runtime_dir` (from `$XDG_RUNTIME_DIR`)
/// 2. `/run/user/{uid}/biomeos/` (Linux, UID from `/proc/self/status`)
/// 3. `temp_dir/biomeos/`
#[must_use]
pub fn resolve_socket_base_dir_with(runtime_dir: Option<&str>) -> std::path::PathBuf {
    if let Some(rd) = runtime_dir {
        return std::path::PathBuf::from(format!("{rd}/biomeos"));
    }
    #[cfg(target_os = "linux")]
    if let Some(path) = linux_run_user_biomeos() {
        return path;
    }
    std::env::temp_dir().join("biomeos")
}

pub use super::env_resolution::{
    actual_jsonrpc_port, actual_tarpc_port, bind_address, jsonrpc_port,
    resolve_primal_socket_with_env, tarpc_port, use_os_assigned_ports,
};

/// Build a complete endpoint URL from parts.
///
/// # Examples
///
/// ```rust
/// use loam_spine_core::constants::network::build_endpoint;
///
/// let endpoint = build_endpoint("http", "localhost", 8080, None);
/// assert_eq!(endpoint, "http://localhost:8080");
///
/// let endpoint = build_endpoint("http", "localhost", 8080, Some("/api"));
/// assert_eq!(endpoint, "http://localhost:8080/api");
/// ```
#[must_use]
pub fn build_endpoint(scheme: &str, host: &str, port: u16, path: Option<&str>) -> String {
    path.map_or_else(
        || format!("{scheme}://{host}:{port}"),
        |p| format!("{scheme}://{host}:{port}{p}"),
    )
}

// ──────────────────────────────────────────────────────────────────────────────
// Generic Primal Discovery Helpers (sweetGrass V0.7.17 pattern)
// ──────────────────────────────────────────────────────────────────────────────

/// Build the environment variable name for a primal's socket path.
///
/// Follows: `{PRIMAL}_SOCKET` where `{PRIMAL}` is uppercased with hyphens
/// replaced by underscores.
///
/// # Examples
///
/// ```rust
/// use loam_spine_core::constants::network::socket_env_var;
///
/// assert_eq!(socket_env_var("rhizoCrypt"), "RHIZOCRYPT_SOCKET");
/// assert_eq!(socket_env_var("sweetGrass"), "SWEETGRASS_SOCKET");
/// assert_eq!(socket_env_var("loamSpine"), "LOAMSPINE_SOCKET");
/// ```
#[must_use]
pub fn socket_env_var(primal_name: &str) -> String {
    format!("{}_SOCKET", primal_name.to_uppercase().replace('-', "_"))
}

/// Build the environment variable name for a primal's address.
///
/// # Examples
///
/// ```rust
/// use loam_spine_core::constants::network::address_env_var;
///
/// assert_eq!(address_env_var("rhizoCrypt"), "RHIZOCRYPT_ADDRESS");
/// assert_eq!(address_env_var("songbird"), "SONGBIRD_ADDRESS");
/// ```
#[must_use]
pub fn address_env_var(primal_name: &str) -> String {
    format!("{}_ADDRESS", primal_name.to_uppercase().replace('-', "_"))
}

/// Resolve a primal's socket path using an optional env override, then
/// falling back to the standard biomeos layout.
///
/// Pure inner: accepts the override value directly.
#[must_use]
pub fn resolve_primal_socket_with(
    env_override: Option<&str>,
    primal: &str,
    family_id: &str,
) -> std::path::PathBuf {
    if let Some(path) = env_override {
        debug!("Using {primal} socket from env override: {path}");
        return std::path::PathBuf::from(path);
    }
    resolve_primal_socket(primal, family_id)
}

// ──────────────────────────────────────────────────────────────────────────────
// Protocol Escalation (UNIVERSAL_IPC_STANDARD_V3)
// ──────────────────────────────────────────────────────────────────────────────

/// IPC protocol level resolved at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcProtocol {
    /// JSON-RPC 2.0 over Unix socket (primary, always available).
    JsonRpc,
    /// tarpc/bincode over Unix socket (high-performance primal-to-primal).
    Tarpc,
}

/// Resolve the IPC socket path for a given primal.
///
/// Socket naming: `{primal}-{family_id}.sock`
#[must_use]
pub fn resolve_primal_socket(primal: &str, family_id: &str) -> std::path::PathBuf {
    let base = resolve_socket_base_dir();
    resolve_primal_socket_from(&base, primal, family_id)
}

/// Resolve the IPC socket from a known base directory (pure, no env reads).
#[must_use]
pub fn resolve_primal_socket_from(
    base_dir: &std::path::Path,
    primal: &str,
    family_id: &str,
) -> std::path::PathBuf {
    base_dir.join(format!("{primal}-{family_id}.sock"))
}

/// Resolve the tarpc socket path for a given primal.
#[must_use]
pub fn resolve_primal_tarpc_socket(primal: &str, family_id: &str) -> std::path::PathBuf {
    let base = resolve_socket_base_dir();
    resolve_primal_tarpc_socket_from(&base, primal, family_id)
}

/// Resolve the tarpc socket from a known base directory (pure, no env reads).
#[must_use]
pub fn resolve_primal_tarpc_socket_from(
    base_dir: &std::path::Path,
    primal: &str,
    family_id: &str,
) -> std::path::PathBuf {
    base_dir.join(format!("{primal}-{family_id}.tarpc.sock"))
}

/// Protocol escalation: prefer tarpc when `.tarpc.sock` exists,
/// fall back to JSON-RPC `.sock`.
#[must_use]
pub fn negotiate_protocol(primal: &str, family_id: &str) -> (IpcProtocol, std::path::PathBuf) {
    let base = resolve_socket_base_dir();
    negotiate_protocol_from(&base, primal, family_id)
}

/// Protocol escalation from a known base directory (pure, no env reads).
#[must_use]
pub fn negotiate_protocol_from(
    base_dir: &std::path::Path,
    primal: &str,
    family_id: &str,
) -> (IpcProtocol, std::path::PathBuf) {
    let tarpc_sock = resolve_primal_tarpc_socket_from(base_dir, primal, family_id);
    if tarpc_sock.exists() {
        debug!(
            "Protocol escalation: tarpc socket found at {}",
            tarpc_sock.display()
        );
        return (IpcProtocol::Tarpc, tarpc_sock);
    }
    let jsonrpc_sock = resolve_primal_socket_from(base_dir, primal, family_id);
    debug!(
        "Using JSON-RPC socket at {} (tarpc not available)",
        jsonrpc_sock.display()
    );
    (IpcProtocol::JsonRpc, jsonrpc_sock)
}

fn resolve_socket_base_dir() -> std::path::PathBuf {
    resolve_socket_base_dir_with(env::var("XDG_RUNTIME_DIR").ok().as_deref())
}

/// Detect the current user's runtime directory via `/proc/self/status`.
///
/// Returns `Some(path)` only if the directory actually exists on disk.
#[cfg(target_os = "linux")]
pub(crate) fn linux_run_user_biomeos() -> Option<std::path::PathBuf> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let uid_str = status
        .lines()
        .find(|l| l.starts_with("Uid:"))
        .and_then(|l| l.split_whitespace().nth(1))?;
    let path = std::path::PathBuf::from(format!("/run/user/{uid_str}"));
    path.exists()
        .then(|| path.join(crate::primal_names::BIOMEOS_SOCKET_DIR))
}

#[cfg(test)]
#[path = "network_tests.rs"]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests;
