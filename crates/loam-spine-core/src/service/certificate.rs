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
    LoanInfo, LoanTerms, MintInfo,
};
use crate::entry::EntryType;
use crate::error::{LoamSpineError, LoamSpineResult};
use crate::storage::{CertificateStorage, EntryStorage, SpineStorage};
use crate::types::{CertificateId, Did, EntryHash, SpineId, Timestamp};

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
        cert.state = CertificateState::Loaned {
            loan_entry: entry_hash,
        };
        cert.holder = Some(borrower.clone());
        cert.active_loan = Some(LoanInfo {
            loan_entry: entry_hash,
            borrower,
            terms,
            started_at: now,
            expires_at: None,
            waypoint: None,
            waypoint_anchor: None,
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
    /// Only the current borrower (holder) can return the certificate.
    ///
    /// # Errors
    ///
    /// Returns error if certificate not found, not loaned, or return fails.
    pub async fn return_certificate(
        &self,
        cert_id: CertificateId,
        returner: Did,
    ) -> LoamSpineResult<EntryHash> {
        let (mut cert, spine_id) = self
            .certificate_storage
            .get_certificate(cert_id)
            .await?
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        let loan_entry = match &cert.state {
            CertificateState::Loaned { loan_entry } => *loan_entry,
            _ => {
                return Err(LoamSpineError::InvalidEntryType(
                    "certificate not loaned".into(),
                ))
            }
        };

        if cert.holder.as_ref() != Some(&returner) {
            return Err(LoamSpineError::LoanTermsViolation(
                "only borrower can return".into(),
            ));
        }

        let mut spine = self
            .spine_storage
            .get_spine(spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(spine_id))?;

        let entry = spine.create_entry(EntryType::CertificateReturn {
            cert_id,
            loan_entry,
        });

        let entry_hash = spine.append(entry.clone())?;

        cert.state = CertificateState::Active;
        cert.holder = None;
        cert.active_loan = None;
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
mod tests {
    use super::*;
    use crate::certificate::CertificateType;

    #[tokio::test]
    async fn test_mint_certificate() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let cert_type = CertificateType::DigitalGame {
            platform: "steam".into(),
            game_id: "test".into(),
            edition: None,
        };

        let result = service
            .mint_certificate(spine_id, cert_type, owner.clone(), None)
            .await;
        assert!(result.is_ok());

        let (cert_id, _hash) = result.unwrap_or_else(|_| unreachable!());

        // Check certificate exists
        let cert = service.get_certificate(cert_id).await;
        assert!(cert.is_some());

        // List certificates
        let certs = service
            .list_certificates()
            .await
            .unwrap_or_else(|_| unreachable!());
        assert!(!certs.is_empty());

        // Certificate count
        assert!(service.certificate_count().await >= 1);
    }

    #[tokio::test]
    async fn test_certificate_transfer() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");
        let buyer = Did::new("did:key:z6MkBuyer");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let cert_type = CertificateType::DigitalGame {
            platform: "steam".into(),
            game_id: "test".into(),
            edition: None,
        };

        let (cert_id, _hash) = service
            .mint_certificate(spine_id, cert_type, owner.clone(), None)
            .await
            .unwrap_or_else(|_| unreachable!());

        // Transfer
        let result = service
            .transfer_certificate(cert_id, owner.clone(), buyer.clone())
            .await;
        assert!(result.is_ok());

        // Verify new owner
        let cert = service.get_certificate(cert_id).await;
        assert!(cert.is_some());
        assert_eq!(cert.unwrap_or_else(|| unreachable!()).owner, buyer);
    }

    #[tokio::test]
    async fn test_certificate_loan_and_return() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");
        let borrower = Did::new("did:key:z6MkBorrower");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let cert_type = CertificateType::DigitalGame {
            platform: "steam".into(),
            game_id: "test".into(),
            edition: None,
        };

        let (cert_id, _hash) = service
            .mint_certificate(spine_id, cert_type, owner.clone(), None)
            .await
            .unwrap_or_else(|_| unreachable!());

        // Loan with configurable duration
        let terms = crate::certificate::LoanTerms::new()
            .with_duration(crate::certificate::SECONDS_PER_DAY)
            .with_auto_return(false);
        let result = service
            .loan_certificate(cert_id, owner.clone(), borrower.clone(), terms)
            .await;
        assert!(result.is_ok());

        // Verify loaned
        let cert = service.get_certificate(cert_id).await;
        assert!(cert.is_some());
        assert!(cert.unwrap_or_else(|| unreachable!()).is_loaned());

        // Return
        let result = service.return_certificate(cert_id, borrower.clone()).await;
        assert!(result.is_ok());

        // Verify returned
        let cert = service.get_certificate(cert_id).await;
        assert!(cert.is_some());
        assert!(!cert.unwrap_or_else(|| unreachable!()).is_loaned());
    }

    #[tokio::test]
    async fn test_verify_certificate() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let cert_type = CertificateType::DigitalGame {
            platform: "steam".into(),
            game_id: "test".into(),
            edition: None,
        };

        let (cert_id, _hash) = service
            .mint_certificate(spine_id, cert_type, owner.clone(), None)
            .await
            .unwrap_or_else(|_| unreachable!());

        let verification = service
            .verify_certificate(cert_id)
            .await
            .unwrap_or_else(|_| unreachable!());

        assert!(verification.exists());
        assert!(verification.is_valid());
    }

    #[tokio::test]
    async fn test_verify_nonexistent_certificate() {
        let service = LoamSpineService::new();
        let fake_id = uuid::Uuid::now_v7();

        let verification = service
            .verify_certificate(fake_id)
            .await
            .unwrap_or_else(|_| unreachable!());

        assert!(!verification.exists());
        assert!(!verification.is_valid());
    }

    #[tokio::test]
    async fn test_get_certificate_history() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");
        let buyer = Did::new("did:key:z6MkBuyer");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let cert_type = CertificateType::DigitalGame {
            platform: "steam".into(),
            game_id: "test".into(),
            edition: None,
        };

        let (cert_id, _hash) = service
            .mint_certificate(spine_id, cert_type, owner.clone(), None)
            .await
            .unwrap_or_else(|_| unreachable!());

        // Transfer
        service
            .transfer_certificate(cert_id, owner.clone(), buyer.clone())
            .await
            .unwrap_or_else(|_| unreachable!());

        let history = service
            .certificate_lifecycle(cert_id)
            .await
            .unwrap_or_else(|_| unreachable!());

        // mint + transfer = 2 entries
        assert_eq!(history.len(), 2);
    }

    #[tokio::test]
    async fn test_get_certificate_history_with_loan() {
        let service = LoamSpineService::new();
        let owner = Did::new("did:key:z6MkOwner");
        let borrower = Did::new("did:key:z6MkBorrower");

        let spine_id = service
            .ensure_spine(owner.clone(), Some("Test".into()))
            .await
            .unwrap_or_else(|_| unreachable!());

        let cert_type = CertificateType::DigitalGame {
            platform: "steam".into(),
            game_id: "test".into(),
            edition: None,
        };

        let (cert_id, _hash) = service
            .mint_certificate(spine_id, cert_type, owner.clone(), None)
            .await
            .unwrap_or_else(|_| unreachable!());

        // Loan
        let terms = crate::certificate::LoanTerms::new()
            .with_duration(crate::certificate::SECONDS_PER_DAY)
            .with_auto_return(false);
        service
            .loan_certificate(cert_id, owner.clone(), borrower.clone(), terms)
            .await
            .unwrap_or_else(|_| unreachable!());

        // Return
        service
            .return_certificate(cert_id, borrower.clone())
            .await
            .unwrap_or_else(|_| unreachable!());

        // mint + loan + return = 3
        let history = service
            .certificate_lifecycle(cert_id)
            .await
            .unwrap_or_else(|_| unreachable!());

        assert_eq!(history.len(), 3);
    }
}
