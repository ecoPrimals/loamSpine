// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON-RPC 2.0 wire types and error codes.
//!
//! These types define the protocol-level structures for JSON-RPC 2.0
//! communication. They are transport-agnostic and used by both TCP
//! and UDS server implementations.

use serde::{Deserialize, Serialize};

// Standard JSON-RPC 2.0 error codes
pub(crate) const PARSE_ERROR: i32 = -32700;
pub(crate) const INVALID_REQUEST: i32 = -32600;
pub(crate) const METHOD_NOT_FOUND: i32 = -32601;
pub(crate) const INVALID_PARAMS: i32 = -32602;
pub(crate) const LOAMSPINE_ERROR: i32 = -32000;

/// A JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Protocol version (must be "2.0").
    pub jsonrpc: String,
    /// Method name.
    pub method: String,
    /// Method parameters.
    #[serde(default)]
    pub params: serde_json::Value,
    /// Request ID (null for notifications).
    #[serde(default)]
    pub id: serde_json::Value,
}

/// A JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version (always `"2.0"`; `Cow` avoids allocation).
    pub jsonrpc: std::borrow::Cow<'static, str>,
    /// Successful result (mutually exclusive with `error`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error (mutually exclusive with `result`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Request ID (echoed from the request).
    pub id: serde_json::Value,
}

/// A JSON-RPC 2.0 error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i32,
    /// Error message.
    pub message: String,
    /// Additional data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    pub(crate) const fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: std::borrow::Cow::Borrowed("2.0"),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub(crate) fn error(id: serde_json::Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: std::borrow::Cow::Borrowed("2.0"),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
            id,
        }
    }
}
