// SPDX-License-Identifier: AGPL-3.0-or-later

//! Environment-reading facades for runtime configuration.
//!
//! Each function reads well-known environment variables and delegates to the
//! pure resolution functions in [`super::network`] (or other pure helpers).
//! Tests should call the inner pure functions directly for concurrency safety.

use std::borrow::Cow;
use std::env;

use super::network;

/// Well-known environment variable names used across LoamSpine.
pub mod keys {
    /// LoamSpine-scoped JSON-RPC port override.
    pub const LOAMSPINE_JSONRPC_PORT: &str = "LOAMSPINE_JSONRPC_PORT";
    /// Generic JSON-RPC port override.
    pub const JSONRPC_PORT: &str = "JSONRPC_PORT";
    /// LoamSpine-scoped tarpc port override.
    pub const LOAMSPINE_TARPC_PORT: &str = "LOAMSPINE_TARPC_PORT";
    /// Generic tarpc port override.
    pub const TARPC_PORT: &str = "TARPC_PORT";
    /// LoamSpine-scoped bind address override.
    pub const LOAMSPINE_BIND_ADDRESS: &str = "LOAMSPINE_BIND_ADDRESS";
    /// Generic bind address override.
    pub const BIND_ADDRESS: &str = "BIND_ADDRESS";
    /// Request OS-assigned ports when truthy.
    pub const USE_OS_ASSIGNED_PORTS: &str = "USE_OS_ASSIGNED_PORTS";
    /// LoamSpine-scoped OS-assigned ports flag.
    pub const LOAMSPINE_OS_PORTS: &str = "LOAMSPINE_OS_PORTS";
    /// Enable or disable service discovery (`0`/`false`/`no` disables).
    pub const LOAMSPINE_DISCOVERY_ENABLED: &str = "LOAMSPINE_DISCOVERY_ENABLED";
    /// Explicit discovery service endpoint URL.
    pub const DISCOVERY_ENDPOINT: &str = "DISCOVERY_ENDPOINT";
    /// Explicit tarpc endpoint URL override.
    pub const TARPC_ENDPOINT: &str = "TARPC_ENDPOINT";
    /// Explicit JSON-RPC endpoint URL override.
    pub const JSONRPC_ENDPOINT: &str = "JSONRPC_ENDPOINT";
    /// HTTP service registry base URL for infant discovery.
    pub const SERVICE_REGISTRY_URL: &str = "SERVICE_REGISTRY_URL";
    /// Infant discovery cache TTL in seconds.
    pub const DISCOVERY_CACHE_TTL: &str = "DISCOVERY_CACHE_TTL";
    /// XDG runtime directory for socket and manifest discovery.
    pub const XDG_RUNTIME_DIR: &str = "XDG_RUNTIME_DIR";
    /// LoamSpine UDS socket path override.
    pub const LOAMSPINE_SOCKET: &str = "LOAMSPINE_SOCKET";
    /// BiomeOS family identifier for scoped sockets and BTSP.
    pub const BIOMEOS_FAMILY_ID: &str = "BIOMEOS_FAMILY_ID";
    /// Allow insecure development sockets alongside a family ID.
    pub const BIOMEOS_INSECURE: &str = "BIOMEOS_INSECURE";
    /// Explicit NeuralAPI socket path override.
    pub const BIOMEOS_NEURAL_API_SOCKET: &str = "BIOMEOS_NEURAL_API_SOCKET";
    /// BiomeOS socket directory override.
    pub const BIOMEOS_SOCKET_DIR: &str = "BIOMEOS_SOCKET_DIR";
    /// BTSP handshake provider prefix override.
    pub const BTSP_PROVIDER: &str = "BTSP_PROVIDER";
    /// BTSP handshake provider socket path override.
    pub const BTSP_PROVIDER_SOCKET: &str = "BTSP_PROVIDER_SOCKET";
    /// Canonical BTSP family seed (hex) for session creation.
    pub const FAMILY_SEED: &str = "FAMILY_SEED";
    /// BTSP-scoped alias for [`FAMILY_SEED`].
    pub const BTSP_FAMILY_SEED: &str = "BTSP_FAMILY_SEED";
    /// Deprecated BearDog-era alias for [`FAMILY_SEED`].
    pub const BEARDOG_FAMILY_SEED: &str = "BEARDOG_FAMILY_SEED";
    /// Tower signer UDS socket path for delegated signing.
    pub const TOWER_SIGNER_SOCKET: &str = "TOWER_SIGNER_SOCKET";
    /// Deprecated BearDog-era alias for [`TOWER_SIGNER_SOCKET`].
    pub const BEARDOG_SOCKET: &str = "BEARDOG_SOCKET";
    /// Tower signer DID for delegated signing.
    pub const TOWER_SIGNER_DID: &str = "TOWER_SIGNER_DID";
    /// JSON-RPC authentication mode (`permissive` or `enforced`).
    pub const LOAMSPINE_AUTH_MODE: &str = "LOAMSPINE_AUTH_MODE";
}

