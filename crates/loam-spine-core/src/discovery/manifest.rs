// SPDX-License-Identifier: AGPL-3.0-or-later

//! Manifest-based primal discovery fallback.
//!
//! Discovers peer primals by scanning `$XDG_RUNTIME_DIR/ecoPrimals/*.json`
//! for primal manifest files. Each manifest advertises a primal's name,
//! socket path, capabilities, and PID.
//!
//! Aligns with rhizoCrypt S16's manifest discovery pattern — a local,
//! zero-network fallback when mDNS or DNS SRV is unavailable.

use std::path::{Path, PathBuf};

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

// ──────────────────────────────────────────────────────────────────────────────
// Inner pure functions (no env reads)
// ──────────────────────────────────────────────────────────────────────────────

/// Resolve the manifest directory from a known base path.
///
/// Returns `base/ecoPrimals/` if the directory exists, otherwise `None`.
#[must_use]
pub fn manifest_dir_from(base: &Path) -> Option<PathBuf> {
    let dir = base.join("ecoPrimals");
    if dir.is_dir() { Some(dir) } else { None }
}

/// Discover all primal manifests under `base/ecoPrimals/*.json`.
#[must_use]
pub fn discover_manifests_from(base: &Path) -> Vec<PrimalManifest> {
    let Some(dir) = manifest_dir_from(base) else {
        return Vec::new();
    };
    scan_manifest_dir(&dir)
}

/// Find a primal manifest by capability under a given base path.
#[must_use]
pub fn find_by_capability_from(base: &Path, capability: &str) -> Option<PrimalManifest> {
    discover_manifests_from(base)
        .into_iter()
        .find(|m| m.capabilities.iter().any(|c| c == capability))
}

/// Find a primal manifest by name under a given base path.
#[must_use]
pub fn find_by_name_from(base: &Path, name: &str) -> Option<PrimalManifest> {
    discover_manifests_from(base)
        .into_iter()
        .find(|m| m.name == name)
}

// ──────────────────────────────────────────────────────────────────────────────
// Outer wrappers (read env, delegate)
// ──────────────────────────────────────────────────────────────────────────────

/// Resolve the manifest directory for primal discovery.
///
/// Returns `$XDG_RUNTIME_DIR/ecoPrimals/` if the env var is set and the
/// directory exists, otherwise `None`.
#[must_use]
pub fn manifest_dir() -> Option<PathBuf> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").ok()?;
    manifest_dir_from(Path::new(&runtime_dir))
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
    scan_manifest_dir(&dir)
}

/// Find a primal manifest by capability.
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

