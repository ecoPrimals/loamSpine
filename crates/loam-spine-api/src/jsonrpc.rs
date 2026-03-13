// SPDX-License-Identifier: AGPL-3.0-only

//! JSON-RPC 2.0 server for `LoamSpine`.
//!
//! Universal, language-agnostic RPC for external clients.
//! Works with Python, JavaScript, curl, etc.

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
    // Permanent Storage Compatibility (rhizoCrypt wire format)
    // ========================================================================
    // rhizoCrypt's LoamSpineHttpClient calls these `permanent-storage.*`
    // methods. They translate to loamSpine's native types internally.

    /// Commit a session using the rhizoCrypt wire format.
    #[method(name = "permanent-storage.commitSession")]
    async fn permanent_storage_commit_session(
        &self,
        request: PermanentStorageCommitRequest,
    ) -> RpcResult<PermanentStorageCommitResponse>;

    /// Verify a commit using the rhizoCrypt wire format.
    #[method(name = "permanent-storage.verifyCommit")]
    async fn permanent_storage_verify_commit(
        &self,
        request: PermanentStorageVerifyRequest,
    ) -> RpcResult<bool>;

    /// Get a commit using the rhizoCrypt wire format.
    #[method(name = "permanent-storage.getCommit")]
    async fn permanent_storage_get_commit(
        &self,
        request: PermanentStorageGetCommitRequest,
    ) -> RpcResult<serde_json::Value>;

    /// Health check using the rhizoCrypt wire format.
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

    async fn permanent_storage_commit_session(
        &self,
        request: PermanentStorageCommitRequest,
    ) -> RpcResult<PermanentStorageCommitResponse> {
        self.service
            .permanent_storage_commit_session(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn permanent_storage_verify_commit(
        &self,
        request: PermanentStorageVerifyRequest,
    ) -> RpcResult<bool> {
        self.service
            .permanent_storage_verify_commit(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn permanent_storage_get_commit(
        &self,
        request: PermanentStorageGetCommitRequest,
    ) -> RpcResult<serde_json::Value> {
        self.service
            .permanent_storage_get_commit(request)
            .await
            .map_err(to_rpc_error)
    }

    async fn permanent_storage_health_check(&self) -> RpcResult<bool> {
        Ok(true)
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
    use crate::types::{CertificateType, Did, EntryType};

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

    #[tokio::test]
    async fn test_jsonrpc_append_entry() {
        let server = LoamSpineJsonRpc::default_server();
        let owner = Did::new("did:key:z6MkTest");

        let create_request = CreateSpineRequest {
            owner: owner.clone(),
            name: "Entry Test".to_string(),
            config: None,
        };
        let create_response = LoamSpineJsonRpcApiServer::create_spine(&server, create_request)
            .await
            .unwrap_or_else(|_| panic!("create failed"));

        let append_request = AppendEntryRequest {
            spine_id: create_response.spine_id,
            entry_type: EntryType::DataAnchor {
                data_hash: [1u8; 32],
                mime_type: Some("text/plain".to_string()),
                size: 50,
            },
            committer: owner.clone(),
            payload: None,
        };
        let result = LoamSpineJsonRpcApiServer::append_entry(&server, append_request).await;
        assert!(result.is_ok());
        let response = result.unwrap_or_else(|_| panic!("append failed"));
        assert!(!response.entry_hash.iter().all(|&b| b == 0));
    }

    #[tokio::test]
    async fn test_jsonrpc_get_entry_and_tip() {
        let server = LoamSpineJsonRpc::default_server();
        let owner = Did::new("did:key:z6MkTest");

        let create_request = CreateSpineRequest {
            owner: owner.clone(),
            name: "Get Entry Test".to_string(),
            config: None,
        };
        let create_response = LoamSpineJsonRpcApiServer::create_spine(&server, create_request)
            .await
            .unwrap_or_else(|_| panic!("create failed"));

        let append_request = AppendEntryRequest {
            spine_id: create_response.spine_id,
            entry_type: EntryType::DataAnchor {
                data_hash: [2u8; 32],
                mime_type: Some("text/plain".to_string()),
                size: 10,
            },
            committer: owner.clone(),
            payload: None,
        };
        let append_response = LoamSpineJsonRpcApiServer::append_entry(&server, append_request)
            .await
            .unwrap_or_else(|_| panic!("append failed"));

        let get_entry_request = GetEntryRequest {
            spine_id: create_response.spine_id,
            entry_hash: append_response.entry_hash,
        };
        let result = LoamSpineJsonRpcApiServer::get_entry(&server, get_entry_request).await;
        assert!(result.is_ok());
        let response = result.unwrap_or_else(|_| panic!("get_entry failed"));
        assert!(response.found);

        let get_tip_request = GetTipRequest {
            spine_id: create_response.spine_id,
        };
        let result = LoamSpineJsonRpcApiServer::get_tip(&server, get_tip_request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_jsonrpc_liveness_and_readiness() {
        let server = LoamSpineJsonRpc::default_server();

        let liveness = LoamSpineJsonRpcApiServer::liveness(&server).await;
        assert!(liveness.is_ok());

        let readiness = LoamSpineJsonRpcApiServer::readiness(&server).await;
        assert!(readiness.is_ok());
    }

    #[tokio::test]
    async fn test_jsonrpc_commit_braid() {
        let server = LoamSpineJsonRpc::default_server();
        let owner = Did::new("did:key:z6MkTest");

        let create_request = CreateSpineRequest {
            owner: owner.clone(),
            name: "Braid Test".to_string(),
            config: None,
        };
        let create_response = LoamSpineJsonRpcApiServer::create_spine(&server, create_request)
            .await
            .unwrap_or_else(|_| panic!("create failed"));

        let braid_request = CommitBraidRequest {
            spine_id: create_response.spine_id,
            committer: owner,
            braid_id: uuid::Uuid::now_v7(),
            braid_hash: [3u8; 32],
            subjects: vec![],
        };
        let result = LoamSpineJsonRpcApiServer::commit_braid(&server, braid_request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_jsonrpc_anchor_slice() {
        let server = LoamSpineJsonRpc::default_server();
        let owner = Did::new("did:key:z6MkTest");

        let waypoint_request = CreateSpineRequest {
            owner: owner.clone(),
            name: "Waypoint".to_string(),
            config: None,
        };
        let waypoint_response = LoamSpineJsonRpcApiServer::create_spine(&server, waypoint_request)
            .await
            .unwrap_or_else(|_| panic!("create waypoint failed"));

        let origin_request = CreateSpineRequest {
            owner: owner.clone(),
            name: "Origin".to_string(),
            config: None,
        };
        let origin_response = LoamSpineJsonRpcApiServer::create_spine(&server, origin_request)
            .await
            .unwrap_or_else(|_| panic!("create origin failed"));

        let anchor_request = AnchorSliceRequest {
            waypoint_spine_id: waypoint_response.spine_id,
            slice_id: uuid::Uuid::now_v7(),
            origin_spine_id: origin_response.spine_id,
            committer: owner,
        };
        let result = LoamSpineJsonRpcApiServer::anchor_slice(&server, anchor_request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_jsonrpc_generate_and_verify_inclusion_proof() {
        let server = LoamSpineJsonRpc::default_server();
        let owner = Did::new("did:key:z6MkTest");

        let create_request = CreateSpineRequest {
            owner: owner.clone(),
            name: "Proof Test".to_string(),
            config: None,
        };
        let create_response = LoamSpineJsonRpcApiServer::create_spine(&server, create_request)
            .await
            .unwrap_or_else(|_| panic!("create failed"));

        let append_request = AppendEntryRequest {
            spine_id: create_response.spine_id,
            entry_type: EntryType::DataAnchor {
                data_hash: [4u8; 32],
                mime_type: Some("text/plain".to_string()),
                size: 20,
            },
            committer: owner,
            payload: None,
        };
        let append_response = LoamSpineJsonRpcApiServer::append_entry(&server, append_request)
            .await
            .unwrap_or_else(|_| panic!("append failed"));

        let gen_request = GenerateInclusionProofRequest {
            spine_id: create_response.spine_id,
            entry_hash: append_response.entry_hash,
        };
        let gen_result =
            LoamSpineJsonRpcApiServer::generate_inclusion_proof(&server, gen_request).await;
        assert!(gen_result.is_ok());

        let proof = gen_result.unwrap_or_else(|_| panic!("generate failed"));
        let verify_request = VerifyInclusionProofRequest { proof: proof.proof };
        let verify_result =
            LoamSpineJsonRpcApiServer::verify_inclusion_proof(&server, verify_request).await;
        assert!(verify_result.is_ok());
        let response = verify_result.unwrap_or_else(|_| panic!("verify failed"));
        assert!(response.valid);
    }
}
