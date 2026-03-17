// SPDX-License-Identifier: AGPL-3.0-or-later

//! API error types for `LoamSpine` RPC.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// API-level errors for RPC operations.
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ApiError {
    /// Spine not found
    #[error("spine not found: {0}")]
    SpineNotFound(String),

    /// Entry not found
    #[error("entry not found: {0}")]
    EntryNotFound(String),

    /// Certificate not found
    #[error("certificate not found: {0}")]
    CertificateNotFound(String),

    /// Invalid request parameters
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// Permission denied
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Internal error
    #[error("internal error: {0}")]
    Internal(String),

    /// Serialization error
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Transport error
    #[error("transport error: {0}")]
    Transport(String),

    /// Spine is sealed
    #[error("spine is sealed: {0}")]
    SpineSealed(String),

    /// Certificate already exists
    #[error("certificate already exists: {0}")]
    CertificateExists(String),

    /// Not the certificate owner
    #[error("not certificate owner: {0}")]
    NotCertificateOwner(String),
}

impl From<loam_spine_core::error::LoamSpineError> for ApiError {
    fn from(err: loam_spine_core::error::LoamSpineError) -> Self {
        use loam_spine_core::error::LoamSpineError;
        match err {
            LoamSpineError::SpineNotFound(id) => Self::SpineNotFound(format!("{id:?}")),
            LoamSpineError::EntryNotFound(hash) => {
                Self::EntryNotFound(loam_spine_core::types::format_hash_short(&hash))
            }
            LoamSpineError::CertificateNotFound(id) => Self::CertificateNotFound(format!("{id:?}")),
            LoamSpineError::SpineSealed(id) => Self::SpineSealed(format!("{id:?}")),
            LoamSpineError::CertificateExists(id) => Self::CertificateExists(format!("{id:?}")),
            LoamSpineError::NotCertificateOwner => {
                Self::NotCertificateOwner("not certificate owner".to_string())
            }
            LoamSpineError::CertificateLoaned(id) => {
                Self::InvalidRequest(format!("certificate is loaned: {id:?}"))
            }
            LoamSpineError::ChainValidation { index, reason } => {
                Self::InvalidRequest(format!("chain validation at {index}: {reason}"))
            }
            LoamSpineError::Config(msg)
            | LoamSpineError::SignatureVerification(msg)
            | LoamSpineError::InvalidEntryType(msg)
            | LoamSpineError::LoanTermsViolation(msg)
            | LoamSpineError::InvalidData(msg) => Self::InvalidRequest(msg),
            LoamSpineError::Storage(msg)
            | LoamSpineError::Internal(msg)
            | LoamSpineError::CapabilityUnavailable(msg)
            | LoamSpineError::Network(msg) => Self::Internal(msg),
            LoamSpineError::Serialization(msg) => Self::Serialization(msg),
            LoamSpineError::CapabilityProvider {
                capability,
                message,
            } => Self::Internal(format!("capability provider ({capability}): {message}")),
            LoamSpineError::Ipc { phase, message } => {
                Self::Transport(format!("{phase}: {message}"))
            }
            LoamSpineError::EscrowNotFound(id) => {
                Self::InvalidRequest(format!("escrow not found: {id:?}"))
            }
        }
    }
}

/// Result type for API operations.
pub type ApiResult<T> = Result<T, ApiError>;

/// Server startup and transport errors.
///
/// Used by `run_tarpc_server` and `run_jsonrpc_server` for typed error handling
/// instead of `Box<dyn Error>`.
#[derive(Debug, Error)]
pub enum ServerError {
    /// TCP/socket bind failure (e.g. address in use, permission denied).
    #[error("bind failed: {0}")]
    Bind(String),

    /// Transport-layer error (e.g. connection accept, I/O).
    #[error("transport error: {0}")]
    Transport(String),
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "tests use expect for conciseness")]
mod tests {
    use super::*;
    use loam_spine_core::error::LoamSpineError;