fn scan_manifest_dir(dir: &Path) -> Vec<PrimalManifest> {
    let Ok(entries) = std::fs::read_dir(dir) else {
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
    fn manifest_dir_from_returns_some_when_directory_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let eco_dir = tmp.path().join("ecoPrimals");
        std::fs::create_dir_all(&eco_dir).unwrap();
        let result = manifest_dir_from(tmp.path());
        assert!(result.is_some());
        assert_eq!(result.unwrap(), eco_dir);
    }

    #[test]
    fn manifest_dir_from_returns_none_when_directory_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let result = manifest_dir_from(tmp.path());
        assert!(
            result.is_none(),
            "should be None when ecoPrimals/ doesn't exist"
        );
    }

    #[test]
    fn discover_manifests_from_finds_valid_json_files() {
        let tmp = tempfile::tempdir().unwrap();
        let eco_dir = tmp.path().join("ecoPrimals");
        std::fs::create_dir_all(&eco_dir).unwrap();

        let manifest_json = r#"{"name":"testPrimal","socket_path":"/tmp/test.sock","pid":42,"capabilities":["signing"],"version":"1.0.0"}"#;
        std::fs::write(eco_dir.join("testPrimal.json"), manifest_json).unwrap();

        let manifests = discover_manifests_from(tmp.path());
        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].name, "testPrimal");
        assert_eq!(manifests[0].pid, Some(42));
        assert_eq!(manifests[0].capabilities, vec!["signing"]);
    }

    #[test]
    fn discover_manifests_from_skips_invalid_json() {
        let tmp = tempfile::tempdir().unwrap();
        let eco_dir = tmp.path().join("ecoPrimals");
        std::fs::create_dir_all(&eco_dir).unwrap();

        std::fs::write(eco_dir.join("valid.json"), r#"{"name":"valid"}"#).unwrap();
        std::fs::write(eco_dir.join("invalid.json"), "not json at all").unwrap();
        std::fs::write(eco_dir.join("readme.txt"), "not a manifest").unwrap();

        let manifests = discover_manifests_from(tmp.path());
        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].name, "valid");
    }

    #[test]
    fn discover_manifests_from_skips_non_json_files() {
        let tmp = tempfile::tempdir().unwrap();
        let eco_dir = tmp.path().join("ecoPrimals");
        std::fs::create_dir_all(&eco_dir).unwrap();

        std::fs::write(eco_dir.join("manifest.toml"), "[primal]\nname=\"test\"").unwrap();
        std::fs::write(eco_dir.join("readme.md"), "# Manifests").unwrap();

        let manifests = discover_manifests_from(tmp.path());
        assert!(manifests.is_empty());
    }

    #[test]
    fn discover_manifests_from_handles_multiple_manifests() {
        let tmp = tempfile::tempdir().unwrap();
        let eco_dir = tmp.path().join("ecoPrimals");
        std::fs::create_dir_all(&eco_dir).unwrap();

        std::fs::write(
            eco_dir.join("primal_a.json"),
            r#"{"name":"primalA","capabilities":["signing","verification"]}"#,
        )
        .unwrap();
        std::fs::write(
            eco_dir.join("primal_b.json"),
            r#"{"name":"primalB","capabilities":["storage"]}"#,
        )
        .unwrap();
        std::fs::write(
            eco_dir.join("primal_c.json"),
            r#"{"name":"primalC","capabilities":["compute"]}"#,
        )
        .unwrap();

        let manifests = discover_manifests_from(tmp.path());
        assert_eq!(manifests.len(), 3);
        let names: Vec<&str> = manifests.iter().map(|m| m.name.as_str()).collect();
        assert!(names.contains(&"primalA"));
        assert!(names.contains(&"primalB"));
        assert!(names.contains(&"primalC"));
    }

    #[test]
    fn find_by_capability_from_returns_matching_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let eco_dir = tmp.path().join("ecoPrimals");
        std::fs::create_dir_all(&eco_dir).unwrap();

        std::fs::write(
            eco_dir.join("signer.json"),
            r#"{"name":"signerPrimal","capabilities":["signing","verification"]}"#,
        )
        .unwrap();
        std::fs::write(
            eco_dir.join("storage.json"),
            r#"{"name":"storagePrimal","capabilities":["storage"]}"#,
        )
        .unwrap();

        assert_eq!(
            find_by_capability_from(tmp.path(), "signing").unwrap().name,
            "signerPrimal"
        );
        assert_eq!(
            find_by_capability_from(tmp.path(), "storage").unwrap().name,
            "storagePrimal"
        );
        assert!(find_by_capability_from(tmp.path(), "nonexistent").is_none());
    }

    #[test]
    fn find_by_name_from_returns_matching_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let eco_dir = tmp.path().join("ecoPrimals");
        std::fs::create_dir_all(&eco_dir).unwrap();

        std::fs::write(
            eco_dir.join("target.json"),
            r#"{"name":"targetPrimal","socket_path":"/tmp/target.sock","capabilities":["signing"]}"#,
        )
        .unwrap();

        let m = find_by_name_from(tmp.path(), "targetPrimal").unwrap();
        assert_eq!(m.name, "targetPrimal");
        assert_eq!(m.socket_path.as_deref(), Some("/tmp/target.sock"));

        assert!(find_by_name_from(tmp.path(), "nonexistent").is_none());
    }

    #[test]
    fn discover_manifests_from_returns_empty_when_no_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let manifests = discover_manifests_from(tmp.path());
        assert!(manifests.is_empty());
    }

    #[test]
    fn manifest_clone_and_debug() {
        let json = r#"{"name":"test","capabilities":["a","b"]}"#;
        let manifest: PrimalManifest = serde_json::from_str(json).unwrap();
        let cloned = manifest.clone();
        assert_eq!(cloned.name, "test");
        let debug = format!("{manifest:?}");
        assert!(debug.contains("test"));
    }
}
