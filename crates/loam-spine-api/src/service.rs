//! RPC service implementation for `LoamSpine`.
//!
//! Implements the `LoamSpineRpc` trait defined in `rpc.rs`.

// Allow unused_async for stub implementations that will be async when completed
#![allow(clippy::unused_async)]
// Allow wildcard imports for re-exported types
#![allow(clippy::wildcard_imports)]

use crate::error::{ApiError, ApiResult};
use crate::types::*;
use loam_spine_core::service::LoamSpineService as CoreService;
use loam_spine_core::traits::{
    BraidAcceptor, BraidSummary, CommitAcceptor, DehydrationSummary, SliceManager, SpineQuery,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// RPC service implementation backed by the core `LoamSpineService`.
#[derive(Clone)]
pub struct LoamSpineRpcService {
    core: Arc<RwLock<CoreService>>,
}

impl LoamSpineRpcService {
    /// Create a new RPC service.
    #[must_use]
    pub fn new(core: CoreService) -> Self {
        Self {
            core: Arc::new(RwLock::new(core)),
        }
    }

    /// Create with default core service.
    #[must_use]
    pub fn default_service() -> Self {
        Self::new(CoreService::new())
    }

    /// Get read access to the core service.
    pub async fn core(&self) -> tokio::sync::RwLockReadGuard<'_, CoreService> {
        self.core.read().await
    }

    /// Get write access to the core service.
    pub async fn core_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, CoreService> {
        self.core.write().await
    }
}

// Implementation of the RPC trait for our service
impl LoamSpineRpcService {
    /// Create a new spine.
    ///
    /// # Errors
    ///
    /// Returns error if spine creation fails.
    pub async fn create_spine(
        &self,
        request: CreateSpineRequest,
    ) -> ApiResult<CreateSpineResponse> {
        let core = self.core_mut().await;
        let spine_id = core
            .ensure_spine(request.owner.clone(), Some(request.name))
            .await
            .map_err(ApiError::from)?;

        // Get the spine to get genesis hash
        let spine = core
            .get_spine(spine_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::SpineNotFound(format!("{spine_id:?}")))?;

        Ok(CreateSpineResponse {
            spine_id,
            genesis_hash: spine.genesis,
        })
    }

    /// Get a spine by ID.
    ///
    /// # Errors
    ///
    /// Returns error if spine lookup fails.
    pub async fn get_spine(&self, request: GetSpineRequest) -> ApiResult<GetSpineResponse> {
        let core = self.core().await;
        match core.get_spine(request.spine_id).await {
            Ok(Some(spine)) => Ok(GetSpineResponse {
                found: true,
                spine: Some(spine),
            }),
            Ok(None) | Err(_) => Ok(GetSpineResponse {
                found: false,
                spine: None,
            }),
        }
    }

    /// Seal a spine.
    ///
    /// # Errors
    ///
    /// Returns error if sealing fails.
    pub async fn seal_spine(&self, request: SealSpineRequest) -> ApiResult<SealSpineResponse> {
        let core = self.core_mut().await;
        match core.seal_spine(request.spine_id, None).await {
            Ok(hash) => Ok(SealSpineResponse {
                success: true,
                seal_hash: Some(hash),
            }),
            Err(e) => Err(ApiError::from(e)),
        }
    }

    /// Append an entry.
    ///
    /// # Errors
    ///
    /// Returns error if append fails.
    pub async fn append_entry(
        &self,
        request: AppendEntryRequest,
    ) -> ApiResult<AppendEntryResponse> {
        let core = self.core_mut().await;
        let entry_hash = core
            .append_entry(request.spine_id, request.entry_type)
            .await
            .map_err(ApiError::from)?;

        // Get the spine to get the new height
        let spine = core
            .get_spine(request.spine_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::SpineNotFound(format!("{:?}", request.spine_id)))?;

        Ok(AppendEntryResponse {
            entry_hash,
            index: spine.height - 1,
        })
    }

