// SPDX-License-Identifier: AGPL-3.0-or-later

//! Certificate loan lifecycle operations.
//!
//! Handles loan, return, sublend, and auto-return for expired certificates.
//! Loan operations maintain a relending chain for multi-hop sublending.

use crate::certificate::{CertificateLocation, CertificateState, LoanInfo, LoanTerms};
use crate::entry::EntryType;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::storage::{CertificateStorage, EntryStorage, SpineStorage};
use crate::types::{CertificateId, Did, EntryHash, Timestamp};
use crate::waypoint::RelendingChain;

use super::LoamSpineService;

impl LoamSpineService {
    /// Loan a certificate to another party.
    ///
    /// The owner retains ownership but the borrower gains temporary access.
    /// Loan terms define duration, auto-return, and sublending permissions.
    ///
    /// # Errors
    ///
    /// Returns error if certificate not found, caller is not owner, or loan fails.
    pub async fn loan_certificate(
        &self,
        cert_id: CertificateId,
        lender: Did,
        borrower: Did,
        terms: LoanTerms,
    ) -> LoamSpineResult<EntryHash> {
        let (mut cert, spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        if cert.owner != lender {
            return Err(LoamSpineError::NotCertificateOwner);
        }

        if cert.is_loaned() {
            return Err(LoamSpineError::CertificateLoaned(cert_id));
        }

        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entry = spine.create_entry(EntryType::CertificateLoan {
            cert_id,
            lender: lender.clone(),
            borrower: borrower.clone(),
            duration_secs: terms.duration_secs,
            auto_return: terms.auto_return,
        });

        let entry_hash = spine.append(entry)?;
        let appended = spine
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;

        let now = Timestamp::now();
        let expires_at = terms
            .duration_secs
            .map(|secs| Timestamp::from_nanos(now.as_nanos() + secs * 1_000_000_000));
        cert.state = CertificateState::Loaned {
            loan_entry: entry_hash,
        };
        cert.holder = Some(borrower.clone());
        cert.active_loan = Some(LoanInfo {
            loan_entry: entry_hash,
            borrower: borrower.clone(),
            terms,
            started_at: now,
            expires_at,
            waypoint: None,
            waypoint_anchor: None,
            relending_chain: Some(RelendingChain::with_initial(borrower, entry_hash)),
        });
        cert.current_location = CertificateLocation {
            spine: spine_id,
            entry: entry_hash,
            index: spine.height - 1,
        };
        cert.updated_at = now;

        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&spine).await?;
        self.certificate_storage
            .save_certificate(&cert, spine_id)
            .await?;

        Ok(entry_hash)
    }

    /// Return a loaned certificate.
    ///
    /// Only the current borrower (holder) can return. With a relending chain,
    /// returns to the previous lender (partial return) or owner (full return).
    ///
    /// # Errors
    ///
    /// Returns error if certificate not found, not loaned, or return fails.
    pub async fn return_certificate(
        &self,
        cert_id: CertificateId,
        returner: Did,
    ) -> LoamSpineResult<EntryHash> {
        self.return_certificate_at(cert_id, returner).await
    }

    /// Return a loaned certificate at any point in the relending chain.
    ///
    /// Unwinds from the given returner: removes that borrower and all
    /// subsequent borrowers. The certificate returns to the previous lender
    /// or owner.
    ///
    /// # Errors
    ///
    /// Returns error if certificate not found, not loaned, returner not in
    /// chain, or return fails.
    pub async fn return_certificate_at(
        &self,
        cert_id: CertificateId,
        returner: Did,
    ) -> LoamSpineResult<EntryHash> {
        let (mut cert, spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        let loan = cert
            .active_loan
            .as_ref()
            .ok_or_else(|| LoamSpineError::InvalidEntryType("certificate not loaned".into()))?;

        if !matches!(&cert.state, CertificateState::Loaned { .. }) {
            return Err(LoamSpineError::InvalidEntryType(
                "certificate not loaned".into(),
            ));
        }

        let mut chain = loan.relending_chain.clone().unwrap_or_else(|| {
            RelendingChain::with_initial(loan.borrower.clone(), loan.loan_entry)
        });

        if !chain.contains(&returner) {
            return Err(LoamSpineError::LoanTermsViolation(
                "returner not in relending chain".into(),
            ));
        }

        let returner_loan_entry = chain
            .links
            .iter()
            .find(|l| l.borrower == returner)
            .map(|l| l.loan_entry)
            .ok_or_else(|| {
                LoamSpineError::LoanTermsViolation("returner not in relending chain".into())
            })?;

        let _unwound = chain.return_at(&returner)?;
        let new_holder = chain.current_holder().cloned();

        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entry = spine.create_entry(EntryType::CertificateReturn {
            cert_id,
            loan_entry: returner_loan_entry,
            usage_summary: None,
        });

        let entry_hash = spine.append(entry)?;
        let appended = spine
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;

        if let Some(holder) = new_holder {
            cert.holder = Some(holder.clone());
            cert.active_loan = Some(LoanInfo {
                borrower: holder,
                relending_chain: if chain.links.is_empty() {
                    None
                } else {
                    Some(chain)
                },
                ..loan.clone()
            });
        } else {
            cert.state = CertificateState::Active;
            cert.holder = None;
            cert.active_loan = None;
        }

        cert.current_location = CertificateLocation {
            spine: spine_id,
            entry: entry_hash,
            index: spine.height - 1,
        };
        cert.updated_at = Timestamp::now();

        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&spine).await?;
        self.certificate_storage
            .save_certificate(&cert, spine_id)
            .await?;

        Ok(entry_hash)
    }