/// Get JSON-RPC port from environment or default.
///
/// Priority: `LOAMSPINE_JSONRPC_PORT` > `JSONRPC_PORT` > default (8080).
#[must_use]
pub fn jsonrpc_port() -> u16 {
    network::resolve_jsonrpc_port(
        env::var(keys::LOAMSPINE_JSONRPC_PORT).ok().as_deref(),
        env::var(keys::JSONRPC_PORT).ok().as_deref(),
    )
}

/// Get tarpc port from environment or default.
///
/// Priority: `LOAMSPINE_TARPC_PORT` > `TARPC_PORT` > default (9001).
#[must_use]
pub fn tarpc_port() -> u16 {
    network::resolve_tarpc_port(
        env::var(keys::LOAMSPINE_TARPC_PORT).ok().as_deref(),
        env::var(keys::TARPC_PORT).ok().as_deref(),
    )
}

/// Get bind address from environment or default.
///
/// Priority: `LOAMSPINE_BIND_ADDRESS` > `BIND_ADDRESS` > `"0.0.0.0"`.
#[must_use]
pub fn bind_address() -> Cow<'static, str> {
    network::resolve_bind_address(
        env::var(keys::LOAMSPINE_BIND_ADDRESS).ok().as_deref(),
        env::var(keys::BIND_ADDRESS).ok().as_deref(),
    )
}

/// Check if we should use OS-assigned ports.
///
/// Returns `true` if `USE_OS_ASSIGNED_PORTS` or `LOAMSPINE_OS_PORTS` is truthy.
#[must_use]
pub fn use_os_assigned_ports() -> bool {
    network::resolve_use_os_assigned_ports(
        env::var(keys::USE_OS_ASSIGNED_PORTS).ok().as_deref(),
        env::var(keys::LOAMSPINE_OS_PORTS).ok().as_deref(),
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

/// Check whether TCP transports were explicitly requested via environment.
///
/// Returns `true` when any TCP-related env var is set (port or OS-assigned).
/// Used by the binary entrypoint to implement opt-in TCP: when no CLI flag
/// and no env var requests TCP, only the UDS socket is started.
#[must_use]
pub fn has_explicit_tcp_config() -> bool {
    env::var(keys::LOAMSPINE_JSONRPC_PORT).is_ok()
        || env::var(keys::JSONRPC_PORT).is_ok()
        || env::var(keys::LOAMSPINE_TARPC_PORT).is_ok()
        || env::var(keys::TARPC_PORT).is_ok()
        || use_os_assigned_ports()
}

/// Resolve a primal's socket path using the environment override pattern.
///
/// Checks `{PRIMAL}_SOCKET` env var first, then falls back to the
/// standard biomeos socket directory resolution.
#[must_use]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "pre-wired for provenance trio socket resolution (strandGate deploy)"
    )
)]
pub(crate) fn resolve_primal_socket_with_env(primal: &str, family_id: &str) -> std::path::PathBuf {
    let env_key = network::socket_env_var(primal);
    network::resolve_primal_socket_with(env::var(&env_key).ok().as_deref(), primal, family_id)
}

/// Read `$XDG_RUNTIME_DIR` when set.
#[must_use]
pub fn xdg_runtime_dir() -> Option<String> {
    env::var(keys::XDG_RUNTIME_DIR).ok()
}

/// Read `$BIOMEOS_FAMILY_ID` when set.
#[must_use]
pub fn biomeos_family_id() -> Option<String> {
    env::var(keys::BIOMEOS_FAMILY_ID).ok()
}

/// Read `$BIOMEOS_FAMILY_ID`, defaulting to `"default"` when unset.
#[must_use]
pub fn biomeos_family_id_or_default() -> String {
    biomeos_family_id().unwrap_or_else(|| "default".into())
}

/// Read `$LOAMSPINE_SOCKET` when set.
#[must_use]
pub fn loamspine_socket() -> Option<String> {
    env::var(keys::LOAMSPINE_SOCKET).ok()
}

/// Read `$BIOMEOS_INSECURE` when set.
#[must_use]
pub fn biomeos_insecure() -> Option<String> {
    env::var(keys::BIOMEOS_INSECURE).ok()
}

/// Read `$BIOMEOS_NEURAL_API_SOCKET` when set.
#[must_use]
pub fn biomeos_neural_api_socket() -> Option<String> {
    env::var(keys::BIOMEOS_NEURAL_API_SOCKET).ok()
}

/// Read `$BIOMEOS_SOCKET_DIR` when set.
#[must_use]
pub fn biomeos_socket_dir() -> Option<String> {
    env::var(keys::BIOMEOS_SOCKET_DIR).ok()
}

