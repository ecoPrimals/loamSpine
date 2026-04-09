// SPDX-License-Identifier: AGPL-3.0-or-later

//! Environment-reading facades for network configuration.
//!
//! Each function reads well-known environment variables and delegates to the
//! pure resolution functions in [`super::network`].  Tests should call the
//! inner pure functions directly for concurrency safety.

use std::borrow::Cow;
use std::env;

use super::network;

/// Get JSON-RPC port from environment or default.
///
/// Priority: `LOAMSPINE_JSONRPC_PORT` > `JSONRPC_PORT` > default (8080).
#[must_use]
pub fn jsonrpc_port() -> u16 {
    network::resolve_jsonrpc_port(
        env::var("LOAMSPINE_JSONRPC_PORT").ok().as_deref(),
        env::var("JSONRPC_PORT").ok().as_deref(),
    )
}

/// Get tarpc port from environment or default.
///
/// Priority: `LOAMSPINE_TARPC_PORT` > `TARPC_PORT` > default (9001).
#[must_use]
pub fn tarpc_port() -> u16 {
    network::resolve_tarpc_port(
        env::var("LOAMSPINE_TARPC_PORT").ok().as_deref(),
        env::var("TARPC_PORT").ok().as_deref(),
    )
}

/// Get bind address from environment or default.
///
/// Priority: `LOAMSPINE_BIND_ADDRESS` > `BIND_ADDRESS` > `"0.0.0.0"`.
#[must_use]
pub fn bind_address() -> Cow<'static, str> {
    network::resolve_bind_address(
        env::var("LOAMSPINE_BIND_ADDRESS").ok().as_deref(),
        env::var("BIND_ADDRESS").ok().as_deref(),
    )
}

/// Check if we should use OS-assigned ports.
///
/// Returns `true` if `USE_OS_ASSIGNED_PORTS` or `LOAMSPINE_OS_PORTS` is truthy.
#[must_use]
pub fn use_os_assigned_ports() -> bool {
    network::resolve_use_os_assigned_ports(
        env::var("USE_OS_ASSIGNED_PORTS").ok().as_deref(),
        env::var("LOAMSPINE_OS_PORTS").ok().as_deref(),
    )
}

/// Get the actual JSON-RPC port to bind to, considering OS assignment.
#[must_use]
pub fn actual_jsonrpc_port() -> u16 {
    network::resolve_actual_jsonrpc_port(use_os_assigned_ports(), jsonrpc_port())
}

/// Get the actual tarpc port to bind to, considering OS assignment.
#[must_use]
pub fn actual_tarpc_port() -> u16 {
    network::resolve_actual_tarpc_port(use_os_assigned_ports(), tarpc_port())
}

/// Resolve a primal's socket path using the environment override pattern.
///
/// Checks `{PRIMAL}_SOCKET` env var first, then falls back to the
/// standard biomeos socket directory resolution.
#[must_use]
pub fn resolve_primal_socket_with_env(primal: &str, family_id: &str) -> std::path::PathBuf {
    let env_key = network::socket_env_var(primal);
    network::resolve_primal_socket_with(env::var(&env_key).ok().as_deref(), primal, family_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonrpc_port_default_without_env() {
        assert_eq!(jsonrpc_port(), 8080);
    }

    #[test]
    fn tarpc_port_default_without_env() {
        assert_eq!(tarpc_port(), 9001);
    }

    #[test]
    fn bind_address_default_without_env() {
        assert_eq!(bind_address().as_ref(), "0.0.0.0");
    }

    #[test]
    fn use_os_assigned_ports_default_without_env() {
        assert!(!use_os_assigned_ports());
    }

    #[test]
    fn actual_jsonrpc_port_default_without_env() {
        assert_eq!(actual_jsonrpc_port(), 8080);
    }

    #[test]
    fn actual_tarpc_port_default_without_env() {
        assert_eq!(actual_tarpc_port(), 9001);
    }

    #[test]
    fn resolve_primal_socket_with_env_default_path_suffix() {
        let path = resolve_primal_socket_with_env("loamspine", "default");
        let s = path.to_string_lossy();
        assert!(
            s.ends_with("loamspine-default.sock"),
            "expected path ending with loamspine-default.sock, got {path:?}"
        );
    }
}
