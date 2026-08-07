// SPDX-License-Identifier: AGPL-3.0-or-later

//! Platform substrate abstraction (G68).
//!
//! Centralizes platform-specific filesystem and OS operations behind
//! a single API surface.  Primals call functions in this module instead
//! of importing `std::os::unix::*` or `std::os::windows::*` directly.
//!
//! ## Layers
//!
//! | Layer | Module | What |
//! |-------|--------|------|
//! | **L1** | [`fs`] | Filesystem links (symlink / junction) |
//! | **L2** | [`access`] | File permissions / access control |
//!
//! Transport (G66) lives in [`crate::transport::stream`].

pub mod access;
pub mod fs;

pub use access::{PlatformAccess, is_executable, set_executable};
pub use fs::{create_link, remove_link};
