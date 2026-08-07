// SPDX-License-Identifier: AGPL-3.0-or-later

//! Platform-abstracted filesystem link operations (G68 L1).
//!
//! Replaces direct `std::os::unix::fs::symlink` calls so that
//! no primal binary imports raw OS modules.
//!
//! | Platform | `create_link` | `remove_link` |
//! |----------|---------------|---------------|
//! | Unix | `symlink(target, link)` | `remove_file` |
//! | Windows | `symlink_file(target, link)` | `remove_file` |
//! | Other | Returns `Unsupported` error | `remove_file` |

use std::io;
use std::path::Path;

/// Create a filesystem link from `link` pointing to `target`.
///
/// # Platform behaviour
///
/// - **Unix**: creates a symbolic link via `std::os::unix::fs::symlink`.
/// - **Windows**: creates a file-level symlink via `std::os::windows::fs::symlink_file`.
///   Requires developer mode or elevated privileges.
/// - **Other**: returns `io::ErrorKind::Unsupported`.
///
/// # Errors
///
/// Returns `io::Error` on filesystem failure or platform unavailability.
pub fn create_link(target: &Path, link: &Path) -> io::Result<()> {
    create_link_impl(target, link)
}

/// Remove a filesystem link (symlink, junction, or regular file).
///
/// Delegates to `std::fs::remove_file`, which handles both symlinks
/// and regular files on all platforms.
///
/// # Errors
///
/// Returns `io::Error` if the file does not exist or cannot be removed.
pub fn remove_link(path: &Path) -> io::Result<()> {
    std::fs::remove_file(path)
}

#[cfg(unix)]
fn create_link_impl(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_link_impl(target: &Path, link: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

#[cfg(not(any(unix, windows)))]
fn create_link_impl(_target: &Path, _link: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "filesystem links not available on this platform",
    ))
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests {
    use super::*;

    #[test]
    fn create_and_remove_link() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("target.txt");
        std::fs::write(&target, "hello").unwrap();

        let link = tmp.path().join("link.txt");
        create_link(&target, &link).unwrap();

        assert!(link.exists() || link.symlink_metadata().is_ok());
        assert_eq!(std::fs::read_to_string(&link).unwrap(), "hello");

        remove_link(&link).unwrap();
        assert!(!link.exists());
    }

    #[test]
    fn create_link_target_missing_still_creates() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("nonexistent");
        let link = tmp.path().join("dangling");

        #[cfg(any(unix, windows))]
        {
            create_link(&target, &link).unwrap();
            assert!(link.symlink_metadata().is_ok());
            remove_link(&link).unwrap();
        }

        #[cfg(not(any(unix, windows)))]
        {
            assert!(create_link(&target, &link).is_err());
        }
    }

    #[test]
    fn remove_link_nonexistent_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("does_not_exist");
        assert!(remove_link(&path).is_err());
    }

    #[test]
    fn create_link_overwrites_after_remove() {
        let tmp = tempfile::tempdir().unwrap();
        let target1 = tmp.path().join("a.txt");
        let target2 = tmp.path().join("b.txt");
        std::fs::write(&target1, "first").unwrap();
        std::fs::write(&target2, "second").unwrap();

        let link = tmp.path().join("alias");
        create_link(&target1, &link).unwrap();
        assert_eq!(std::fs::read_to_string(&link).unwrap(), "first");

        remove_link(&link).unwrap();
        create_link(&target2, &link).unwrap();
        assert_eq!(std::fs::read_to_string(&link).unwrap(), "second");

        remove_link(&link).unwrap();
    }
}
