// SPDX-License-Identifier: AGPL-3.0-only

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
/// std::env::set_var("LOAMSPINE_JSONRPC_PORT", "8888");
/// assert_eq!(jsonrpc_port(), 8888);
///
/// // Falls back to default
/// std::env::remove_var("LOAMSPINE_JSONRPC_PORT");
/// std::env::remove_var("JSONRPC_PORT");
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
/// std::env::set_var("LOAMSPINE_TARPC_PORT", "9999");
/// assert_eq!(tarpc_port(), 9999);
///
/// // Falls back to default
/// std::env::remove_var("LOAMSPINE_TARPC_PORT");
/// std::env::remove_var("TARPC_PORT");
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
/// std::env::set_var("LOAMSPINE_BIND_ADDRESS", "127.0.0.1");
/// assert_eq!(bind_address(), "127.0.0.1");
///
/// // Default to all interfaces
/// std::env::remove_var("LOAMSPINE_BIND_ADDRESS");
/// std::env::remove_var("BIND_ADDRESS");
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
/// std::env::set_var("USE_OS_ASSIGNED_PORTS", "true");
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
/// std::env::set_var("USE_OS_ASSIGNED_PORTS", "true");
/// assert_eq!(actual_jsonrpc_port(), 0);
///
/// // OS assignment disabled, uses configured port
/// std::env::remove_var("USE_OS_ASSIGNED_PORTS");
/// std::env::set_var("LOAMSPINE_JSONRPC_PORT", "8888");
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
/// std::env::set_var("USE_OS_ASSIGNED_PORTS", "true");
/// assert_eq!(actual_tarpc_port(), 0);
///
/// // OS assignment disabled, uses configured port
/// std::env::remove_var("USE_OS_ASSIGNED_PORTS");
/// std::env::set_var("LOAMSPINE_TARPC_PORT", "9999");
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
/// Resolution order:
/// 1. `$XDG_RUNTIME_DIR/biomeos/`
/// 2. `/run/user/{uid}/biomeos/`
/// 3. `/tmp/biomeos/`
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
    if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
        return std::path::PathBuf::from(format!("{runtime_dir}/biomeos"));
    }

    // Fallback: /tmp/biomeos/
    std::path::PathBuf::from("/tmp/biomeos")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Tests use unwrap for clarity
mod tests {
    use super::*;
    use serial_test::serial;

    /// Clean up all environment variables that might affect tests
    fn cleanup_env_vars() {
        env::remove_var("LOAMSPINE_JSONRPC_PORT");
        env::remove_var("JSONRPC_PORT");
        env::remove_var("LOAMSPINE_TARPC_PORT");
        env::remove_var("TARPC_PORT");
        env::remove_var("USE_OS_ASSIGNED_PORTS");
        env::remove_var("LOAMSPINE_OS_PORTS");
        env::remove_var("LOAMSPINE_USE_OS_ASSIGNED_PORTS");
        env::remove_var("LOAMSPINE_BIND_ADDRESS");
        env::remove_var("BIND_ADDRESS");
    }

