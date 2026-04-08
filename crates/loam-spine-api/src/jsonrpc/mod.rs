// SPDX-License-Identifier: AGPL-3.0-or-later

//! Pure Rust JSON-RPC 2.0 server for `LoamSpine`.
//!
//! Universal, language-agnostic RPC for external clients.
//! Works with Python, JavaScript, curl, etc.
//!
//! Zero C dependencies — replaces jsonrpsee (which pulled ring/C-asm)
//! with a hand-rolled JSON-RPC dispatcher over raw HTTP/TCP.

mod server;
mod wire;

pub use server::{run_jsonrpc_server, ServerHandle};
#[cfg(unix)]
pub use server::{run_jsonrpc_uds_server, UdsServerHandle};
pub use wire::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};

// Re-exports for internal use by sibling test modules
#[cfg(test)]
pub(crate) use server::{
    is_notification, process_request, serialize_response, serialize_response_batch,
};

use crate::service::LoamSpineRpcService;
use loam_spine_core::error::{DispatchOutcome, IpcErrorPhase, LoamSpineError};
use wire::{INVALID_PARAMS, LOAMSPINE_ERROR, METHOD_NOT_FOUND};

// ============================================================================
// Method normalization (backward-compatible alias resolution)
// ============================================================================

/// Normalize legacy method names to canonical forms.
///
/// Maps older naming conventions (e.g. camelCase, reversed domain,
/// `primal.capabilities`, `commit.session`) to the canonical semantic
/// names defined in the wateringHole Semantic Method Naming Standard v2.1.
///
/// Absorbed from barraCuda v0.3.7's `normalize_method()` pattern — a
/// single normalization step before dispatch, instead of duplicated
/// match arms.
#[must_use]
pub fn normalize_method(method: &str) -> &str {
    match method {
        "commit.session" => "session.commit",
        "permanent-storage.commitSession" => "permanence.commit_session",
        "permanent-storage.verifyCommit" => "permanence.verify_commit",
        "permanent-storage.getCommit" => "permanence.get_commit",
        "permanent-storage.healthCheck" => "permanence.health_check",
        "capability.list" | "primal.capabilities" => "capabilities.list",
        other => other,
    }
}

// ============================================================================
// JSON-RPC dispatch
// ============================================================================

/// The JSON-RPC dispatch handler wrapping a `LoamSpineRpcService`.
pub struct LoamSpineJsonRpc {
    pub(crate) service: LoamSpineRpcService,
}

impl LoamSpineJsonRpc {
    /// Create a new handler from a service.
    #[must_use]
    pub const fn new(service: LoamSpineRpcService) -> Self {
        Self { service }
    }

    /// Create a handler with default service (in-memory storage).
    #[must_use]
    pub fn default_server() -> Self {
        Self::new(LoamSpineRpcService::default_service())
    }

