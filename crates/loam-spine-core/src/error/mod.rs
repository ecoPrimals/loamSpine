// SPDX-License-Identifier: AGPL-3.0-or-later

//! LoamSpine error types.
//!
//! Structured IPC error phases align with the ecosystem convention
//! established by the ecosystem (`IpcErrorPhase`, `SendError`),
//! enabling typed retry logic and observability across primals.

use std::fmt;

use thiserror::Error;

use crate::types::{CertificateId, EntryHash, SpineId, format_hash_short};

mod dispatch;
mod ipc;
mod storage_ext;

pub use dispatch::DispatchOutcome;
pub use ipc::{IpcErrorPhase, IpcPhase};
pub use storage_ext::StorageResultExt;

/// Errors specific to LoamSpine.
#[derive(Debug, Error)]
#[non_exhaustive]
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
    /// Follows the ecosystem `CapabilityProvider` convention for consistency.
    #[error("capability provider error ({capability}): {message}")]
    CapabilityProvider {
        /// The capability that failed.
        capability: String,
        /// Error detail.
        message: String,
    },

    /// Structured IPC error with phase information.
    ///
    /// Aligns with the ecosystem `Ipc { phase, message }` and `SendError`
    /// conventions for cross-primal typed IPC error handling. Enables
    /// phase-aware retry logic and observability.
    #[error("ipc error ({phase}): {message}")]
    Ipc {
        /// Phase of the IPC call that failed.
        phase: IpcPhase,
        /// Human-readable error detail.
        message: String,
    },

    /// Network error (non-IPC: configuration, transport selection).
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

    /// Create a structured IPC error.
    #[must_use]
    pub fn ipc(phase: IpcErrorPhase, message: impl Into<String>) -> Self {
        Self::Ipc {
            phase,
            message: message.into(),
        }
    }

    /// Whether this error represents a transient failure worth retrying.
    ///
    /// IPC errors at Connect, Write, Read, and HttpStatus(5xx) phases are
    /// considered recoverable. JsonRpcError and NoResult are not (they
    /// indicate a logic or protocol mismatch, not a transient failure).
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        match self {
            Self::Ipc { phase, .. } => matches!(
                phase,
                IpcErrorPhase::Connect
                    | IpcErrorPhase::Write
                    | IpcErrorPhase::Read
                    | IpcErrorPhase::HttpStatus(500..=599)
            ),
            Self::CapabilityUnavailable(_) | Self::Network(_) => true,
            _ => false,
        }
    }

    /// Whether this is a "method not found" IPC error (JSON-RPC -32601).
    #[must_use]
    pub const fn is_method_not_found(&self) -> bool {
        matches!(
            self,
            Self::Ipc {
                phase: IpcErrorPhase::JsonRpcError(-32601),
                ..
            }
        )
    }

    /// Whether this error likely indicates a timeout (Connect, Read, Write phases).
    ///
    /// Follows the ecosystem `is_timeout_likely()` convention for consistency.
    #[must_use]
    pub const fn is_timeout_likely(&self) -> bool {
        matches!(
            self,
            Self::Ipc {
                phase: IpcErrorPhase::Connect | IpcErrorPhase::Read | IpcErrorPhase::Write,
                ..
            }
        )
    }

    /// Whether this is an application-level JSON-RPC error (as opposed to protocol).
    ///
    /// Returns `true` for `IpcPhase::JsonRpcError(_)` — the remote primal
    /// understood the request but returned an error object.
    #[must_use]
    pub const fn is_application_error(&self) -> bool {
        matches!(
            self,
            Self::Ipc {
                phase: IpcErrorPhase::JsonRpcError(_),
                ..
            }
        )
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// JSON-RPC helpers — centralized extraction from response payloads
// ──────────────────────────────────────────────────────────────────────────────

/// Extracts `(code, message)` from a JSON-RPC 2.0 error object.
///
/// Centralizes the pattern used by every IPC adapter to parse the `error`
/// field from a JSON-RPC 2.0 response. Returns `None` if no error is present.
///
/// Follows the ecosystem `extract_rpc_error` convention for consistency.
#[must_use]
pub fn extract_rpc_error(response: &serde_json::Value) -> Option<(i64, String)> {
    let error = response.get("error")?;
    let code = error
        .get("code")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(-1);
    let message = error
        .get("message")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Unknown error")
        .to_string();
    Some((code, message))
}

/// Extracts a typed result from a JSON-RPC 2.0 response.
///
/// Returns `Ok(result_value)` if the response contains a `result` field,
/// or `Err(LoamSpineError::Ipc)` if it contains an `error` field or
/// is malformed. This is the counterpart to [`extract_rpc_error`].
///
/// Aligns with the ecosystem `extract_rpc_result` / `classify_response`
/// pattern for consistent outbound RPC handling.
///
/// # Errors
///
/// - `IpcErrorPhase::JsonRpcError(code)` if the response contains an `error` object
/// - `IpcErrorPhase::NoResult` if neither `result` nor `error` is present
pub fn extract_rpc_result(response: &serde_json::Value) -> LoamSpineResult<&serde_json::Value> {
    if let Some((code, message)) = extract_rpc_error(response) {
        return Err(LoamSpineError::ipc(
            IpcErrorPhase::JsonRpcError(code),
            message,
        ));
    }
    response
        .get("result")
        .ok_or_else(|| LoamSpineError::ipc(IpcErrorPhase::NoResult, "response missing 'result'"))
}

/// Extracts and deserializes a typed result from a JSON-RPC 2.0 response.
///
/// Combines [`extract_rpc_result`] with `serde_json::from_value` for the
/// common pattern of extracting and deserializing in a single step.
///
/// # Errors
///
/// - `IpcErrorPhase::JsonRpcError(code)` if the response contains an error
/// - `IpcErrorPhase::NoResult` if neither `result` nor `error` is present
/// - `IpcErrorPhase::InvalidJson` if deserialization fails
pub fn extract_rpc_result_typed<T: serde::de::DeserializeOwned>(
    response: &serde_json::Value,
) -> LoamSpineResult<T> {
    let result = extract_rpc_result(response)?;
    T::deserialize(result).map_err(|e| {
        LoamSpineError::ipc(
            IpcErrorPhase::InvalidJson,
            format!("result deserialization failed: {e}"),
        )
    })
}

// ──────────────────────────────────────────────────────────────────────────────
// OrExit — zero-panic startup validation
// ──────────────────────────────────────────────────────────────────────────────

/// Extension trait for `Result<T, E>` and `Option<T>` that exits the
/// process cleanly on error instead of panicking.
///
/// Follows the ecosystem `OrExit` pattern. Validation
/// and startup code should never panic — it should print a structured
/// error message and exit with a non-zero status code.
///
/// # Examples
///
/// ```no_run
/// use loam_spine_core::error::OrExit;
///
/// let config = std::fs::read_to_string("config.toml")
///     .or_exit("Failed to read configuration file");
/// ```
pub trait OrExit<T> {
    /// Unwrap the value or print the context message + error and exit with code 1.
    fn or_exit(self, context: &str) -> T;
}

impl<T, E: fmt::Display> OrExit<T> for std::result::Result<T, E> {
    fn or_exit(self, context: &str) -> T {
        match self {
            Ok(val) => val,
            Err(e) => {
                tracing::error!("fatal: {context}: {e}");
                std::process::exit(1);
            }
        }
    }
}

impl<T> OrExit<T> for Option<T> {
    fn or_exit(self, context: &str) -> T {
        if let Some(val) = self {
            return val;
        }
        tracing::error!("fatal: {context}");
        std::process::exit(1);
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
#[path = "tests.rs"]
mod tests;
