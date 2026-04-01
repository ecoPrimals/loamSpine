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

// ──────────────────────────────────────────────────────────────────────────────
// Outer wrappers (read env, delegate to pure functions)
// ──────────────────────────────────────────────────────────────────────────────

/// Get JSON-RPC port from environment or default.
///
/// Priority: `LOAMSPINE_JSONRPC_PORT` > `JSONRPC_PORT` > default (8080).
#[must_use]
pub fn jsonrpc_port() -> u16 {
    resolve_jsonrpc_port(
        env::var("LOAMSPINE_JSONRPC_PORT").ok().as_deref(),
        env::var("JSONRPC_PORT").ok().as_deref(),
    )
}

/// Get tarpc port from environment or default.
///
/// Priority: `LOAMSPINE_TARPC_PORT` > `TARPC_PORT` > default (9001).
#[must_use]
pub fn tarpc_port() -> u16 {
    resolve_tarpc_port(
        env::var("LOAMSPINE_TARPC_PORT").ok().as_deref(),
        env::var("TARPC_PORT").ok().as_deref(),
    )
}

/// Get bind address from environment or default.
///
/// Priority: `LOAMSPINE_BIND_ADDRESS` > `BIND_ADDRESS` > `"0.0.0.0"`.
#[must_use]
pub fn bind_address() -> Cow<'static, str> {
    resolve_bind_address(
        env::var("LOAMSPINE_BIND_ADDRESS").ok().as_deref(),
        env::var("BIND_ADDRESS").ok().as_deref(),
    )
}

/// Check if we should use OS-assigned ports.
///
/// Returns `true` if `USE_OS_ASSIGNED_PORTS` or `LOAMSPINE_OS_PORTS` is truthy.
#[must_use]
pub fn use_os_assigned_ports() -> bool {
    resolve_use_os_assigned_ports(
        env::var("USE_OS_ASSIGNED_PORTS").ok().as_deref(),
        env::var("LOAMSPINE_OS_PORTS").ok().as_deref(),
    )
}

/// Get the actual JSON-RPC port to bind to, considering OS assignment.
#[must_use]
pub fn actual_jsonrpc_port() -> u16 {
    resolve_actual_jsonrpc_port(use_os_assigned_ports(), jsonrpc_port())
}

/// Get the actual tarpc port to bind to, considering OS assignment.
#[must_use]
pub fn actual_tarpc_port() -> u16 {
    resolve_actual_tarpc_port(use_os_assigned_ports(), tarpc_port())
}

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

