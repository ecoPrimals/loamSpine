// SPDX-License-Identifier: AGPL-3.0-only

//! JSON-RPC 2.0 server for `LoamSpine`.
//!
//! Universal, language-agnostic RPC for external clients.
//! Works with Python, JavaScript, curl, etc.

use crate::error::ServerError;
use crate::health::{LivenessProbe, ReadinessProbe};
use crate::service::LoamSpineRpcService;
use crate::types::{
    AnchorSliceRequest, AnchorSliceResponse, AppendEntryRequest, AppendEntryResponse,
    CheckoutSliceRequest, CheckoutSliceResponse, CommitBraidRequest, CommitBraidResponse,
    CommitSessionRequest, CommitSessionResponse, CreateSpineRequest, CreateSpineResponse,
    GenerateInclusionProofRequest, GenerateInclusionProofResponse, GetCertificateRequest,
    GetCertificateResponse, GetEntryRequest, GetEntryResponse, GetSpineRequest, GetSpineResponse,
    GetTipRequest, GetTipResponse, HealthCheckRequest, HealthCheckResponse, LoanCertificateRequest,
    LoanCertificateResponse, MintCertificateRequest, MintCertificateResponse,
    PermanentStorageCommitRequest, PermanentStorageCommitResponse,
    PermanentStorageGetCommitRequest, PermanentStorageVerifyRequest, ReturnCertificateRequest,
    ReturnCertificateResponse, SealSpineRequest, SealSpineResponse, TransferCertificateRequest,
    TransferCertificateResponse, VerifyInclusionProofRequest, VerifyInclusionProofResponse,
};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::{Server, ServerHandle};
use jsonrpsee::types::ErrorObject;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

// ============================================================================
// JSON-RPC 2.0 Error Codes
// ============================================================================
// Standard JSON-RPC 2.0 error codes: -32700 to -32600
// Server error codes (reserved): -32099 to -32000
// Application error codes: -32000 and below

/// Application-level error code for `LoamSpine` operations.
/// Per JSON-RPC 2.0 spec, -32000 to -32099 are reserved for implementation-defined server errors.
const LOAMSPINE_ERROR_CODE: i32 = -32000;

/// Convert a service error to a JSON-RPC error.
fn to_rpc_error<E: std::fmt::Display>(e: E) -> ErrorObject<'static> {
    ErrorObject::owned(LOAMSPINE_ERROR_CODE, e.to_string(), None::<()>)
}

/// JSON-RPC 2.0 API for `LoamSpine`.
///
/// Exposes the same operations as tarpc, but over JSON-RPC
/// for universal client access.
#[rpc(server)]
pub trait LoamSpineJsonRpcApi {
    // ========================================================================
    // Spine Operations
    // ========================================================================

    /// Create a new spine.
    #[method(name = "spine.create")]
    async fn create_spine(&self, request: CreateSpineRequest) -> RpcResult<CreateSpineResponse>;

    /// Get a spine by ID.
    #[method(name = "spine.get")]
    async fn get_spine(&self, request: GetSpineRequest) -> RpcResult<GetSpineResponse>;

    /// Seal a spine (make immutable).
    #[method(name = "spine.seal")]
    async fn seal_spine(&self, request: SealSpineRequest) -> RpcResult<SealSpineResponse>;

    // ========================================================================
    // Entry Operations
    // ========================================================================

    /// Append an entry to a spine.
    #[method(name = "entry.append")]
    async fn append_entry(&self, request: AppendEntryRequest) -> RpcResult<AppendEntryResponse>;

    /// Get an entry by hash.
    #[method(name = "entry.get")]
    async fn get_entry(&self, request: GetEntryRequest) -> RpcResult<GetEntryResponse>;

    /// Get the tip entry of a spine.
    #[method(name = "entry.get_tip")]
    async fn get_tip(&self, request: GetTipRequest) -> RpcResult<GetTipResponse>;

    // ========================================================================
    // Certificate Operations
    // ========================================================================

    /// Mint a new certificate.
    #[method(name = "certificate.mint")]
    async fn mint_certificate(
        &self,
        request: MintCertificateRequest,
    ) -> RpcResult<MintCertificateResponse>;

