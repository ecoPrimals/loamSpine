// SPDX-License-Identifier: AGPL-3.0-or-later

//! LoamSpine error types.

use thiserror::Error;

use crate::types::{CertificateId, EntryHash, SpineId, format_hash_short};

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

    /// Escrow not found.
    #[error("escrow not found: {0}")]
    EscrowNotFound(uuid::Uuid),

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

    /// Capability provider error (structured, vendor-agnostic).
    ///
    /// Used when a capability (signing, storage, etc.) fails at the provider level.
    /// Matches rhizoCrypt's `CapabilityProvider` for ecosystem consistency.
    #[error("capability provider error ({capability}): {message}")]
    CapabilityProvider {
        /// The capability that failed.
        capability: String,
        /// Error detail.
        message: String,
    },

    /// Network error (service registry, HTTP, etc.).
    #[error("network error: {0}")]
    Network(String),

    /// Invalid data (conversion, parsing, validation).
    #[error("invalid data: {0}")]
    InvalidData(String),
}

/// Result type for LoamSpine operations.
pub type LoamSpineResult<T> = Result<T, LoamSpineError>;

impl LoamSpineError {
    /// Create a capability provider error.
    #[must_use]
    pub fn capability_provider(capability: impl Into<String>, message: impl Into<String>) -> Self {
        Self::CapabilityProvider {
            capability: capability.into(),
            message: message.into(),
        }
    }
}

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

    #[test]
    fn capability_provider_error() {
        let err = LoamSpineError::capability_provider("signing", "HSM unavailable");
        assert!(err.to_string().contains("capability provider error"));
        assert!(err.to_string().contains("signing"));
        assert!(err.to_string().contains("HSM unavailable"));
    }
}