/// Resolve a primal's socket path using the environment override pattern.
///
/// Checks `{PRIMAL}_SOCKET` env var first, then falls back to the
/// standard biomeos socket directory resolution.
#[must_use]
pub fn resolve_primal_socket_with_env(primal: &str, family_id: &str) -> std::path::PathBuf {
    let env_key = socket_env_var(primal);
    resolve_primal_socket_with(env::var(&env_key).ok().as_deref(), primal, family_id)
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
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests {
    use super::*;

    // ── Port resolution ──────────────────────────────────────────────────

    #[test]
    fn jsonrpc_port_from_loamspine_env() {
        assert_eq!(resolve_jsonrpc_port(Some("8888"), None), 8888);
    }

    #[test]
    fn jsonrpc_port_default_when_unset() {
        assert_eq!(resolve_jsonrpc_port(None, None), DEFAULT_JSONRPC_PORT);
    }

    #[test]
    fn jsonrpc_port_invalid_loamspine_falls_back_to_generic() {
        assert_eq!(resolve_jsonrpc_port(Some("invalid"), Some("7777")), 7777);
    }

    #[test]
    fn jsonrpc_port_invalid_both_falls_back_to_default() {
        assert_eq!(
            resolve_jsonrpc_port(Some("not-a-number"), Some("also-invalid")),
            DEFAULT_JSONRPC_PORT,
        );
    }

    #[test]
    fn jsonrpc_port_generic_env_var() {
        assert_eq!(resolve_jsonrpc_port(None, Some("5555")), 5555);
    }

    #[test]
    fn tarpc_port_from_env() {
        assert_eq!(resolve_tarpc_port(Some("9999"), None), 9999);
    }

    #[test]
    fn tarpc_port_default() {
        assert_eq!(resolve_tarpc_port(None, None), DEFAULT_TARPC_PORT);
    }

    #[test]
    fn tarpc_port_invalid_loamspine_falls_back_to_generic() {
        assert_eq!(resolve_tarpc_port(Some("invalid"), Some("8888")), 8888);
    }

    #[test]
    fn tarpc_port_generic_env_var() {
        assert_eq!(resolve_tarpc_port(None, Some("7777")), 7777);
    }

    #[test]
    fn tarpc_port_invalid_both_falls_back_to_default() {
        assert_eq!(
            resolve_tarpc_port(Some("bad"), Some("worse")),
            DEFAULT_TARPC_PORT,
        );
    }

    // ── OS-assigned ports ────────────────────────────────────────────────

    #[test]
    fn os_assigned_ports_on_1() {
        assert!(resolve_use_os_assigned_ports(Some("1"), None));
    }

    #[test]
    fn os_assigned_ports_off_0() {
        assert!(!resolve_use_os_assigned_ports(Some("0"), None));
    }

    #[test]
    fn os_assigned_ports_yes() {
        assert!(resolve_use_os_assigned_ports(Some("yes"), None));
    }

    #[test]
    fn os_assigned_ports_true() {
        assert!(resolve_use_os_assigned_ports(Some("true"), None));
    }

    #[test]
    fn os_assigned_ports_loamspine_os_ports() {
        assert!(resolve_use_os_assigned_ports(None, Some("true")));
    }

    #[test]
    fn os_assigned_ports_unset() {
        assert!(!resolve_use_os_assigned_ports(None, None));
    }

    // ── Actual ports ─────────────────────────────────────────────────────

    #[test]
    fn actual_ports_with_os_assignment() {
        assert_eq!(resolve_actual_jsonrpc_port(true, 8080), OS_ASSIGNED_PORT);
        assert_eq!(resolve_actual_tarpc_port(true, 9001), OS_ASSIGNED_PORT);
    }

    #[test]
    fn actual_ports_without_os_assignment() {
        assert_eq!(resolve_actual_jsonrpc_port(false, 3333), 3333);
        assert_eq!(resolve_actual_tarpc_port(false, 4444), 4444);
    }

    // ── Bind address ─────────────────────────────────────────────────────

    #[test]
    fn bind_address_loamspine_specific() {
        assert_eq!(resolve_bind_address(Some("127.0.0.1"), None), "127.0.0.1");
    }

    #[test]
    fn bind_address_generic() {
        assert_eq!(
            resolve_bind_address(None, Some("192.0.2.1")),
            "192.0.2.1"
        );
    }

    #[test]
    fn bind_address_default() {
        assert_eq!(
            resolve_bind_address(None, None),
            crate::constants::BIND_ALL_IPV4
        );
    }

    // ── Build endpoint ───────────────────────────────────────────────────

    #[test]
    fn build_endpoint_without_path() {
        assert_eq!(
            build_endpoint("http", "localhost", 8080, None),
            "http://localhost:8080",
        );
    }

    #[test]
    fn build_endpoint_with_path() {
        assert_eq!(
            build_endpoint("http", "localhost", 8080, Some("/api")),
            "http://localhost:8080/api",
        );
    }

    // ── Env var name builders ────────────────────────────────────────────

    #[test]
    fn socket_env_var_formatting() {
        assert_eq!(socket_env_var("rhizoCrypt"), "RHIZOCRYPT_SOCKET");
        assert_eq!(socket_env_var("sweetGrass"), "SWEETGRASS_SOCKET");
        assert_eq!(socket_env_var("loamSpine"), "LOAMSPINE_SOCKET");
        assert_eq!(socket_env_var("bear-dog"), "BEAR_DOG_SOCKET");
    }

    #[test]
    fn address_env_var_formatting() {
        assert_eq!(address_env_var("rhizoCrypt"), "RHIZOCRYPT_ADDRESS");
        assert_eq!(address_env_var("songbird"), "SONGBIRD_ADDRESS");
        assert_eq!(address_env_var("loamSpine"), "LOAMSPINE_ADDRESS");
    }

    // ── Socket base dir ──────────────────────────────────────────────────

    #[test]
    fn socket_base_dir_with_xdg() {
        let base = resolve_socket_base_dir_with(Some("/run/user/1000"));
        assert_eq!(base, std::path::PathBuf::from("/run/user/1000/biomeos"));
    }

    #[test]
    fn socket_base_dir_fallback() {
        let base = resolve_socket_base_dir_with(None);
        assert!(
            base.to_string_lossy().ends_with("biomeos"),
            "got: {}",
            base.display(),
        );
    }

    // ── Primal socket resolution ─────────────────────────────────────────

    #[test]
    fn primal_socket_path() {
        let base = resolve_socket_base_dir_with(None);
        let path = resolve_primal_socket_from(&base, "loamspine", "default");
        assert!(
            path.to_string_lossy()
                .ends_with("biomeos/loamspine-default.sock"),
            "got: {}",
            path.display(),
        );
    }

    #[test]
    fn primal_tarpc_socket_path() {
        let base = resolve_socket_base_dir_with(None);
        let path = resolve_primal_tarpc_socket_from(&base, "loamspine", "default");
        assert!(
            path.to_string_lossy()
                .ends_with("biomeos/loamspine-default.tarpc.sock"),
            "got: {}",
            path.display(),
        );
    }

    #[test]
    fn primal_socket_with_xdg() {
        let base = resolve_socket_base_dir_with(Some("/run/user/1000"));
        let path = resolve_primal_socket_from(&base, "rhizocrypt", "myfamily");
        assert_eq!(
            path.to_string_lossy(),
            "/run/user/1000/biomeos/rhizocrypt-myfamily.sock",
        );
    }

    #[test]
    fn primal_socket_with_env_override() {
        let path =
            resolve_primal_socket_with(Some("/tmp/override.sock"), "testprimal", "dev");
        assert_eq!(path, std::path::PathBuf::from("/tmp/override.sock"));
    }

    #[test]
    fn primal_socket_with_env_fallback() {
        let path = resolve_primal_socket_with(None, "testprimal", "dev");
        assert!(path.to_string_lossy().contains("testprimal-dev.sock"));
    }

    // ── Protocol negotiation ─────────────────────────────────────────────

    #[test]
    fn negotiate_protocol_prefers_tarpc_when_available() {
        let tmp = tempfile::tempdir().unwrap();
        let biomeos_dir = tmp.path().join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).unwrap();
        std::fs::write(biomeos_dir.join("testprimal-dev.sock"), "").unwrap();
        std::fs::write(biomeos_dir.join("testprimal-dev.tarpc.sock"), "").unwrap();

        let (protocol, path) = negotiate_protocol_from(&biomeos_dir, "testprimal", "dev");
        assert_eq!(protocol, IpcProtocol::Tarpc);
        assert!(path.to_string_lossy().contains("tarpc.sock"));
    }

    #[test]
    fn negotiate_protocol_falls_back_to_jsonrpc() {
        let tmp = tempfile::tempdir().unwrap();
        let biomeos_dir = tmp.path().join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).unwrap();
        std::fs::write(biomeos_dir.join("testprimal-dev.sock"), "").unwrap();

        let (protocol, path) = negotiate_protocol_from(&biomeos_dir, "testprimal", "dev");
        assert_eq!(protocol, IpcProtocol::JsonRpc);
        assert!(!path.to_string_lossy().contains("tarpc"));
    }

    #[test]
    fn ipc_protocol_equality() {
        assert_eq!(IpcProtocol::JsonRpc, IpcProtocol::JsonRpc);
        assert_eq!(IpcProtocol::Tarpc, IpcProtocol::Tarpc);
        assert_ne!(IpcProtocol::JsonRpc, IpcProtocol::Tarpc);
    }
}
