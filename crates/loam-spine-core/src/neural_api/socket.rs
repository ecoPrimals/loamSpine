// SPDX-License-Identifier: AGPL-3.0-or-later

//! Socket path resolution for LoamSpine IPC endpoints.
//!
//! Follows `PRIMAL_SELF_KNOWLEDGE_STANDARD.md` §3 Socket Naming Convention:
//! - Primary socket uses the **capability domain** (`permanence`), not the
//!   primal name (`loamspine`).
//! - Family-scoped: `permanence-{family_id}.sock` when `BIOMEOS_FAMILY_ID` is set.
//! - Development mode: `permanence.sock` when `BIOMEOS_INSECURE=1`.
//! - Legacy symlink: `loamspine.sock → permanence.sock` for backward compat.

use std::path::PathBuf;

use crate::error::LoamSpineError;

/// Resolve the socket path from explicit config values (pure, no env reads).
///
/// Resolution order:
/// 1. `socket_override` (from `LOAMSPINE_SOCKET`)
/// 2. `runtime_dir/biomeos/permanence-{family_id}.sock`
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
    let sock_name = domain_socket_name(family_id);

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

/// Build the domain-based socket filename.
///
/// Per `PRIMAL_SELF_KNOWLEDGE_STANDARD.md` §3:
/// - With family: `permanence-{family_id}.sock`
/// - Without family: `permanence.sock`
#[must_use]
pub fn domain_socket_name(family_id: Option<&str>) -> String {
    match family_id {
        Some(fid) if !fid.is_empty() && fid != "default" => {
            format!("{}-{fid}.sock", crate::primal_names::DOMAIN)
        }
        _ => format!("{}.sock", crate::primal_names::DOMAIN),
    }
}

/// Build the legacy primal-named socket filename for backward compatibility.
///
/// Creates `loamspine.sock` or `loamspine-{family_id}.sock` to support
/// consumers still using identity-based discovery (Tier 5–6).
#[must_use]
pub fn legacy_socket_name(family_id: Option<&str>) -> String {
    match family_id {
        Some(fid) if !fid.is_empty() && fid != "default" => {
            format!("{}-{fid}.sock", crate::primal_names::SELF_ID)
        }
        _ => format!("{}.sock", crate::primal_names::SELF_ID),
    }
}

/// Resolve the legacy symlink path (same directory as the primary socket).
#[must_use]
pub fn resolve_legacy_symlink_path(primary: &std::path::Path, family_id: Option<&str>) -> PathBuf {
    let parent = primary
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    parent.join(legacy_socket_name(family_id))
}

/// Validate the `BIOMEOS_INSECURE` + `FAMILY_ID` invariant.
///
/// Per `PRIMAL_SELF_KNOWLEDGE_STANDARD.md` §3: "If both FAMILY_ID (non-default)
/// and BIOMEOS_INSECURE=1 are set, the primal MUST refuse to start."
///
/// # Errors
///
/// Returns error if conflicting configuration is detected.
pub fn validate_security_config(
    family_id: Option<&str>,
    insecure: Option<&str>,
) -> Result<(), LoamSpineError> {
    let has_family = family_id.is_some_and(|fid| !fid.is_empty() && fid != "default");
    let is_insecure = insecure.is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));

    if has_family && is_insecure {
        return Err(LoamSpineError::Internal(
            "BIOMEOS_INSECURE=1 cannot be used with a non-default FAMILY_ID. \
             Family-scoped sockets require BTSP authentication. \
             Either unset BIOMEOS_INSECURE or use FAMILY_ID=default for development."
                .to_string(),
        ));
    }
    Ok(())
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

/// Validate security config from environment (reads env).
///
/// # Errors
///
/// Returns error if `BIOMEOS_INSECURE=1` is set alongside a non-default `BIOMEOS_FAMILY_ID`.
pub fn validate_security_config_from_env() -> Result<(), LoamSpineError> {
    validate_security_config(
        std::env::var("BIOMEOS_FAMILY_ID").ok().as_deref(),
        std::env::var("BIOMEOS_INSECURE").ok().as_deref(),
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
pub(super) fn resolve_neural_api_socket() -> Option<PathBuf> {
    resolve_neural_api_socket_with(
        std::env::var("BIOMEOS_NEURAL_API_SOCKET").ok().as_deref(),
        std::env::var("XDG_RUNTIME_DIR").ok().as_deref(),
        std::env::var("BIOMEOS_FAMILY_ID").ok().as_deref(),
    )
}
