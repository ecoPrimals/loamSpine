// SPDX-License-Identifier: AGPL-3.0-or-later

//! Platform-abstracted file access control (G68 L2).
//!
//! Replaces direct `PermissionsExt::from_mode()` / `.mode()` calls
//! so that no code imports `std::os::unix::fs::PermissionsExt`.
//!
//! | Platform | `set_executable` | `is_executable` | `PlatformAccess::apply` |
//! |----------|------------------|-----------------|------------------------|
//! | Unix | `chmod 0o755` | `mode & 0o111 != 0` | mode bits |
//! | Windows | no-op (extension-based) | `!readonly` | readonly flag |
//! | Other | no-op | `!readonly` | readonly flag |

use std::io;
use std::path::Path;

/// Semantic access levels for file permissions.
///
/// Abstracts Unix mode bits and Windows ACLs into a small
/// vocabulary that covers primal use cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformAccess {
    /// Readable + writable + executable (Unix `0o755`).
    Executable,
    /// Readable only (Unix `0o444`, Windows `readonly = true`).
    ReadOnly,
    /// Readable + writable (Unix `0o644`, Windows `readonly = false`).
    ReadWrite,
}

impl PlatformAccess {
    /// Apply this access level to a file.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if permissions cannot be set.
    pub fn apply(self, path: &Path) -> io::Result<()> {
        apply_access(self, path)
    }
}

/// Make a file executable.
///
/// Convenience wrapper for `PlatformAccess::Executable.apply(path)`.
///
/// - **Unix**: sets mode `0o755`
/// - **Windows/Other**: no-op (executability determined by extension)
///
/// # Errors
///
/// Returns `io::Error` on permission failure.
pub fn set_executable(path: &Path) -> io::Result<()> {
    PlatformAccess::Executable.apply(path)
}

/// Check whether a file is executable.
///
/// - **Unix**: checks `mode & 0o111 != 0`
/// - **Windows/Other**: checks `!readonly` (heuristic)
///
/// # Errors
///
/// Returns `io::Error` if metadata cannot be read.
pub fn is_executable(path: &Path) -> io::Result<bool> {
    is_executable_impl(path)
}

#[cfg(unix)]
fn apply_access(access: PlatformAccess, path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = match access {
        PlatformAccess::Executable => 0o755,
        PlatformAccess::ReadOnly => 0o444,
        PlatformAccess::ReadWrite => 0o644,
    };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn apply_access(access: PlatformAccess, path: &Path) -> io::Result<()> {
    let mut perms = std::fs::metadata(path)?.permissions();
    match access {
        PlatformAccess::ReadOnly => perms.set_readonly(true),
        PlatformAccess::Executable | PlatformAccess::ReadWrite => perms.set_readonly(false),
    }
    std::fs::set_permissions(path, perms)
}

#[cfg(unix)]
fn is_executable_impl(path: &Path) -> io::Result<bool> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable_impl(path: &Path) -> io::Result<bool> {
    let metadata = std::fs::metadata(path)?;
    Ok(!metadata.permissions().readonly())
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests {
    use super::*;

    #[test]
    fn set_and_check_executable() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("script.sh");
        std::fs::write(&file, "#!/bin/sh\nexit 0").unwrap();

        set_executable(&file).unwrap();
        assert!(is_executable(&file).unwrap());
    }

    #[test]
    fn apply_read_only() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("data.txt");
        std::fs::write(&file, "content").unwrap();

        PlatformAccess::ReadOnly.apply(&file).unwrap();
        let perms = std::fs::metadata(&file).unwrap().permissions();
        assert!(perms.readonly());

        PlatformAccess::ReadWrite.apply(&file).unwrap();
        let perms = std::fs::metadata(&file).unwrap().permissions();
        assert!(!perms.readonly());
    }

    #[test]
    fn is_executable_nonexistent_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("nope");
        assert!(is_executable(&missing).is_err());
    }

    #[test]
    fn platform_access_variants_are_distinct() {
        assert_ne!(PlatformAccess::Executable, PlatformAccess::ReadOnly);
        assert_ne!(PlatformAccess::ReadOnly, PlatformAccess::ReadWrite);
        assert_ne!(PlatformAccess::Executable, PlatformAccess::ReadWrite);
    }

    #[test]
    fn apply_executable_makes_file_executable() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("tool");
        std::fs::write(&file, "data").unwrap();

        PlatformAccess::Executable.apply(&file).unwrap();
        assert!(is_executable(&file).unwrap());
    }
}