    /// Get an entry by hash.
    ///
    /// # Errors
    ///
    /// Returns error if lookup fails.
    pub async fn get_entry(&self, request: GetEntryRequest) -> ApiResult<GetEntryResponse> {
        let core = self.core().await;
        // Note: Core get_entry takes only hash, not spine_id
        match core.get_entry(request.entry_hash).await {
            Ok(Some(entry)) => Ok(GetEntryResponse {
                found: true,
                entry: Some(entry),
            }),
            Ok(None) | Err(_) => Ok(GetEntryResponse {
                found: false,
                entry: None,
            }),
        }
    }

    /// Get the tip entry.
    ///
    /// # Errors
    ///
    /// Returns error if spine not found.
    pub async fn get_tip(&self, request: GetTipRequest) -> ApiResult<GetTipResponse> {
        let core = self.core().await;
        let mut entry = core
            .get_tip(request.spine_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::SpineNotFound(format!("{:?}", request.spine_id)))?;

        let spine = core
            .get_spine(request.spine_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::SpineNotFound(format!("{:?}", request.spine_id)))?;

        let tip_hash = entry.hash();

        Ok(GetTipResponse {
            tip_hash,
            entry,
            height: spine.height,
        })
    }

    /// Mint a certificate.
    ///
    /// # Errors
    ///
    /// Returns error if minting fails.
    pub async fn mint_certificate(
        &self,
        request: MintCertificateRequest,
    ) -> ApiResult<MintCertificateResponse> {
        let core = self.core_mut().await;
        let (cert_id, mint_hash) = core
            .mint_certificate(
                request.spine_id,
                request.cert_type,
                request.owner,
                request.metadata,
            )
            .await
            .map_err(ApiError::from)?;

        Ok(MintCertificateResponse {
            certificate_id: cert_id,
            mint_hash,
        })
    }

    /// Transfer a certificate.
    ///
    /// # Errors
    ///
    /// Returns error if transfer fails.
    pub async fn transfer_certificate(
        &self,
        request: TransferCertificateRequest,
    ) -> ApiResult<TransferCertificateResponse> {
        let core = self.core_mut().await;
        match core
            .transfer_certificate(request.certificate_id, request.from, request.to)
            .await
        {
            Ok(hash) => Ok(TransferCertificateResponse {
                success: true,
                transfer_hash: Some(hash),
            }),
            Err(e) => Err(ApiError::from(e)),
        }
    }

    /// Loan a certificate.
    ///
    /// # Errors
    ///
    /// Returns error if loan fails.
    pub async fn loan_certificate(
        &self,
        request: LoanCertificateRequest,
    ) -> ApiResult<LoanCertificateResponse> {
        let core = self.core_mut().await;
        match core
            .loan_certificate(
                request.certificate_id,
                request.lender,
                request.borrower,
                request.terms,
            )
            .await
        {
            Ok(hash) => Ok(LoanCertificateResponse {
                success: true,
                loan_hash: Some(hash),
            }),
            Err(e) => Err(ApiError::from(e)),
        }
    }

    /// Return a certificate.
    ///
    /// # Errors
    ///
    /// Returns error if return fails.
    pub async fn return_certificate(
        &self,
        request: ReturnCertificateRequest,
    ) -> ApiResult<ReturnCertificateResponse> {
        let core = self.core_mut().await;
        match core
            .return_certificate(request.certificate_id, request.returner)
            .await
        {
            Ok(hash) => Ok(ReturnCertificateResponse {
                success: true,
                return_hash: Some(hash),
            }),
            Err(e) => Err(ApiError::from(e)),
        }
    }

    /// Get a certificate.
    ///
    /// # Errors
    ///
    /// Returns error if lookup fails.
    pub async fn get_certificate(
        &self,
        request: GetCertificateRequest,
    ) -> ApiResult<GetCertificateResponse> {
        let core = self.core().await;
        match core.get_certificate(request.certificate_id).await {
            Some(cert) => Ok(GetCertificateResponse {
                found: true,
                certificate: Some(cert),
            }),
            None => Ok(GetCertificateResponse {
                found: false,
                certificate: None,
            }),
        }
    }

