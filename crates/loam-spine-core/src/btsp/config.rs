// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP configuration and handshake provider socket resolution.
//!
//! Derives BTSP handshake requirements from environment variables and
//! resolves the handshake-as-a-service provider UDS socket path.
//!
//! The provider prefix defaults to the BTSP protocol standard naming
//! convention but can be overridden via `BTSP_PROVIDER` environment
//! variable for capability-based discovery in non-standard deployments.

use std::path::PathBuf;

/// Default BTSP handshake provider socket prefix per `BTSP_PROTOCOL_STANDARD.md`.
///
/// Overridable via `BTSP_PROVIDER` environment variable for deployments
/// where the handshake-as-a-service provider has a different socket name.
const DEFAULT_BTSP_PROVIDER_PREFIX: &str = "beardog";

/// Resolve the BTSP provider prefix from environment or fall back to default.
///
/// Checks `BTSP_PROVIDER` env var first, allowing runtime configuration
/// of the handshake provider without compile-time primal coupling.
fn btsp_provider_prefix() -> String {
    std::env::var("BTSP_PROVIDER").unwrap_or_else(|_| DEFAULT_BTSP_PROVIDER_PREFIX.into())
}

/// Pure variant for testing and explicit configuration.
#[must_use]
fn btsp_provider_prefix_with(provider_override: Option<&str>) -> String {
    provider_override
        .filter(|s| !s.is_empty())
        .map_or_else(|| DEFAULT_BTSP_PROVIDER_PREFIX.into(), Into::into)
}

/// BTSP handshake configuration, derived from environment.
///
/// When `required` is `true`, every incoming UDS connection must complete the
/// BTSP handshake before any JSON-RPC methods are exposed. When `false`,
/// raw newline-delimited JSON-RPC is accepted (development mode).
#[derive(Debug, Clone)]
pub struct BtspHandshakeConfig {
    /// Whether BTSP handshake is mandatory.
    pub required: bool,
    /// Path to the handshake provider UDS socket.
    pub provider_socket: PathBuf,
    /// Family ID (for logging/diagnostics).
    pub family_id: String,
}

impl BtspHandshakeConfig {
    /// Derive BTSP configuration from explicit values (pure, no env reads).
    ///
    /// BTSP is required when `family_id` is set and not `"default"`.
    #[must_use]
    pub fn from_values(
        family_id: Option<&str>,
        provider_socket_override: Option<&str>,
        socket_dir: Option<&str>,
    ) -> Option<Self> {
        let fid = family_id.filter(|s| !s.is_empty() && *s != "default")?;

        let provider_socket = if let Some(s) = provider_socket_override {
            PathBuf::from(s)
        } else {
            resolve_provider_socket_with(Some(fid), socket_dir, None)
        };

        Some(Self {
            required: true,
            provider_socket,
            family_id: fid.into(),
        })
    }

    /// Derive BTSP configuration from environment variables.
    ///
    /// Returns `Some` when `BIOMEOS_FAMILY_ID` is set to a non-default value,
    /// meaning BTSP is required. Returns `None` in development mode.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let provider_socket_override = std::env::var("BTSP_PROVIDER_SOCKET")
            .or_else(|_| std::env::var("BEARDOG_SOCKET"))
            .ok();
        Self::from_values(
            std::env::var("BIOMEOS_FAMILY_ID").ok().as_deref(),
            provider_socket_override.as_deref(),
            std::env::var("BIOMEOS_SOCKET_DIR").ok().as_deref(),
        )
    }
}

/// Resolve the BTSP handshake provider UDS socket path from explicit values.
///
/// Resolution order:
/// 1. `$BIOMEOS_SOCKET_DIR/{provider}-{family_id}.sock`
/// 2. `/run/user/{uid}/biomeos/{provider}-{family_id}.sock` (Linux)
/// 3. `$TMPDIR/biomeos/{provider}-{family_id}.sock`
#[must_use]
pub fn resolve_provider_socket_with(
    family_id: Option<&str>,
    socket_dir: Option<&str>,
    provider_override: Option<&str>,
) -> PathBuf {
    let sock_name = provider_socket_name(family_id, provider_override);

    if let Some(dir) = socket_dir {
        return PathBuf::from(dir).join(&sock_name);
    }

    #[cfg(target_os = "linux")]
    if let Some(base) = crate::constants::network::linux_run_user_biomeos() {
        return base.join(&sock_name);
    }

    std::env::temp_dir()
        .join(crate::primal_names::BIOMEOS_SOCKET_DIR)
        .join(sock_name)
}

/// Resolve the provider socket using environment variables.
#[must_use]
pub fn resolve_provider_socket(family_id: Option<&str>) -> PathBuf {
    let prefix = btsp_provider_prefix();
    resolve_provider_socket_with(family_id, None, Some(&prefix))
}

/// Build the BTSP provider socket filename.
///
/// - With family: `{provider}-{family_id}.sock`
/// - Without family: `{provider}.sock`
#[must_use]
pub(crate) fn provider_socket_name(
    family_id: Option<&str>,
    provider_override: Option<&str>,
) -> String {
    let prefix = btsp_provider_prefix_with(provider_override);
    match family_id {
        Some(fid) if !fid.is_empty() && fid != "default" => {
            format!("{prefix}-{fid}.sock")
        }
        _ => format!("{prefix}.sock"),
    }
}

/// Check whether BTSP is required based on the environment.
///
/// Returns `true` when `BIOMEOS_FAMILY_ID` is set and not `"default"`.
#[must_use]
pub fn is_btsp_required() -> bool {
    is_btsp_required_with(std::env::var("BIOMEOS_FAMILY_ID").ok().as_deref())
}

/// Pure inner function: check BTSP requirement from explicit values.
#[must_use]
pub fn is_btsp_required_with(family_id: Option<&str>) -> bool {
    family_id.is_some_and(|fid| !fid.is_empty() && fid != "default")
}