/// Read `$BTSP_PROVIDER` when set.
#[must_use]
pub fn btsp_provider() -> Option<String> {
    env::var(keys::BTSP_PROVIDER).ok()
}

/// Read `$BTSP_PROVIDER_SOCKET` when set.
#[must_use]
pub fn btsp_provider_socket() -> Option<String> {
    env::var(keys::BTSP_PROVIDER_SOCKET).ok()
}

/// Whether service discovery is enabled (defaults to `true`).
#[must_use]
pub fn discovery_enabled() -> bool {
    env::var(keys::LOAMSPINE_DISCOVERY_ENABLED)
        .map_or(true, |v| !matches!(v.as_str(), "0" | "false" | "no"))
}

/// Read `$DISCOVERY_ENDPOINT` when set.
#[must_use]
pub fn discovery_endpoint() -> Option<String> {
    env::var(keys::DISCOVERY_ENDPOINT).ok()
}

/// Read `$TARPC_ENDPOINT` when set.
#[must_use]
pub fn tarpc_endpoint() -> Option<String> {
    env::var(keys::TARPC_ENDPOINT).ok()
}

/// Read `$JSONRPC_ENDPOINT` when set.
#[must_use]
pub fn jsonrpc_endpoint() -> Option<String> {
    env::var(keys::JSONRPC_ENDPOINT).ok()
}

/// Read `$SERVICE_REGISTRY_URL` when set.
#[must_use]
pub fn service_registry_url() -> Option<String> {
    env::var(keys::SERVICE_REGISTRY_URL).ok()
}

/// Read `$DISCOVERY_CACHE_TTL` when set and parseable as `u64`.
#[must_use]
pub fn discovery_cache_ttl() -> Option<u64> {
    env::var(keys::DISCOVERY_CACHE_TTL)
        .ok()
        .and_then(|s| s.parse().ok())
}

/// Read `$LOAMSPINE_AUTH_MODE` when set.
#[must_use]
pub fn loamspine_auth_mode() -> Option<String> {
    env::var(keys::LOAMSPINE_AUTH_MODE).ok()
}

/// Resolve the BTSP family seed from environment.
///
/// Priority: `FAMILY_SEED` > `BTSP_FAMILY_SEED` > `BEARDOG_FAMILY_SEED` (deprecated).
///
/// # Errors
///
/// Returns [`env::VarError`] when none of the seed variables are set.
pub fn family_seed() -> Result<String, env::VarError> {
    if let Ok(v) = env::var(keys::FAMILY_SEED) {
        return Ok(v);
    }
    if let Ok(v) = env::var(keys::BTSP_FAMILY_SEED) {
        return Ok(v);
    }
    if let Ok(v) = env::var(keys::BEARDOG_FAMILY_SEED) {
        tracing::warn!(
            "BEARDOG_FAMILY_SEED is deprecated — migrate to FAMILY_SEED or BTSP_FAMILY_SEED"
        );
        return Ok(v);
    }
    Err(env::VarError::NotPresent)
}

/// Resolve the Tower signer socket from environment.
///
/// Priority: `TOWER_SIGNER_SOCKET` > `BEARDOG_SOCKET` (deprecated).
#[must_use]
pub fn tower_signer_socket() -> Option<String> {
    if let Ok(v) = env::var(keys::TOWER_SIGNER_SOCKET) {
        return Some(v);
    }
    if let Ok(v) = env::var(keys::BEARDOG_SOCKET) {
        tracing::warn!("BEARDOG_SOCKET is deprecated — migrate to TOWER_SIGNER_SOCKET");
        return Some(v);
    }
    None
}

