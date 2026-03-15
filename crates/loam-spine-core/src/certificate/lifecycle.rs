// SPDX-License-Identifier: AGPL-3.0-only

//! Certificate lifecycle state and loan management.
//!
//! Defines certificate states (Active, Loaned, Revoked, etc.), loan terms,
//! and revocation reasons.

use serde::{Deserialize, Serialize};

use crate::types::{CertificateId, Did, EntryHash, SpineId, Timestamp};
use crate::waypoint::RelendingChain;

/// Certificate state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateState {
    /// Active and owned.
    Active,

    /// Currently loaned out.
    Loaned {
        /// Loan entry hash.
        loan_entry: EntryHash,
    },

    /// Pending transfer (escrow).
    PendingTransfer {
        /// Transfer entry hash.
        transfer_entry: EntryHash,
        /// Recipient.
        to: Did,
    },

    /// Revoked (no longer valid).
    Revoked {
        /// Revoke entry hash.
        revoke_entry: EntryHash,
        /// Revocation reason.
        reason: RevocationReason,
    },

    /// Expired (time-limited certificate).
    Expired {
        /// Expiration timestamp.
        expired_at: Timestamp,
    },
}

/// Revocation reason.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevocationReason {
    /// Fraudulent certificate.
    Fraud,
    /// Terms violated.
    TermsViolation,
    /// Replaced by new certificate.
    Superseded {
        /// Replacement certificate.
        replacement: CertificateId,
    },
    /// Administrative revocation.
    Administrative {
        /// Reason text.
        reason: String,
    },
    /// Owner requested revocation.
    OwnerRequest,
}

/// Loan information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoanInfo {
    /// Loan entry hash.
    pub loan_entry: EntryHash,
    /// Borrower DID (current holder).
    pub borrower: Did,
    /// Loan terms.
    pub terms: LoanTerms,
    /// Start time.
    pub started_at: Timestamp,
    /// Expected end time.
    pub expires_at: Option<Timestamp>,
    /// Waypoint spine (if anchored).
    pub waypoint: Option<SpineId>,
    /// Waypoint anchor entry.
    pub waypoint_anchor: Option<EntryHash>,
    /// Relending chain (when sub-lent).
    #[serde(default)]
    pub relending_chain: Option<RelendingChain>,
}

/// Loan terms.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LoanTerms {
    /// Loan duration in seconds.
    pub duration_secs: Option<u64>,
    /// Grace period in seconds.
    pub grace_period_secs: Option<u64>,
    /// Automatic return on expiry.
    pub auto_return: bool,
    /// Allowed operations during loan.
    pub allowed_operations: Vec<String>,
    /// Forbidden operations during loan.
    pub forbidden_operations: Vec<String>,
    /// Allow sub-lending.
    pub allow_sublend: bool,
    /// Maximum sublend depth.
    pub max_sublend_depth: Option<u32>,
}

impl LoanTerms {
    /// Create default loan terms.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set duration.
    #[must_use]
    pub const fn with_duration(mut self, secs: u64) -> Self {
        self.duration_secs = Some(secs);
        self
    }

    /// Set auto-return.
    #[must_use]
    pub const fn with_auto_return(mut self, auto: bool) -> Self {
        self.auto_return = auto;
        self
    }

    /// Allow sublending.
    #[must_use]
    pub const fn with_sublend(mut self, allow: bool, max_depth: Option<u32>) -> Self {
        self.allow_sublend = allow;
        self.max_sublend_depth = max_depth;
        self
    }
}
