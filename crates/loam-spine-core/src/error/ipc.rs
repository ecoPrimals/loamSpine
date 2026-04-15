// SPDX-License-Identifier: AGPL-3.0-or-later

//! IPC error phase classification.
//!
//! Aligns with the ecosystem's `IpcErrorPhase` and `SendError` conventions
//! for cross-primal structured IPC error handling.

use std::fmt;

/// Phase of an IPC call that failed.
///
/// Aligns with the ecosystem's `IpcErrorPhase` and `SendError` conventions
/// for cross-primal structured IPC error handling.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum IpcErrorPhase {
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

/// Backward-compatible alias for [`IpcErrorPhase`].
///
/// Ecosystem naming converged on `IpcErrorPhase` (primalSpring, biomeOS).
/// This alias preserves backward compatibility for downstream
/// code that imported the original name.
pub type IpcPhase = IpcErrorPhase;

impl fmt::Display for IpcErrorPhase {
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