    /// Transfer a certificate to a new owner.
    #[method(name = "certificate.transfer")]
    async fn transfer_certificate(
        &self,
        request: TransferCertificateRequest,
    ) -> RpcResult<TransferCertificateResponse>;

    /// Loan a certificate temporarily.
    #[method(name = "certificate.loan")]
    async fn loan_certificate(
        &self,
        request: LoanCertificateRequest,
    ) -> RpcResult<LoanCertificateResponse>;

    /// Return a loaned certificate.
    #[method(name = "certificate.return")]
    async fn return_certificate(
        &self,
        request: ReturnCertificateRequest,
    ) -> RpcResult<ReturnCertificateResponse>;

    /// Get a certificate by ID.
    #[method(name = "certificate.get")]
    async fn get_certificate(
        &self,
        request: GetCertificateRequest,
    ) -> RpcResult<GetCertificateResponse>;

    // ========================================================================
    // Health
    // ========================================================================

    /// Health check (detailed status).
    #[method(name = "health.check")]
    async fn health_check(&self, request: HealthCheckRequest) -> RpcResult<HealthCheckResponse>;

    /// Liveness probe (container orchestrator endpoint).
    #[method(name = "health.liveness")]
    async fn liveness(&self) -> RpcResult<LivenessProbe>;

    /// Readiness probe (container orchestrator endpoint).
    #[method(name = "health.readiness")]
    async fn readiness(&self) -> RpcResult<ReadinessProbe>;

    // ========================================================================
    // Ephemeral Storage Integration
    // ========================================================================

    /// Commit a session from an ephemeral storage primal.
    #[method(name = "session.commit")]
    async fn commit_session(
        &self,
        request: CommitSessionRequest,
    ) -> RpcResult<CommitSessionResponse>;

    // ========================================================================
    // Semantic Attribution Integration
    // ========================================================================

    /// Commit a braid from a semantic attribution primal.
    #[method(name = "braid.commit")]
    async fn commit_braid(&self, request: CommitBraidRequest) -> RpcResult<CommitBraidResponse>;

    // ========================================================================
    // Waypoint Operations
    // ========================================================================

    /// Anchor a slice on a waypoint spine.
    #[method(name = "slice.anchor")]
    async fn anchor_slice(&self, request: AnchorSliceRequest) -> RpcResult<AnchorSliceResponse>;

    /// Checkout a slice from a waypoint.
    #[method(name = "slice.checkout")]
    async fn checkout_slice(
        &self,
        request: CheckoutSliceRequest,
    ) -> RpcResult<CheckoutSliceResponse>;

    // ========================================================================
    // Proof Operations
    // ========================================================================

    /// Generate an inclusion proof.
    #[method(name = "proof.generate_inclusion")]
    async fn generate_inclusion_proof(
        &self,
        request: GenerateInclusionProofRequest,
    ) -> RpcResult<GenerateInclusionProofResponse>;

    /// Verify an inclusion proof.
    #[method(name = "proof.verify_inclusion")]
    async fn verify_inclusion_proof(
        &self,
        request: VerifyInclusionProofRequest,
    ) -> RpcResult<VerifyInclusionProofResponse>;

    // ========================================================================
    // Permanence Operations (semantic naming: permanence.{operation})
    // ========================================================================

    /// Commit a session to permanent storage.
    #[method(name = "permanence.commit_session")]
    async fn permanence_commit_session(
        &self,
        request: PermanentStorageCommitRequest,
    ) -> RpcResult<PermanentStorageCommitResponse>;

    /// Verify a commit in permanent storage.
    #[method(name = "permanence.verify_commit")]
    async fn permanence_verify_commit(
        &self,
        request: PermanentStorageVerifyRequest,
    ) -> RpcResult<bool>;

    /// Get a commit from permanent storage.
    #[method(name = "permanence.get_commit")]
    async fn permanence_get_commit(
        &self,
        request: PermanentStorageGetCommitRequest,
    ) -> RpcResult<serde_json::Value>;

    /// Health check for permanence layer.
    #[method(name = "permanence.health_check")]
    async fn permanence_health_check(&self) -> RpcResult<bool>;