    #[test]
    #[serial]
    fn test_jsonrpc_port_from_env() {
        cleanup_env_vars();
        env::set_var("LOAMSPINE_JSONRPC_PORT", "8888");
        assert_eq!(jsonrpc_port(), 8888);
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_jsonrpc_port_default() {
        cleanup_env_vars();
        assert_eq!(jsonrpc_port(), DEFAULT_JSONRPC_PORT);
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_tarpc_port_from_env() {
        cleanup_env_vars();
        env::set_var("LOAMSPINE_TARPC_PORT", "9999");
        assert_eq!(tarpc_port(), 9999);
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_os_assigned_ports() {
        cleanup_env_vars();
        env::set_var("USE_OS_ASSIGNED_PORTS", "1");
        assert!(use_os_assigned_ports());
        cleanup_env_vars();

        env::set_var("USE_OS_ASSIGNED_PORTS", "0");
        assert!(!use_os_assigned_ports());
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_actual_ports_with_os_assignment() {
        cleanup_env_vars();
        env::set_var("USE_OS_ASSIGNED_PORTS", "1");

        // Verify the env var is set correctly
        assert!(
            use_os_assigned_ports(),
            "use_os_assigned_ports() should return true when USE_OS_ASSIGNED_PORTS=1"
        );

        let json_port = actual_jsonrpc_port();
        let tarpc_port_val = actual_tarpc_port();

        assert_eq!(
            json_port, 0,
            "actual_jsonrpc_port() should return 0 when OS assignment is enabled"
        );
        assert_eq!(
            tarpc_port_val, 0,
            "actual_tarpc_port() should return 0 when OS assignment is enabled"
        );

        cleanup_env_vars();
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
        cleanup_env_vars();
        env::set_var("LOAMSPINE_JSONRPC_PORT", "invalid");
        env::set_var("JSONRPC_PORT", "7777");

        assert_eq!(jsonrpc_port(), 7777);
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_jsonrpc_port_invalid_both_falls_back_to_default() {
        cleanup_env_vars();
        env::set_var("LOAMSPINE_JSONRPC_PORT", "not-a-number");
        env::set_var("JSONRPC_PORT", "also-invalid");

        assert_eq!(jsonrpc_port(), DEFAULT_JSONRPC_PORT);
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_jsonrpc_port_generic_env_var() {
        cleanup_env_vars();
        env::remove_var("LOAMSPINE_JSONRPC_PORT");
        env::set_var("JSONRPC_PORT", "5555");

        assert_eq!(jsonrpc_port(), 5555);
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_tarpc_port_invalid_loamspine_falls_back_to_generic() {
        cleanup_env_vars();
        env::set_var("LOAMSPINE_TARPC_PORT", "invalid");
        env::set_var("TARPC_PORT", "8888");

        assert_eq!(tarpc_port(), 8888);
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_tarpc_port_generic_env_var() {
        cleanup_env_vars();
        env::remove_var("LOAMSPINE_TARPC_PORT");
        env::set_var("TARPC_PORT", "7777");

        assert_eq!(tarpc_port(), 7777);
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_tarpc_port_invalid_both_falls_back_to_default() {
        cleanup_env_vars();
        env::set_var("LOAMSPINE_TARPC_PORT", "bad");
        env::set_var("TARPC_PORT", "worse");

        assert_eq!(tarpc_port(), DEFAULT_TARPC_PORT);
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_use_os_assigned_ports_yes() {
        cleanup_env_vars();
        env::set_var("USE_OS_ASSIGNED_PORTS", "yes");
        assert!(use_os_assigned_ports());
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_use_os_assigned_ports_true() {
        cleanup_env_vars();
        env::set_var("USE_OS_ASSIGNED_PORTS", "true");
        assert!(use_os_assigned_ports());
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_use_os_assigned_ports_loamspine_os_ports() {
        cleanup_env_vars();
        env::remove_var("USE_OS_ASSIGNED_PORTS");
        env::set_var("LOAMSPINE_OS_PORTS", "true");
        assert!(use_os_assigned_ports());
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_bind_address_loamspine_specific() {
        cleanup_env_vars();
        env::set_var("LOAMSPINE_BIND_ADDRESS", "127.0.0.1");
        assert_eq!(bind_address(), "127.0.0.1");
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_bind_address_generic() {
        cleanup_env_vars();
        env::remove_var("LOAMSPINE_BIND_ADDRESS");
        env::set_var("BIND_ADDRESS", "192.0.2.1");
        assert_eq!(bind_address(), "192.0.2.1");
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_bind_address_default() {
        cleanup_env_vars();
        assert_eq!(bind_address(), crate::constants::BIND_ALL_IPV4);
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_actual_ports_without_os_assignment() {
        cleanup_env_vars();
        env::set_var("LOAMSPINE_JSONRPC_PORT", "3333");
        env::set_var("LOAMSPINE_TARPC_PORT", "4444");

        assert_eq!(actual_jsonrpc_port(), 3333);
        assert_eq!(actual_tarpc_port(), 4444);
        cleanup_env_vars();
    }

    // ──────────────────────────────────────────────────────────────────────
    // Protocol escalation tests
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    #[serial]
    fn test_resolve_primal_socket_path() {
        cleanup_env_vars();
        env::remove_var("XDG_RUNTIME_DIR");
        let path = resolve_primal_socket("loamspine", "default");
        assert_eq!(
            path.to_string_lossy(),
            "/tmp/biomeos/loamspine-default.sock"
        );
    }

    #[test]
    #[serial]
    fn test_resolve_primal_tarpc_socket_path() {
        cleanup_env_vars();
        env::remove_var("XDG_RUNTIME_DIR");
        let path = resolve_primal_tarpc_socket("loamspine", "default");
        assert_eq!(
            path.to_string_lossy(),
            "/tmp/biomeos/loamspine-default.tarpc.sock"
        );
    }

    #[test]
    #[serial]
    fn test_resolve_primal_socket_with_xdg() {
        cleanup_env_vars();
        env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");
        let path = resolve_primal_socket("rhizocrypt", "myfamily");
        assert_eq!(
            path.to_string_lossy(),
            "/run/user/1000/biomeos/rhizocrypt-myfamily.sock"
        );
        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_negotiate_protocol_prefers_tarpc_when_available() {
        cleanup_env_vars();
        let tmp = tempfile::tempdir().unwrap();
        env::set_var("XDG_RUNTIME_DIR", tmp.path().to_str().unwrap());

        let biomeos_dir = tmp.path().join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).unwrap();

        // Create both socket files
        std::fs::write(biomeos_dir.join("testprimal-dev.sock"), "").unwrap();
        std::fs::write(biomeos_dir.join("testprimal-dev.tarpc.sock"), "").unwrap();

        let (protocol, path) = negotiate_protocol("testprimal", "dev");
        assert_eq!(protocol, IpcProtocol::Tarpc);
        assert!(path.to_string_lossy().contains("tarpc.sock"));

        cleanup_env_vars();
    }

    #[test]
    #[serial]
    fn test_negotiate_protocol_falls_back_to_jsonrpc() {
        cleanup_env_vars();
        let tmp = tempfile::tempdir().unwrap();
        env::set_var("XDG_RUNTIME_DIR", tmp.path().to_str().unwrap());

        let biomeos_dir = tmp.path().join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).unwrap();

        // Only JSON-RPC socket exists
        std::fs::write(biomeos_dir.join("testprimal-dev.sock"), "").unwrap();

        let (protocol, path) = negotiate_protocol("testprimal", "dev");
        assert_eq!(protocol, IpcProtocol::JsonRpc);
        assert!(!path.to_string_lossy().contains("tarpc"));

        cleanup_env_vars();
    }

    #[test]
    fn test_ipc_protocol_equality() {
        assert_eq!(IpcProtocol::JsonRpc, IpcProtocol::JsonRpc);
        assert_eq!(IpcProtocol::Tarpc, IpcProtocol::Tarpc);
        assert_ne!(IpcProtocol::JsonRpc, IpcProtocol::Tarpc);
    }
}
