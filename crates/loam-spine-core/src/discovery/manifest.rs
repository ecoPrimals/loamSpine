// SPDX-License-Identifier: AGPL-3.0-or-later

//! Manifest-based primal discovery fallback.
//!
//! Discovers peer primals by scanning `$XDG_RUNTIME_DIR/ecoPrimals/*.json`
//! for primal manifest files. Each manifest advertises a primal's name,
//! socket path, capabilities, and PID.
//!
//! Aligns with rhizoCrypt S16's manifest discovery pattern — a local,
//! zero-network fallback when Songbird or mDNS is unavailable.

use std::path::PathBuf;

use serde::Deserialize;

/// A primal manifest discovered on the local filesystem.
#[derive(Debug, Clone, Deserialize)]
pub struct PrimalManifest {
    /// Primal name (e.g. `"rhizoCrypt"`, `"sweetGrass"`).
    pub name: String,
    /// Unix socket or TCP endpoint for IPC.
    pub socket_path: Option<String>,
    /// Process ID of the running primal.
    pub pid: Option<u32>,
    /// Flat capability list.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Version string.
    pub version: Option<String>,
}

/// Resolve the manifest directory for primal discovery.
///
/// Returns `$XDG_RUNTIME_DIR/ecoPrimals/` if the env var is set and the
/// directory exists, otherwise `None`.
#[must_use]
pub fn manifest_dir() -> Option<PathBuf> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").ok()?;
    let dir = PathBuf::from(runtime_dir).join("ecoPrimals");
    if dir.is_dir() { Some(dir) } else { None }
}

/// Discover all primal manifests in the manifest directory.
///
/// Scans `$XDG_RUNTIME_DIR/ecoPrimals/*.json` and parses each file
/// as a `PrimalManifest`. Invalid or unparseable files are silently
/// skipped (defensive — a malformed neighbor should not crash us).
#[must_use]
pub fn discover_manifests() -> Vec<PrimalManifest> {
    let Some(dir) = manifest_dir() else {
        return Vec::new();
    };

    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut manifests = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json")
            && let Ok(contents) = std::fs::read_to_string(&path)
            && let Ok(manifest) = serde_json::from_str::<PrimalManifest>(&contents)
        {
            manifests.push(manifest);
        }
    }
    manifests
}

/// Find a primal manifest by capability.
///
/// Returns the first manifest that advertises the given capability string.
#[must_use]
pub fn find_by_capability(capability: &str) -> Option<PrimalManifest> {
    discover_manifests()
        .into_iter()
        .find(|m| m.capabilities.iter().any(|c| c == capability))
}

/// Find a primal manifest by name.
#[must_use]
pub fn find_by_name(name: &str) -> Option<PrimalManifest> {
    discover_manifests().into_iter().find(|m| m.name == name)
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests {
    use super::*;

    #[test]
    fn parse_manifest_json() {
        let json = r#"{
            "name": "rhizoCrypt",
            "socket_path": "/run/user/1000/ecoPrimals/rhizocrypt.sock",
            "pid": 12345,
            "capabilities": ["signing", "verification", "key-management"],
            "version": "0.13.0"
        }"#;
        let manifest: PrimalManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.name, "rhizoCrypt");
        assert_eq!(manifest.pid, Some(12345));
        assert_eq!(manifest.capabilities.len(), 3);
    }

    #[test]
    fn parse_minimal_manifest() {
        let json = r#"{ "name": "sweetGrass" }"#;
        let manifest: PrimalManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.name, "sweetGrass");
        assert!(manifest.socket_path.is_none());
        assert!(manifest.capabilities.is_empty());
    }

    #[test]
    fn manifest_dir_without_env() {
        // Without XDG_RUNTIME_DIR set to a real dir, returns None
        // (safe to run in CI where the env var may or may not exist)
        let result = manifest_dir();
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn discover_manifests_returns_vec() {
        let manifests = discover_manifests();
        // Should return empty vec if no manifest dir exists
        let _ = manifests;
    }
}
