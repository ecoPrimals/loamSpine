// SPDX-License-Identifier: AGPL-3.0-only

//! Certificate mint, transfer, loan, return, and get operations.

use super::LoamSpineRpcService;
use crate::error::{ApiError, ApiResult};
use crate::types::*;

impl LoamSpineRpcService {
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
}
