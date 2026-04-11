// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP configuration and BearDog socket resolution.
//!
//! Derives BTSP handshake requirements from environment variables and
//! resolves the BearDog UDS socket path for handshake-as-a-service calls.

use std::path::PathBuf;

/// BTSP handshake configuration, derived from environment.
///
/// When `required` is `true`, every incoming UDS connection must complete the
/// BTSP handshake before any JSON-RPC methods are exposed. When `false`,
/// raw newline-delimited JSON-RPC is accepted (development mode).
#[derive(Debug, Clone)]
pub struct BtspHandshakeConfig {
    /// Whether BTSP handshake is mandatory.
    pub required: bool,
    /// Path to the BearDog UDS socket for handshake-as-a-service calls.
    pub beardog_socket: PathBuf,
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
        beardog_socket_override: Option<&str>,
        socket_dir: Option<&str>,
    ) -> Option<Self> {
        let fid = family_id.filter(|s| !s.is_empty() && *s != "default")?;

        let beardog_socket = if let Some(s) = beardog_socket_override {
            PathBuf::from(s)
        } else {
            resolve_beardog_socket_with(Some(fid), socket_dir)
        };

        Some(Self {
            required: true,
            beardog_socket,
            family_id: fid.to_string(),
        })
    }

    /// Derive BTSP configuration from environment variables.
    ///
    /// Returns `Some` when `BIOMEOS_FAMILY_ID` is set to a non-default value,
    /// meaning BTSP is required. Returns `None` in development mode.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        Self::from_values(
            std::env::var("BIOMEOS_FAMILY_ID").ok().as_deref(),
            std::env::var("BEARDOG_SOCKET").ok().as_deref(),
            std::env::var("BIOMEOS_SOCKET_DIR").ok().as_deref(),
        )
    }
}

/// Resolve the BearDog UDS socket path from explicit values.
///
/// Resolution order:
/// 1. `$BIOMEOS_SOCKET_DIR/beardog-{family_id}.sock`
/// 2. `/run/user/{uid}/biomeos/beardog-{family_id}.sock` (Linux)
/// 3. `$TMPDIR/biomeos/beardog-{family_id}.sock`
#[must_use]
pub fn resolve_beardog_socket_with(family_id: Option<&str>, socket_dir: Option<&str>) -> PathBuf {
    let sock_name = beardog_socket_name(family_id);

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

/// Build the BearDog socket filename.
///
/// - With family: `beardog-{family_id}.sock`
/// - Without family: `beardog.sock`
#[must_use]
pub(crate) fn beardog_socket_name(family_id: Option<&str>) -> String {
    match family_id {
        Some(fid) if !fid.is_empty() && fid != "default" => format!("beardog-{fid}.sock"),
        _ => "beardog.sock".to_string(),
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
