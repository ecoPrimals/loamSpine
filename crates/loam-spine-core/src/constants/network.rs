// SPDX-License-Identifier: AGPL-3.0-or-later

//! Network configuration helpers with environment-first discovery
//!
//! This module provides functions to resolve network configuration at runtime,
//! following the "infant discovery" pattern where all configuration is
//! discovered from the environment rather than hardcoded.

use std::borrow::Cow;
use std::env;
use tracing::{debug, warn};

use crate::constants::{DEFAULT_JSONRPC_PORT, DEFAULT_TARPC_PORT, OS_ASSIGNED_PORT};

/// Get JSON-RPC port from environment or default
///
/// Priority order:
/// 1. `LOAMSPINE_JSONRPC_PORT` environment variable
/// 2. `JSONRPC_PORT` environment variable (generic)
/// 3. Default development port (8080)
///
/// # Examples
///
/// ```rust
/// use loam_spine_core::constants::network::jsonrpc_port;
///
/// // Set via environment
/// unsafe { std::env::set_var("LOAMSPINE_JSONRPC_PORT", "8888"); }
/// assert_eq!(jsonrpc_port(), 8888);
///
/// // Falls back to default
/// unsafe { std::env::remove_var("LOAMSPINE_JSONRPC_PORT"); }
/// unsafe { std::env::remove_var("JSONRPC_PORT"); }
/// assert_eq!(jsonrpc_port(), 8080);
/// ```
pub fn jsonrpc_port() -> u16 {
    // Try LoamSpine-specific env var first
    if let Ok(port_str) = env::var("LOAMSPINE_JSONRPC_PORT") {
        if let Ok(port) = port_str.parse::<u16>() {
            debug!("Using JSON-RPC port from LOAMSPINE_JSONRPC_PORT: {}", port);
            return port;
        }
        warn!(
            "Invalid LOAMSPINE_JSONRPC_PORT value: {}, using default",
            port_str
        );
    }

    // Try generic env var
    if let Ok(port_str) = env::var("JSONRPC_PORT") {
        if let Ok(port) = port_str.parse::<u16>() {
            debug!("Using JSON-RPC port from JSONRPC_PORT: {}", port);
            return port;
        }
        warn!("Invalid JSONRPC_PORT value: {}, using default", port_str);
    }

    // Development default
    debug!("Using default JSON-RPC port: {}", DEFAULT_JSONRPC_PORT);
    DEFAULT_JSONRPC_PORT
}

/// Get tarpc port from environment or default
///
/// Priority order:
/// 1. `LOAMSPINE_TARPC_PORT` environment variable
/// 2. `TARPC_PORT` environment variable (generic)
/// 3. Default development port (9001)
///
/// # Examples
///
/// ```rust
/// use loam_spine_core::constants::network::tarpc_port;
///
/// // Set via environment
/// unsafe { std::env::set_var("LOAMSPINE_TARPC_PORT", "9999"); }
/// assert_eq!(tarpc_port(), 9999);
///
/// // Falls back to default
/// unsafe { std::env::remove_var("LOAMSPINE_TARPC_PORT"); }
/// unsafe { std::env::remove_var("TARPC_PORT"); }
/// assert_eq!(tarpc_port(), 9001);
/// ```
pub fn tarpc_port() -> u16 {
    // Try LoamSpine-specific env var first
    if let Ok(port_str) = env::var("LOAMSPINE_TARPC_PORT") {
        if let Ok(port) = port_str.parse::<u16>() {
            debug!("Using tarpc port from LOAMSPINE_TARPC_PORT: {}", port);
            return port;
        }
        warn!(
            "Invalid LOAMSPINE_TARPC_PORT value: {}, using default",
            port_str
        );
    }

    // Try generic env var
    if let Ok(port_str) = env::var("TARPC_PORT") {
        if let Ok(port) = port_str.parse::<u16>() {
            debug!("Using tarpc port from TARPC_PORT: {}", port);
            return port;
        }
        warn!("Invalid TARPC_PORT value: {}, using default", port_str);
    }

    // Development default
    debug!("Using default tarpc port: {}", DEFAULT_TARPC_PORT);
    DEFAULT_TARPC_PORT
}