    /// Anchor a slice.
    ///
    /// # Errors
    ///
    /// Returns error if anchoring fails.
    pub async fn anchor_slice(
        &self,
        request: AnchorSliceRequest,
    ) -> ApiResult<AnchorSliceResponse> {
        let core = self.core_mut().await;

        // We need an origin entry hash - use a placeholder for now
        // In a real implementation, this would be looked up from the origin spine
        let origin_entry = [0u8; 32]; // Placeholder

        let anchor_hash = core
            .anchor_slice(
                request.waypoint_spine_id,
                request.slice_id,
                request.origin_spine_id,
                origin_entry,
            )
            .await
            .map_err(ApiError::from)?;

        Ok(AnchorSliceResponse { anchor_hash })
    }

    /// Checkout a slice.
    ///
    /// # Errors
    ///
    /// Returns error if checkout fails.
    pub async fn checkout_slice(
        &self,
        request: CheckoutSliceRequest,
    ) -> ApiResult<CheckoutSliceResponse> {
        let core = self.core_mut().await;

        // Generate a session ID for this checkout
        let session_id = loam_spine_core::types::SessionId::now_v7();

        // We need the entry hash - for now use a placeholder
        // In a real implementation, we'd look up the slice's entry hash
        let entry_hash = [0u8; 32];

        match core
            .checkout_slice(
                request.waypoint_spine_id,
                entry_hash,
                request.requester,
                session_id,
            )
            .await
        {
            Ok(_origin) => Ok(CheckoutSliceResponse {
                success: true,
                checkout_hash: Some(entry_hash), // Return the checkout entry hash
            }),
            Err(e) => Err(ApiError::from(e)),
        }
    }

    /// Generate inclusion proof.
    ///
    /// # Errors
    ///
    /// Returns error if proof generation fails.
    pub async fn generate_inclusion_proof(
        &self,
        request: GenerateInclusionProofRequest,
    ) -> ApiResult<GenerateInclusionProofResponse> {
        let core = self.core().await;
        let proof = core
            .generate_inclusion_proof(request.spine_id, request.entry_hash)
            .await
            .map_err(ApiError::from)?;

        Ok(GenerateInclusionProofResponse { proof })
    }

    /// Verify inclusion proof.
    ///
    /// # Errors
    ///
    /// Returns error if verification fails.
    pub async fn verify_inclusion_proof(
        &self,
        request: VerifyInclusionProofRequest,
    ) -> ApiResult<VerifyInclusionProofResponse> {
        let valid = request.proof.verify();
        Ok(VerifyInclusionProofResponse {
            valid,
            message: if valid {
                "Proof verified".to_string()
            } else {
                "Proof verification failed".to_string()
            },
        })
    }

    /// Health check.
    ///
    /// # Errors
    ///
    /// Returns error if health check fails.
    pub async fn health_check(
        &self,
        request: HealthCheckRequest,
    ) -> ApiResult<HealthCheckResponse> {
        let core = self.core().await;
        let status = HealthStatus::Healthy;
        let spine_count = core.spine_count().await;

        let report = if request.include_details {
            Some(HealthReport {
                name: "LoamSpine".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                status: status.clone(),
                uptime_secs: Some(0),
                components: vec![loam_spine_core::primal::ComponentHealth::healthy(format!(
                    "storage: {spine_count} spines"
                ))],
            })
        } else {
            None
        };

        Ok(HealthCheckResponse { status, report })
    }
    
    /// Liveness probe (Kubernetes-style).
    ///
    /// Returns whether the process is alive.
    pub async fn liveness(&self) -> crate::health::LivenessProbe {
        // If we can execute this code, we're alive
        crate::health::LivenessProbe { alive: true }
    }
    
