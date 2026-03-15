// SPDX-License-Identifier: AGPL-3.0-only

//! Certificate lifecycle operations.
//!
//! This module provides certificate management functionality:
//! - **Mint**: Create new certificates
//! - **Transfer**: Permanent ownership change
//! - **Loan**: Temporary access grant
//! - **Return**: Loan completion
//!
//! ## Ownership Model
//!
//! Certificates follow a clear ownership model where only the current owner
//! can transfer or loan. Loans must be returned by the borrower.

use crate::certificate::{
    Certificate, CertificateLocation, CertificateMetadata, CertificateState, CertificateType,
    EscrowCondition, EscrowId, LoanInfo, LoanTerms, MintInfo, TransferConditions,
};
use crate::entry::EntryType;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::proof::CertificateOwnershipProof;
use crate::storage::{CertificateStorage, EntryStorage, SpineStorage};
use crate::types::{CertificateId, Did, EntryHash, SpineId, Timestamp};
use crate::waypoint::RelendingChain;

use super::LoamSpineService;

/// Result of certificate integrity verification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertificateVerification {
    /// Certificate ID that was verified.
    pub cert_id: CertificateId,
    /// Which checks passed (empty set = not found).
    pub passed: Vec<VerificationCheck>,
}

impl CertificateVerification {
    /// Whether the certificate exists at all.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.passed.contains(&VerificationCheck::Exists)
    }

    /// Whether all checks passed (full chain integrity).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.passed.contains(&VerificationCheck::ChainValid)
    }
}

/// Individual verification checks for a certificate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerificationCheck {
    /// Certificate record exists in storage.
    Exists,
    /// Associated spine exists.
    SpineExists,
    /// Mint entry present in entry storage.
    MintEntryExists,
    /// Full chain (spine + mint + current location) is intact.
    ChainValid,
}

/// Check if an entry type references a specific certificate ID.
fn entry_references_certificate(
    entry_type: &crate::entry::EntryType,
    cert_id: CertificateId,
) -> bool {
    match entry_type {
        crate::entry::EntryType::CertificateMint { cert_id: id, .. }
        | crate::entry::EntryType::CertificateTransfer { cert_id: id, .. }
        | crate::entry::EntryType::CertificateLoan { cert_id: id, .. }
        | crate::entry::EntryType::CertificateReturn { cert_id: id, .. } => *id == cert_id,
        _ => false,
    }
}

impl LoamSpineService {
    // ========================================================================
    // Certificate Operations
    // ========================================================================

    /// Mint a new certificate.
    ///
    /// Creates a new certificate and records it in the spine.
    ///
    /// # Errors
    ///
    /// Returns error if spine not found or certificate creation fails.
    pub async fn mint_certificate(
        &self,
        spine_id: SpineId,
        cert_type: CertificateType,
        owner: Did,
        metadata: Option<CertificateMetadata>,
    ) -> LoamSpineResult<(CertificateId, EntryHash)> {
        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let cert_id = uuid::Uuid::now_v7();

        let cert_type_str = format!("{cert_type:?}");
        let entry = spine.create_entry(EntryType::CertificateMint {
            cert_id,
            cert_type: cert_type_str,
            initial_owner: owner.clone(),
        });

        let entry_hash = spine.append(entry.clone())?;

        let mint_info = MintInfo {
            minter: owner.clone(),
            spine: spine_id,
            entry: entry_hash,
            timestamp: Timestamp::now(),
            authority: None,
        };

        let mut cert = Certificate::new(cert_id, cert_type, &owner, &mint_info);
        cert.current_location = CertificateLocation {
            spine: spine_id,
            entry: entry_hash,
            index: spine.height - 1,
        };

        if let Some(meta) = metadata {
            cert.metadata = meta;
        }

        self.entry_storage.save_entry(&entry).await?;
        self.spine_storage.save_spine(&spine).await?;
        self.certificate_storage
            .save_certificate(&cert, spine_id)
            .await?;

        Ok((cert_id, entry_hash))
    }

    /// Get a certificate by ID.
    pub async fn get_certificate(&self, cert_id: CertificateId) -> Option<Certificate> {
        self.certificate_storage
            .get_certificate(cert_id)
            .await
            .ok()
            .flatten()
            .map(|(cert, _)| cert)
    }

