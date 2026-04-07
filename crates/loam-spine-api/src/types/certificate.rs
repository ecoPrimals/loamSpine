// SPDX-License-Identifier: AGPL-3.0-or-later

//! Certificate operation RPC types.

use serde::{Deserialize, Serialize};

use super::{
    Certificate, CertificateId, CertificateMetadata, CertificateType, Did, EntryHash, LoanTerms,
    SpineId,
};

/// Request to mint a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintCertificateRequest {
    /// Spine ID to mint on
    pub spine_id: SpineId,
    /// Certificate type
    pub cert_type: CertificateType,
    /// Owner DID
    pub owner: Did,
    /// Certificate metadata
    pub metadata: Option<CertificateMetadata>,
}

/// Response from minting a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintCertificateResponse {
    /// Certificate ID
    pub certificate_id: CertificateId,
    /// Mint entry hash
    pub mint_hash: EntryHash,
}

/// Request to transfer a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferCertificateRequest {
    /// Certificate ID
    pub certificate_id: CertificateId,
    /// Current owner DID
    pub from: Did,
    /// New owner DID
    pub to: Did,
}

/// Response from transferring a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferCertificateResponse {
    /// Whether transfer succeeded
    pub success: bool,
    /// Transfer entry hash
    pub transfer_hash: Option<EntryHash>,
}

/// Request to loan a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanCertificateRequest {
    /// Certificate ID
    pub certificate_id: CertificateId,
    /// Lender DID
    pub lender: Did,
    /// Borrower DID
    pub borrower: Did,
    /// Loan terms
    pub terms: LoanTerms,
}

/// Response from loaning a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanCertificateResponse {
    /// Whether loan succeeded
    pub success: bool,
    /// Loan entry hash
    pub loan_hash: Option<EntryHash>,
}

/// Request to return a loaned certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnCertificateRequest {
    /// Certificate ID
    pub certificate_id: CertificateId,
    /// Returner DID (borrower)
    pub returner: Did,
}

/// Response from returning a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnCertificateResponse {
    /// Whether return succeeded
    pub success: bool,
    /// Return entry hash
    pub return_hash: Option<EntryHash>,
}

/// Request to get a certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCertificateRequest {
    /// Certificate ID
    pub certificate_id: CertificateId,
}

/// Response containing certificate data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCertificateResponse {
    /// Whether the certificate was found
    pub found: bool,
    /// The certificate if found
    pub certificate: Option<Certificate>,
}
