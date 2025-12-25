//! JSON-RPC 2.0 server for `LoamSpine`.
//!
//! Universal, language-agnostic RPC for external clients.
//! Works with Python, JavaScript, curl, etc.

#![allow(clippy::wildcard_imports)]

use crate::health::{LivenessProbe, ReadinessProbe};
use crate::service::LoamSpineRpcService;
use crate::types::*;
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
    #[method(name = "loamspine.createSpine")]
    async fn create_spine(&self, request: CreateSpineRequest) -> RpcResult<CreateSpineResponse>;

    /// Get a spine by ID.
    #[method(name = "loamspine.getSpine")]
    async fn get_spine(&self, request: GetSpineRequest) -> RpcResult<GetSpineResponse>;

    /// Seal a spine.
    #[method(name = "loamspine.sealSpine")]
    async fn seal_spine(&self, request: SealSpineRequest) -> RpcResult<SealSpineResponse>;

    // ========================================================================
    // Entry Operations
    // ========================================================================

    /// Append an entry.
    #[method(name = "loamspine.appendEntry")]
    async fn append_entry(&self, request: AppendEntryRequest) -> RpcResult<AppendEntryResponse>;

    /// Get an entry by hash.
    #[method(name = "loamspine.getEntry")]
    async fn get_entry(&self, request: GetEntryRequest) -> RpcResult<GetEntryResponse>;

    /// Get the tip entry.
    #[method(name = "loamspine.getTip")]
    async fn get_tip(&self, request: GetTipRequest) -> RpcResult<GetTipResponse>;

    // ========================================================================
    // Certificate Operations
    // ========================================================================

    /// Mint a certificate.
    #[method(name = "loamspine.mintCertificate")]
    async fn mint_certificate(
        &self,
        request: MintCertificateRequest,
    ) -> RpcResult<MintCertificateResponse>;

    /// Transfer a certificate.
    #[method(name = "loamspine.transferCertificate")]
    async fn transfer_certificate(
        &self,
        request: TransferCertificateRequest,
    ) -> RpcResult<TransferCertificateResponse>;

    /// Loan a certificate.
    #[method(name = "loamspine.loanCertificate")]
    async fn loan_certificate(
        &self,
        request: LoanCertificateRequest,
    ) -> RpcResult<LoanCertificateResponse>;

    /// Return a loaned certificate.
    #[method(name = "loamspine.returnCertificate")]
    async fn return_certificate(
        &self,
        request: ReturnCertificateRequest,
    ) -> RpcResult<ReturnCertificateResponse>;

    /// Get a certificate.
    #[method(name = "loamspine.getCertificate")]
    async fn get_certificate(
        &self,
        request: GetCertificateRequest,
    ) -> RpcResult<GetCertificateResponse>;

    // ========================================================================
    // Health
    // ========================================================================

    /// Health check (detailed status).
    #[method(name = "loamspine.healthCheck")]
    async fn health_check(&self, request: HealthCheckRequest) -> RpcResult<HealthCheckResponse>;
    
    /// Liveness probe (Kubernetes-style).
    #[method(name = "loamspine.liveness")]
    async fn liveness(&self) -> RpcResult<LivenessProbe>;
    
    /// Readiness probe (Kubernetes-style).
    #[method(name = "loamspine.readiness")]
    async fn readiness(&self) -> RpcResult<ReadinessProbe>;

    // ========================================================================
    // Ephemeral Storage Integration
    // ========================================================================

    /// Commit a session from an ephemeral storage primal.
    #[method(name = "loamspine.commitSession")]
    async fn commit_session(
        &self,
        request: CommitSessionRequest,
    ) -> RpcResult<CommitSessionResponse>;

    // ========================================================================
    // Semantic Attribution Integration
    // ========================================================================

    /// Commit a braid from a semantic attribution primal.
    #[method(name = "loamspine.commitBraid")]
    async fn commit_braid(&self, request: CommitBraidRequest) -> RpcResult<CommitBraidResponse>;

    // ========================================================================
    // Waypoint Operations
    // ========================================================================

    /// Anchor a slice on a waypoint spine.
    #[method(name = "loamspine.anchorSlice")]
    async fn anchor_slice(&self, request: AnchorSliceRequest) -> RpcResult<AnchorSliceResponse>;

    /// Checkout a slice from a waypoint.
    #[method(name = "loamspine.checkoutSlice")]
    async fn checkout_slice(
        &self,
        request: CheckoutSliceRequest,
    ) -> RpcResult<CheckoutSliceResponse>;

    // ========================================================================
    // Proof Operations
    // ========================================================================

    /// Generate an inclusion proof.
    #[method(name = "loamspine.generateInclusionProof")]
    async fn generate_inclusion_proof(
        &self,
        request: GenerateInclusionProofRequest,
    ) -> RpcResult<GenerateInclusionProofResponse>;

    /// Verify an inclusion proof.
    #[method(name = "loamspine.verifyInclusionProof")]
    async fn verify_inclusion_proof(
        &self,
        request: VerifyInclusionProofRequest,
    ) -> RpcResult<VerifyInclusionProofResponse>;
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
}