    #[test]
    fn api_error_display() {
        let err = ApiError::SpineNotFound("test-id".into());
        assert!(err.to_string().contains("spine not found"));

        let err = ApiError::EntryNotFound("abc123".into());
        assert!(err.to_string().contains("entry not found"));

        let err = ApiError::CertificateNotFound("cert-id".into());
        assert!(err.to_string().contains("certificate not found"));

        let err = ApiError::InvalidRequest("bad input".into());
        assert!(err.to_string().contains("invalid request"));

        let err = ApiError::PermissionDenied("access denied".into());
        assert!(err.to_string().contains("permission denied"));

        let err = ApiError::Internal("oops".into());
        assert!(err.to_string().contains("internal error"));

        let err = ApiError::Serialization("parse failed".into());
        assert!(err.to_string().contains("serialization error"));

        let err = ApiError::Transport("network error".into());
        assert!(err.to_string().contains("transport error"));

        let err = ApiError::SpineSealed("sealed-id".into());
        assert!(err.to_string().contains("spine is sealed"));

        let err = ApiError::CertificateExists("exists-id".into());
        assert!(err.to_string().contains("certificate already exists"));

        let err = ApiError::NotCertificateOwner("not owner".into());
        assert!(err.to_string().contains("not certificate owner"));
    }

    #[test]
    #[expect(
        clippy::cognitive_complexity,
        reason = "exhaustive variant coverage test"
    )]
    fn from_loamspine_error() {
        // SpineNotFound
        let core_err = LoamSpineError::SpineNotFound(uuid::Uuid::nil());
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::SpineNotFound(_)));

        // EntryNotFound
        let core_err = LoamSpineError::EntryNotFound([0u8; 32]);
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::EntryNotFound(_)));

        // CertificateNotFound
        let core_err = LoamSpineError::CertificateNotFound(uuid::Uuid::nil());
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::CertificateNotFound(_)));

        // SpineSealed
        let core_err = LoamSpineError::SpineSealed(uuid::Uuid::nil());
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::SpineSealed(_)));

        // CertificateExists
        let core_err = LoamSpineError::CertificateExists(uuid::Uuid::nil());
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::CertificateExists(_)));

        // NotCertificateOwner
        let core_err = LoamSpineError::NotCertificateOwner;
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::NotCertificateOwner(_)));

        // CertificateLoaned
        let core_err = LoamSpineError::CertificateLoaned(uuid::Uuid::nil());
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::InvalidRequest(_)));

        // ChainValidation
        let core_err = LoamSpineError::ChainValidation {
            index: 5,
            reason: "bad hash".into(),
        };
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::InvalidRequest(_)));

        // Config -> InvalidRequest
        let core_err = LoamSpineError::Config("bad config".into());
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::InvalidRequest(_)));

        // SignatureVerification -> InvalidRequest
        let core_err = LoamSpineError::SignatureVerification("bad sig".into());
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::InvalidRequest(_)));

        // InvalidEntryType -> InvalidRequest
        let core_err = LoamSpineError::InvalidEntryType("wrong type".into());
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::InvalidRequest(_)));

        // LoanTermsViolation -> InvalidRequest
        let core_err = LoamSpineError::LoanTermsViolation("terms broken".into());
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::InvalidRequest(_)));

        // Storage -> Internal
        let core_err = LoamSpineError::Storage("db error".into());
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::Internal(_)));

        // Internal -> Internal
        let core_err = LoamSpineError::Internal("bug".into());
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::Internal(_)));

        // CapabilityUnavailable -> Internal
        let core_err = LoamSpineError::CapabilityUnavailable("signer".into());
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::Internal(_)));

        // Serialization -> Serialization
        let core_err = LoamSpineError::Serialization("json failed".into());
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::Serialization(_)));

        // InvalidData -> InvalidRequest
        let core_err = LoamSpineError::InvalidData("bad data".into());
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::InvalidRequest(_)));

        // Ipc -> Transport
        let core_err =
            LoamSpineError::ipc(loam_spine_core::error::IpcPhase::Connect, "socket timeout");
        let api_err: ApiError = core_err.into();
        assert!(matches!(api_err, ApiError::Transport(_)));
        assert!(api_err.to_string().contains("connect"));
    }

    #[test]
    fn api_error_serialization() {
        let err = ApiError::SpineNotFound("test".into());
        let json = serde_json::to_string(&err).expect("serialize");
        let parsed: ApiError = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, ApiError::SpineNotFound(_)));
    }
}
