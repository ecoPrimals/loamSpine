// SPDX-License-Identifier: AGPL-3.0-only

//! LoamSpine error types.

use thiserror::Error;

use crate::types::{format_hash_short, CertificateId, EntryHash, SpineId};

/// Errors specific to LoamSpine.
#[derive(Debug, Error)]
pub enum LoamSpineError {
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Spine not found.
    #[error("spine not found: {0}")]
    SpineNotFound(SpineId),

    /// Entry not found.
    #[error("entry not found: {}", format_hash_short(.0))]
    EntryNotFound(EntryHash),

    /// Certificate not found.
    #[error("certificate not found: {0}")]
    CertificateNotFound(CertificateId),

    /// Chain validation error.
    #[error("chain validation error at index {index}: {reason}")]
    ChainValidation {
        /// Entry index where validation failed.
        index: u64,
        /// Reason for validation failure.
        reason: String,
    },

    /// Signature verification failed.
    #[error("signature verification failed: {0}")]
    SignatureVerification(String),

    /// Spine is sealed (read-only).
    #[error("spine is sealed: {0}")]
    SpineSealed(SpineId),

    /// Invalid entry type for operation.
    #[error("invalid entry type: {0}")]
    InvalidEntryType(String),

    /// Certificate already exists.
    #[error("certificate already exists: {0}")]
    CertificateExists(CertificateId),

    /// Certificate is not owned by caller.
    #[error("not certificate owner")]
    NotCertificateOwner,

    /// Certificate is currently loaned.
    #[error("certificate is loaned: {0}")]
    CertificateLoaned(CertificateId),

    /// Loan terms violation.
    #[error("loan terms violation: {0}")]
    LoanTermsViolation(String),

    /// Storage error.
    #[error("storage error: {0}")]
    Storage(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),

    /// Capability unavailable (runtime discovery).
    #[error("capability unavailable: {0}")]
    CapabilityUnavailable(String),

    /// Network error (Songbird, HTTP, etc.).
    #[error("network error: {0}")]
    Network(String),
}

/// Result type for LoamSpine operations.
pub type LoamSpineResult<T> = Result<T, LoamSpineError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let spine_id = SpineId::nil();
        let err = LoamSpineError::SpineNotFound(spine_id);
        assert!(err.to_string().contains("spine not found"));
    }

    #[test]
    fn chain_validation_error() {
        let err = LoamSpineError::ChainValidation {
            index: 42,
            reason: "hash mismatch".into(),
        };
        assert!(err.to_string().contains("42"));
        assert!(err.to_string().contains("hash mismatch"));
    }
}
