// SPDX-License-Identifier: AGPL-3.0-only

//! Certificate types for LoamSpine.
//!
//! Certificates are memory-bound objects with verifiable ownership,
//! transferability, and complete provenance history. Unlike blockchain NFTs,
//! Loam Certificates are sovereign, lendable, and have complete history.

mod escrow;
mod lifecycle;
mod metadata;
mod provenance;
mod types;
mod usage;

#[cfg(test)]
mod tests;

pub use escrow::{EscrowCondition, EscrowId, TransferConditions};
pub use lifecycle::{CertificateState, LoanInfo, LoanTerms, RevocationReason};
pub use metadata::{
    CertificateMetadata, SCYBORG_LICENSE_SCHEMA_VERSION, SCYBORG_LICENSE_TYPE_URI,
    SCYBORG_META_CATEGORY, SCYBORG_META_COPYRIGHT, SCYBORG_META_SHARE_ALIKE, SCYBORG_META_SPDX,
    SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE, SECONDS_PER_WEEK, SECONDS_PER_YEAR,
};
pub use provenance::{
    AcquisitionType, CertificateLocation, LoanRecord, MintInfo, MintingAuthority, OwnershipRecord,
};
pub use types::{CertificateType, MediaType, Rarity};
pub use usage::UsageSummary;

use serde::{Deserialize, Serialize};

use crate::types::{CertificateId, Did, Timestamp};

/// A Loam Certificate (memory-bound object).
///
/// Certificates represent digital ownership with full provenance tracking.
/// They can be minted, transferred, loaned, and returned.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Certificate {
    /// Unique certificate ID.
    pub id: CertificateId,

    /// Certificate type.
    pub cert_type: CertificateType,

    /// Certificate version (for schema evolution).
    pub version: u32,

    /// Current owner.
    pub owner: Did,

    /// Current holder (if loaned, different from owner).
    pub holder: Option<Did>,

    /// Minting information.
    pub mint_info: MintInfo,

    /// Current location.
    pub current_location: CertificateLocation,

    /// Certificate state.
    pub state: CertificateState,

    /// Transfer count.
    pub transfer_count: u64,

    /// Active loan (if any).
    pub active_loan: Option<LoanInfo>,

    /// Certificate metadata.
    pub metadata: CertificateMetadata,

    /// Creation timestamp.
    pub created_at: Timestamp,

    /// Last update timestamp.
    pub updated_at: Timestamp,
}

impl Certificate {
    /// Create a new certificate.
    #[must_use]
    pub fn new(
        id: CertificateId,
        cert_type: CertificateType,
        owner: &Did,
        mint_info: &MintInfo,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id,
            cert_type,
            version: 1,
            owner: owner.clone(),
            holder: None,
            mint_info: mint_info.clone(),
            current_location: CertificateLocation {
                spine: mint_info.spine,
                entry: mint_info.entry,
                index: 0,
            },
            state: CertificateState::Active,
            transfer_count: 0,
            active_loan: None,
            metadata: CertificateMetadata::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if the certificate is active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self.state, CertificateState::Active)
    }

    /// Check if the certificate is loaned.
    #[must_use]
    pub const fn is_loaned(&self) -> bool {
        matches!(self.state, CertificateState::Loaned { .. })
    }

    /// Check if the certificate is revoked.
    #[must_use]
    pub const fn is_revoked(&self) -> bool {
        matches!(self.state, CertificateState::Revoked { .. })
    }

    /// Get the effective holder (holder if loaned, owner otherwise).
    #[must_use]
    pub fn effective_holder(&self) -> &Did {
        self.holder.as_ref().unwrap_or(&self.owner)
    }

    /// Set metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: CertificateMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Full certificate history.
///
/// Contains the certificate and its complete ownership and loan records.
#[derive(Clone, Debug)]
pub struct CertificateHistory {
    /// The certificate.
    pub certificate: Certificate,
    /// Ownership records.
    pub ownership_records: Vec<OwnershipRecord>,
    /// Loan records.
    pub loan_records: Vec<LoanRecord>,
}
