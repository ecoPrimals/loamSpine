//! Network configuration helpers with environment-first discovery
//!
//! This module provides functions to resolve network configuration at runtime,
//! following the "infant discovery" pattern where all configuration is
//! discovered from the environment rather than hardcoded.

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
pub fn bind_address() -> String {
    // Try LoamSpine-specific env var first
    if let Ok(addr) = env::var("LOAMSPINE_BIND_ADDRESS") {
        debug!("Using bind address from LOAMSPINE_BIND_ADDRESS: {}", addr);
        return addr;
    }

    // Try generic env var
    if let Ok(addr) = env::var("BIND_ADDRESS") {
        debug!("Using bind address from BIND_ADDRESS: {}", addr);
        return addr;
    }

    // Default to all interfaces
    debug!("Using default bind address: 0.0.0.0");
    "0.0.0.0".to_string()
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
pub fn build_endpoint(scheme: &str, host: &str, port: u16, path: Option<&str>) -> String {
    match path {
        Some(p) => format!("{scheme}://{host}:{port}{p}"),
        None => format!("{scheme}://{host}:{port}"),
    }
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
        env::remove_var("LOAMSPINE_USE_OS_ASSIGNED_PORTS");
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
}