/// Run the JSON-RPC server.
///
/// # Errors
///
/// Returns error if server fails to start.
pub async fn run_jsonrpc_server(
    addr: SocketAddr,
    service: LoamSpineRpcService,
) -> Result<ServerHandle, Box<dyn std::error::Error + Send + Sync>> {
    let server = Server::builder().build(addr).await?;
    let jsonrpc = LoamSpineJsonRpc::new(service);

    info!("🌐 LoamSpine JSON-RPC server listening on http://{}", addr);

    let handle = server.start(jsonrpc.into_rpc());
    Ok(handle)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_creation() {
        let _server = LoamSpineJsonRpc::default_server();
    }

    #[test]
    fn test_jsonrpc_with_service() {
        let service = LoamSpineRpcService::default_service();
        let server = LoamSpineJsonRpc::new(service);
        // Verify server is created with service
        assert!(Arc::strong_count(&server.service) >= 1);
    }

    #[tokio::test]
    async fn test_jsonrpc_health_check() {
        let server = LoamSpineJsonRpc::default_server();
        let request = HealthCheckRequest {
            include_details: false,
        };

        let result = LoamSpineJsonRpcApiServer::health_check(&server, request).await;
        assert!(result.is_ok());

        let response = result.unwrap_or_else(|_| panic!("unexpected error"));
        assert!(response.status.is_healthy());
    }

    #[tokio::test]
    async fn test_jsonrpc_create_spine() {
        let server = LoamSpineJsonRpc::default_server();
        let request = CreateSpineRequest {
            owner: Did::new("did:key:z6MkTest"),
            name: "Test Spine".to_string(),
            config: None,
        };

        let result = LoamSpineJsonRpcApiServer::create_spine(&server, request).await;
        assert!(result.is_ok());

        let response = result.unwrap_or_else(|_| panic!("unexpected error"));
        assert!(!response.spine_id.is_nil());
    }

    #[tokio::test]
    async fn test_jsonrpc_get_nonexistent_spine() {
        let server = LoamSpineJsonRpc::default_server();
        let request = GetSpineRequest {
            spine_id: uuid::Uuid::nil(),
        };

        let result = LoamSpineJsonRpcApiServer::get_spine(&server, request).await;
        assert!(result.is_ok());

        let response = result.unwrap_or_else(|_| panic!("unexpected error"));
        assert!(response.spine.is_none());
    }

    #[tokio::test]
    async fn test_jsonrpc_seal_spine() {
        let server = LoamSpineJsonRpc::default_server();

        // First create a spine
        let owner = Did::new("did:key:z6MkTest");
        let create_request = CreateSpineRequest {
            owner: owner.clone(),
            name: "Test".to_string(),
            config: None,
        };
        let create_response = LoamSpineJsonRpcApiServer::create_spine(&server, create_request)
            .await
            .unwrap_or_else(|_| panic!("create failed"));

        // Then seal it
        let seal_request = SealSpineRequest {
            spine_id: create_response.spine_id,
            sealer: owner,
        };

        let result = LoamSpineJsonRpcApiServer::seal_spine(&server, seal_request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_jsonrpc_mint_and_get_certificate() {
        let server = LoamSpineJsonRpc::default_server();
        let owner = Did::new("did:key:z6MkTest");

        // Create spine first
        let create_request = CreateSpineRequest {
            owner: owner.clone(),
            name: "Cert Test".to_string(),
            config: None,
        };
        let create_response = LoamSpineJsonRpcApiServer::create_spine(&server, create_request)
            .await
            .unwrap_or_else(|_| panic!("create failed"));

        // Mint certificate
        let mint_request = MintCertificateRequest {
            spine_id: create_response.spine_id,
            cert_type: CertificateType::DigitalGame {
                platform: "steam".to_string(),
                game_id: "hl3".to_string(),
                edition: None,
            },
            owner: owner.clone(),
            metadata: None,
        };

        let mint_result = LoamSpineJsonRpcApiServer::mint_certificate(&server, mint_request).await;
        assert!(mint_result.is_ok());

        let mint_response = mint_result.unwrap_or_else(|_| panic!("mint failed"));

        // Get certificate
        let get_request = GetCertificateRequest {
            certificate_id: mint_response.certificate_id,
        };

        let get_result = LoamSpineJsonRpcApiServer::get_certificate(&server, get_request).await;
        assert!(get_result.is_ok());
    }

    #[tokio::test]
    async fn test_jsonrpc_commit_session() {
        let server = LoamSpineJsonRpc::default_server();
        let owner = Did::new("did:key:z6MkTest");

        // Create spine first
        let create_request = CreateSpineRequest {
            owner: owner.clone(),
            name: "Session Test".to_string(),
            config: None,
        };
        let create_response = LoamSpineJsonRpcApiServer::create_spine(&server, create_request)
            .await
            .unwrap_or_else(|_| panic!("create failed"));

        // Commit session
        let commit_request = CommitSessionRequest {
            spine_id: create_response.spine_id,
            committer: owner,
            session_id: uuid::Uuid::now_v7(),
            session_hash: [0u8; 32],
            vertex_count: 42,
        };

        let result = LoamSpineJsonRpcApiServer::commit_session(&server, commit_request).await;
        assert!(result.is_ok());
    }
}
