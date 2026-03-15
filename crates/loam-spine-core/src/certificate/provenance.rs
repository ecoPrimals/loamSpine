// SPDX-License-Identifier: AGPL-3.0-only

//! Certificate provenance and ownership tracking.
//!
//! Defines minting information, location, ownership records, and loan
//! history for full provenance chain verification.

use serde::{Deserialize, Serialize};

use crate::types::{Did, EntryHash, SpineId, Timestamp};

use super::lifecycle::LoanTerms;

/// Minting information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MintInfo {
    /// Who minted the certificate.
    pub minter: Did,
    /// Spine where minted.
    pub spine: SpineId,
    /// Mint entry hash.
    pub entry: EntryHash,
    /// Mint timestamp.
    pub timestamp: Timestamp,
    /// Minting authority (if delegated).
    pub authority: Option<MintingAuthority>,
}

impl MintInfo {
    /// Create new mint info.
    #[must_use]
    pub fn new(minter: Did, spine: SpineId, entry: EntryHash) -> Self {
        Self {
            minter,
            spine,
            entry,
            timestamp: Timestamp::now(),
            authority: None,
        }
    }
}

/// Minting authority (for delegated minting).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MintingAuthority {
    /// Authority DID.
    pub authority: Did,
    /// Authorization entry.
    pub authorization_entry: EntryHash,
}

/// Current certificate location.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificateLocation {
    /// Current spine.
    pub spine: SpineId,
    /// Latest state entry.
    pub entry: EntryHash,
    /// Entry index in spine.
    pub index: u64,
}

/// Certificate ownership record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OwnershipRecord {
    /// Owner at this point.
    pub owner: Did,
    /// Entry that established ownership.
    pub entry: EntryHash,
    /// Spine where entry exists.
    pub spine: SpineId,
    /// Ownership start time.
    pub from: Timestamp,
    /// Ownership end time (None if current).
    pub until: Option<Timestamp>,
    /// How ownership was acquired.
    pub acquisition: AcquisitionType,
}

/// How ownership was acquired.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AcquisitionType {
    /// Original mint.
    Mint,
    /// Transfer from previous owner.
    Transfer {
        /// Previous owner.
        from: Did,
    },
    /// Inherited (original owner dissolved).
    Inherited {
        /// Previous owner.
        from: Did,
    },
    /// Returned from loan.
    LoanReturn {
        /// Borrower who returned.
        borrower: Did,
    },
}

/// Loan record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoanRecord {
    /// Loan entry hash.
    pub loan_entry: EntryHash,
    /// Lender.
    pub lender: Did,
    /// Borrower.
    pub borrower: Did,
    /// Loan terms.
    pub terms: LoanTerms,
    /// Start time.
    pub started_at: Timestamp,
    /// End time.
    pub ended_at: Option<Timestamp>,
    /// Return entry hash.
    pub return_entry: Option<EntryHash>,
}
