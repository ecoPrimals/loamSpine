//! LoamSpine error types.

use thiserror::Error;

/// Errors specific to LoamSpine.
#[derive(Debug, Error)]
pub enum LoamSpineError {
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),
    
    // TODO: Add LoamSpine-specific errors
    
    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}