    /// Handle a single JSON-RPC request and return a response.
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let canonical = normalize_method(&request.method);
        match self.dispatch(canonical, request.params).await {
            Ok(val) => JsonRpcResponse::success(request.id, val),
            Err(e) => JsonRpcResponse::error(request.id, e.code, e.message),
        }
    }

    /// Handle a request through the `DispatchOutcome` pathway.
    ///
    /// This entry point is used by the tarpc bridge to preserve
    /// the distinction between protocol errors and application errors.
    pub async fn dispatch_typed(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> DispatchOutcome<serde_json::Value> {
        let canonical = normalize_method(method);
        match self.dispatch(canonical, params).await {
            Ok(val) => DispatchOutcome::Ok(val),
            Err(e)
                if e.code == METHOD_NOT_FOUND
                    || e.code == INVALID_PARAMS
                    || e.code == wire::PARSE_ERROR =>
            {
                DispatchOutcome::ProtocolError(LoamSpineError::Ipc {
                    phase: IpcErrorPhase::JsonRpcError(i64::from(e.code)),
                    message: e.message,
                })
            }
            Err(e) => DispatchOutcome::ApplicationError {
                code: i64::from(e.code),
                message: e.message,
            },
        }
    }

    fn dispatch<'a>(
        &'a self,
        method: &'a str,
        params: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<serde_json::Value, wire::JsonRpcError>> + Send + 'a>,
    > {
        Box::pin(self.dispatch_inner(method, params))
    }

    async fn dispatch_inner(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, wire::JsonRpcError> {
        macro_rules! rpc {
            ($params:expr, $method:ident) => {{
                let req = deser($params)?;
                let res = self.service.$method(req).await.map_err(app_err)?;
                ser(res)
            }};
        }

        match method {
            "spine.create" => rpc!(params, create_spine),
            "spine.get" => rpc!(params, get_spine),
            "spine.seal" => rpc!(params, seal_spine),

            "entry.append" => rpc!(params, append_entry),
            "entry.get" => rpc!(params, get_entry),
            "entry.get_tip" => rpc!(params, get_tip),

            "certificate.mint" => rpc!(params, mint_certificate),
            "certificate.transfer" => rpc!(params, transfer_certificate),
            "certificate.loan" => rpc!(params, loan_certificate),
            "certificate.return" => rpc!(params, return_certificate),
            "certificate.get" => rpc!(params, get_certificate),

            "health.check" => rpc!(params, health_check),
            "health.liveness" => ser(self.service.liveness().await),
            "health.readiness" => {
                let probe = self.service.readiness().await.map_err(app_err)?;
                ser(probe)
            }

            "session.commit" => rpc!(params, commit_session),
            "braid.commit" => rpc!(params, commit_braid),

            "slice.anchor" => rpc!(params, anchor_slice),
            "slice.checkout" => rpc!(params, checkout_slice),

            "proof.generate_inclusion" => rpc!(params, generate_inclusion_proof),
            "proof.verify_inclusion" => rpc!(params, verify_inclusion_proof),

            "anchor.publish" => rpc!(params, publish_anchor),
            "anchor.verify" => rpc!(params, verify_anchor),

            "permanence.commit_session" => rpc!(params, permanent_storage_commit_session),
            "permanence.verify_commit" => rpc!(params, permanent_storage_verify_commit),
            "permanence.get_commit" => rpc!(params, permanent_storage_get_commit),
            "permanence.health_check" => ser(self.service.permanence_healthy().await),

            "capabilities.list" => Ok(loam_spine_core::neural_api::capability_list().clone()),
            "identity.get" => Ok(loam_spine_core::neural_api::identity_response().clone()),

            "tools.list" => Ok(loam_spine_core::neural_api::mcp_tools_list().clone()),

            "tools.call" => {
                let tool_name = params
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| wire::JsonRpcError {
                        code: INVALID_PARAMS,
                        message: "tools.call requires 'name' string".to_string(),
                        data: None,
                    })?;
                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
                let (rpc_method, rpc_params) = loam_spine_core::neural_api::mcp_tool_to_rpc(
                    tool_name, arguments,
                )
                .ok_or_else(|| wire::JsonRpcError {
                    code: METHOD_NOT_FOUND,
                    message: format!("unknown tool: {tool_name}"),
                    data: None,
                })?;
                let inner_result = self.dispatch(rpc_method, rpc_params).await?;
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": inner_result.to_string() }],
                    "isError": false,
                }))
            }

            _ => Err(wire::JsonRpcError {
                code: METHOD_NOT_FOUND,
                message: format!("method not found: {method}"),
                data: None,
            }),
        }
    }
}

fn app_err(e: impl std::fmt::Display) -> wire::JsonRpcError {
    wire::JsonRpcError {
        code: LOAMSPINE_ERROR,
        message: e.to_string(),
        data: None,
    }
}

fn deser<T: serde::de::DeserializeOwned>(params: serde_json::Value) -> Result<T, wire::JsonRpcError> {
    serde_json::from_value(params).map_err(|e| wire::JsonRpcError {
        code: INVALID_PARAMS,
        message: format!("invalid params: {e}"),
        data: None,
    })
}

fn ser<T: serde::Serialize>(val: T) -> Result<serde_json::Value, wire::JsonRpcError> {
    serde_json::to_value(val).map_err(|e| wire::JsonRpcError {
        code: LOAMSPINE_ERROR,
        message: format!("serialization error: {e}"),
        data: None,
    })
}

/// Convert a [`DispatchOutcome`] into a [`JsonRpcResponse`].
///
/// Protocol errors (method not found, invalid params) carry their
/// original JSON-RPC error code via [`IpcErrorPhase::JsonRpcError`].
/// Application errors use the code embedded in the outcome.
#[cfg(test)]
pub(crate) fn outcome_to_response(
    id: serde_json::Value,
    outcome: DispatchOutcome<serde_json::Value>,
) -> JsonRpcResponse {
    match outcome {
        DispatchOutcome::Ok(val) => JsonRpcResponse::success(id, val),
        DispatchOutcome::ApplicationError { code, message } => {
            JsonRpcResponse::error(id, i32::try_from(code).unwrap_or(LOAMSPINE_ERROR), message)
        }
        DispatchOutcome::ProtocolError(ref err) => {
            let (code, message) = match err {
                LoamSpineError::Ipc {
                    phase: IpcErrorPhase::JsonRpcError(c),
                    message,
                } => (
                    i32::try_from(*c).unwrap_or(LOAMSPINE_ERROR),
                    message.clone(),
                ),
                other => (LOAMSPINE_ERROR, other.to_string()),
            };
            JsonRpcResponse::error(id, code, message)
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests;
#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests_permanence_cert;
#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests_protocol;
#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
mod tests_validation;
