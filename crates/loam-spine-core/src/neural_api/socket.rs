// SPDX-License-Identifier: AGPL-3.0-or-later

//! Socket path resolution for LoamSpine IPC endpoints.

use std::path::PathBuf;

/// Resolve the socket path from explicit config values (pure, no env reads).
///
/// Resolution order:
/// 1. `socket_override` (from `LOAMSPINE_SOCKET`)
/// 2. `runtime_dir/biomeos/loamspine-{family_id}.sock`
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
    let sock_name = match family_id {
        Some(fid) if !fid.is_empty() => {
            format!("{}-{fid}.sock", crate::primal_names::SELF_ID)
        }
        _ => format!("{}.sock", crate::primal_names::SELF_ID),
    };

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

/// Resolve the socket path for LoamSpine's IPC endpoint (reads env).
#[must_use]
pub fn resolve_socket_path() -> PathBuf {
    resolve_socket_path_with(
        std::env::var("LOAMSPINE_SOCKET").ok().as_deref(),
        std::env::var("BIOMEOS_FAMILY_ID").ok().as_deref(),
        std::env::var("XDG_RUNTIME_DIR").ok().as_deref(),
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
