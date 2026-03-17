// SPDX-License-Identifier: AGPL-3.0-or-later

//! LoamSpine error types.
//!
//! Structured IPC error phases align with the ecosystem convention
//! established by rhizoCrypt (`IpcErrorPhase`) and healthSpring (`SendError`),
//! enabling typed retry logic and observability across primals.

use std::fmt;

use thiserror::Error;

use crate::types::{CertificateId, EntryHash, SpineId, format_hash_short};

/// Phase of an IPC call that failed.
///
/// Aligns with rhizoCrypt's `IpcErrorPhase` and healthSpring's `SendError`
/// for ecosystem-wide structured IPC error handling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpcPhase {
    /// Socket or TCP connection failed (primal unreachable).
    Connect,
    /// Request write to socket/stream failed (broken pipe, timeout).
    Write,
    /// Response read from socket/stream failed (timeout, truncated).
    Read,
    /// Response is not valid JSON.
    InvalidJson,
    /// HTTP response status was not 2xx.
    HttpStatus(u16),
    /// Response lacks a `result` field (JSON-RPC protocol violation).
    NoResult,
    /// JSON-RPC error object returned by the remote primal.
    JsonRpcError(i64),
    /// Request serialization failed before sending.
    Serialization,
}

impl fmt::Display for IpcPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect => write!(f, "connect"),
            Self::Write => write!(f, "write"),
            Self::Read => write!(f, "read"),
            Self::InvalidJson => write!(f, "invalid_json"),
            Self::HttpStatus(code) => write!(f, "http_{code}"),
            Self::NoResult => write!(f, "no_result"),
            Self::JsonRpcError(code) => write!(f, "jsonrpc_{code}"),
            Self::Serialization => write!(f, "serialization"),
        }
    }
}

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

    /// Structured IPC error with phase information.
    ///
    /// Aligns with rhizoCrypt's `Ipc { phase, message }` and healthSpring's
    /// `SendError` for ecosystem-wide typed IPC error handling. Enables
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
    pub fn ipc(phase: IpcPhase, message: impl Into<String>) -> Self {
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
                IpcPhase::Connect
                    | IpcPhase::Write
                    | IpcPhase::Read
                    | IpcPhase::HttpStatus(500..=599)
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
                phase: IpcPhase::JsonRpcError(-32601),
                ..
            }
        )
    }

    /// Whether this error likely indicates a timeout (Connect, Read, Write phases).
    ///
    /// Aligns with sweetGrass's `is_timeout_likely()` for ecosystem consistency.
    #[must_use]
    pub const fn is_timeout_likely(&self) -> bool {
        matches!(
            self,
            Self::Ipc {
                phase: IpcPhase::Connect | IpcPhase::Read | IpcPhase::Write,
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
                phase: IpcPhase::JsonRpcError(_),
                ..
            }
        )
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// DispatchOutcome — protocol vs application error separation
// ──────────────────────────────────────────────────────────────────────────────

/// Outcome of a dispatched JSON-RPC call, separating protocol errors
/// from application results.
///
/// Absorbed from rhizoCrypt / airSpring / biomeOS dispatch patterns.
/// Protocol errors (transport failures, malformed responses) are
/// fundamentally different from application errors (method returned an
/// error object). Callers can pattern-match to decide retry strategy.
#[derive(Debug)]
pub enum DispatchOutcome<T> {
    /// The call succeeded and returned a result.
    Ok(T),
    /// The remote primal returned a JSON-RPC error object.
    ApplicationError {
        /// JSON-RPC error code.
        code: i64,
        /// Human-readable error message.
        message: String,
    },
    /// A transport or protocol-level failure occurred.
    ProtocolError(LoamSpineError),
}

impl<T> DispatchOutcome<T> {
    /// Returns `true` if the outcome is a successful result.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Returns `true` if the outcome is an application-level error.
    #[must_use]
    pub const fn is_application_error(&self) -> bool {
        matches!(self, Self::ApplicationError { .. })
    }

    /// Convert into a `Result`, folding both error variants into `LoamSpineError`.
    ///
    /// # Errors
    ///
    /// Returns `LoamSpineError::Ipc` for application errors and the
    /// original `LoamSpineError` for protocol errors.
    pub fn into_result(self) -> LoamSpineResult<T> {
        match self {
            Self::Ok(val) => Ok(val),
            Self::ApplicationError { code, message } => {
                Err(LoamSpineError::ipc(IpcPhase::JsonRpcError(code), message))
            }
            Self::ProtocolError(e) => Err(e),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// extract_rpc_error — centralized JSON-RPC error extraction
// ──────────────────────────────────────────────────────────────────────────────

/// Extracts `(code, message)` from a JSON-RPC 2.0 error object.
///
/// Centralizes the pattern used by every IPC adapter to parse the `error`
/// field from a JSON-RPC 2.0 response. Returns `None` if no error is present.
///
/// Aligns with rhizoCrypt's `extract_rpc_error` for ecosystem consistency.
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

// ──────────────────────────────────────────────────────────────────────────────
// OrExit — zero-panic startup validation
// ──────────────────────────────────────────────────────────────────────────────

/// Extension trait for `Result<T, E>` and `Option<T>` that exits the
/// process cleanly on error instead of panicking.
///
/// Absorbed from wetSpring V123 / rhizoCrypt `OrExit` pattern. Validation
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
                eprintln!("fatal: {context}: {e}");
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
        eprintln!("fatal: {context}");
        std::process::exit(1);
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
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

    #[test]
    fn ipc_error_display_and_helper() {
        let err = LoamSpineError::ipc(IpcPhase::Connect, "socket not found");
        assert!(err.to_string().contains("ipc error (connect)"));
        assert!(err.to_string().contains("socket not found"));
    }

    #[test]
    fn ipc_phase_display() {
        assert_eq!(IpcPhase::Connect.to_string(), "connect");
        assert_eq!(IpcPhase::Write.to_string(), "write");
        assert_eq!(IpcPhase::Read.to_string(), "read");
        assert_eq!(IpcPhase::InvalidJson.to_string(), "invalid_json");
        assert_eq!(IpcPhase::HttpStatus(404).to_string(), "http_404");
        assert_eq!(IpcPhase::NoResult.to_string(), "no_result");
        assert_eq!(IpcPhase::JsonRpcError(-32601).to_string(), "jsonrpc_-32601");
        assert_eq!(IpcPhase::Serialization.to_string(), "serialization");
    }

    #[test]
    fn is_recoverable_ipc_phases() {
        assert!(LoamSpineError::ipc(IpcPhase::Connect, "timeout").is_recoverable());
        assert!(LoamSpineError::ipc(IpcPhase::Write, "broken pipe").is_recoverable());
        assert!(LoamSpineError::ipc(IpcPhase::Read, "eof").is_recoverable());
        assert!(LoamSpineError::ipc(IpcPhase::HttpStatus(503), "unavail").is_recoverable());
        assert!(!LoamSpineError::ipc(IpcPhase::HttpStatus(404), "not found").is_recoverable());
        assert!(!LoamSpineError::ipc(IpcPhase::NoResult, "missing").is_recoverable());
        assert!(!LoamSpineError::ipc(IpcPhase::JsonRpcError(-32601), "method").is_recoverable());
        assert!(!LoamSpineError::ipc(IpcPhase::InvalidJson, "parse").is_recoverable());
    }

    #[test]
    fn is_recoverable_other_variants() {
        assert!(LoamSpineError::Network("timeout".into()).is_recoverable());
        assert!(LoamSpineError::CapabilityUnavailable("signer".into()).is_recoverable());
        assert!(!LoamSpineError::Storage("corrupt".into()).is_recoverable());
        assert!(!LoamSpineError::Config("bad".into()).is_recoverable());
    }

    #[test]
    fn is_method_not_found() {
        assert!(
            LoamSpineError::ipc(IpcPhase::JsonRpcError(-32601), "not found").is_method_not_found()
        );
        assert!(
            !LoamSpineError::ipc(IpcPhase::JsonRpcError(-32600), "other").is_method_not_found()
        );
        assert!(!LoamSpineError::ipc(IpcPhase::Connect, "timeout").is_method_not_found());
        assert!(!LoamSpineError::Network("err".into()).is_method_not_found());
    }

    #[test]
    fn dispatch_outcome_ok() {
        let outcome: DispatchOutcome<i32> = DispatchOutcome::Ok(42);
        assert!(outcome.is_ok());
        assert!(!outcome.is_application_error());
        assert_eq!(outcome.into_result().unwrap(), 42);
    }

    #[test]
    fn dispatch_outcome_application_error() {
        let outcome: DispatchOutcome<i32> = DispatchOutcome::ApplicationError {
            code: -32601,
            message: "method not found".into(),
        };
        assert!(!outcome.is_ok());
        assert!(outcome.is_application_error());
        let err = outcome.into_result().unwrap_err();
        assert!(err.is_method_not_found());
    }

    #[test]
    fn dispatch_outcome_protocol_error() {
        let outcome: DispatchOutcome<i32> =
            DispatchOutcome::ProtocolError(LoamSpineError::ipc(IpcPhase::Connect, "refused"));
        assert!(!outcome.is_ok());
        assert!(!outcome.is_application_error());
        let err = outcome.into_result().unwrap_err();
        assert!(err.is_recoverable());
    }

    #[test]
    fn extract_rpc_error_present() {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": -32601, "message": "method not found" },
            "id": 1
        });
        let (code, msg) = extract_rpc_error(&response).unwrap();
        assert_eq!(code, -32601);
        assert_eq!(msg, "method not found");
    }

    #[test]
    fn extract_rpc_error_absent() {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": 42,
            "id": 1
        });
        assert!(extract_rpc_error(&response).is_none());
    }

    #[test]
    fn extract_rpc_error_missing_fields() {
        let response = serde_json::json!({
            "error": {}
        });
        let (code, msg) = extract_rpc_error(&response).unwrap();
        assert_eq!(code, -1);
        assert_eq!(msg, "Unknown error");
    }

    #[test]
    fn is_timeout_likely_phases() {
        assert!(LoamSpineError::ipc(IpcPhase::Connect, "timeout").is_timeout_likely());
        assert!(LoamSpineError::ipc(IpcPhase::Read, "timeout").is_timeout_likely());
        assert!(LoamSpineError::ipc(IpcPhase::Write, "timeout").is_timeout_likely());
        assert!(!LoamSpineError::ipc(IpcPhase::InvalidJson, "parse").is_timeout_likely());
        assert!(!LoamSpineError::ipc(IpcPhase::JsonRpcError(-32601), "m").is_timeout_likely());
        assert!(!LoamSpineError::Network("err".into()).is_timeout_likely());
    }

    #[test]
    fn is_application_error_phases() {
        assert!(
            LoamSpineError::ipc(IpcPhase::JsonRpcError(-32601), "not found").is_application_error()
        );
        assert!(
            LoamSpineError::ipc(IpcPhase::JsonRpcError(-32000), "app err").is_application_error()
        );
        assert!(!LoamSpineError::ipc(IpcPhase::Connect, "refused").is_application_error());
        assert!(!LoamSpineError::ipc(IpcPhase::InvalidJson, "parse").is_application_error());
        assert!(!LoamSpineError::Network("err".into()).is_application_error());
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "proptest assertions use unwrap_err for error-path validation"
)]
#[expect(
    clippy::redundant_clone,
    reason = "proptest macro takes ownership; clone needed for subsequent assertions"
)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_ipc_phase() -> impl Strategy<Value = IpcPhase> {
        prop_oneof![
            Just(IpcPhase::Connect),
            Just(IpcPhase::Write),
            Just(IpcPhase::Read),
            Just(IpcPhase::InvalidJson),
            (0u16..=999u16).prop_map(IpcPhase::HttpStatus),
            Just(IpcPhase::NoResult),
            any::<i64>().prop_map(IpcPhase::JsonRpcError),
            Just(IpcPhase::Serialization),
        ]
    }

    proptest! {
        #[test]
        fn ipc_phase_display_never_panics(phase in arb_ipc_phase()) {
            let s = phase.to_string();
            prop_assert!(!s.is_empty());
        }

        #[test]
        fn ipc_error_helpers_consistent(phase in arb_ipc_phase(), msg in ".*") {
            let err = LoamSpineError::ipc(phase.clone(), msg);
            if err.is_method_not_found() {
                prop_assert!(err.is_application_error());
            }
            if err.is_timeout_likely() {
                prop_assert!(err.is_recoverable());
            }
        }

        #[test]
        fn extract_rpc_error_never_panics(json_str in "\\PC{0,200}") {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                let _ = extract_rpc_error(&val);
            }
        }

        #[test]
        fn dispatch_outcome_into_result_consistent(code in any::<i64>(), msg in ".*") {
            let outcome: DispatchOutcome<i32> = DispatchOutcome::ApplicationError {
                code,
                message: msg,
            };
            let err = outcome.into_result().unwrap_err();
            prop_assert!(err.is_application_error());
        }
    }
}
