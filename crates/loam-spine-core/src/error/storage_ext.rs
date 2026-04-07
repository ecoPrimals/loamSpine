// SPDX-License-Identifier: AGPL-3.0-or-later

//! Storage error conversion extension trait.

use super::{LoamSpineError, LoamSpineResult};

/// Extension trait for converting storage-layer errors into [`LoamSpineError::Storage`].
///
/// Replaces the verbose `.map_err(|e| LoamSpineError::Storage(e.to_string()))`
/// pattern with `.storage_err()` or `.storage_ctx("context")`.
pub trait StorageResultExt<T> {
    /// Convert the error into `LoamSpineError::Storage` using its `Display` impl.
    ///
    /// # Errors
    ///
    /// Returns [`LoamSpineError::Storage`] wrapping the original error's display text.
    fn storage_err(self) -> LoamSpineResult<T>;

    /// Convert the error into `LoamSpineError::Storage` with additional context.
    ///
    /// # Errors
    ///
    /// Returns [`LoamSpineError::Storage`] with `"{ctx}: {error}"` message.
    fn storage_ctx(self, ctx: &str) -> LoamSpineResult<T>;
}

impl<T, E: std::fmt::Display> StorageResultExt<T> for Result<T, E> {
    fn storage_err(self) -> LoamSpineResult<T> {
        self.map_err(|e| LoamSpineError::Storage(e.to_string()))
    }

    fn storage_ctx(self, ctx: &str) -> LoamSpineResult<T> {
        self.map_err(|e| LoamSpineError::Storage(format!("{ctx}: {e}")))
    }
}