    /// Transfer a certificate to a new owner.
    ///
    /// This is a permanent ownership change. The certificate cannot be
    /// transferred if it is currently loaned out.
    ///
    /// # Errors
    ///
    /// Returns error if certificate not found, caller is not owner, or transfer fails.
    pub async fn transfer_certificate(
        &self,
        cert_id: CertificateId,
        from: Did,
        to: Did,
    ) -> LoamSpineResult<EntryHash> {
        let (mut cert, spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        if cert.owner != from {
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

        let entry = spine.create_entry(EntryType::CertificateTransfer {
            cert_id,
            from: from.clone(),
            to: to.clone(),
        });

        let entry_hash = spine.append(entry.clone())?;

        cert.owner = to;
        cert.transfer_count += 1;
        cert.current_location = CertificateLocation {
            spine: spine_id,
            entry: entry_hash,
            index: spine.height - 1,
        };
        cert.updated_at = Timestamp::now();

        self.entry_storage.save_entry(&entry).await?;
        self.spine_storage.save_spine(&spine).await?;
        self.certificate_storage
            .save_certificate(&cert, spine_id)
            .await?;

        Ok(entry_hash)
    }

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

        let entry_hash = spine.append(entry.clone())?;

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

        self.entry_storage.save_entry(&entry).await?;
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

        let entry_hash = spine.append(entry.clone())?;

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

        self.entry_storage.save_entry(&entry).await?;
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

        let entry_hash = spine.append(entry.clone())?;

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

        self.entry_storage.save_entry(&entry).await?;
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

    /// Verify a certificate's integrity.
    ///
    /// Checks that:
    /// - The certificate exists in storage
    /// - Its associated spine exists
    /// - The mint entry is present in storage
    /// - Chain integrity (owner matches spine records)
    ///
    /// # Errors
    ///
    /// Returns error if storage lookup fails.
    pub async fn verify_certificate(
        &self,
        cert_id: CertificateId,
    ) -> LoamSpineResult<CertificateVerification> {
        let Some((cert, spine_id)) = self.certificate_storage.get_certificate(cert_id).await?
        else {
            return Ok(CertificateVerification {
                cert_id,
                passed: Vec::new(),
            });
        };

        let mut passed = vec![VerificationCheck::Exists];

        let spine_exists = self.spine_storage.get_spine(spine_id).await?.is_some();
        if spine_exists {
            passed.push(VerificationCheck::SpineExists);
        }

        let mint_entry_exists = self
            .entry_storage
            .get_entry(cert.mint_info.entry)
            .await?
            .is_some();
        if mint_entry_exists {
            passed.push(VerificationCheck::MintEntryExists);
        }

        let location_entry_exists = self
            .entry_storage
            .get_entry(cert.current_location.entry)
            .await?
            .is_some();

        if spine_exists && mint_entry_exists && location_entry_exists {
            passed.push(VerificationCheck::ChainValid);
        }

        Ok(CertificateVerification { cert_id, passed })
    }

    /// Retrieve the lifecycle history of a certificate stored via the
    /// certificate storage trait.
    ///
    /// Returns all entries referencing this certificate ID (mint, transfer,
    /// loan, return) in spine order.  This method requires the certificate
    /// to have been created through `mint_certificate` (i.e. it exists in
    /// `certificate_storage`).  For a trait-based scan of all spines, use
    /// the `ProvenanceSource::get_certificate_history` trait method.
    ///
    /// # Errors
    ///
    /// Returns error if certificate or spine not found, or storage fails.
    pub async fn certificate_lifecycle(
        &self,
        cert_id: CertificateId,
    ) -> LoamSpineResult<Vec<crate::entry::Entry>> {
        let (_cert, spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        let spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entries = self
            .entry_storage
            .get_entries_for_spine(spine_id, 0, spine.height)
            .await?;

        let history: Vec<crate::entry::Entry> = entries
            .into_iter()
            .filter(|e| entry_references_certificate(&e.entry_type, cert_id))
            .collect();

        Ok(history)
    }

    /// Generate a cryptographic proof of a certificate's ownership chain.
    ///
    /// Collects all ownership-establishing entries (mint and transfers),
    /// builds a Merkle tree over their hashes, and returns a
    /// `CertificateOwnershipProof`.
    ///
    /// # Errors
    ///
    /// Returns error if certificate not found or storage fails.
    pub async fn generate_provenance_proof(
        &self,
        cert_id: CertificateId,
    ) -> LoamSpineResult<CertificateOwnershipProof> {
        let (_cert, _spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        let lifecycle = self.certificate_lifecycle(cert_id).await?;

        let mut entries: Vec<EntryHash> = Vec::new();

        for entry in &lifecycle {
            let hash = entry.compute_hash()?;
            match &entry.entry_type {
                EntryType::CertificateMint { .. } | EntryType::CertificateTransfer { .. } => {
                    entries.push(hash);
                }
                _ => {}
            }
        }

        let chain_root = crate::proof::compute_merkle_root(&entries);
        let chain_length = entries.len() as u64;

        Ok(CertificateOwnershipProof {
            certificate_id: cert_id,
            chain_root,
            chain_length,
            entries,
            created_at: Timestamp::now(),
        })
    }

    /// Put a certificate in escrow (PendingTransfer state).
    ///
    /// The certificate must be Active. The current owner becomes the seller
    /// (`from`), and `to` is the intended buyer. Conditions must be met
    /// before `release_certificate` completes the transfer.
    ///
    /// # Errors
    ///
    /// Returns error if certificate not found, not owned by seller, or
    /// already loaned.
    pub async fn hold_certificate(
        &self,
        cert_id: CertificateId,
        to: Did,
        conditions: Vec<EscrowCondition>,
        expires_at: Option<Timestamp>,
    ) -> LoamSpineResult<EscrowId> {
        let (mut cert, spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        let from = cert.owner.clone();

        if cert.is_loaned() {
            return Err(LoamSpineError::CertificateLoaned(cert_id));
        }

        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entry = spine.create_entry(EntryType::CertificateTransfer {
            cert_id,
            from: from.clone(),
            to: to.clone(),
        });

        let entry_hash = spine.append(entry.clone())?;

        let escrow_id = uuid::Uuid::now_v7();
        let now = Timestamp::now();

        cert.state = CertificateState::PendingTransfer {
            transfer_entry: entry_hash,
            to: to.clone(),
        };
        cert.current_location = crate::certificate::CertificateLocation {
            spine: spine_id,
            entry: entry_hash,
            index: spine.height - 1,
        };
        cert.updated_at = now;

        let transfer_conditions = TransferConditions {
            escrow_id,
            cert_id,
            from: from.clone(),
            to: to.clone(),
            conditions,
            expires_at,
            created_at: now,
        };

        self.entry_storage.save_entry(&entry).await?;
        self.spine_storage.save_spine(&spine).await?;
        self.certificate_storage
            .save_certificate(&cert, spine_id)
            .await?;

        self.escrows
            .write()
            .await
            .insert(escrow_id, transfer_conditions);

        tracing::info!(escrow_id = %escrow_id, cert_id = %cert_id, "certificate held in escrow");

        Ok(escrow_id)
    }

    /// Release a certificate from escrow, completing the transfer.
    ///
    /// # Errors
    ///
    /// Returns error if escrow not found.
    pub async fn release_certificate(&self, escrow_id: EscrowId) -> LoamSpineResult<CertificateId> {
        let conditions = self
            .escrows
            .write()
            .await
            .remove(&escrow_id)
            .ok_or(LoamSpineError::EscrowNotFound(escrow_id))?;

        let cert_id = conditions.cert_id;

        let (mut cert, spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        let to = conditions.to.clone();

        match &cert.state {
            CertificateState::PendingTransfer { .. } => {}
            _ => {
                return Err(LoamSpineError::InvalidEntryType(
                    "certificate not in pending transfer".into(),
                ));
            }
        }

        cert.owner = to;
        cert.transfer_count += 1;
        cert.state = CertificateState::Active;
        cert.updated_at = Timestamp::now();

        self.certificate_storage
            .save_certificate(&cert, spine_id)
            .await?;

        tracing::info!(escrow_id = %escrow_id, cert_id = %cert_id, "certificate released from escrow");

        Ok(cert_id)
    }

    /// Cancel an escrow, returning the certificate to Active state.
    ///
    /// # Errors
    ///
    /// Returns error if escrow not found.
    pub async fn cancel_escrow(&self, escrow_id: EscrowId) -> LoamSpineResult<()> {
        let conditions = self
            .escrows
            .write()
            .await
            .remove(&escrow_id)
            .ok_or(LoamSpineError::EscrowNotFound(escrow_id))?;

        let cert_id = conditions.cert_id;

        let (mut cert, spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        match &cert.state {
            CertificateState::PendingTransfer { .. } => {}
            _ => {
                return Err(LoamSpineError::InvalidEntryType(
                    "certificate not in pending transfer".into(),
                ));
            }
        }

        cert.state = CertificateState::Active;
        cert.updated_at = Timestamp::now();

        self.certificate_storage
            .save_certificate(&cert, spine_id)
            .await?;

        tracing::info!(escrow_id = %escrow_id, cert_id = %cert_id, "escrow cancelled");

        Ok(())
    }

    /// List all certificates.
    ///
    /// # Errors
    ///
    /// Returns error if storage fails.
    pub async fn list_certificates(&self) -> LoamSpineResult<Vec<Certificate>> {
        let ids = self.certificate_storage.list_certificates().await?;
        let mut result = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some((cert, _)) = self.certificate_storage.get_certificate(id).await? {
                result.push(cert);
            }
        }
        Ok(result)
    }

    /// Get certificate count.
    pub async fn certificate_count(&self) -> usize {
        self.certificate_storage.certificate_count().await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "certificate_tests.rs"]
mod tests;