    /// Sub-lend a loaned certificate to another party.
    ///
    /// The current holder (sublender) must have permission per `LoanTerms`.
    /// Validates `allow_sublend` and `max_sublend_depth`.
    ///
    /// # Errors
    ///
    /// Returns error if certificate not found, not loaned, sublender not
    /// current holder, or terms forbid sublending.
    pub async fn sublend_certificate(
        &self,
        cert_id: CertificateId,
        sublender: Did,
        new_borrower: Did,
    ) -> LoamSpineResult<EntryHash> {
        let (mut cert, spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        let loan = cert
            .active_loan
            .as_ref()
            .ok_or_else(|| LoamSpineError::InvalidEntryType("certificate not loaned".into()))?
            .clone();

        if cert.holder.as_ref() != Some(&sublender) {
            return Err(LoamSpineError::LoanTermsViolation(
                "only current holder can sublend".into(),
            ));
        }

        let mut chain = loan.relending_chain.clone().unwrap_or_else(|| {
            RelendingChain::with_initial(loan.borrower.clone(), loan.loan_entry)
        });

        chain.can_sublend(loan.terms.allow_sublend, loan.terms.max_sublend_depth)?;

        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entry = spine.create_entry(EntryType::CertificateLoan {
            cert_id,
            lender: sublender.clone(),
            borrower: new_borrower.clone(),
            duration_secs: loan.terms.duration_secs,
            auto_return: loan.terms.auto_return,
        });

        let entry_hash = spine.append(entry)?;
        let appended = spine
            .tip_entry()
            .ok_or_else(|| LoamSpineError::Internal("tip empty after append".into()))?;

        chain.links.push(crate::waypoint::RelendingLink {
            borrower: new_borrower.clone(),
            loan_entry: entry_hash,
        });

        cert.holder = Some(new_borrower.clone());
        cert.active_loan = Some(LoanInfo {
            borrower: new_borrower,
            relending_chain: Some(chain),
            ..loan
        });
        cert.current_location = CertificateLocation {
            spine: spine_id,
            entry: entry_hash,
            index: spine.height - 1,
        };
        cert.updated_at = Timestamp::now();

        self.entry_storage.save_entry(appended).await?;
        self.spine_storage.save_spine(&spine).await?;
        self.certificate_storage
            .save_certificate(&cert, spine_id)
            .await?;

        Ok(entry_hash)
    }

    /// Auto-return an expired certificate when `terms.auto_return` is true.
    ///
    /// Used by the expiry sweeper. Returns the certificate as the current
    /// holder (system-initiated return on behalf of expiry).
    ///
    /// # Errors
    ///
    /// Returns error if certificate not found, not loaned, not expired, or
    /// auto_return is false.
    pub async fn return_certificate_expired(
        &self,
        cert_id: CertificateId,
    ) -> LoamSpineResult<EntryHash> {
        let (cert, _spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        let loan = cert
            .active_loan
            .as_ref()
            .ok_or_else(|| LoamSpineError::InvalidEntryType("certificate not loaned".into()))?;

        if !loan.terms.auto_return {
            return Err(LoamSpineError::LoanTermsViolation(
                "auto_return not enabled for this loan".into(),
            ));
        }

        let Some(expires_at) = loan.expires_at else {
            return Err(LoamSpineError::LoanTermsViolation(
                "loan has no expiry".into(),
            ));
        };

        if Timestamp::now() <= expires_at {
            return Err(LoamSpineError::LoanTermsViolation(
                "loan not yet expired".into(),
            ));
        }

        let mut last_hash = self
            .return_certificate_at(cert_id, loan.borrower.clone())
            .await?;

        while self
            .get_certificate(cert_id)
            .await
            .is_some_and(|c| c.is_loaned())
        {
            let Some(cert) = self.get_certificate(cert_id).await else {
                break;
            };
            let Some(holder) = cert.holder else {
                break;
            };
            last_hash = self.return_certificate_at(cert_id, holder).await?;
        }

        Ok(last_hash)
    }
}
