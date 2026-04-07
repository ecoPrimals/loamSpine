// SPDX-License-Identifier: AGPL-3.0-or-later

//! Dispatch outcome type for protocol vs application error separation.

use super::{IpcErrorPhase, LoamSpineError, LoamSpineResult};

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
            Self::ApplicationError { code, message } => Err(LoamSpineError::ipc(
                IpcErrorPhase::JsonRpcError(code),
                message,
            )),
            Self::ProtocolError(e) => Err(e),
        }
    }
}
