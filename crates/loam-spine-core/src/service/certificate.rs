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
use crate::storage::{EntryStorage, SpineStorage};
use crate::types::{CertificateId, Did, EntryHash, SpineId, Timestamp};

use super::LoamSpineService;

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

        let mint_info = MintInfo {
            minter: owner.clone(),
            spine: spine_id,
            entry: [0u8; 32],
            timestamp: Timestamp::now(),
            authority: None,
        };

        let cert_type_str = format!("{cert_type:?}");
        let certificate = Certificate::new(cert_id, cert_type, &owner, &mint_info);

        let entry = spine.create_entry(EntryType::CertificateMint {
            cert_id,
            cert_type: cert_type_str,
            initial_owner: owner.clone(),
        });

        let entry_hash = spine.append(entry.clone())?;

        let mut cert = certificate;
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

        self.certificates
            .write()
            .await
            .insert(cert_id, (cert, spine_id));

        Ok((cert_id, entry_hash))
    }

    /// Get a certificate by ID.
    pub async fn get_certificate(&self, cert_id: CertificateId) -> Option<Certificate> {
        let certs = self.certificates.read().await;
        certs.get(&cert_id).map(|(cert, _)| cert.clone())
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
        let mut certs = self.certificates.write().await;
        let (cert, spine_id) = certs
            .get_mut(&cert_id)
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        if cert.owner != from {
            return Err(LoamSpineError::NotCertificateOwner);
        }

        if cert.is_loaned() {
            return Err(LoamSpineError::CertificateLoaned(cert_id));
        }

        let mut spine = self
            .spine_storage
            .get_spine(*spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(*spine_id))?;

        let entry = spine.create_entry(EntryType::CertificateTransfer {
            cert_id,
            from: from.clone(),
            to: to.clone(),
        });

        let entry_hash = spine.append(entry.clone())?;

        cert.owner = to;
        cert.transfer_count += 1;
        cert.current_location = CertificateLocation {
            spine: *spine_id,
            entry: entry_hash,
            index: spine.height - 1,
        };
        cert.updated_at = Timestamp::now();

        self.entry_storage.save_entry(&entry).await?;
        self.spine_storage.save_spine(&spine).await?;

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
        let mut certs = self.certificates.write().await;
        let (cert, spine_id) = certs
            .get_mut(&cert_id)
            .ok_or(LoamSpineError::CertificateNotFound(cert_id))?;

        if cert.owner != lender {
            return Err(LoamSpineError::NotCertificateOwner);
        }

        if cert.is_loaned() {
            return Err(LoamSpineError::CertificateLoaned(cert_id));
        }

        let mut spine = self
            .spine_storage
            .get_spine(*spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(*spine_id))?;

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
            spine: *spine_id,
            entry: entry_hash,
            index: spine.height - 1,
        };
        cert.updated_at = now;

        self.entry_storage.save_entry(&entry).await?;
        self.spine_storage.save_spine(&spine).await?;

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
        let mut certs = self.certificates.write().await;
        let (cert, spine_id) = certs
            .get_mut(&cert_id)
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
            .get_spine(*spine_id)
            .await?
            .ok_or(LoamSpineError::SpineNotFound(*spine_id))?;

        let entry = spine.create_entry(EntryType::CertificateReturn {
            cert_id,
            loan_entry,
        });

        let entry_hash = spine.append(entry.clone())?;

        cert.state = CertificateState::Active;
        cert.holder = None;
        cert.active_loan = None;
        cert.current_location = CertificateLocation {
            spine: *spine_id,
            entry: entry_hash,
            index: spine.height - 1,
        };
        cert.updated_at = Timestamp::now();

        self.entry_storage.save_entry(&entry).await?;
        self.spine_storage.save_spine(&spine).await?;

        Ok(entry_hash)
    }

    /// List all certificates.
    pub async fn list_certificates(&self) -> Vec<Certificate> {
        let certs = self.certificates.read().await;
        certs.values().map(|(cert, _)| cert.clone()).collect()
    }

    /// Get certificate count.
    pub async fn certificate_count(&self) -> usize {
        self.certificates.read().await.len()
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
        let certs = service.list_certificates().await;
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
}
