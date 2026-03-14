// SPDX-License-Identifier: AGPL-3.0-only

//! Inclusion proof generation and verification.

use super::LoamSpineRpcService;
use crate::error::ApiResult;
use crate::types::*;

impl LoamSpineRpcService {
    /// Generate inclusion proof.
    ///
    /// # Errors
    ///
    /// Returns error if proof generation fails.
    pub async fn generate_inclusion_proof(
        &self,
        request: GenerateInclusionProofRequest,
    ) -> ApiResult<GenerateInclusionProofResponse> {
        let proof = {
            let core = self.core().await;
            core.generate_inclusion_proof(request.spine_id, request.entry_hash)
                .await
                .map_err(crate::error::ApiError::from)?
        };

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
        let valid = request.proof.verify()?;
        Ok(VerifyInclusionProofResponse {
            valid,
            message: if valid {
                "Proof verified".to_string()
            } else {
                "Proof verification failed".to_string()
            },
        })
    }
}