/// Read `$TOWER_SIGNER_DID` when set.
#[must_use]
pub fn tower_signer_did() -> Option<String> {
    env::var(keys::TOWER_SIGNER_DID).ok()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap for concise error paths"
)]
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
    fn has_explicit_tcp_config_default_without_env() {
        assert!(!has_explicit_tcp_config());
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

    #[test]
    fn discovery_enabled_defaults_true_without_env() {
        assert!(discovery_enabled());
    }

    #[test]
    fn biomeos_family_id_or_default_without_env() {
        assert_eq!(biomeos_family_id_or_default(), "default");
    }

    #[test]
    fn family_seed_primary_env() {
        temp_env::with_vars(
            [
                ("FAMILY_SEED", Some("primary_seed")),
                ("BTSP_FAMILY_SEED", Some("btsp_seed")),
                ("BEARDOG_FAMILY_SEED", Some("beardog_seed")),
            ],
            || {
                assert_eq!(family_seed().unwrap(), "primary_seed");
            },
        );
    }

    #[test]
    fn family_seed_btsp_fallback() {
        temp_env::with_vars(
            [
                ("FAMILY_SEED", None::<&str>),
                ("BTSP_FAMILY_SEED", Some("btsp_seed")),
                ("BEARDOG_FAMILY_SEED", Some("beardog_seed")),
            ],
            || {
                assert_eq!(family_seed().unwrap(), "btsp_seed");
            },
        );
    }

    #[test]
    fn family_seed_deprecated_fallback() {
        temp_env::with_vars(
            [
                ("FAMILY_SEED", None::<&str>),
                ("BTSP_FAMILY_SEED", None::<&str>),
                ("BEARDOG_FAMILY_SEED", Some("beardog_seed")),
            ],
            || {
                assert_eq!(family_seed().unwrap(), "beardog_seed");
            },
        );
    }

    #[test]
    fn family_seed_missing_returns_err() {
        temp_env::with_vars(
            [
                ("FAMILY_SEED", None::<&str>),
                ("BTSP_FAMILY_SEED", None::<&str>),
                ("BEARDOG_FAMILY_SEED", None::<&str>),
            ],
            || {
                assert!(family_seed().is_err());
            },
        );
    }

    #[test]
    fn tower_signer_socket_primary_env() {
        temp_env::with_vars(
            [
                ("TOWER_SIGNER_SOCKET", Some("/run/tower.sock")),
                ("BEARDOG_SOCKET", Some("/run/beardog.sock")),
            ],
            || {
                assert_eq!(tower_signer_socket().unwrap(), "/run/tower.sock");
            },
        );
    }

    #[test]
    fn tower_signer_socket_deprecated_fallback() {
        temp_env::with_vars(
            [
                ("TOWER_SIGNER_SOCKET", None::<&str>),
                ("BEARDOG_SOCKET", Some("/run/beardog.sock")),
            ],
            || {
                assert_eq!(tower_signer_socket().unwrap(), "/run/beardog.sock");
            },
        );
    }

    #[test]
    fn tower_signer_socket_missing_returns_none() {
        temp_env::with_vars(
            [
                ("TOWER_SIGNER_SOCKET", None::<&str>),
                ("BEARDOG_SOCKET", None::<&str>),
            ],
            || {
                assert!(tower_signer_socket().is_none());
            },
        );
    }

    #[test]
    fn key_constants_are_valid_env_names() {
        let key_names = [
            keys::LOAMSPINE_JSONRPC_PORT,
            keys::JSONRPC_PORT,
            keys::LOAMSPINE_TARPC_PORT,
            keys::TARPC_PORT,
            keys::LOAMSPINE_BIND_ADDRESS,
            keys::BIND_ADDRESS,
            keys::USE_OS_ASSIGNED_PORTS,
            keys::BIOMEOS_FAMILY_ID,
            keys::BIOMEOS_INSECURE,
            keys::FAMILY_SEED,
            keys::BTSP_FAMILY_SEED,
            keys::BEARDOG_FAMILY_SEED,
            keys::TOWER_SIGNER_SOCKET,
            keys::BEARDOG_SOCKET,
            keys::TOWER_SIGNER_DID,
            keys::LOAMSPINE_AUTH_MODE,
        ];
        for name in key_names {
            assert!(!name.is_empty(), "env key constant must not be empty");
            assert!(
                name.chars().all(|c| c.is_ascii_uppercase() || c == '_'),
                "env key {name} should be UPPER_SNAKE_CASE"
            );
        }
    }

    #[test]
    fn discovery_cache_ttl_default_none() {
        assert!(discovery_cache_ttl().is_none());
    }

    #[test]
    fn discovery_cache_ttl_invalid_value_returns_none() {
        temp_env::with_var("DISCOVERY_CACHE_TTL", Some("not-a-number"), || {
            assert!(discovery_cache_ttl().is_none());
        });
    }

    #[test]
    fn discovery_cache_ttl_valid_value() {
        temp_env::with_var("DISCOVERY_CACHE_TTL", Some("300"), || {
            assert_eq!(discovery_cache_ttl(), Some(300));
        });
    }

    #[test]
    fn discovery_enabled_false_variants() {
        for val in &["0", "false", "no"] {
            temp_env::with_var("LOAMSPINE_DISCOVERY_ENABLED", Some(val), || {
                assert!(!discovery_enabled(), "expected disabled for value '{val}'");
            });
        }
    }

    #[test]
    fn discovery_enabled_truthy_variants() {
        for val in &["1", "true", "yes", "anything"] {
            temp_env::with_var("LOAMSPINE_DISCOVERY_ENABLED", Some(val), || {
                assert!(discovery_enabled(), "expected enabled for value '{val}'");
            });
        }
    }

    #[test]
    fn loamspine_auth_mode_default_none() {
        assert!(loamspine_auth_mode().is_none());
    }

    #[test]
    fn tower_signer_did_default_none() {
        assert!(tower_signer_did().is_none());
    }
}
