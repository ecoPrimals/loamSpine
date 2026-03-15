// SPDX-License-Identifier: AGPL-3.0-only

//! Certificate and Spine management traits.
//!
//! These traits define the operations for managing spines and certificates.

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use crate::certificate::{
    Certificate, CertificateLocation, CertificateMetadata, CertificateState, CertificateType,
    LoanInfo, LoanTerms, MintInfo,
};
use crate::entry::EntryType;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::spine::Spine;
use crate::types::{CertificateId, Did, EntryHash, Timestamp};

/// Certificate manager for a spine.
///
/// Provides operations for minting, transferring, loaning, and returning certificates.
pub struct CertificateManager {
    /// The underlying spine.
    spine: Spine,
    /// Certificate registry (id -> certificate).
    certificates: HashMap<CertificateId, Certificate>,
}

impl CertificateManager {
    /// Create a new certificate manager for a spine.
    #[must_use]
    pub fn new(spine: Spine) -> Self {
        Self {
            spine,
            certificates: HashMap::new(),
        }
    }

    /// Get the underlying spine.
    #[must_use]
    pub const fn spine(&self) -> &Spine {
        &self.spine
    }

    /// Get a mutable reference to the underlying spine.
    pub const fn spine_mut(&mut self) -> &mut Spine {
        &mut self.spine
    }

    /// Get a certificate by ID.
    #[must_use]
    pub fn get_certificate(&self, id: &CertificateId) -> Option<&Certificate> {
        self.certificates.get(id)
    }

    /// List all certificates.
    #[must_use]
    pub fn list_certificates(&self) -> Vec<&Certificate> {
        self.certificates.values().collect()
    }

    /// Mint a new certificate.
    ///
    /// # Errors
    ///
    /// Returns an error if the spine is sealed or the certificate already exists.
    pub fn mint(
        &mut self,
        cert_type: CertificateType,
        initial_owner: &Did,
        metadata: CertificateMetadata,
    ) -> LoamSpineResult<(Certificate, EntryHash)> {
        let cert_id = CertificateId::now_v7();

        if self.certificates.contains_key(&cert_id) {
            return Err(LoamSpineError::CertificateExists(cert_id));
        }

        let entry = self.spine.create_entry(EntryType::CertificateMint {
            cert_id,
            cert_type: cert_type.category().to_string(),
            initial_owner: initial_owner.clone(),
        });

        let entry_hash = self.spine.append(entry)?;

        let mint_info = MintInfo::new(initial_owner.clone(), self.spine.id, entry_hash);

        let cert =
            Certificate::new(cert_id, cert_type, initial_owner, &mint_info).with_metadata(metadata);

        self.certificates.insert(cert_id, cert.clone());

        Ok((cert, entry_hash))
    }

    /// Transfer a certificate to a new owner.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The certificate doesn't exist
    /// - The caller is not the owner
    /// - The certificate is loaned
    pub fn transfer(
        &mut self,
        cert_id: CertificateId,
        caller: &Did,
        to: &Did,
    ) -> LoamSpineResult<EntryHash> {
        let cert = self
            .certificates
            .get(&cert_id)
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        if &cert.owner != caller {
            return Err(LoamSpineError::NotCertificateOwner);
        }

        if cert.is_loaned() {
            return Err(LoamSpineError::CertificateLoaned(cert_id));
        }

        let entry = self.spine.create_entry(EntryType::CertificateTransfer {
            cert_id,
            from: caller.clone(),
            to: to.clone(),
        });

        let entry_hash = self.spine.append(entry)?;

        if let Some(cert) = self.certificates.get_mut(&cert_id) {
            cert.owner = to.clone();
            cert.transfer_count += 1;
            cert.updated_at = Timestamp::now();
            cert.current_location = CertificateLocation {
                spine: self.spine.id,
                entry: entry_hash,
                index: self.spine.height - 1,
            };
        }

        Ok(entry_hash)
    }