    // ========================================================================
    // NeuralAPI Semantic Aliases (biomeOS capability routing)
    // These align with biomeOS capability_domains.rs translations.
    // ========================================================================

    /// Semantic alias for `session.commit` (used by biomeOS `commit.session` routing).
    #[method(name = "commit.session")]
    async fn commit_session_semantic(
        &self,
        request: CommitSessionRequest,
    ) -> RpcResult<CommitSessionResponse>;

    /// List capabilities provided by this primal (Spring-as-Niche standard).
    #[method(name = "capability.list")]
    async fn capability_list(&self) -> RpcResult<serde_json::Value>;

    // ========================================================================
    // Legacy compatibility (deprecated camelCase wire format)
    // These aliases will be removed in v1.0.0.
    // ========================================================================

    /// DEPRECATED: Use `permanence.commit_session` instead.
    #[method(name = "permanent-storage.commitSession")]
    async fn permanent_storage_commit_session(
        &self,
        request: PermanentStorageCommitRequest,
    ) -> RpcResult<PermanentStorageCommitResponse>;

    /// DEPRECATED: Use `permanence.verify_commit` instead.
    #[method(name = "permanent-storage.verifyCommit")]
    async fn permanent_storage_verify_commit(
        &self,
        request: PermanentStorageVerifyRequest,
    ) -> RpcResult<bool>;

    /// DEPRECATED: Use `permanence.get_commit` instead.
    #[method(name = "permanent-storage.getCommit")]
    async fn permanent_storage_get_commit(
        &self,
        request: PermanentStorageGetCommitRequest,
    ) -> RpcResult<serde_json::Value>;

    /// DEPRECATED: Use `permanence.health_check` instead.
    #[method(name = "permanent-storage.healthCheck")]
    async fn permanent_storage_health_check(&self) -> RpcResult<bool>;
}

/// JSON-RPC server implementation.
pub struct LoamSpineJsonRpc {
    service: Arc<LoamSpineRpcService>,
}

impl LoamSpineJsonRpc {
    /// Create a new JSON-RPC server.
    #[must_use]
    pub fn new(service: LoamSpineRpcService) -> Self {
        Self {
            service: Arc::new(service),
        }
    }

    /// Create with default service.
    #[must_use]
    pub fn default_server() -> Self {
        Self::new(LoamSpineRpcService::default_service())
    }
}