/// Get bind address from environment or default
///
/// Priority order:
/// 1. `LOAMSPINE_BIND_ADDRESS` environment variable
/// 2. `BIND_ADDRESS` environment variable (generic)
/// 3. Default: "0.0.0.0" (all interfaces)
///
/// # Examples
///
/// ```rust
/// use loam_spine_core::constants::network::bind_address;
///
/// // Set specific interface
/// unsafe { std::env::set_var("LOAMSPINE_BIND_ADDRESS", "127.0.0.1"); }
/// assert_eq!(bind_address(), "127.0.0.1");
///
/// // Default to all interfaces
/// unsafe { std::env::remove_var("LOAMSPINE_BIND_ADDRESS"); }
/// unsafe { std::env::remove_var("BIND_ADDRESS"); }
/// assert_eq!(bind_address(), "0.0.0.0");
/// ```
pub fn bind_address() -> Cow<'static, str> {
    // Try LoamSpine-specific env var first
    if let Ok(addr) = env::var("LOAMSPINE_BIND_ADDRESS") {
        debug!("Using bind address from LOAMSPINE_BIND_ADDRESS: {}", addr);
        return Cow::Owned(addr);
    }

    // Try generic env var
    if let Ok(addr) = env::var("BIND_ADDRESS") {
        debug!("Using bind address from BIND_ADDRESS: {}", addr);
        return Cow::Owned(addr);
    }

    // Default to all interfaces (zero-copy: borrow static constant)
    debug!(
        "Using default bind address: {}",
        crate::constants::BIND_ALL_IPV4
    );
    Cow::Borrowed(crate::constants::BIND_ALL_IPV4)
}

/// Check if we should use OS-assigned ports (recommended for production)
///
/// Returns `true` if:
/// - `USE_OS_ASSIGNED_PORTS=true` environment variable is set, or
/// - `LOAMSPINE_OS_PORTS=true` environment variable is set
///
/// When true, services should bind to port 0 and let the OS assign ports.
///
/// # Examples
///
/// ```rust
/// use loam_spine_core::constants::network::{use_os_assigned_ports, tarpc_port};
///
/// unsafe { std::env::set_var("USE_OS_ASSIGNED_PORTS", "true"); }
/// assert!(use_os_assigned_ports());
///
/// // In application code
/// let port = if use_os_assigned_ports() {
///     0  // OS assigns port
/// } else {
///     tarpc_port()  // Use configured or default port
/// };
/// ```
#[must_use]
pub fn use_os_assigned_ports() -> bool {
    env::var("USE_OS_ASSIGNED_PORTS")
        .or_else(|_| env::var("LOAMSPINE_OS_PORTS"))
        .map(|v| v.to_lowercase() == "true" || v == "1" || v.to_lowercase() == "yes")
        .unwrap_or(false)
}

/// Get the actual port to bind to, considering OS assignment preference
///
/// This is the recommended function to use when starting services.
///
/// # Examples
///
/// ```rust
/// use loam_spine_core::constants::network::actual_jsonrpc_port;
///
/// // OS assignment enabled
/// unsafe { std::env::set_var("USE_OS_ASSIGNED_PORTS", "true"); }
/// assert_eq!(actual_jsonrpc_port(), 0);
///
/// // OS assignment disabled, uses configured port
/// unsafe { std::env::remove_var("USE_OS_ASSIGNED_PORTS"); }
/// unsafe { std::env::set_var("LOAMSPINE_JSONRPC_PORT", "8888"); }
/// assert_eq!(actual_jsonrpc_port(), 8888);
/// ```
pub fn actual_jsonrpc_port() -> u16 {
    if use_os_assigned_ports() {
        debug!("Using OS-assigned port for JSON-RPC");
        OS_ASSIGNED_PORT
    } else {
        jsonrpc_port()
    }
}

/// Get the actual tarpc port to bind to, considering OS assignment preference
///
/// # Examples
///
/// ```rust
/// use loam_spine_core::constants::network::actual_tarpc_port;
///
/// // OS assignment enabled
/// unsafe { std::env::set_var("USE_OS_ASSIGNED_PORTS", "true"); }
/// assert_eq!(actual_tarpc_port(), 0);
///
/// // OS assignment disabled, uses configured port
/// unsafe { std::env::remove_var("USE_OS_ASSIGNED_PORTS"); }
/// unsafe { std::env::set_var("LOAMSPINE_TARPC_PORT", "9999"); }
/// assert_eq!(actual_tarpc_port(), 9999);
/// ```
pub fn actual_tarpc_port() -> u16 {
    if use_os_assigned_ports() {
        debug!("Using OS-assigned port for tarpc");
        OS_ASSIGNED_PORT
    } else {
        tarpc_port()
    }
}

/// Build a complete endpoint URL from parts
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