    /// Loan a certificate to a borrower.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The certificate doesn't exist
    /// - The caller is not the owner
    /// - The certificate is already loaned
    pub fn loan(
        &mut self,
        cert_id: CertificateId,
        caller: &Did,
        borrower: &Did,
        terms: LoanTerms,
    ) -> LoamSpineResult<EntryHash> {
        let cert = self
            .certificates
            .get(&cert_id)
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        if &cert.owner != caller {
            return Err(LoamSpineError::NotCertificateOwner);
        }

        if cert.is_loaned() {
            return Err(LoamSpineError::CertificateLoaned(cert_id));
        }

        let entry = self.spine.create_entry(EntryType::CertificateLoan {
            cert_id,
            lender: caller.clone(),
            borrower: borrower.clone(),
            duration_secs: terms.duration_secs,
            auto_return: terms.auto_return,
        });

        let entry_hash = self.spine.append(entry)?;

        let expires_at = terms.duration_secs.map(|secs| {
            let now = Timestamp::now();
            Timestamp::from_nanos(now.as_nanos() + secs * 1_000_000_000)
        });

        if let Some(cert) = self.certificates.get_mut(&cert_id) {
            cert.holder = Some(borrower.clone());
            cert.state = CertificateState::Loaned {
                loan_entry: entry_hash,
            };
            cert.active_loan = Some(LoanInfo {
                loan_entry: entry_hash,
                borrower: borrower.clone(),
                terms,
                started_at: Timestamp::now(),
                expires_at,
                waypoint: None,
                waypoint_anchor: None,
                relending_chain: Some(crate::waypoint::RelendingChain::with_initial(
                    borrower.clone(),
                    entry_hash,
                )),
            });
            cert.updated_at = Timestamp::now();
            cert.current_location = CertificateLocation {
                spine: self.spine.id,
                entry: entry_hash,
                index: self.spine.height - 1,
            };
        }

        Ok(entry_hash)
    }

    /// Return a loaned certificate.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The certificate doesn't exist
    /// - The certificate is not loaned
    /// - The caller is not the borrower
    pub fn return_loan(
        &mut self,
        cert_id: CertificateId,
        caller: &Did,
    ) -> LoamSpineResult<EntryHash> {
        let cert = self
            .certificates
            .get(&cert_id)
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        let loan_entry = match &cert.state {
            CertificateState::Loaned { loan_entry } => *loan_entry,
            _ => {
                return Err(LoamSpineError::LoanTermsViolation(
                    "certificate is not loaned".into(),
                ))
            }
        };

        if cert.holder.as_ref() != Some(caller) {
            return Err(LoamSpineError::LoanTermsViolation(
                "caller is not the borrower".into(),
            ));
        }

        let entry = self.spine.create_entry(EntryType::CertificateReturn {
            cert_id,
            loan_entry,
        });

        let entry_hash = self.spine.append(entry)?;

        if let Some(cert) = self.certificates.get_mut(&cert_id) {
            cert.holder = None;
            cert.state = CertificateState::Active;
            cert.active_loan = None;
            cert.updated_at = Timestamp::now();
            cert.current_location = CertificateLocation {
                spine: self.spine.id,
                entry: entry_hash,
                index: self.spine.height - 1,
            };
        }

        Ok(entry_hash)
    }

    /// Check for expired loans and auto-return them.
    ///
    /// Returns the number of certificates auto-returned.
    pub fn process_expired_loans(&mut self) -> usize {
        let now = Timestamp::now();
        let mut expired: Vec<(CertificateId, Did)> = Vec::new();

        for cert in self.certificates.values() {
            if let Some(loan) = &cert.active_loan {
                if loan.terms.auto_return {
                    if let Some(expires_at) = loan.expires_at {
                        if now.as_nanos() > expires_at.as_nanos() {
                            expired.push((cert.id, loan.borrower.clone()));
                        }
                    }
                }
            }
        }

        let mut count = 0;
        for (cert_id, borrower) in expired {
            if self.return_loan(cert_id, &borrower).is_ok() {
                count += 1;
            }
        }

        count
    }
}