    /// Readiness probe (Kubernetes-style).
    ///
    /// Returns whether the service is ready for traffic.
    ///
    /// # Errors
    ///
    /// Returns error if readiness check fails.
    pub async fn readiness(&self) -> ApiResult<crate::health::ReadinessProbe> {
        // Check if we can access core service
        let _core = self.core().await;
        
        // If we got here, we're ready
        Ok(crate::health::ReadinessProbe {
            ready: true,
            reason: None,
        })
    }

    /// Commit a session from an ephemeral storage primal.
    ///
    /// # Errors
    ///
    /// Returns error if commit fails.
    pub async fn commit_session(
        &self,
        request: CommitSessionRequest,
    ) -> ApiResult<CommitSessionResponse> {
        let core = self.core_mut().await;

        // Build dehydration summary from request
        let summary = DehydrationSummary::new(request.session_id, "session", request.session_hash)
            .with_vertex_count(request.vertex_count);

        let commit_ref = core
            .commit_session(request.spine_id, request.committer, summary)
            .await
            .map_err(ApiError::from)?;

        Ok(CommitSessionResponse {
            commit_hash: commit_ref.entry_hash,
            index: commit_ref.index,
        })
    }

    /// Commit a braid from a semantic attribution primal.
    ///
    /// # Errors
    ///
    /// Returns error if commit fails.
    pub async fn commit_braid(
        &self,
        request: CommitBraidRequest,
    ) -> ApiResult<CommitBraidResponse> {
        let core = self.core_mut().await;

        // Build braid summary from request
        // BraidSummary::new takes (braid_id, braid_type, subject_hash, braid_hash)
        let mut braid = BraidSummary::new(
            request.braid_id,
            "attribution",
            request.braid_hash, // Using braid_hash as subject_hash
            request.braid_hash,
        );

        // Add agents from subjects
        for agent in request.subjects {
            braid = braid.with_agent(agent);
        }

        let hash = core
            .commit_braid(request.spine_id, request.committer, braid)
            .await
            .map_err(ApiError::from)?;

        // Note: commit_braid returns EntryHash, not index
        // We return 0 for index since we don't have it
        Ok(CommitBraidResponse {
            commit_hash: hash,
            index: 0,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use loam_spine_core::KB;

    #[tokio::test]
    async fn test_service_creation() {
        let service = LoamSpineRpcService::default_service();
        let result = service
            .health_check(HealthCheckRequest {
                include_details: true,
            })
            .await;
        assert!(result.is_ok());
        let resp = result.expect("health check should succeed");
        assert!(matches!(resp.status, HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_create_and_get_spine() {
        let service = LoamSpineRpcService::default_service();
        let owner = Did::new("did:key:test");

        let create_resp = service
            .create_spine(CreateSpineRequest {
                name: "test".to_string(),
                owner: owner.clone(),
                config: None,
            })
            .await
            .expect("create should succeed");

        let get_resp = service
            .get_spine(GetSpineRequest {
                spine_id: create_resp.spine_id,
            })
            .await
            .expect("get should succeed");

        assert!(get_resp.found);
        assert!(get_resp.spine.is_some());
    }

    #[tokio::test]
    async fn test_mint_certificate() {
        let service = LoamSpineRpcService::default_service();
        let owner = Did::new("did:key:test-owner");

        // Create spine first
        let spine_resp = service
            .create_spine(CreateSpineRequest {
                name: "cert-test".to_string(),
                owner: owner.clone(),
                config: None,
            })
            .await
            .expect("create spine should succeed");

        // Mint a certificate
        let mint_resp = service
            .mint_certificate(MintCertificateRequest {
                spine_id: spine_resp.spine_id,
                cert_type: CertificateType::DigitalGame {
                    platform: "steam".into(),
                    game_id: "hl3".into(),
                    edition: None,
                },
                owner: owner.clone(),
                metadata: None,
            })
            .await
            .expect("mint should succeed");

        assert_ne!(mint_resp.mint_hash, [0u8; 32]);

        // Get the certificate
        let get_resp = service
            .get_certificate(GetCertificateRequest {
                certificate_id: mint_resp.certificate_id,
            })
            .await
            .expect("get certificate should succeed");

        assert!(get_resp.found);
        assert!(get_resp.certificate.is_some());
    }

    #[tokio::test]
    async fn test_certificate_transfer() {
        let service = LoamSpineRpcService::default_service();
        let owner = Did::new("did:key:owner");
        let new_owner = Did::new("did:key:new-owner");

        // Create spine and mint certificate
        let spine_resp = service
            .create_spine(CreateSpineRequest {
                name: "transfer-test".to_string(),
                owner: owner.clone(),
                config: None,
            })
            .await
            .expect("create should succeed");

        let mint_resp = service
            .mint_certificate(MintCertificateRequest {
                spine_id: spine_resp.spine_id,
                cert_type: CertificateType::SoftwareLicense {
                    software_id: "cursor".into(),
                    license_type: "pro".into(),
                    seats: Some(1),
                    expires: None,
                },
                owner: owner.clone(),
                metadata: None,
            })
            .await
            .expect("mint should succeed");

        // Transfer
        let transfer_resp = service
            .transfer_certificate(TransferCertificateRequest {
                certificate_id: mint_resp.certificate_id,
                from: owner,
                to: new_owner.clone(),
            })
            .await
            .expect("transfer should succeed");

        assert!(transfer_resp.success);
        assert!(transfer_resp.transfer_hash.is_some());

        // Verify new owner
        let get_resp = service
            .get_certificate(GetCertificateRequest {
                certificate_id: mint_resp.certificate_id,
            })
            .await
            .expect("get should succeed");

        assert!(get_resp.found);
        let cert = get_resp.certificate.expect("certificate should exist");
        assert_eq!(cert.owner, new_owner);
    }

    #[tokio::test]
    async fn test_certificate_loan_and_return() {
        let service = LoamSpineRpcService::default_service();
        let owner = Did::new("did:key:lender");
        let borrower = Did::new("did:key:borrower");

        // Create spine and mint certificate
        let spine_resp = service
            .create_spine(CreateSpineRequest {
                name: "loan-test".to_string(),
                owner: owner.clone(),
                config: None,
            })
            .await
            .expect("create should succeed");

        let mint_resp = service
            .mint_certificate(MintCertificateRequest {
                spine_id: spine_resp.spine_id,
                cert_type: CertificateType::DigitalCollectible {
                    collection_id: "cards".into(),
                    item_number: Some(42),
                    total_supply: Some(1000),
                    rarity: None,
                },
                owner: owner.clone(),
                metadata: None,
            })
            .await
            .expect("mint should succeed");

        // Loan
        let loan_resp = service
            .loan_certificate(LoanCertificateRequest {
                certificate_id: mint_resp.certificate_id,
                lender: owner.clone(),
                borrower: borrower.clone(),
                terms: LoanTerms::default(),
            })
            .await
            .expect("loan should succeed");

        assert!(loan_resp.success);
        assert!(loan_resp.loan_hash.is_some());

        // Return
        let return_resp = service
            .return_certificate(ReturnCertificateRequest {
                certificate_id: mint_resp.certificate_id,
                returner: borrower,
            })
            .await
            .expect("return should succeed");

        assert!(return_resp.success);
        assert!(return_resp.return_hash.is_some());

        // Verify certificate is back with owner
        let get_resp = service
            .get_certificate(GetCertificateRequest {
                certificate_id: mint_resp.certificate_id,
            })
            .await
            .expect("get should succeed");

        let cert = get_resp.certificate.expect("certificate should exist");
        assert_eq!(cert.owner, owner);
        assert!(!cert.is_loaned());
    }

    #[tokio::test]
    async fn test_seal_spine() {
        let service = LoamSpineRpcService::default_service();
        let owner = Did::new("did:key:sealer");

        // Create spine
        let create_resp = service
            .create_spine(CreateSpineRequest {
                name: "seal-test".to_string(),
                owner: owner.clone(),
                config: None,
            })
            .await
            .expect("create should succeed");

        // Seal the spine
        let seal_resp = service
            .seal_spine(SealSpineRequest {
                spine_id: create_resp.spine_id,
                sealer: owner,
            })
            .await
            .expect("seal should succeed");

        assert!(seal_resp.success);
        assert!(seal_resp.seal_hash.is_some());

        // Verify spine is sealed (attempting to seal again should fail)
        let seal_again = service
            .seal_spine(SealSpineRequest {
                spine_id: create_resp.spine_id,
                sealer: Did::new("did:key:other"),
            })
            .await;

        assert!(seal_again.is_err());
    }

    #[tokio::test]
    async fn test_append_entry() {
        let service = LoamSpineRpcService::default_service();
        let owner = Did::new("did:key:appender");

        // Create spine
        let create_resp = service
            .create_spine(CreateSpineRequest {
                name: "append-test".to_string(),
                owner: owner.clone(),
                config: None,
            })
            .await
            .expect("create should succeed");

        // Append a data anchor entry
        let append_resp = service
            .append_entry(AppendEntryRequest {
                spine_id: create_resp.spine_id,
                entry_type: EntryType::DataAnchor {
                    data_hash: [42u8; 32],
                    mime_type: Some("application/json".into()),
                    size: KB,
                },
                committer: owner.clone(),
                payload: None,
            })
            .await
            .expect("append should succeed");

        assert_ne!(append_resp.entry_hash, [0u8; 32]);
        assert_eq!(append_resp.index, 1); // After genesis

        // Get the entry
        let get_resp = service
            .get_entry(GetEntryRequest {
                spine_id: create_resp.spine_id,
                entry_hash: append_resp.entry_hash,
            })
            .await
            .expect("get should succeed");

        assert!(get_resp.found);
        assert!(get_resp.entry.is_some());
    }

    #[tokio::test]
    async fn test_anchor_slice() {
        let service = LoamSpineRpcService::default_service();
        let owner = Did::new("did:key:waypoint-owner");

        // Create a waypoint spine
        let waypoint_resp = service
            .create_spine(CreateSpineRequest {
                name: "waypoint-spine".to_string(),
                owner: owner.clone(),
                config: None,
            })
            .await
            .expect("create waypoint should succeed");

        // Create an origin spine
        let origin_resp = service
            .create_spine(CreateSpineRequest {
                name: "origin-spine".to_string(),
                owner: owner.clone(),
                config: None,
            })
            .await
            .expect("create origin should succeed");

        // Anchor a slice
        let slice_id = loam_spine_core::types::SliceId::now_v7();
        let anchor_resp = service
            .anchor_slice(AnchorSliceRequest {
                waypoint_spine_id: waypoint_resp.spine_id,
                slice_id,
                origin_spine_id: origin_resp.spine_id,
                committer: owner,
            })
            .await
            .expect("anchor should succeed");

        assert_ne!(anchor_resp.anchor_hash, [0u8; 32]);
    }

    #[tokio::test]
    async fn test_generate_inclusion_proof() {
        let service = LoamSpineRpcService::default_service();
        let owner = Did::new("did:key:prover");

        // Create spine
        let create_resp = service
            .create_spine(CreateSpineRequest {
                name: "proof-test".to_string(),
                owner: owner.clone(),
                config: None,
            })
            .await
            .expect("create should succeed");

        // Append an entry
        let append_resp = service
            .append_entry(AppendEntryRequest {
                spine_id: create_resp.spine_id,
                entry_type: EntryType::DataAnchor {
                    data_hash: [99u8; 32],
                    mime_type: Some("text/plain".into()),
                    size: 512,
                },
                committer: owner,
                payload: None,
            })
            .await
            .expect("append should succeed");

        // Generate inclusion proof for the entry
        let proof_resp = service
            .generate_inclusion_proof(GenerateInclusionProofRequest {
                spine_id: create_resp.spine_id,
                entry_hash: append_resp.entry_hash,
            })
            .await
            .expect("proof generation should succeed");

        // Verify the proof
        assert!(proof_resp.proof.verify());
        assert_eq!(proof_resp.proof.spine_id, create_resp.spine_id);
    }
}