/// Resolve the IPC socket path for a given primal, following the wateringHole
/// `UNIVERSAL_IPC_STANDARD_V3` and `PRIMAL_IPC_PROTOCOL` conventions.
///
/// Socket naming: `{primal}-{family_id}.sock` for JSON-RPC,
/// `{primal}-{family_id}.tarpc.sock` for tarpc.
///
/// 5-tier resolution order:
/// 1. `$XDG_RUNTIME_DIR/biomeos/`
/// 2. `/run/user/{uid}/biomeos/` (Linux, UID from `/proc/self/status`)
/// 3. `{temp_dir}/biomeos/`
#[must_use]
pub fn resolve_primal_socket(primal: &str, family_id: &str) -> std::path::PathBuf {
    let base = resolve_socket_base_dir();
    base.join(format!("{primal}-{family_id}.sock"))
}

/// Resolve the tarpc socket path for a given primal.
#[must_use]
pub fn resolve_primal_tarpc_socket(primal: &str, family_id: &str) -> std::path::PathBuf {
    let base = resolve_socket_base_dir();
    base.join(format!("{primal}-{family_id}.tarpc.sock"))
}

/// Protocol escalation: prefer tarpc when `.tarpc.sock` exists,
/// fall back to JSON-RPC `.sock`.
///
/// This implements the wateringHole `PRIMAL_IPC_PROTOCOL` standard:
/// JSON-RPC is always available; tarpc is optional and higher-performance.
#[must_use]
pub fn negotiate_protocol(primal: &str, family_id: &str) -> (IpcProtocol, std::path::PathBuf) {
    let tarpc_sock = resolve_primal_tarpc_socket(primal, family_id);
    if tarpc_sock.exists() {
        debug!(
            "Protocol escalation: tarpc socket found at {}",
            tarpc_sock.display()
        );
        return (IpcProtocol::Tarpc, tarpc_sock);
    }

    let jsonrpc_sock = resolve_primal_socket(primal, family_id);
    debug!(
        "Using JSON-RPC socket at {} (tarpc not available)",
        jsonrpc_sock.display()
    );
    (IpcProtocol::JsonRpc, jsonrpc_sock)
}

fn resolve_socket_base_dir() -> std::path::PathBuf {
    // Tier 1: XDG_RUNTIME_DIR (standard, set by pam_systemd)
    if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
        return std::path::PathBuf::from(format!("{runtime_dir}/biomeos"));
    }

    // Tier 2: /run/user/{uid}/biomeos/ (Linux fallback when XDG unset)
    #[cfg(target_os = "linux")]
    if let Some(path) = linux_run_user_biomeos() {
        return path;
    }

    // Tier 3: temp_dir/biomeos/
    std::env::temp_dir().join("biomeos")
}