#[jsonrpsee::core::async_trait]
impl LoamSpineJsonRpcApiServer for LoamSpineJsonRpc {
    async fn create_spine(&self, request: CreateSpineRequest) -> RpcResult<CreateSpineResponse> {
        self.service
            .create_spine(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn get_spine(&self, request: GetSpineRequest) -> RpcResult<GetSpineResponse> {
        self.service.get_spine(request).await.map_err(to_rpc_error)
    }

    async fn seal_spine(&self, request: SealSpineRequest) -> RpcResult<SealSpineResponse> {
        self.service.seal_spine(request).await.map_err(to_rpc_error)
    }

    async fn append_entry(&self, request: AppendEntryRequest) -> RpcResult<AppendEntryResponse> {
        self.service
            .append_entry(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn get_entry(&self, request: GetEntryRequest) -> RpcResult<GetEntryResponse> {
        self.service.get_entry(request).await.map_err(to_rpc_error)
    }

    async fn get_tip(&self, request: GetTipRequest) -> RpcResult<GetTipResponse> {
        self.service.get_tip(request).await.map_err(to_rpc_error)
    }

    async fn mint_certificate(
        &self,
        request: MintCertificateRequest,
    ) -> RpcResult<MintCertificateResponse> {
        self.service
            .mint_certificate(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn transfer_certificate(
        &self,
        request: TransferCertificateRequest,
    ) -> RpcResult<TransferCertificateResponse> {
        self.service
            .transfer_certificate(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn loan_certificate(
        &self,
        request: LoanCertificateRequest,
    ) -> RpcResult<LoanCertificateResponse> {
        self.service
            .loan_certificate(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn return_certificate(
        &self,
        request: ReturnCertificateRequest,
    ) -> RpcResult<ReturnCertificateResponse> {
        self.service
            .return_certificate(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn get_certificate(
        &self,
        request: GetCertificateRequest,
    ) -> RpcResult<GetCertificateResponse> {
        self.service
            .get_certificate(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn health_check(&self, request: HealthCheckRequest) -> RpcResult<HealthCheckResponse> {
        self.service
            .health_check(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn liveness(&self) -> RpcResult<crate::health::LivenessProbe> {
        Ok(self.service.liveness().await)
    }

    async fn readiness(&self) -> RpcResult<crate::health::ReadinessProbe> {
        self.service.readiness().await.map_err(to_rpc_error)
    }

    async fn commit_session(
        &self,
        request: CommitSessionRequest,
    ) -> RpcResult<CommitSessionResponse> {
        self.service
            .commit_session(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn commit_braid(&self, request: CommitBraidRequest) -> RpcResult<CommitBraidResponse> {
        self.service
            .commit_braid(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn anchor_slice(&self, request: AnchorSliceRequest) -> RpcResult<AnchorSliceResponse> {
        self.service
            .anchor_slice(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn checkout_slice(
        &self,
        request: CheckoutSliceRequest,
    ) -> RpcResult<CheckoutSliceResponse> {
        self.service
            .checkout_slice(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn generate_inclusion_proof(
        &self,
        request: GenerateInclusionProofRequest,
    ) -> RpcResult<GenerateInclusionProofResponse> {
        self.service
            .generate_inclusion_proof(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn verify_inclusion_proof(
        &self,
        request: VerifyInclusionProofRequest,
    ) -> RpcResult<VerifyInclusionProofResponse> {
        self.service
            .verify_inclusion_proof(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn permanence_commit_session(
        &self,
        request: PermanentStorageCommitRequest,
    ) -> RpcResult<PermanentStorageCommitResponse> {
        self.service
            .permanent_storage_commit_session(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn permanence_verify_commit(
        &self,
        request: PermanentStorageVerifyRequest,
    ) -> RpcResult<bool> {
        self.service
            .permanent_storage_verify_commit(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn permanence_get_commit(
        &self,
        request: PermanentStorageGetCommitRequest,
    ) -> RpcResult<serde_json::Value> {
        self.service
            .permanent_storage_get_commit(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn permanence_health_check(&self) -> RpcResult<bool> {
        Ok(self.service.permanence_healthy().await)
    }

    async fn permanent_storage_commit_session(
        &self,
        request: PermanentStorageCommitRequest,
    ) -> RpcResult<PermanentStorageCommitResponse> {
        self.permanence_commit_session(request).await
    }

    async fn permanent_storage_verify_commit(
        &self,
        request: PermanentStorageVerifyRequest,
    ) -> RpcResult<bool> {
        self.permanence_verify_commit(request).await
    }

    async fn permanent_storage_get_commit(
        &self,
        request: PermanentStorageGetCommitRequest,
    ) -> RpcResult<serde_json::Value> {
        self.permanence_get_commit(request).await
    }

    async fn permanent_storage_health_check(&self) -> RpcResult<bool> {
        self.permanence_health_check().await
    }

    async fn commit_session_semantic(
        &self,
        request: CommitSessionRequest,
    ) -> RpcResult<CommitSessionResponse> {
        self.commit_session(request).await
    }

    async fn capability_list(&self) -> RpcResult<serde_json::Value> {
        Ok(loam_spine_core::neural_api::capability_list())
    }
}

/// Run the JSON-RPC server.
///
/// # Errors
///
/// Returns error if server fails to start.
pub async fn run_jsonrpc_server(
    addr: SocketAddr,
    service: LoamSpineRpcService,
) -> Result<ServerHandle, ServerError> {
    let server = Server::builder()
        .build(addr)
        .await
        .map_err(|e| ServerError::Bind(e.to_string()))?;
    let jsonrpc = LoamSpineJsonRpc::new(service);

    info!("🌐 LoamSpine JSON-RPC server listening on http://{}", addr);

    let handle = server.start(jsonrpc.into_rpc());
    Ok(handle)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests;