/// Detect the current user's runtime directory via `/proc/self/status`.
///
/// Returns `Some(path)` only if the directory actually exists on disk,
/// avoiding phantom paths in containers or minimal environments.
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serial_test::serial;

    const CLEAN: [(&str, Option<&str>); 9] = [
        ("LOAMSPINE_JSONRPC_PORT", None),
        ("JSONRPC_PORT", None),
        ("LOAMSPINE_TARPC_PORT", None),
        ("TARPC_PORT", None),
        ("USE_OS_ASSIGNED_PORTS", None),
        ("LOAMSPINE_OS_PORTS", None),
        ("LOAMSPINE_USE_OS_ASSIGNED_PORTS", None),
        ("LOAMSPINE_BIND_ADDRESS", None),
        ("BIND_ADDRESS", None),
    ];

    #[test]
    #[serial]
    fn test_jsonrpc_port_from_env() {
        let mut vars = CLEAN.to_vec();
        vars.push(("LOAMSPINE_JSONRPC_PORT", Some("8888")));
        temp_env::with_vars(vars, || assert_eq!(jsonrpc_port(), 8888));
    }

    #[test]
    #[serial]
    fn test_jsonrpc_port_default() {
        temp_env::with_vars(CLEAN, || assert_eq!(jsonrpc_port(), DEFAULT_JSONRPC_PORT));
    }

    #[test]
    #[serial]
    fn test_tarpc_port_from_env() {
        let mut vars = CLEAN.to_vec();
        vars.push(("LOAMSPINE_TARPC_PORT", Some("9999")));
        temp_env::with_vars(vars, || assert_eq!(tarpc_port(), 9999));
    }

    #[test]
    #[serial]
    fn test_os_assigned_ports() {
        let mut on = CLEAN.to_vec();
        on.push(("USE_OS_ASSIGNED_PORTS", Some("1")));
        temp_env::with_vars(on, || assert!(use_os_assigned_ports()));

        let mut off = CLEAN.to_vec();
        off.push(("USE_OS_ASSIGNED_PORTS", Some("0")));
        temp_env::with_vars(off, || assert!(!use_os_assigned_ports()));
    }

    #[test]
    #[serial]
    fn test_actual_ports_with_os_assignment() {
        let mut vars = CLEAN.to_vec();
        vars.push(("USE_OS_ASSIGNED_PORTS", Some("1")));
        temp_env::with_vars(vars, || {
            assert!(use_os_assigned_ports());
            assert_eq!(actual_jsonrpc_port(), 0);
            assert_eq!(actual_tarpc_port(), 0);
        });
    }

    #[test]
    fn test_build_endpoint() {
        assert_eq!(
            build_endpoint("http", "localhost", 8080, None),
            "http://localhost:8080"
        );
        assert_eq!(
            build_endpoint("http", "localhost", 8080, Some("/api")),
            "http://localhost:8080/api"
        );
    }

    #[test]
    #[serial]
    fn test_jsonrpc_port_invalid_loamspine_falls_back_to_generic() {
        let mut vars = CLEAN.to_vec();
        vars.push(("LOAMSPINE_JSONRPC_PORT", Some("invalid")));
        vars.push(("JSONRPC_PORT", Some("7777")));
        temp_env::with_vars(vars, || assert_eq!(jsonrpc_port(), 7777));
    }

    #[test]
    #[serial]
    fn test_jsonrpc_port_invalid_both_falls_back_to_default() {
        let mut vars = CLEAN.to_vec();
        vars.push(("LOAMSPINE_JSONRPC_PORT", Some("not-a-number")));
        vars.push(("JSONRPC_PORT", Some("also-invalid")));
        temp_env::with_vars(vars, || assert_eq!(jsonrpc_port(), DEFAULT_JSONRPC_PORT));
    }

    #[test]
    #[serial]
    fn test_jsonrpc_port_generic_env_var() {
        let mut vars = CLEAN.to_vec();
        vars.push(("JSONRPC_PORT", Some("5555")));
        temp_env::with_vars(vars, || assert_eq!(jsonrpc_port(), 5555));
    }

    #[test]
    #[serial]
    fn test_tarpc_port_invalid_loamspine_falls_back_to_generic() {
        let mut vars = CLEAN.to_vec();
        vars.push(("LOAMSPINE_TARPC_PORT", Some("invalid")));
        vars.push(("TARPC_PORT", Some("8888")));
        temp_env::with_vars(vars, || assert_eq!(tarpc_port(), 8888));
    }

    #[test]
    #[serial]
    fn test_tarpc_port_generic_env_var() {
        let mut vars = CLEAN.to_vec();
        vars.push(("TARPC_PORT", Some("7777")));
        temp_env::with_vars(vars, || assert_eq!(tarpc_port(), 7777));
    }

    #[test]
    #[serial]
    fn test_tarpc_port_invalid_both_falls_back_to_default() {
        let mut vars = CLEAN.to_vec();
        vars.push(("LOAMSPINE_TARPC_PORT", Some("bad")));
        vars.push(("TARPC_PORT", Some("worse")));
        temp_env::with_vars(vars, || assert_eq!(tarpc_port(), DEFAULT_TARPC_PORT));
    }

    #[test]
    #[serial]
    fn test_use_os_assigned_ports_yes() {
        let mut vars = CLEAN.to_vec();
        vars.push(("USE_OS_ASSIGNED_PORTS", Some("yes")));
        temp_env::with_vars(vars, || assert!(use_os_assigned_ports()));
    }

    #[test]
    #[serial]
    fn test_use_os_assigned_ports_true() {
        let mut vars = CLEAN.to_vec();
        vars.push(("USE_OS_ASSIGNED_PORTS", Some("true")));
        temp_env::with_vars(vars, || assert!(use_os_assigned_ports()));
    }

    #[test]
    #[serial]
    fn test_use_os_assigned_ports_loamspine_os_ports() {
        let mut vars = CLEAN.to_vec();
        vars.push(("LOAMSPINE_OS_PORTS", Some("true")));
        temp_env::with_vars(vars, || assert!(use_os_assigned_ports()));
    }

    #[test]
    #[serial]
    fn test_bind_address_loamspine_specific() {
        let mut vars = CLEAN.to_vec();
        vars.push(("LOAMSPINE_BIND_ADDRESS", Some("127.0.0.1")));
        temp_env::with_vars(vars, || assert_eq!(bind_address(), "127.0.0.1"));
    }

    #[test]
    #[serial]
    fn test_bind_address_generic() {
        let mut vars = CLEAN.to_vec();
        vars.push(("BIND_ADDRESS", Some("192.0.2.1")));
        temp_env::with_vars(vars, || assert_eq!(bind_address(), "192.0.2.1"));
    }

    #[test]
    #[serial]
    fn test_bind_address_default() {
        temp_env::with_vars(CLEAN, || {
            assert_eq!(bind_address(), crate::constants::BIND_ALL_IPV4);
        });
    }

    #[test]
    #[serial]
    fn test_actual_ports_without_os_assignment() {
        let mut vars = CLEAN.to_vec();
        vars.push(("LOAMSPINE_JSONRPC_PORT", Some("3333")));
        vars.push(("LOAMSPINE_TARPC_PORT", Some("4444")));
        temp_env::with_vars(vars, || {
            assert_eq!(actual_jsonrpc_port(), 3333);
            assert_eq!(actual_tarpc_port(), 4444);
        });
    }

    // ──────────────────────────────────────────────────────────────────────
    // Protocol escalation tests
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    #[serial]
    fn test_resolve_primal_socket_path() {
        temp_env::with_vars([("XDG_RUNTIME_DIR", None::<&str>)], || {
            let path = resolve_primal_socket("loamspine", "default");
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
    fn test_resolve_primal_tarpc_socket_path() {
        temp_env::with_vars([("XDG_RUNTIME_DIR", None::<&str>)], || {
            let path = resolve_primal_tarpc_socket("loamspine", "default");
            assert!(
                path.to_string_lossy()
                    .ends_with("biomeos/loamspine-default.tarpc.sock"),
                "got: {}",
                path.display()
            );
        });
    }

    #[test]
    #[serial]
    fn test_resolve_primal_socket_with_xdg() {
        temp_env::with_vars([("XDG_RUNTIME_DIR", Some("/run/user/1000"))], || {
            let path = resolve_primal_socket("rhizocrypt", "myfamily");
            assert_eq!(
                path.to_string_lossy(),
                "/run/user/1000/biomeos/rhizocrypt-myfamily.sock"
            );
        });
    }

    #[test]
    #[serial]
    fn test_negotiate_protocol_prefers_tarpc_when_available() {
        let tmp = tempfile::tempdir().unwrap();
        let xdg = tmp.path().to_str().unwrap().to_string();
        temp_env::with_vars([("XDG_RUNTIME_DIR", Some(xdg.as_str()))], || {
            let biomeos_dir = tmp.path().join("biomeos");
            std::fs::create_dir_all(&biomeos_dir).unwrap();
            std::fs::write(biomeos_dir.join("testprimal-dev.sock"), "").unwrap();
            std::fs::write(biomeos_dir.join("testprimal-dev.tarpc.sock"), "").unwrap();

            let (protocol, path) = negotiate_protocol("testprimal", "dev");
            assert_eq!(protocol, IpcProtocol::Tarpc);
            assert!(path.to_string_lossy().contains("tarpc.sock"));
        });
    }

    #[test]
    #[serial]
    fn test_negotiate_protocol_falls_back_to_jsonrpc() {
        let tmp = tempfile::tempdir().unwrap();
        let xdg = tmp.path().to_str().unwrap().to_string();
        temp_env::with_vars([("XDG_RUNTIME_DIR", Some(xdg.as_str()))], || {
            let biomeos_dir = tmp.path().join("biomeos");
            std::fs::create_dir_all(&biomeos_dir).unwrap();
            std::fs::write(biomeos_dir.join("testprimal-dev.sock"), "").unwrap();

            let (protocol, path) = negotiate_protocol("testprimal", "dev");
            assert_eq!(protocol, IpcProtocol::JsonRpc);
            assert!(!path.to_string_lossy().contains("tarpc"));
        });
    }

    #[test]
    fn test_ipc_protocol_equality() {
        assert_eq!(IpcProtocol::JsonRpc, IpcProtocol::JsonRpc);
        assert_eq!(IpcProtocol::Tarpc, IpcProtocol::Tarpc);
        assert_ne!(IpcProtocol::JsonRpc, IpcProtocol::Tarpc);
    }
}
